// RUN: fun_main_9
// EXPECT: 42

pragma solidity >0.7.0;

contract MyContract {
    function main() public pure returns(uint256) {
        return 42;
    }
}
