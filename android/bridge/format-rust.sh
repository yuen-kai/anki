#!/bin/bash

set -e

(cd ../../cargo/format && cargo fmt --all --manifest-path ../../../Cargo.toml)
