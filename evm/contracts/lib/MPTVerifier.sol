pragma solidity ^0.8.27;

// custom bytes calldata pointer storing (length | offset) in one word,
// also allows calldata pointers to be stored in memory
type BytesCalldata is uint256;

using BytesCalldataOps for BytesCalldata global;

// can't introduce global using .. for non UDTs
// each consumer should add the following line:
using BytesCalldataOps for bytes;

/**
 * @author Theori, Inc
 * @title BytesCalldataOps
 * @notice Common operations for bytes calldata, implemented for both the builtin
 *         type and our BytesCalldata type. These operations are heavily optimized
 *         and omit safety checks, so this library should only be used when memory
 *         safety is not a security issue.
 */
library BytesCalldataOps {
    function length(
        BytesCalldata bc
    ) internal pure returns (uint256 result) {
        assembly {
            result := shr(128, shl(128, bc))
        }
    }

    function offset(
        BytesCalldata bc
    ) internal pure returns (uint256 result) {
        assembly {
            result := shr(128, bc)
        }
    }

    function convert(
        BytesCalldata bc
    ) internal pure returns (bytes calldata value) {
        assembly {
            value.offset := shr(128, bc)
            value.length := shr(128, shl(128, bc))
        }
    }

    function convert(
        bytes calldata inp
    ) internal pure returns (BytesCalldata bc) {
        assembly {
            bc := or(shl(128, inp.offset), inp.length)
        }
    }

    function slice(
        BytesCalldata bc,
        uint256 start,
        uint256 len
    ) internal pure returns (BytesCalldata result) {
        assembly {
            result := shl(128, add(shr(128, bc), start)) // add to the offset and clear the length
            result := or(result, len) // set the new length
        }
    }

    function slice(
        bytes calldata value,
        uint256 start,
        uint256 len
    ) internal pure returns (bytes calldata result) {
        assembly {
            result.offset := add(value.offset, start)
            result.length := len
        }
    }

    function prefix(
        BytesCalldata bc,
        uint256 len
    ) internal pure returns (BytesCalldata result) {
        assembly {
            result := shl(128, shr(128, bc)) // clear out the length
            result := or(result, len) // set it to the new length
        }
    }

    function prefix(
        bytes calldata value,
        uint256 len
    ) internal pure returns (bytes calldata result) {
        assembly {
            result.offset := value.offset
            result.length := len
        }
    }

    function suffix(
        BytesCalldata bc,
        uint256 start
    ) internal pure returns (BytesCalldata result) {
        assembly {
            result := add(bc, shl(128, start)) // add to the offset
            result := sub(result, start) // subtract from the length
        }
    }

    function suffix(
        bytes calldata value,
        uint256 start
    ) internal pure returns (bytes calldata result) {
        assembly {
            result.offset := add(value.offset, start)
            result.length := sub(value.length, start)
        }
    }

    function split(
        BytesCalldata bc,
        uint256 start
    ) internal pure returns (BytesCalldata, BytesCalldata) {
        return (prefix(bc, start), suffix(bc, start));
    }

    function split(
        bytes calldata value,
        uint256 start
    ) internal pure returns (bytes calldata, bytes calldata) {
        return (prefix(value, start), suffix(value, start));
    }
}

/**
 * @title RLP
 * @author Theori, Inc.
 * @notice Gas optimized RLP parsing code. Note that some parsing logic is
 *         duplicated because helper functions are oddly expensive.
 */
