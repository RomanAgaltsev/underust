# The canonical measurement environment: x86_64-unknown-linux-gnu, Tier 1.
#
# Used by `underust test --docker` and by the CI parity job. Keep the tag in step with
# rust-toolchain.toml and with cargo::IMAGE in crates/underust/src/cargo.rs; Renovate
# proposes bumps for the FROM line.
FROM rust:1.98.1-bookworm

# Nightly plus miri: the SOUNDNESS track's oracle. rust-src is what miri builds its
# sysroot from, and without it `cargo miri setup` fails with an error about a missing
# std manifest rather than anything mentioning miri.
RUN rustup toolchain install nightly --component miri --component rust-src \
 && rustup component add clippy rustfmt

# The miri sysroot is deliberately NOT prebuilt here. Building it at image-build time and
# using it at run time fails with "detected a concurrent sysroot build with different
# settings" -- the cached sysroot does not match what miri asks for once the repository is
# mounted. Miri builds it on first use instead, which costs about half a minute and works.

WORKDIR /w
