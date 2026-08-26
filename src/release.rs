//! Verification and safe extraction components for signed release bundles.

mod archive;
mod build;
mod receipt;
mod schema;
mod signature;
mod verify;

use std::path::PathBuf;

use crate::coord::CoordError;

const BUILD_USAGE: &str = build::BUILD_USAGE;
const VERIFY_USAGE: &str =
    "usage: bullet-family release verify --bundle ABSOLUTE_PATH --allowed-signers ABSOLUTE_PATH";
const EXTRACT_USAGE: &str = "usage: bullet-family release extract --bundle ABSOLUTE_PATH --allowed-signers ABSOLUTE_PATH --target TARGET --destination ABSOLUTE_PATH";
const RECEIPT_VERIFY_USAGE: &str = "usage: bullet-family release receipt-verify --receipt ABSOLUTE_PATH --signature ABSOLUTE_PATH --policy ABSOLUTE_PATH";

pub(crate) use receipt::verify_detached;
pub use receipt::{
    RELEASE_RECEIPT_POLICY_SCHEMA_VERSION, RELEASE_RECEIPT_SCHEMA_VERSION, ReleaseReceipt,
    ReleaseReceiptKind, ReleaseReceiptPolicy, ReleaseReceiptResult, ReleaseReceiptSigner,
};
pub use schema::{
    RELEASE_MANIFEST_SCHEMA_VERSION, ReleaseFile, ReleaseManifest, ReleasePackage,
    SignedReleaseFile,
};

pub fn run(args: &[String]) -> Result<String, CoordError> {
    if args.first().is_some_and(|action| action == "build") {
        return Err(release_build_containment_unavailable());
    }
    match parse_args(args)? {
        Command::Build(options) => {
            let _preserved_builder: fn(&build::BuildArgs) -> Result<String, CoordError> =
                build::run;
            let _ = options;
            Err(release_build_containment_unavailable())
        }
        Command::Verify(options) => {
            let receipt = verify::verify(&options.bundle, &options.allowed_signers)?;
            Ok(format!(
                "release bundle verified (read-only component): {} packages at {} by {}",
                receipt.manifest.package.len(),
                receipt.manifest.tag,
                receipt.manifest.release_signing_identity
            ))
        }
        Command::Extract(options) => {
            let _receipt = verify::verify(&options.bundle, &options.allowed_signers)?;
            Err(CoordError::new(
                "RELEASE_PUBLICATION_CONTAINMENT_UNAVAILABLE",
                format!(
                    "verified archive {} cannot be published at {} without a different-UID or privileged containment backend",
                    options.target,
                    options.destination.display()
                ),
            ))
        }
        Command::VerifyReceipt(options) => {
            let verified = receipt::verify(&options.receipt, &options.signature, &options.policy)?;
            Ok(format!(
                "release receipt verified (contract only; no release gate cleared): {} {} at {} under {}",
                verified.receipt.receipt_kind.as_str(),
                verified.receipt.result.as_str(),
                verified.receipt.tag,
                verified.policy_digest
            ))
        }
    }
}

fn release_build_containment_unavailable() -> CoordError {
    CoordError::new(
        "RELEASE_BUILD_CONTAINMENT_UNAVAILABLE",
        "release builds require private exact-OID reconstruction, a sealed toolchain, and a \
         different-identity build broker; the public command refuses before validation or mutation",
    )
}

#[derive(Debug)]
enum Command {
    Build(build::BuildArgs),
    Verify(CommonArgs),
    Extract(ExtractArgs),
    VerifyReceipt(ReceiptArgs),
}

#[derive(Debug)]
struct CommonArgs {
    bundle: PathBuf,
    allowed_signers: PathBuf,
}

#[derive(Debug)]
struct ExtractArgs {
    bundle: PathBuf,
    allowed_signers: PathBuf,
    target: String,
    destination: PathBuf,
}

#[derive(Debug)]
struct ReceiptArgs {
    receipt: PathBuf,
    signature: PathBuf,
    policy: PathBuf,
}

