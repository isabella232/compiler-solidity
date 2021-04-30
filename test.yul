Optimized IR:
/*******************************************************
 *                       WARNING                       *
 *  Solidity to Yul compilation is still EXPERIMENTAL  *
 *       It can result in LOSS OF FUNDS or worse       *
 *                !USE AT YOUR OWN RISK!               *
 *******************************************************/

object "Test_138" {
    code {
        {
            mstore(64, 128)
            if callvalue() { revert(0, 0) }
            let _1 := datasize("Test_138_deployed")
            codecopy(0, dataoffset("Test_138_deployed"), _1)
            return(0, _1)
        }
    }
    object "Test_138_deployed" {
        code {
            {
                mstore(64, 128)
                let _1 := 0
                if callvalue() { revert(_1, _1) }
                if slt(add(calldatasize(), not(3)), _1) { revert(_1, _1) }
                let expr_mpos := allocate_memory(array_allocation_size_array_uint8(11))
                write_to_memory_uint8(expr_mpos, _1)
                write_to_memory_uint8(add(expr_mpos, 32), 0x01)
                write_to_memory_uint8(add(expr_mpos, 64), 0x02)
                write_to_memory_uint8(add(expr_mpos, 96), 0x03)
                write_to_memory_uint8(add(expr_mpos, 128), 4)
                write_to_memory_uint8(add(expr_mpos, 160), 0x05)
                write_to_memory_uint8(add(expr_mpos, 192), 0x06)
                write_to_memory_uint8(add(expr_mpos, 224), 0x07)
                write_to_memory_uint8(add(expr_mpos, 256), 0x08)
                write_to_memory_uint8(add(expr_mpos, 288), 0x09)
                write_to_memory_uint8(add(expr_mpos, 320), 0x0a)
                let var := cleanup_uint8(fun_main(expr_mpos))
                let memPos := allocate_memory(_1)
                return(memPos, sub(abi_encode_uint64(memPos, var), memPos))
            }
            function abi_encode_uint64(headStart, value0) -> tail
            {
                tail := add(headStart, 32)
                mstore(headStart, and(value0, 0xffffffffffffffff))
            }
            function allocate_memory(size) -> memPtr
            {
                memPtr := mload(64)
                let newFreePtr := add(memPtr, and(add(size, 31), not(31)))
                if or(gt(newFreePtr, 0xffffffffffffffff), lt(newFreePtr, memPtr)) { panic_error_0x41() }
                mstore(64, newFreePtr)
            }
            function array_allocation_size_array_uint8(length) -> size
            {
                if gt(length, 0xffffffffffffffff) { panic_error_0x41() }
                size := shl(5, length)
            }
            function checked_add_uint8(x, y) -> sum
            {
                let x_1 := and(x, 0xff)
                let y_1 := and(y, 0xff)
                if gt(x_1, sub(0xff, y_1))
                {
                    mstore(sum, shl(224, 0x4e487b71))
                    mstore(4, 0x11)
                    revert(sum, 0x24)
                }
                sum := add(x_1, y_1)
            }
            function cleanup_uint8(value) -> cleaned
            { cleaned := and(value, 0xff) }
            function fun_main(var_v_mpos) -> var
            {
                let cleaned := and(mload(memory_array_index_access_uint8(var_v_mpos, var)), 0xff)
                pop(memory_array_index_access_uint8(var_v_mpos, 0x01))
                let returnValue := and(mload(memory_array_index_access_uint8(var_v_mpos, 0x02)), 0xff)
                let _1 := read_from_memoryt_uint8(memory_array_index_access_uint8(var_v_mpos, 0x03))
                let _2 := read_from_memoryt_uint8(memory_array_index_access_uint8(var_v_mpos, 0x04))
                let _3 := read_from_memoryt_uint8(memory_array_index_access_uint8(var_v_mpos, 0x05))
                let _4 := read_from_memoryt_uint8(memory_array_index_access_uint8(var_v_mpos, 0x06))
                pop(memory_array_index_access_uint8(var_v_mpos, 0x07))
                let _5 := read_from_memoryt_uint8(memory_array_index_access_uint8(var_v_mpos, 0x08))
                pop(memory_array_index_access_uint8(var_v_mpos, 0x09))
                var := checked_add_uint8(checked_add_uint8(checked_add_uint8(checked_add_uint8(checked_add_uint8(checked_add_uint8(checked_add_uint8(checked_add_uint8(checked_add_uint8(checked_add_uint8(cleaned, 0x2a), returnValue), _1), _2), _3), _4), 0x2a), _5), 0x2a), read_from_memoryt_uint8(memory_array_index_access_uint8(var_v_mpos, 0x0a)))
            }
            function memory_array_index_access_uint8(baseRef, index) -> addr
            {
                if iszero(lt(index, 0x0b))
                {
                    mstore(addr, shl(224, 0x4e487b71))
                    mstore(4, 0x32)
                    revert(addr, 0x24)
                }
                addr := add(baseRef, shl(5, index))
            }
            function panic_error_0x41()
            {
                mstore(0, shl(224, 0x4e487b71))
                mstore(4, 0x41)
                revert(0, 0x24)
            }
            function read_from_memoryt_uint8(ptr) -> returnValue
            {
                returnValue := and(mload(ptr), 0xff)
            }
            function write_to_memory_uint8(memPtr, value)
            {
                mstore(memPtr, and(value, 0xff))
            }
        }
    }
}

