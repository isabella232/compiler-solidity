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

# The LLVM rebuild if it has been updated
if [[ -n ${LLVM_HOME} ]]; then
  LLVM_LOCK_FILE='./LLVM.lock'
  LLVM_COMMIT_CURRENT=$(git --git-dir "${LLVM_HOME}/.git/" rev-parse HEAD)
  LLVM_COMMIT_LAST=$(cat "${LLVM_LOCK_FILE}" 2>'/dev/null' || echo '0000000000000000000000000000000000000000')

  if [[ "${LLVM_COMMIT_CURRENT}" != "${LLVM_COMMIT_LAST}" ]]; then
      (cd "${LLVM_HOME}" && ./build.sh ${BUILD_TYPE})
      rm -rfv ./target/${BUILD_TYPE}/build/llvm-sys-*
      echo -n "${LLVM_COMMIT_CURRENT}" > "${LLVM_LOCK_FILE}"
  fi
fi

# Prevents the stack overflow when running some unit tests
export RUST_MIN_STACK=$(( 64 * 1024 * 1024 ))

cargo fmt --all
cargo clippy
cargo build ${CARGO_LOG_LEVEL} ${RELEASE_FLAG}