library RLP {
    function parseUint(
        bytes calldata buf
    ) internal pure returns (uint256 result, uint256 size) {
        assembly {
            // check that we have at least one byte of input
            if iszero(buf.length) { revert(0, 0) }
            let first32 := calldataload(buf.offset)
            let kind := shr(248, first32)

            // ensure it's a not a long string or list (> 0xB7)
            // also ensure it's not a short string longer than 32 bytes (> 0xA0)
            if gt(kind, 0xA0) { revert(0, 0) }

            switch lt(kind, 0x80)
            case true {
                // small single byte
                result := kind
                size := 1
            }
            case false {
                // short string
                size := sub(kind, 0x80)

                // ensure it's not reading out of bounds
                if lt(buf.length, size) { revert(0, 0) }

                switch eq(size, 32)
                case true {
                    // if it's exactly 32 bytes, read it from calldata
                    result := calldataload(add(buf.offset, 1))
                }
                case false {
                    // if it's < 32 bytes, we've already read it from calldata
                    result := shr(shl(3, sub(32, size)), shl(8, first32))
                }
                size := add(size, 1)
            }
        }
    }

    function nextSize(
        bytes calldata buf
    ) internal pure returns (uint256 size) {
        assembly {
            if iszero(buf.length) { revert(0, 0) }
            let first32 := calldataload(buf.offset)
            let kind := shr(248, first32)

            switch lt(kind, 0x80)
            case true {
                // small single byte
                size := 1
            }
            case false {
                switch lt(kind, 0xB8)
                case true {
                    // short string
                    size := add(1, sub(kind, 0x80))
                }
                case false {
                    switch lt(kind, 0xC0)
                    case true {
                        // long string
                        let lengthSize := sub(kind, 0xB7)

                        // ensure that we don't overflow
                        if gt(lengthSize, 31) { revert(0, 0) }

                        // ensure that we don't read out of bounds
                        if lt(buf.length, lengthSize) { revert(0, 0) }
                        size :=
                            shr(mul(8, sub(32, lengthSize)), shl(8, first32))
                        size := add(size, add(1, lengthSize))
                    }
                    case false {
                        switch lt(kind, 0xF8)
                        case true {
                            // short list
                            size := add(1, sub(kind, 0xC0))
                        }
                        case false {
                            let lengthSize := sub(kind, 0xF7)

                            // ensure that we don't overflow
                            if gt(lengthSize, 31) { revert(0, 0) }
                            // ensure that we don't read out of bounds
                            if lt(buf.length, lengthSize) { revert(0, 0) }
                            size :=
                                shr(mul(8, sub(32, lengthSize)), shl(8, first32))
                            size := add(size, add(1, lengthSize))
                        }
                    }
                }
            }
        }
    }

    function skip(
        bytes calldata buf
    ) internal pure returns (bytes calldata) {
        uint256 size = RLP.nextSize(buf);
        assembly {
            buf.offset := add(buf.offset, size)
            buf.length := sub(buf.length, size)
        }
        return buf;
    }

    function parseList(
        bytes calldata buf
    ) internal pure returns (uint256 listSize, uint256 offset) {
        assembly {
            // check that we have at least one byte of input
            if iszero(buf.length) { revert(0, 0) }
            let first32 := calldataload(buf.offset)
            let kind := shr(248, first32)

            // ensure it's a list
            if lt(kind, 0xC0) { revert(0, 0) }

            switch lt(kind, 0xF8)
            case true {
                // short list
                listSize := sub(kind, 0xC0)
                offset := 1
            }
            case false {
                // long list
                let lengthSize := sub(kind, 0xF7)

                // ensure that we don't overflow
                if gt(lengthSize, 31) { revert(0, 0) }
                // ensure that we don't read out of bounds
                if lt(buf.length, lengthSize) { revert(0, 0) }
                listSize := shr(mul(8, sub(32, lengthSize)), shl(8, first32))
                offset := add(lengthSize, 1)
            }
        }
    }

    function splitBytes(
        bytes calldata buf
    ) internal pure returns (bytes calldata result, bytes calldata rest) {
        uint256 offset;
        uint256 size;
        assembly {
            // check that we have at least one byte of input
            if iszero(buf.length) { revert(0, 0) }
            let first32 := calldataload(buf.offset)
            let kind := shr(248, first32)

            // ensure it's a not list
            if gt(kind, 0xBF) { revert(0, 0) }

            switch lt(kind, 0x80)
            case true {
                // small single byte
                offset := 0
                size := 1
            }
            case false {
                switch lt(kind, 0xB8)
                case true {
                    // short string
                    offset := 1
                    size := sub(kind, 0x80)
                }
                case false {
                    // long string
                    let lengthSize := sub(kind, 0xB7)

                    // ensure that we don't overflow
                    if gt(lengthSize, 31) { revert(0, 0) }
                    // ensure we don't read out of bounds
                    if lt(buf.length, lengthSize) { revert(0, 0) }
                    size := shr(mul(8, sub(32, lengthSize)), shl(8, first32))
                    offset := add(lengthSize, 1)
                }
            }

            result.offset := add(buf.offset, offset)
            result.length := size

            let end := add(offset, size)
            rest.offset := add(buf.offset, end)
            rest.length := sub(buf.length, end)
        }
    }

    function encodeUint(
        uint256 value
    ) internal pure returns (bytes memory) {
        // allocate our result bytes
        bytes memory result = new bytes(33);

        if (value == 0) {
            // store length = 1, value = 0x80
            assembly {
                mstore(add(result, 1), 0x180)
            }
            return result;
        }

        if (value < 128) {
            // store length = 1, value = value
            assembly {
                mstore(add(result, 1), or(0x100, value))
            }
            return result;
        }

        if (
            value
                > 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
        ) {
            // length 33, prefix 0xa0 followed by value
            assembly {
                mstore(add(result, 1), 0x21a0)
                mstore(add(result, 33), value)
            }
            return result;
        }

        if (
            value
                > 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
        ) {
            // length 32, prefix 0x9f followed by value
            assembly {
                mstore(add(result, 1), 0x209f)
                mstore(add(result, 33), shl(8, value))
            }
            return result;
        }

        assembly {
            let length := 1
            for { let min := 0x100 } lt(sub(min, 1), value) {
                min := shl(8, min)
            } { length := add(length, 1) }

            let bytesLength := add(length, 1)

            // bytes length field
            let hi := shl(mul(bytesLength, 8), bytesLength)

            // rlp encoding of value
            let lo := or(shl(mul(length, 8), add(length, 0x80)), value)

            mstore(add(result, bytesLength), or(hi, lo))
        }
        return result;
    }
}

