// RUN: fun_main_26
// EXPECT: 42

pragma solidity >0.7.0;

contract MyContract {
    function foo(uint256 a, uint256 b) public pure returns(uint256) {
      return a + b;
    }

    function main() public pure returns(uint256) {
        return foo(40, 2);
    }
}
