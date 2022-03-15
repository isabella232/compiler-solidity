# Solidity compiler for zkEVM

The compiler from Solidity to zkEVM bytecode.

## Building (only for developers)

1. Get the access to the private [LLVM repository](https://github.com/matter-labs/compiler-llvm).
2. Remove all the existing LLVM artifacts from your system.
3. Build the `main` branch of LLVM by running `./build.sh release` at its root.
4. Perform a clean build of this repository: `cargo clean && ./run.sh`.

## Usage

```
zksolc ERC20.sol --asm --bin --optimize --output-dir './build/'
```

The latest patch of the **solc v0.8** must be available through `PATH`.

**Do not use the former patches of *solc*, as each version introduces important bug fixes!**
