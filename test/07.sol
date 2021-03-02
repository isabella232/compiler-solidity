// RUN: fun_main_69
// EXPECT: 5050
pragma solidity >0.7.0;

contract MyContract {
    function sum(uint256 a, bool is_odd) public pure returns(uint256) {
        uint256 result = 0;
        for (uint256 i = 1; i <= a; ++i) {
            if (is_odd) {
                if (i % 2 != 0)
                    result += i;
            } else {
                if (i % 2 == 0)
                    result += i;
            }
        }
        return result;
    }
    function main() public pure returns(uint256) {
        return sum(100, true) + sum(100, false);
    }
}
