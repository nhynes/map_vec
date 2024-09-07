#!/bin/sh

# The checks with the `aarch64-unknown-none` target ensure that the library can 
# target a `no_std` environment.

set -x

# Lint
cargo clippy -- -Dwarnings && \
cargo clippy --no-default-features -- -Dwarnings && \
cargo clippy --no-default-features --features serde -- -Dwarnings && \
cargo +nightly clippy --all-features && \

cargo clippy --tests -- -Dwarnings && \
cargo clippy --tests --no-default-features -- -Dwarnings && \
cargo clippy --tests --no-default-features --features serde -- -Dwarnings && \
cargo +nightly clippy --tests --all-features && \

# Check against a target that does _not_ support `std` to ensure it doesn't 
# creep in from a dependency or anything.
cargo clippy --target aarch64-unknown-none -- -Dwarnings && \
cargo clippy --target aarch64-unknown-none --no-default-features -- -Dwarnings && \
cargo clippy --target aarch64-unknown-none --no-default-features --features serde -- -Dwarnings && \
cargo +nightly clippy --target aarch64-unknown-none --features serde && \

# Tests
cargo test && \
cargo test --no-default-features && \
cargo test --no-default-features --features serde && \
cargo +nightly test --all-features && \

# Documentation
cargo +nightly doc --all-features && \

# Publishable?
cargo publish --dry-run -v --allow-dirty

set +x
