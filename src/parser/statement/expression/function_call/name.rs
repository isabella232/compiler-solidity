//!
//! The function name.
//!

///
/// The function name.
///
#[derive(Debug, PartialEq, Clone)]
pub enum Name {
    /// The user-defined function.
    UserDefined(String),

    /// `x + y`
    Add,
    /// `x - y`
    Sub,
    /// `x * y`
    Mul,
    /// `x / y` or `0` if `y == 0`
    Div,
    /// `x % y` or `0` if `y == 0`
    Mod,
    /// bitwise "not" of `x` (every bit of `x` is negated)
    Not,
    /// `1` if `x < y`, `0` otherwise
    Lt,
    /// `1` if `x > y`, `0` otherwise
    Gt,
    /// `1` if `x == y`, `0` otherwise
    Eq,
    /// `1` if `x == 0`, `0` otherwise
    IsZero,
    /// bitwise "and" of `x` and `y`
    And,
    /// bitwise "or" of `x` and `y`
    Or,
    /// bitwise "xor" of `x` and `y`
    Xor,
    /// `(x + y) % m` with arbitrary precision arithmetic, `0` if `m == 0`
    AddMod,
    /// `(x * y) % m` with arbitrary precision arithmetic, `0` if `m == 0`
    MulMod,

    /// `x / y`, for signed numbers in two’s complement, `0` if `y == 0`
    Sdiv,
    /// `x % y`, for signed numbers in two’s complement, `0` if `y == 0`
    Smod,
    /// `x` to the power of `y`
    Exp,
    /// `1` if `x < y`, `0` otherwise, for signed numbers in two’s complement
    Slt,
    /// `1` if `x > y`, `0` otherwise, for signed numbers in two’s complement
    Sgt,
    /// `n`th byte of `x`, where the most significant byte is the `0`th byte
    Byte,
    /// logical shift left `y` by `x` bits
    Shl,
    /// logical shift right `y` by `x` bits
    Shr,
    /// signed arithmetic shift right `y` by `x` bits
    Sar,
    /// sign extend from `(i*8+7)`th bit counting from least significant
    SignExtend,
    /// `keccak(mem[p…(p+n)))`
    Keccak256,
    /// current position in code
    Pc,

    /// discard value x
    Pop,
    /// `mem[p…(p+32))`
    MLoad,
    /// `mem[p…(p+32)) := v`
    MStore,
    /// `mem[p] := v & 0xff` (only modifies a single byte)
    MStore8,

    /// `storage[p]`
    SLoad,
    /// `storage[p] := v`
    SStore,

    /// call sender (excluding `delegatecall`)
    Caller,
    /// wei sent together with the current call
    CallValue,
    /// call data starting from position `p` (32 bytes)
    CallDataLoad,
    /// size of call data in bytes
    CallDataSize,
    /// copy `s` bytes from calldata at position `f` to memory at position `t`
    CallDataCopy,

    /// size of memory, i.e. largest accessed memory index
    MSize,
    /// gas still available to execution
    Gas,
    /// address of the current contract / execution context
    Address,
    /// wei balance at address `a`
    Balance,
    /// equivalent to `balance(address())`, but cheaper
    SelfBalance,

    /// ID of the executing chain (EIP 1344)
    ChainId,
    /// transaction sender
    Origin,
    /// gas price of the transaction
    GasPrice,
    /// hash of block nr b - only for last 256 blocks excluding current
    BlockHash,
    /// current mining beneficiary
    CoinBase,
    /// timestamp of the current block in seconds since the epoch
    Timestamp,
    /// current block number
    Number,
    /// difficulty of the current block
    Difficulty,
    /// block gas limit of the current block
    GasLimit,

    /// create new contract with code `mem[p…(p+n))` and send `v` wei and return the new address
    Create,
    /// create new contract with code `mem[p…(p+n))` at address
    /// `keccak256(0xff . this . s . keccak256(mem[p…(p+n)))` and send `v` wei and return the
    /// new address, where `0xff` is a 1-byte value, this is the current contract’s address as a
    /// 20-byte value and `s` is a big-endian 256-bit value
    Create2,

    /// log without topics and data `mem[p…(p+s))`
    Log0,
    /// log with topic t1 and data `mem[p…(p+s))`
    Log1,
    /// log with topics t1, t2 and data `mem[p…(p+s))`
    Log2,
    /// log with topics t1, t2, t3 and data `mem[p…(p+s))`
    Log3,
    /// log with topics t1, t2, t3, t4 and data `mem[p…(p+s))`
    Log4,