fn parse_args(args: &[String]) -> Result<Command, CoordError> {
    let action = args
        .first()
        .ok_or_else(|| CoordError::new("USAGE", VERIFY_USAGE))?;
    if action == "build" {
        return parse_build(&args[1..]).map(Command::Build);
    }
    let usage = match action.as_str() {
        "verify" => VERIFY_USAGE,
        "extract" => EXTRACT_USAGE,
        "receipt-verify" => RECEIPT_VERIFY_USAGE,
        _ => return Err(CoordError::new("USAGE", VERIFY_USAGE)),
    };
    let mut bundle = None;
    let mut allowed_signers = None;
    let mut target = None;
    let mut destination = None;
    let mut receipt = None;
    let mut signature = None;
    let mut policy = None;
    let mut index = 1;
    while index < args.len() {
        let value = args
            .get(index + 1)
            .ok_or_else(|| CoordError::new("USAGE", usage))?;
        let duplicate = match args[index].as_str() {
            "--bundle" => bundle.replace(PathBuf::from(value)).is_some(),
            "--allowed-signers" => allowed_signers.replace(PathBuf::from(value)).is_some(),
            "--target" if action == "extract" => target.replace(value.clone()).is_some(),
            "--destination" if action == "extract" => {
                destination.replace(PathBuf::from(value)).is_some()
            }
            "--receipt" if action == "receipt-verify" => {
                receipt.replace(PathBuf::from(value)).is_some()
            }
            "--signature" if action == "receipt-verify" => {
                signature.replace(PathBuf::from(value)).is_some()
            }
            "--policy" if action == "receipt-verify" => {
                policy.replace(PathBuf::from(value)).is_some()
            }
            _ => return Err(CoordError::new("USAGE", usage)),
        };
        if duplicate {
            return Err(CoordError::new("DUPLICATE_OPTION", usage));
        }
        index += 2;
    }
    match action.as_str() {
        "verify" => Ok(Command::Verify(CommonArgs {
            bundle: bundle.ok_or_else(|| CoordError::new("USAGE", usage))?,
            allowed_signers: allowed_signers.ok_or_else(|| CoordError::new("USAGE", usage))?,
        })),
        "extract" => Ok(Command::Extract(ExtractArgs {
            bundle: bundle.ok_or_else(|| CoordError::new("USAGE", usage))?,
            allowed_signers: allowed_signers.ok_or_else(|| CoordError::new("USAGE", usage))?,
            target: target.ok_or_else(|| CoordError::new("USAGE", usage))?,
            destination: destination.ok_or_else(|| CoordError::new("USAGE", usage))?,
        })),
        "receipt-verify" => Ok(Command::VerifyReceipt(ReceiptArgs {
            receipt: receipt.ok_or_else(|| CoordError::new("USAGE", usage))?,
            signature: signature.ok_or_else(|| CoordError::new("USAGE", usage))?,
            policy: policy.ok_or_else(|| CoordError::new("USAGE", usage))?,
        })),
        _ => unreachable!("action was admitted above"),
    }
}

fn parse_build(args: &[String]) -> Result<build::BuildArgs, CoordError> {
    let mut target = None;
    let mut out = None;
    let mut family_root = None;
    let mut cache_dir = None;
    let mut offline = false;
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--offline" {
            if offline {
                return Err(CoordError::new("DUPLICATE_OPTION", BUILD_USAGE));
            }
            offline = true;
            index += 1;
            continue;
        }
        let value = args
            .get(index + 1)
            .ok_or_else(|| CoordError::new("USAGE", BUILD_USAGE))?;
        let duplicate = match args[index].as_str() {
            "--target" => target.replace(value.clone()).is_some(),
            "--out" => out.replace(PathBuf::from(value)).is_some(),
            "--family-root" => family_root.replace(PathBuf::from(value)).is_some(),
            "--cache-dir" => cache_dir.replace(PathBuf::from(value)).is_some(),
            _ => return Err(CoordError::new("USAGE", BUILD_USAGE)),
        };
        if duplicate {
            return Err(CoordError::new("DUPLICATE_OPTION", BUILD_USAGE));
        }
        index += 2;
    }
    Ok(build::BuildArgs {
        target: target.ok_or_else(|| CoordError::new("USAGE", BUILD_USAGE))?,
        out: out.ok_or_else(|| CoordError::new("USAGE", BUILD_USAGE))?,
        family_root,
        cache_dir,
        offline,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments_are_exact() {
        for args in [
            vec![],
            vec!["verify".into()],
            vec!["install".into()],
            vec!["verify".into(), "--unknown".into(), "x".into()],
            vec!["extract".into(), "--bundle".into(), "x".into()],
            vec!["receipt-verify".into()],
            vec!["build".into()],
            vec!["build".into(), "--target".into(), "x".into()],
            vec![
                "build".into(),
                "--target".into(),
                "x".into(),
                "--out".into(),
                "/out".into(),
                "--unknown".into(),
                "y".into(),
            ],
            vec![
                "receipt-verify".into(),
                "--receipt".into(),
                "x".into(),
                "--signature".into(),
                "y".into(),
            ],
        ] {
            assert_eq!(parse_args(&args).unwrap_err().code(), "USAGE");
        }
        assert!(matches!(
            parse_args(&[
                "build".into(),
                "--target".into(),
                "x86_64-unknown-linux-gnu".into(),
                "--out".into(),
                "/absolute/out".into(),
                "--offline".into(),
            ])
            .expect("exact build command"),
            Command::Build(_)
        ));
        assert_eq!(
            parse_args(&[
                "build".into(),
                "--target".into(),
                "x".into(),
                "--out".into(),
                "/o".into(),
                "--offline".into(),
                "--offline".into(),
            ])
            .unwrap_err()
            .code(),
            "DUPLICATE_OPTION"
        );
        assert!(matches!(
            parse_args(&[
                "receipt-verify".into(),
                "--receipt".into(),
                "/receipt".into(),
                "--signature".into(),
                "/receipt.sig".into(),
                "--policy".into(),
                "/policy".into(),
            ])
            .expect("exact receipt command"),
            Command::VerifyReceipt(_)
        ));
    }
}
