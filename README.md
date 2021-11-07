# Solidity compiler for zkEVM

The compiler from Solidity to zkEVM bytecode.

## Warning 

**This project cannot be built. It is read-only and published only to demonstrate the workflow of LLVM IR generation.**

## Usage

```
zksolc ERC20.sol --asm --bin --optimize --output-dir './build/'
```

The latest patch of the `solc 0.8.x` Solidity compiler must be available in `PATH`.

## Tested

Solidity compiler versions:
```
0.8.0
0.8.1
0.8.2
0.8.3
0.8.4
0.8.5
0.8.6
0.8.7
0.8.8
0.8.9
```
