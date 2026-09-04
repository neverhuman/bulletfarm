//! Replay checked-in corpus seeds against exact, filename-bound outcomes.

#[path = "replay/custody.rs"]
mod custody;

use bullet_wire::WireError;
use bullet_wire_fuzz::fuzz_canonical;
use std::path::Path;
use std::process::ExitCode;

use custody::{CorpusDir, ReadStage};

const MAX_SEED_BYTES: u64 = 1_048_576;

#[derive(Clone, Copy)]
enum Expected {
    Admit,
    Refuse(&'static str),
}

#[derive(Clone, Copy)]
struct Case {
    name: &'static str,
    expected: Expected,
}

const CASES: &[Case] = &[
    Case {
        name: "admit_object",
        expected: Expected::Admit,
    },
    Case {
        name: "admit_true",
        expected: Expected::Admit,
    },
    Case {
        name: "bom",
        expected: Expected::Refuse("UTF8_BOM_FORBIDDEN"),
    },
    Case {
        name: "duplicate",
        expected: Expected::Refuse("DUPLICATE_JSON_KEY"),
    },
    Case {
        name: "empty",
        expected: Expected::Refuse("EMPTY_DOCUMENT"),
    },
    Case {
        name: "whitespace",
        expected: Expected::Refuse("NON_CANONICAL_JSON"),
    },
];

fn main() -> ExitCode {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("corpus/canonical");
    match replay_corpus(&root, CASES, fuzz_canonical) {
        Ok(seeds) => {
            println!("fuzz-replay: {seeds} exact outcomes closed");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("fuzz-replay: {error}");
            ExitCode::FAILURE
        }
    }
}

fn replay_corpus(
    dir: &Path,
    cases: &[Case],
    runner: impl Fn(&[u8]) -> Result<(), WireError>,
) -> Result<usize, String> {
    replay_corpus_with_hook(dir, cases, runner, |_, _| {})
}

fn replay_corpus_with_hook(
    dir: &Path,
    cases: &[Case],
    runner: impl Fn(&[u8]) -> Result<(), WireError>,
    mut hook: impl FnMut(&Path, ReadStage),
) -> Result<usize, String> {
    let corpus = CorpusDir::open(dir)?;
    let actual = corpus.inventory()?;
    let mut expected = cases
        .iter()
        .map(|case| case.name.to_owned())
        .collect::<Vec<_>>();
    expected.sort();
    if actual != expected {
        return Err(format!(
            "{} inventory mismatch: expected {expected:?}, found {actual:?}",
            dir.display()
        ));
    }

    for case in cases {
        let bytes = corpus.read_seed(case.name, MAX_SEED_BYTES, |path, stage| hook(path, stage))?;
        verify(case, runner(&bytes))?;
    }
    corpus.revalidate()?;
    Ok(cases.len())
}

fn verify(case: &Case, outcome: Result<(), WireError>) -> Result<(), String> {
    match (case.expected, outcome) {
        (Expected::Admit, Ok(())) => {
            println!("fuzz-replay: admit {}", case.name);
            Ok(())
        }
        (Expected::Admit, Err(error)) => Err(format!(
            "{} expected admission, got {}: {error}",
            case.name,
            error.code()
        )),
        (Expected::Refuse(expected), Err(error)) if error.code() == expected => {
            println!("fuzz-replay: refuse {} {expected}", case.name);
            Ok(())
        }
        (Expected::Refuse(expected), Err(error)) => Err(format!(
            "{} expected {expected}, got {}: {error}",
            case.name,
            error.code()
        )),
        (Expected::Refuse(expected), Ok(())) => Err(format!(
            "{} expected {expected}, but the oracle admitted it",
            case.name
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::cell::Cell;
    #[cfg(unix)]
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::symlink;

    #[cfg(unix)]
    #[test]
    fn corpus_directory_symlink_is_refused_before_enumeration() {
        let root = tempfile::tempdir().expect("temporary root");
        let real = root.path().join("real");
        fs::create_dir(&real).expect("real corpus directory");
        let linked = root.path().join("linked");
        symlink(&real, &linked).expect("corpus directory symlink");

        let error = replay_corpus(&linked, &[], |_| Ok(())).expect_err("symlink refused");
        assert!(error.contains("not a non-symlink corpus directory"));
    }

    #[cfg(unix)]
    #[test]
    fn unexpected_inventory_is_refused_before_an_oracle_runs() {
        let root = tempfile::tempdir().expect("temporary corpus");
        fs::write(root.path().join("unexpected"), b"seed").expect("unexpected seed");

        let error = replay_corpus(root.path(), &[], |_| {
            panic!("inventory mismatch reached oracle")
        })
        .expect_err("unexpected inventory refused");
        assert!(error.contains("inventory mismatch"));
    }

    #[cfg(unix)]
    #[test]
    fn over_bound_seed_is_refused_before_an_oracle_runs() {
        let root = tempfile::tempdir().expect("temporary corpus");
        let path = root.path().join("large");
        let file = fs::File::create(&path).expect("large seed");
        file.set_len(MAX_SEED_BYTES + 1).expect("sparse large seed");
        let cases = [Case {
            name: "large",
            expected: Expected::Admit,
        }];

        let error = replay_corpus(root.path(), &cases, |_| {
            panic!("over-bound seed reached oracle")
        })
        .expect_err("over-bound seed refused");
        assert!(error.contains("replay bound"));
    }

    #[cfg(unix)]
    #[test]
    fn seed_symlink_is_refused_before_an_oracle_runs() {
        let root = tempfile::tempdir().expect("temporary corpus");
        let outside_root = tempfile::tempdir().expect("outside root");
        let outside = outside_root.path().join("outside");
        fs::write(&outside, b"outside seed").expect("outside seed");
        symlink(&outside, root.path().join("seed")).expect("seed symlink");
        let cases = [Case {
            name: "seed",
            expected: Expected::Admit,
        }];

        let error = replay_corpus(root.path(), &cases, |_| {
            panic!("symlinked seed reached oracle")
        })
        .expect_err("seed symlink refused");
        assert!(error.contains("not a regular single-link corpus file"));
    }

    #[cfg(unix)]
    #[test]
    fn inspected_seed_cannot_be_exchanged_for_a_symlink() {
        let root = tempfile::tempdir().expect("temporary corpus");
        let outside_root = tempfile::tempdir().expect("outside root");
        let seed = root.path().join("seed");
        let retained = root.path().join("retained");
        let outside = outside_root.path().join("outside");
        fs::write(&seed, b"original").expect("seed");
        fs::write(&outside, b"outside").expect("outside seed");
        let cases = [Case {
            name: "seed",
            expected: Expected::Admit,
        }];
        let swapped = Cell::new(false);

        let error = replay_corpus_with_hook(
            root.path(),
            &cases,
            |_| panic!("substituted symlink reached oracle"),
            |path, stage| {
                if stage == ReadStage::BeforeOpen && !swapped.replace(true) {
                    fs::rename(path, &retained).expect("retain inspected seed");
                    symlink(&outside, path).expect("substitute seed symlink");
                }
            },
        )
        .expect_err("symlink substitution refused");
        assert!(error.contains("cannot open"));
        assert_eq!(fs::read(&retained).expect("retained seed"), b"original");
        assert_eq!(fs::read(&outside).expect("outside seed"), b"outside");
    }

    #[cfg(unix)]
    #[test]
    fn inspected_seed_cannot_be_exchanged_for_another_regular_file() {
        let root = tempfile::tempdir().expect("temporary corpus");
        let seed = root.path().join("seed");
        let retained = root.path().join("retained");
        fs::write(&seed, b"original").expect("seed");
        let cases = [Case {
            name: "seed",
            expected: Expected::Admit,
        }];
        let swapped = Cell::new(false);

        let error = replay_corpus_with_hook(
            root.path(),
            &cases,
            |_| panic!("substituted regular file reached oracle"),
            |path, stage| {
                if stage == ReadStage::BeforeOpen && !swapped.replace(true) {
                    fs::rename(path, &retained).expect("retain inspected seed");
                    fs::write(path, b"replacement").expect("substitute regular seed");
                }
            },
        )
        .expect_err("regular substitution refused");
        assert!(error.contains("identity changed"));
        assert_eq!(fs::read(&retained).expect("retained seed"), b"original");
        assert_eq!(fs::read(&seed).expect("replacement seed"), b"replacement");
    }

    #[cfg(unix)]
    #[test]
    fn opened_seed_path_substitution_is_refused_and_both_files_are_retained() {
        let root = tempfile::tempdir().expect("temporary corpus");
        let seed = root.path().join("seed");
        let retained = root.path().join("retained");
        fs::write(&seed, b"original").expect("seed");
        let cases = [Case {
            name: "seed",
            expected: Expected::Admit,
        }];
        let swapped = Cell::new(false);

        let error = replay_corpus_with_hook(
            root.path(),
            &cases,
            |_| Ok(()),
            |path, stage| {
                if stage == ReadStage::AfterOpen && !swapped.replace(true) {
                    fs::rename(path, &retained).expect("retain opened seed");
                    fs::write(path, b"replacement").expect("replacement seed");
                }
            },
        )
        .expect_err("path substitution refused");
        assert!(error.contains("identity changed"));
        assert_eq!(fs::read(&retained).expect("retained seed"), b"original");
        assert_eq!(fs::read(&seed).expect("replacement seed"), b"replacement");
    }

    #[cfg(unix)]
    #[test]
    fn seed_growth_after_the_first_read_is_bounded_and_refused() {
        let root = tempfile::tempdir().expect("temporary corpus");
        let seed = root.path().join("seed");
        fs::write(&seed, b"initial").expect("seed");
        let cases = [Case {
            name: "seed",
            expected: Expected::Admit,
        }];
        let grew = Cell::new(false);

        let error = replay_corpus_with_hook(
            root.path(),
            &cases,
            |_| panic!("grown seed reached oracle"),
            |path, stage| {
                if stage == ReadStage::AfterFirstRead && !grew.replace(true) {
                    let file = fs::OpenOptions::new()
                        .write(true)
                        .open(path)
                        .expect("open seed for growth");
                    file.set_len(MAX_SEED_BYTES + 1).expect("grow seed");
                }
            },
        )
        .expect_err("growth refused");
        assert!(error.contains("replay bound"));
        assert!(grew.get(), "growth hook did not run");
    }

    #[cfg(not(unix))]
    #[test]
    fn replay_fails_closed_without_descriptor_custody() {
        let error = replay_corpus(Path::new("corpus"), &[], |_| Ok(()))
            .expect_err("unsupported platform refused");
        assert!(error.contains("corpus custody is unavailable"));
    }

    #[test]
    fn admission_and_reason_mismatches_fail_closed() {
        let admission = Case {
            name: "admit",
            expected: Expected::Admit,
        };
        let refusal = Case {
            name: "refuse",
            expected: Expected::Refuse("NON_CANONICAL_JSON"),
        };

        assert!(verify(&refusal, Ok(())).is_err());
        assert!(
            verify(
                &admission,
                Err(WireError::new("NON_CANONICAL_JSON", "fixture"))
            )
            .is_err()
        );
        assert!(verify(&refusal, Err(WireError::new("EMPTY_DOCUMENT", "fixture"))).is_err());
    }
}
