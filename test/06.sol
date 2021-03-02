// RUN: fun_main_59
// EXPECT: 42
pragma solidity >0.7.0;

contract MyContract {
    function foo(uint256 a, uint256 b, uint256 c) public pure returns(uint256) {
        if (c > 42) {
            return a + b;
        }
        if (c == 42) {
            return a - b;
        }
        return a * b;
    }
    function main() public pure returns(uint256) {
        return foo(84, 2, 43) + foo(23, 23, 42) - foo(22, 2, 3);
    }
}