    /// call contract at address a with input `mem[in…(in+insize))` providing `g` gas and `v` wei
    /// and output area `mem[out…(out+outsize))` returning 0 on error (e.g. out of gas)
    /// and 1 on success
    /// [See more](https://docs.soliditylang.org/en/v0.8.2/yul.html#yul-call-return-area)
    Call,
    /// identical to call but only use the code from a and stay in the context of the current
    /// contract otherwise
    CallCode,
    /// identical to `callcode` but also keeps `caller` and `callvalue`
    DelegateCall,
    /// identical to `call(g, a, 0, in, insize, out, outsize)` but do not allows state modifications
    StaticCall,
    /// `setimmutable` is called in library constructors
    SetImmutable,

    /// size of the code of the current contract / execution context
    CodeSize,
    /// copy `s` bytes from code at position `f` to mem at position `t`
    CodeCopy,
    /// size of the code at address `a`
    ExtCodeSize,
    /// like `codecopy(t, f, s)` but take code at address `a`
    ExtCodeCopy,
    /// size of the last returndata
    ReturnDataSize,
    /// copy `s` bytes from returndata at position `f` to mem at position `t`
    ReturnDataCopy,
    /// code hash of address `a`
    ExtCodeHash,

    /// returns the size in the data area
    DataSize,
    /// returns the offset in the data area
    DataOffset,
    ///  is equivalent to `CodeCopy`
    DataCopy,

    /// end execution, return data `mem[p…(p+s))`
    Return,
    /// end execution, revert state changes, return data `mem[p…(p+s))`
    Revert,
    /// stop execution, identical to `return(0, 0)`
    Stop,
    /// end execution, destroy current contract and send funds to `a`
    SelfDestruct,
    /// end execution with invalid instruction
    Invalid,
}

impl From<&str> for Name {
    fn from(input: &str) -> Self {
        match input {
            "add" => Self::Add,
            "sub" => Self::Sub,
            "mul" => Self::Mul,
            "div" => Self::Div,
            "mod" => Self::Mod,
            "not" => Self::Not,
            "lt" => Self::Lt,
            "gt" => Self::Gt,
            "eq" => Self::Eq,
            "iszero" => Self::IsZero,
            "and" => Self::And,
            "or" => Self::Or,
            "xor" => Self::Xor,
            "addmod" => Self::AddMod,
            "mulmod" => Self::MulMod,

            "sdiv" => Self::Sdiv,
            "smod" => Self::Smod,
            "exp" => Self::Exp,
            "slt" => Self::Slt,
            "sgt" => Self::Sgt,
            "byte" => Self::Byte,
            "shl" => Self::Shl,
            "shr" => Self::Shr,
            "sar" => Self::Sar,
            "signextend" => Self::SignExtend,
            "keccak256" => Self::Keccak256,
            "pc" => Self::Pc,

            "pop" => Self::Pop,
            "mload" => Self::MLoad,
            "mstore" => Self::MStore,
            "mstore8" => Self::MStore8,

            "sload" => Self::SLoad,
            "sstore" => Self::SStore,

            "caller" => Self::Caller,
            "callvalue" => Self::CallValue,
            "calldataload" => Self::CallDataLoad,
            "calldatasize" => Self::CallDataSize,
            "calldatacopy" => Self::CallDataCopy,

            "msize" => Self::MSize,
            "gas" => Self::Gas,
            "address" => Self::Address,
            "balance" => Self::Balance,
            "selfbalance" => Self::SelfBalance,

            "chainid" => Self::ChainId,
            "origin" => Self::Origin,
            "gasprice" => Self::GasPrice,
            "blockhash" => Self::BlockHash,
            "coinbase" => Self::CoinBase,
            "timestamp" => Self::Timestamp,
            "number" => Self::Number,
            "difficulty" => Self::Difficulty,
            "gaslimit" => Self::GasLimit,

            "create" => Self::Create,
            "create2" => Self::Create2,

            "log0" => Self::Log0,
            "log1" => Self::Log1,
            "log2" => Self::Log2,
            "log3" => Self::Log3,
            "log4" => Self::Log4,

            "call" => Self::Call,
            "callcode" => Self::CallCode,
            "delegatecall" => Self::DelegateCall,
            "staticcall" => Self::StaticCall,
            "setimmutable" => Self::SetImmutable,

            "codesize" => Self::CodeSize,
            "codecopy" => Self::CodeCopy,
            "extcodesize" => Self::ExtCodeSize,
            "extcodecopy" => Self::ExtCodeCopy,
            "returndatasize" => Self::ReturnDataSize,
            "returndatacopy" => Self::ReturnDataCopy,
            "extcodehash" => Self::ExtCodeHash,

            "datasize" => Self::DataSize,
            "dataoffset" => Self::DataOffset,
            "datacopy" => Self::DataCopy,

            "return" => Self::Return,
            "revert" => Self::Revert,
            "stop" => Self::Stop,
            "selfdestruct" => Self::SelfDestruct,
            "invalid" => Self::Invalid,

            input => Self::UserDefined(input.to_owned()),
        }
    }
}
