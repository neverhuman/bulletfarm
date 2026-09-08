    use super::{
        MANIFEST_NAME, MAX_FILE_BYTES, MAX_MANIFEST_BYTES, MAX_PACKAGE_LOCK_BYTES, MAX_TOOL_BYTES,
        MAX_TOOL_TREE_BYTES, decode_manifest, manifest_root, read_manifest, read_regular_bounded,
        read_regular_bounded_with_hooks,
    };

    fn manifest_value() -> serde_json::Value {
        serde_json::json!({
            "schema_version": "bullet.portal.bundle.v1",
            "source": {
                "repository": "bullet-portal",
                "commit_oid": "sha1:0000000000000000000000000000000000000000",
                "tree_oid": "sha1:1111111111111111111111111111111111111111"
            },
            "package_lock": {
                "path": "package-lock.json",
                "size": 1,
                "blake3": "blake3:0000000000000000000000000000000000000000000000000000000000000000"
            },
            "tools": [
                {"name":"git","version":"git version 2.44.0","size":1,"blake3":"blake3:2222222222222222222222222222222222222222222222222222222222222222"},
                {"name":"node","version":"v24.1.0","size":2,"blake3":"blake3:3333333333333333333333333333333333333333333333333333333333333333","platform":"linux","architecture":"x64"},
                {"name":"npm","version":"11.2.0","size":3,"blake3":"blake3:4444444444444444444444444444444444444444444444444444444444444444","file_count":1}
            ],
            "files": [{"path":"index.html","size":0,"mime":"text/html; charset=utf-8","blake3":"blake3:5555555555555555555555555555555555555555555555555555555555555555"}],
            "total_size": 0
        })
    }

    fn rooted_manifest(value: &serde_json::Value) -> serde_json::Value {
        let mut rooted = value.clone();
        rooted["root"] = serde_json::Value::String(format!("blake3:{}", "0".repeat(64)));
        let root = manifest_root(rooted.clone()).expect("manifest root");
        rooted["root"] = serde_json::Value::String(root);
        rooted
    }

    fn raw_manifest_bytes(value: &serde_json::Value) -> Vec<u8> {
        let mut bytes = bullet_wire::canonical_json(value).expect("canonical manifest");
        bytes.push(b'\n');
        bytes
    }

    fn manifest_bytes(value: &serde_json::Value) -> Vec<u8> {
        raw_manifest_bytes(&rooted_manifest(value))
    }

    fn file(path: &str, mime: &str) -> serde_json::Value {
        serde_json::json!({
            "path": path,
            "size": 0,
            "mime": mime,
            "blake3": "blake3:5555555555555555555555555555555555555555555555555555555555555555"
        })
    }

    #[test]
    fn portal_manifest_refuses_ambiguous_or_unsafe_projections() {
        let value = manifest_value();
        let valid = manifest_bytes(&value);
        let decoded = decode_manifest(&valid).expect("full canonical producer schema");
        assert_eq!(decoded.tools.len(), 3);
        assert_eq!(decoded.files[0].mime, "text/html; charset=utf-8");
        let mut unicode = value.clone();
        unicode["files"] = serde_json::json!([
            file("assets/\u{e9}.js", "text/javascript; charset=utf-8"),
            file("index.html", "text/html; charset=utf-8")
        ]);
        decode_manifest(&manifest_bytes(&unicode)).expect("NFC UTF-8 Portal path");
        let mut utf16_order = value.clone();
        utf16_order["files"] = serde_json::json!([
            file("assets/\u{1f600}.js", "text/javascript; charset=utf-8"),
            file("assets/\u{e000}.js", "text/javascript; charset=utf-8"),
            file("index.html", "text/html; charset=utf-8")
        ]);
        decode_manifest(&manifest_bytes(&utf16_order)).expect("producer UTF-16 path order");
        utf16_order["files"]
            .as_array_mut()
            .expect("file records")
            .swap(0, 1);
        assert!(
            decode_manifest(&manifest_bytes(&utf16_order)).is_err(),
            "Rust scalar ordering was admitted instead of producer UTF-16 ordering"
        );

        let mut wrong_root = rooted_manifest(&value);
        wrong_root["root"] = serde_json::Value::String(format!("blake3:{}", "f".repeat(64)));
        let error = decode_manifest(&raw_manifest_bytes(&wrong_root))
            .err()
            .expect("a root not binding the canonical body must fail closed");
        assert!(error.to_string().contains("root does not bind"));

        let canonical = String::from_utf8(valid.clone()).expect("UTF-8 manifest");
        let unsafe_size =
            canonical.replacen("\"total_size\":0", "\"total_size\":9007199254740992", 1);
        let error = decode_manifest(unsafe_size.as_bytes())
            .err()
            .expect("an unsafe Portal byte total must fail closed");
        assert!(error.to_string().contains("UNSAFE_JSON_INTEGER"));

        let duplicate =
            canonical.replacen("\"total_size\":0", "\"total_size\":0,\"total_size\":0", 1);
        let error = decode_manifest(duplicate.as_bytes())
            .err()
            .expect("a duplicate Portal manifest member must fail closed");
        assert!(error.to_string().contains("DUPLICATE_JSON_KEY"));

        let mut unknown = value.clone();
        unknown["unexpected"] = serde_json::Value::Bool(true);
        let error = decode_manifest(&manifest_bytes(&unknown))
            .err()
            .expect("unknown Portal manifest members must fail closed");
        assert!(error.to_string().contains("unknown field"));

        let mut pretty =
            serde_json::to_vec_pretty(&rooted_manifest(&value)).expect("pretty manifest");
        pretty.push(b'\n');
        assert!(
            decode_manifest(&pretty).is_err(),
            "noncanonical JSON was admitted"
        );
        assert!(
            decode_manifest(&valid[..valid.len() - 1]).is_err(),
            "a manifest without its one LF was admitted"
        );

        let mut hostile_subjects = Vec::new();
        let mut hostile = value.clone();
        hostile["tools"][0]["version"] = serde_json::Value::String("git 2.44".to_owned());
        hostile_subjects.push(hostile);
        let mut hostile = value.clone();
        hostile["tools"][0]["version"] =
            serde_json::Value::String(format!("git version {}.1.1", "1".repeat(150)));
        hostile_subjects.push(hostile);
        let mut hostile = value.clone();
        hostile["tools"][0]["size"] = serde_json::Value::from(MAX_TOOL_BYTES + 1);
        hostile_subjects.push(hostile);
        let mut hostile = value.clone();
        hostile["tools"][0]["platform"] = serde_json::Value::String("linux".to_owned());
        hostile_subjects.push(hostile);
        let mut hostile = value.clone();
        hostile["tools"][1]
            .as_object_mut()
            .expect("node subject")
            .remove("platform");
        hostile_subjects.push(hostile);
        let mut hostile = value.clone();
        hostile["tools"][1]["architecture"] = serde_json::Value::String("arm64".to_owned());
        hostile_subjects.push(hostile);
        let mut hostile = value.clone();
        hostile["tools"][1]["file_count"] = serde_json::Value::from(1);
        hostile_subjects.push(hostile);
        let mut hostile = value.clone();
        hostile["tools"][2]["size"] = serde_json::Value::from(MAX_TOOL_TREE_BYTES + 1);
        hostile_subjects.push(hostile);
        let mut hostile = value.clone();
        hostile["tools"][2]["file_count"] = serde_json::Value::from(4_097);
        hostile_subjects.push(hostile);
        let mut hostile = value.clone();
        hostile["tools"][2]
            .as_object_mut()
            .expect("npm subject")
            .remove("file_count");
        hostile_subjects.push(hostile);
        let mut hostile = value.clone();
        hostile["tools"][2]["architecture"] = serde_json::Value::String("x64".to_owned());
        hostile_subjects.push(hostile);
        for (index, key) in [
            (0, "platform"),
            (0, "architecture"),
            (0, "file_count"),
            (1, "file_count"),
            (2, "platform"),
            (2, "architecture"),
            (1, "platform"),
            (1, "architecture"),
            (2, "file_count"),
        ] {
            let mut hostile = value.clone();
            hostile["tools"][index][key] = serde_json::Value::Null;
            hostile_subjects.push(hostile);
        }
        for hostile in hostile_subjects {
            assert!(
                decode_manifest(&manifest_bytes(&hostile)).is_err(),
                "an invalid tool subject was admitted"
            );
        }

        let mut hostile_inventories = Vec::new();
        let mut hostile = value.clone();
        hostile["files"] = serde_json::json!([
            file("index.html", "text/html; charset=utf-8"),
            file("index.html", "text/html; charset=utf-8")
        ]);
        hostile_inventories.push(hostile);
        let mut hostile = value.clone();
        hostile["files"] = serde_json::json!([
            file("assets/App.js", "text/javascript; charset=utf-8"),
            file("assets/app.js", "text/javascript; charset=utf-8"),
            file("index.html", "text/html; charset=utf-8")
        ]);
        hostile_inventories.push(hostile);
        let mut hostile = value.clone();
        hostile["files"] =
            serde_json::json!([file("assets/app.js", "text/javascript; charset=utf-8")]);
        hostile_inventories.push(hostile);
        let mut hostile = value.clone();
        hostile["files"] = serde_json::json!([
            file("assets/nested/app.js", "text/javascript; charset=utf-8"),
            file("index.html", "text/html; charset=utf-8")
        ]);
        hostile_inventories.push(hostile);
        for path in ["assets/bad:name.js", "assets/bad.", "assets/.GIT"] {
            let mut hostile = value.clone();
            hostile["files"] = serde_json::json!([
                file(path, "text/javascript; charset=utf-8"),
                file("index.html", "text/html; charset=utf-8")
            ]);
            hostile_inventories.push(hostile);
        }
        let mut hostile = value.clone();
        hostile["files"] = serde_json::json!([
            file("index.html", "text/html; charset=utf-8"),
            file("assets/app.js", "text/javascript; charset=utf-8")
        ]);
        hostile_inventories.push(hostile);
        let mut hostile = value.clone();
        hostile["files"] = serde_json::json!([
            file("assets/e\u{301}.js", "text/javascript; charset=utf-8"),
            file("index.html", "text/html; charset=utf-8")
        ]);
        hostile_inventories.push(hostile);
        let mut hostile = value.clone();
        hostile["files"][0]["mime"] = serde_json::Value::String("text/plain".to_owned());
        hostile_inventories.push(hostile);
        let mut hostile = value.clone();
        hostile["files"][0]["size"] = serde_json::Value::from(MAX_FILE_BYTES + 1);
        hostile["total_size"] = serde_json::Value::from(MAX_FILE_BYTES + 1);
        hostile_inventories.push(hostile);
        let mut hostile = value.clone();
        hostile["total_size"] = serde_json::Value::from(1);
        hostile_inventories.push(hostile);
        for hostile in hostile_inventories {
            assert!(
                decode_manifest(&manifest_bytes(&hostile)).is_err(),
                "an invalid Portal file inventory was admitted"
            );
        }

        for (pointer, hostile_value) in [
            ("/source/repository", serde_json::json!("other")),
            ("/source/commit_oid", serde_json::json!("deadbeef")),
            ("/package_lock/path", serde_json::json!("other-lock.json")),
            (
                "/package_lock/size",
                serde_json::json!(MAX_PACKAGE_LOCK_BYTES + 1),
            ),
        ] {
            let mut hostile = value.clone();
            *hostile.pointer_mut(pointer).expect("projection pointer") = hostile_value;
            assert!(
                decode_manifest(&manifest_bytes(&hostile)).is_err(),
                "an invalid release projection at {pointer} was admitted"
            );
        }

        let dist = tempfile::tempdir().expect("Portal dist");
        let oversized_manifest = dist.path().join(MANIFEST_NAME);
        std::fs::File::create(&oversized_manifest)
            .expect("oversized manifest")
            .set_len(MAX_MANIFEST_BYTES + 1)
            .expect("size oversized manifest");
        assert!(read_manifest(dist.path()).is_err());
        let oversized_lock = dist.path().join("package-lock.json");
        std::fs::File::create(&oversized_lock)
            .expect("oversized lock")
            .set_len(MAX_PACKAGE_LOCK_BYTES + 1)
            .expect("size oversized lock");
        assert!(
            read_regular_bounded(&oversized_lock, MAX_PACKAGE_LOCK_BYTES, "package-lock.json")
                .is_err()
        );

        #[cfg(unix)]
        {
            use std::{io::Write, os::unix::fs::symlink};

            let race = tempfile::tempdir().expect("race directory");
            let target = race.path().join("target");
            std::fs::write(&target, b"target").expect("race target");
            let swapped = race.path().join("swapped");
            std::fs::write(&swapped, b"original").expect("swap source");
            assert!(
                read_regular_bounded_with_hooks(
                    &swapped,
                    64,
                    "swapped input",
                    || {
                        std::fs::remove_file(&swapped).expect("remove swap source");
                        symlink(&target, &swapped).expect("replace source with symlink");
                    },
                    || {},
                )
                .is_err(),
                "a symlink swapped between lstat and open was admitted"
            );

            let mutated = race.path().join("mutated");
            std::fs::write(&mutated, b"before").expect("mutation source");
            assert!(
                read_regular_bounded_with_hooks(
                    &mutated,
                    64,
                    "mutated input",
                    || {},
                    || {
                        std::fs::OpenOptions::new()
                            .append(true)
                            .open(&mutated)
                            .expect("open mutation source")
                            .write_all(b"after")
                            .expect("mutate opened source");
                    },
                )
                .is_err(),
                "a file mutated after descriptor admission was accepted"
            );

            let replaced = race.path().join("replaced");
            let displaced = race.path().join("displaced");
            std::fs::write(&replaced, b"subject").expect("replacement source");
            assert!(
                read_regular_bounded_with_hooks(
                    &replaced,
                    64,
                    "replaced input",
                    || {},
                    || {
                        std::fs::rename(&replaced, &displaced).expect("displace opened path");
                        std::fs::write(&replaced, b"subject").expect("replace opened path");
                    },
                )
                .is_err(),
                "a pathname replaced after descriptor admission was accepted"
            );
        }
    }
