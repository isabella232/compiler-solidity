# Solidity compiler for zkEVM

The compiler from Solidity to zkEVM bytecode.

## Warning 

This project cannot be built. It is read-only and published only to demonstrate the workflow of LLVM IR generation.

## Usage

```
zksolc ERC20.sol --asm --bin --optimize --output-dir './build/'
```

The Solidity compiler **v0.8.11** must be available in `PATH`.

**Do not use the former patches of *solc*, as each version introduces important bug fixes!**
