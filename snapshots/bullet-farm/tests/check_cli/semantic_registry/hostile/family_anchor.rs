use super::*;
use crate::semantic_registry::fixture::{fixture_family_lock_path, rewrite_family_anchor_policy};

pub(super) fn assert_refusals(registry: &Path) {
    write_structural_registry(registry);
    fs::write(fixture_family_lock_path(), "schema_version = \"2\"\n").unwrap();
    assert_registry_rejected_with(
        registry,
        "provider-codex",
        Some("not an installable schema-3 lock"),
    );

    write_structural_registry(registry);
    mutate_registry_manifest(registry, |manifest| {
        manifest["family_lock_digest"] = format!(
            "blake3:{}",
            std::iter::repeat_n('f', 64).collect::<String>()
        )
        .into();
    });
    assert_registry_rejected_with(
        registry,
        "provider-codex",
        Some("differs from the exact admitted family.lock bytes"),
    );

    write_structural_registry(registry);
    rewrite_family_anchor_policy(
        registry,
        &format!(
            "blake3:{}",
            std::iter::repeat_n('e', 64).collect::<String>()
        ),
    );
    assert_registry_rejected_with(
        registry,
        "provider-codex",
        Some("differs from the policy locked by family.lock"),
    );

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        write_structural_registry(registry);
        let lock = fixture_family_lock_path();
        let real = lock.with_extension("real");
        fs::rename(&lock, &real).unwrap();
        symlink(&real, &lock).unwrap();
        assert_registry_rejected_with(
            registry,
            "provider-codex",
            Some("family.lock admission failed"),
        );
    }

    write_structural_registry(registry);
    fs::write(fixture_family_lock_path(), vec![b'x'; 1024 * 1024 + 1]).unwrap();
    assert_registry_rejected_with(
        registry,
        "provider-codex",
        Some("exceeds the 1 MiB admission limit"),
    );
}
