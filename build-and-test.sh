#!/bin/sh -e

cargo test
cargo fmt --all -- --check
cargo clippy --all
