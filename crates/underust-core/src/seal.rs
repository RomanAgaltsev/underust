//! The seal: honest obfuscation, not secrecy.
//!
//! A reference solution is gzip-compressed and base64-encoded so it cannot be read by
//! scrolling past it. Anyone who wants the answer can have it, and the design says so
//! out loud -- the gate that matters is [`crate::progress`], not this encoding.

use std::collections::BTreeMap;
use std::io::Write as _;

use base64::Engine as _;
use flate2::{Compression, GzBuilder, read::GzDecoder};
use serde::{Deserialize, Serialize};

/// A reference solution: an explanation plus the files it overlays onto the task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sealed {
    /// Prose shown by `reveal`, explaining the mechanism.
    pub explanation: String,
    /// Paths relative to the task directory, mapped to their full contents.
    /// A `BTreeMap` rather than a `HashMap` because the seal must be byte-stable.
    pub files: BTreeMap<String, String>,
}

/// Everything that can go wrong sealing or unsealing.
#[derive(Debug, thiserror::Error)]
pub enum SealError {
    /// The payload could not be encoded or decoded as JSON.
    #[error("seal payload: {0}")]
    Payload(#[from] serde_json::Error),
    /// Compression or decompression failed.
    #[error("seal codec: {0}")]
    Codec(#[from] std::io::Error),
    /// The blob was not valid base64.
    #[error("seal encoding: {0}")]
    Encoding(#[from] base64::DecodeError),
}

/// Compress and encode a reference solution.
///
/// The output is byte-stable: the gzip header's mtime is zeroed and its OS byte is set
/// to 255 ("unknown"), so the same input yields the same blob on every platform.
///
/// # Errors
/// Propagates serialisation and compression failures.
pub fn seal(sealed: &Sealed) -> Result<String, SealError> {
    let json = serde_json::to_vec(sealed)?;
    let mut encoder = GzBuilder::new()
        .mtime(0)
        .operating_system(255)
        .write(Vec::new(), Compression::new(9));
    encoder.write_all(&json)?;
    let gz = encoder.finish()?;
    Ok(base64::engine::general_purpose::STANDARD.encode(gz))
}

/// Decode and decompress a sealed blob.
///
/// # Errors
/// Propagates base64, decompression and deserialisation failures.
pub fn unseal(blob: &str) -> Result<Sealed, SealError> {
    let compact: String = blob.split_whitespace().collect();
    let gz = base64::engine::general_purpose::STANDARD.decode(compact)?;
    let mut json = String::new();
    std::io::Read::read_to_string(&mut GzDecoder::new(&gz[..]), &mut json)?;
    Ok(serde_json::from_str(&json)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Sealed {
        let mut files = std::collections::BTreeMap::new();
        files.insert(
            "src/lib.rs".to_owned(),
            "pub fn answer() -> u8 { 42 }\n".to_owned(),
        );
        Sealed {
            explanation: "Fields drop in declaration order.\n".to_owned(),
            files,
        }
    }

    /// Regenerates the committed golden file. Run deliberately:
    /// `cargo test -p underust-core seal::tests::write_golden -- --ignored`
    #[test]
    #[ignore = "authoring tool: rewrites the committed golden bytes"]
    fn write_golden() {
        let blob = seal(&fixture()).expect("seal");
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/fixture.seal");
        std::fs::write(path, &blob).expect("write golden");
        println!("wrote {} bytes to {path}", blob.len());
    }

    #[test]
    fn round_trips() {
        let blob = seal(&fixture()).expect("seal");
        let back = unseal(&blob).expect("unseal");
        assert_eq!(back, fixture());
    }

    #[test]
    fn is_byte_stable_across_runs() {
        let a = seal(&fixture()).expect("seal");
        let b = seal(&fixture()).expect("seal");
        assert_eq!(a, b, "the seal must not embed a timestamp or an OS byte");
    }

    #[test]
    fn matches_the_committed_golden_bytes() {
        // Pinned so the three-OS CI matrix proves determinism across platforms,
        // not merely across two calls in one process.
        const GOLDEN: &str = include_str!("../tests/golden/fixture.seal");
        assert_eq!(seal(&fixture()).expect("seal"), GOLDEN.trim_end());
    }

    #[test]
    fn plaintext_does_not_appear_in_the_blob() {
        let blob = seal(&fixture()).expect("seal");
        assert!(
            !blob.contains("answer"),
            "the seal must not be readable at a glance"
        );
        assert!(!blob.contains("declaration order"));
    }

    #[test]
    fn rejects_a_corrupt_blob() {
        assert!(unseal("not base64 at all !!!").is_err());
    }
}
