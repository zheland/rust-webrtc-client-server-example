#!/bin/bash

set -ex

which cargo &> /dev/null || curl https://sh.rustup.rs -Sf | sh -s -- -y
rustup target add wasm32-unknown-unknown
cargo install trunk wasm-bindgen-cli