library MPTVerifier {
    using BytesCalldataOps for bytes;

    struct Node {
        BytesCalldata data;
        bytes32 hash;
    }

    // prefix constants
    uint8 constant ODD_LENGTH = 1;
    uint8 constant LEAF = 2;
    uint8 constant MAX_PREFIX = 3;

    function parseHash(
        bytes calldata buf
    ) internal pure returns (bytes32 result, uint256 offset) {
        uint256 value;
        (value, offset) = RLP.parseUint(buf);
        result = bytes32(value);
    }

    /**
     * @notice parses concatenated MPT nodes into processed Node structs
     * @param input the concatenated MPT nodes
     * @return result the parsed nodes array, containing a calldata slice and hash
     *                for each node
     */
    function parseNodes(
        bytes calldata input
    ) internal pure returns (Node[] memory result) {
        uint256 freePtr;
        uint256 firstNode;

        // we'll use a dynamic amount of memory starting at the free pointer
        // it is crucial that no other allocations happen during parsing
        assembly {
            freePtr := mload(0x40)

            // corrupt free pointer to cause out-of-gas if allocation occurs
            mstore(
                0x40,
                0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc
            )

            firstNode := freePtr
        }

        uint256 count;
        while (input.length > 0) {
            (uint256 listsize, uint256 offset) = RLP.parseList(input);
            bytes calldata node = input.slice(offset, listsize);
            BytesCalldata slice = node.convert();

            uint256 len;
            assembly {
                len := add(listsize, offset)

                // compute node hash
                calldatacopy(freePtr, input.offset, len)
                let nodeHash := keccak256(freePtr, len)

                // store the Node struct (calldata slice and hash)
                mstore(freePtr, slice)
                mstore(add(freePtr, 0x20), nodeHash)

                // advance pointer
                count := add(count, 1)
                freePtr := add(freePtr, 0x40)
            }

            input = input.suffix(len);
        }

        assembly {
            // allocate the result array and fill it with the node pointers
            result := freePtr
            mstore(result, count)
            freePtr := add(freePtr, 0x20)
            for { let i := 0 } lt(i, count) { i := add(i, 1) } {
                mstore(freePtr, add(firstNode, mul(0x40, i)))
                freePtr := add(freePtr, 0x20)
            }

            // update the free pointer
            mstore(0x40, freePtr)
        }
    }

    /**
     * @notice parses a compressed MPT proof into arrays of Node structs
     * @param nodes the set of nodes used in the compressed proofs
     * @param compressed the compressed MPT proof
     * @param count the number of proofs expected from the compressed proof
     * @return result the array of proofs
     */
    function parseCompressedProofs(
        Node[] memory nodes,
        bytes calldata compressed,
        uint256 count
    ) internal pure returns (Node[][] memory result) {
        uint256 resultPtr;
        uint256 freePtr;

        // we'll use a dynamic amount of memory starting at the free pointer
        // it is crucial that no other allocations happen during parsing
        assembly {
            result := mload(0x40)

            // corrupt free pointer to cause out-of-gas if allocation occurs
            mstore(
                0x40,
                0xcccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc
            )

            mstore(result, count)
            resultPtr := add(result, 0x20)
            freePtr := add(resultPtr, mul(0x20, count))
        }

        (uint256 listSize, uint256 offset) = RLP.parseList(compressed);
        compressed = compressed.slice(offset, listSize);

        // parse the indices and populate the proof list
        for (; count > 0; count--) {
            bytes calldata indices;
            (listSize, offset) = RLP.parseList(compressed);
            indices = compressed.slice(offset, listSize);
            compressed = compressed.suffix(listSize + offset);

            // begin next proof array
            uint256 arr;
            assembly {
                arr := freePtr
                freePtr := add(freePtr, 0x20)
            }

            // fill proof array
            uint256 len;
            for (len = 0; indices.length > 0; len++) {
                uint256 idx;
                (idx, offset) = RLP.parseUint(indices);
                indices = indices.suffix(offset);
                require(
                    idx < nodes.length, "invalid node index in compressed proof"
                );
                assembly {
                    let node := mload(add(add(nodes, 0x20), mul(0x20, idx)))
                    mstore(freePtr, node)
                    freePtr := add(freePtr, 0x20)
                }
            }

            assembly {
                // store the array length
                mstore(arr, len)

                // store the array pointer in the result
                mstore(resultPtr, arr)
                resultPtr := add(resultPtr, 0x20)
            }
        }

        assembly {
            // update the free pointer
            mstore(0x40, freePtr)
        }
    }

    /**
     * @notice Checks if the provided bytes match the key at a given offset
     * @param key the MPT key to check against
     * @param keyLen the length (in nibbles) of the key
     * @param testBytes the subkey to check
     */
    function subkeysEqual(
        bytes32 key,
        uint256 keyLen,
        bytes calldata testBytes
    ) private pure returns (bool result) {
        // arithmetic cannot overflow because testBytes is from calldata
        uint256 nibbleLength;
        unchecked {
            nibbleLength = 2 * testBytes.length;
            require(nibbleLength <= keyLen);
        }

        assembly {
            let shiftAmount := sub(256, shl(2, nibbleLength))
            let testValue := shr(shiftAmount, calldataload(testBytes.offset))
            let subkey := shr(shiftAmount, key)
            result := eq(testValue, subkey)
        }
    }

    /**
     * @notice checks the MPT proof. Note: for certain optimizations, we assume
     *         that the rootHash belongs to a valid ethereum block. Correctness
     *         is only guaranteed in that case.
     *         Gas usage depends on both proof size and key nibble values.
     *         Gas usage for actual ethereum account proofs: ~ 30000 - 45000
     * @param nodes MPT proof nodes, parsed using parseNodes()
     * @param key the MPT key, padded with trailing 0s if needed
     * @param keyLen the byte length of the MPT key, must be <= 32
     * @param expectedHash the root hash of the MPT
     */
    function verifyTrieValueWithNodes(
        Node[] memory nodes,
        bytes32 key,
        uint256 keyLen,
        bytes32 expectedHash
    ) internal pure returns (bool exists, bytes calldata value) {
        // handle completely empty trie case
        if (nodes.length == 0) {
            require(keccak256(hex"80") == expectedHash, "root hash incorrect");
            return (false, msg.data[:0]);
        }

        // we will read the key nibble by nibble, so double the length
        unchecked {
            keyLen *= 2;
        }

        // initialize return values to make solc happy;
        // one will always be overwritten before returing
        assembly {
            value.offset := 0
            value.length := 0
        }
        exists = true;

        // we'll use nodes as a pointer, advancing through each element
        // end will point to the end of the array
        uint256 end;
        assembly {
            end := add(nodes, add(0x20, mul(0x20, mload(nodes))))
            nodes := add(nodes, 0x20)
        }

        while (true) {
            bytes calldata node;
            {
                BytesCalldata slice;
                bytes32 nodeHash;

                // load the element and advance the proof pointer
                assembly {
                    // bounds checking
                    if iszero(lt(nodes, end)) { revert(0, 0) }

                    let ptr := mload(nodes)
                    nodes := add(nodes, 0x20)

                    slice := mload(ptr)
                    nodeHash := mload(add(ptr, 0x20))
                }
                node = slice.convert();

                require(nodeHash == expectedHash, "node hash incorrect");
            }

            // find the length of the first two elements
            uint256 size = RLP.nextSize(node);
            unchecked {
                size += RLP.nextSize(node.suffix(size));
            }

            // we now know which type of node we're looking at:
            // leaf + extension nodes have 2 list elements, branch nodes have 17
            if (size == node.length) {
                // only two elements, leaf or extension node
                bytes calldata encodedPath;
                (encodedPath, node) = RLP.splitBytes(node);

                // keep track of whether the key nibbles match
                bool keysMatch;

                // the first nibble of the encodedPath tells us the type of
                // node and if it contains an even or odd number of nibbles
                uint8 firstByte = uint8(encodedPath[0]);
                uint8 prefix = firstByte >> 4;
                require(prefix <= MAX_PREFIX);
                if (prefix & ODD_LENGTH == 0) {
                    // second nibble is padding, must be 0
                    require(firstByte & 0xf == 0);
                    keysMatch = true;
                } else {
                    // second nibble is part of key
                    keysMatch = (firstByte & 0xf) == (uint8(bytes1(key)) >> 4);
                    unchecked {
                        key <<= 4;
                        keyLen--;
                    }
                }

                // check the remainder of the encodedPath
                encodedPath = encodedPath.suffix(1);
                keysMatch = keysMatch && subkeysEqual(key, keyLen, encodedPath);
                // cannot overflow because encodedPath is from calldata
                unchecked {
                    key <<= 8 * encodedPath.length;
                    keyLen -= 2 * encodedPath.length;
                }

                if (prefix & LEAF == 0) {
                    // extension can't prove nonexistence, subkeys must match
                    require(keysMatch);

                    (expectedHash,) = parseHash(node);
                } else {
                    // leaf node, must have used all of key
                    require(keyLen == 0);

                    if (keysMatch) {
                        // if keys equal, we found the value
                        (value, node) = RLP.splitBytes(node);
                        break;
                    } else {
                        // if keys aren't equal, key doesn't exist
                        exists = false;
                        break;
                    }
                }
            } else {
                // branch node, this is the hotspot for gas usage

                // there should be 17 elements (16 branch hashes + a value)
                // we won't explicitly check this in order to save gas, since
                // it's implied by inclusion in a valid ethereum block

                // also note, we never need the value element because we assume
                // uniquely-prefixed keys, so branch nodes never hold values

                // fetch the branch for the next nibble of the key
                uint256 keyNibble = uint256(key >> 252);

                // skip past the branches we don't need
                // we already skipped past 2 elements; start there if we can
                uint256 i = 0;
                if (keyNibble >= 2) {
                    i = 2;
                    node = node.suffix(size);
                }
                while (i < keyNibble) {
                    node = RLP.skip(node);
                    unchecked {
                        i++;
                    }
                }

                (expectedHash,) = parseHash(node);
                // if we've reached an empty branch, key doesn't exist
                if (expectedHash == 0) {
                    exists = false;
                    break;
                }
                unchecked {
                    key <<= 4;
                    keyLen -= 1;
                }
            }
        }
    }

    /**
     * @notice checks the MPT proof. Note: for certain optimizations, we assume
     *         that the rootHash belongs to a valid ethereum block. Correctness
     *         is only guaranteed in that case.
     *         Gas usage depends on both proof size and key nibble values.
     *         Gas usage for actual ethereum account proofs: ~ 30000 - 45000
     * @param proof the encoded MPT proof noodes concatenated
     * @param key the MPT key, padded with trailing 0s if needed
     * @param rootHash the root hash of the MPT
     */
    function verifyTrieValue(
        bytes calldata proof,
        bytes32 key,
        bytes32 rootHash
    ) internal pure returns (bool exists, bytes calldata value) {
        Node[] memory nodes = parseNodes(proof);
        return verifyTrieValueWithNodes(nodes, key, 32, rootHash);
    }
}
