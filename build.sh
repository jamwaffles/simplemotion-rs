#!/bin/bash

set -xe

cargo clean --doc

cargo fmt --all -- --check
cargo build
cargo build --examples
cargo test --release
cargo test --release --all-features
cargo bench --no-run

cargo +nightly doc --all-features
linkchecker target/doc/simplemotion_sys/index.html
linkchecker target/doc/simplemotion/index.html

# Check that sys package builds into something pushable to crates.io
pushd simplemotion-sys
cargo package
popd

# Check that higher level package builds correctly
pushd simplemotion
cargo package
popd
