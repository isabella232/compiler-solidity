#!/usr/bin/env bash

set -Ce

# Logging level: 'error' | 'warn' | 'info' | 'debug' | 'trace' (default is 'warn')
case "${1}" in
    info)
        export CARGO_LOG_LEVEL="--verbose"
        ;;
    debug)
        export RUST_BACKTRACE=1
        export CARGO_LOG_LEVEL="--verbose"
        ;;
    trace)
        export RUST_BACKTRACE="full"
        export CARGO_LOG_LEVEL="--verbose"
        ;;
esac

# Target build: 'debug' | 'release' (default is 'release')
case "${2}" in
    debug)
        export LLVM_SYS_130_PREFIX="${HOME}/opt/llvm-debug/"
        ;;
    *)
        export RELEASE_FLAG="--release"
        export LLVM_SYS_130_PREFIX="${HOME}/opt/llvm-release/"
        ;;
esac

# The LLVM static binding library build cleanup
if [[ "${3}" == '--clean' ]]; then
    rm -rfv ./target/release/build/llvm-sys-*
fi

# Prevents the stack overflow when running some unit tests
export RUST_MIN_STACK=$(( 64 * 1024 * 1024 ))

cargo fmt --all
cargo clippy
cargo test
cargo build ${CARGO_LOG_LEVEL} ${RELEASE_FLAG}
