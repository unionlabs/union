pragma solidity ^0.8.27;

library Hex {
    error ErrInvalidHexAddress();

    // Convert 32 hexadecimal digits into 16 bytes.
    function hexToBytes16(
        bytes32 h
    ) internal pure returns (bytes16 b) {
        unchecked {
            // Ensure all chars below 128
            if (
                h
                    &
                    0x8080808080808080808080808080808080808080808080808080808080808080
                    != 0
            ) {
                revert ErrInvalidHexAddress();
            }
            // Subtract '0' from every char
            h = bytes32(
                uint256(h)
                    -
                    0x3030303030303030303030303030303030303030303030303030303030303030
            );
            // Ensure all chars still below 128, i.e. no underflow in the previous line
            if (
                h
                    &
                    0x8080808080808080808080808080808080808080808080808080808080808080
                    != 0
            ) {
                revert ErrInvalidHexAddress();
            }
            // Calculate mask for chars that originally were above '9'
            bytes32 ndm = bytes32(
                (
                    (
                        (
                            uint256(h)
                                +
                                0x7676767676767676767676767676767676767676767676767676767676767676
                        )
                            &
                            0x8080808080808080808080808080808080808080808080808080808080808080
                    ) >> 7
                ) * 0xFF
            );
            // Subtract 7 ('A' - '0') from every char that originally was above '9'
            h = bytes32(
                uint256(h)
                    - uint256(
                        ndm
                            &
                            0x0707070707070707070707070707070707070707070707070707070707070707
                    )
            );
            // Ensure all chars still below 128, i.e. no underflow in the previous line
            if (
                h
                    &
                    0x8080808080808080808080808080808080808080808080808080808080808080
                    != 0
            ) {
                revert ErrInvalidHexAddress();
            }
            // Ensure chars that originally were above '9' are now above 9
            if (
                (
                    uint256(h)
                        - uint256(
                            ndm
                                &
                                0x0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A
                        )
                )
                    &
                    0x8080808080808080808080808080808080808080808080808080808080808080
                    != 0
            ) {
                revert ErrInvalidHexAddress();
            }
            // Calculate Mask for chars that originally were above 'F'
            bytes32 lcm = bytes32(
                (
                    (
                        (
                            uint256(h)
                                +
                                0x7070707070707070707070707070707070707070707070707070707070707070
                        )
                            &
                            0x8080808080808080808080808080808080808080808080808080808080808080
                    ) >> 7
                ) * 0xFF
            );
            // Subtract 32 ('a' - 'A') from all chars that oroginally were above 'F'
            h = bytes32(
                uint256(h)
                    - uint256(
                        lcm
                            &
                            0x2020202020202020202020202020202020202020202020202020202020202020
                    )
            );
            // Ensure chars that originally were above 'F' are now above 9
            if (
                (
                    uint256(h)
                        - uint256(
                            lcm
                                &
                                0x0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A
                        )
                )
                    &
                    0x8080808080808080808080808080808080808080808080808080808080808080
                    != 0
            ) {
                revert ErrInvalidHexAddress();
            }
            // Ensure all chars are below 16
            if (
                h
                    &
                    0xF0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0
                    != 0
            ) {
                revert ErrInvalidHexAddress();
            }
            // 0x0A0B0C0D... -> 0xAB00CD00...
            h = (
                (
                    h
                        &
                        0x0F000F000F000F000F000F000F000F000F000F000F000F000F000F000F000F00
                ) << 4
            )
                | (
                    (
                        h
                            &
                            0x000F000F000F000F000F000F000F000F000F000F000F000F000F000F000F000F
                    ) << 8
                );
            // 0xAA00BB00CC00DD00... -> 0xAABB0000CCDD0000...
            h = (
                h
                    &
                    0xFF000000FF000000FF000000FF000000FF000000FF000000FF000000FF000000
            )
                | (
                    (
                        h
                            &
                            0x0000FF000000FF000000FF000000FF000000FF000000FF000000FF000000FF00
                    ) << 8
                );
            // 0xAAAA0000BBBB0000CCCC0000DDDD0000... -> 0xAAAABBBB00000000CCCCDDDD00000000...
            h = (
                h
                    &
                    0xFFFF000000000000FFFF000000000000FFFF000000000000FFFF000000000000
            )
                | (
                    (
                        h
                            &
                            0x00000000FFFF000000000000FFFF000000000000FFFF000000000000FFFF0000
                    ) << 16
                );
            // 0xAAAAAAAA00000000BBBBBBBB00000000CCCCCCCC00000000DDDDDDDD00000000 -> 0xAAAAAAAABBBBBBBB0000000000000000CCCCCCCCDDDDDDDD0000000000000000
            h = (
                h
                    &
                    0xFFFFFFFF000000000000000000000000FFFFFFFF000000000000000000000000
            )
                | (
                    (
                        h
                            &
                            0x0000000000000000FFFFFFFF000000000000000000000000FFFFFFFF00000000
                    ) << 32
                );
            // 0xAAAAAAAAAAAAAAAA0000000000000000BBBBBBBBBBBBBBBB0000000000000000 -> 0xAAAAAAAAAAAAAAAABBBBBBBBBBBBBBBB00000000000000000000000000000000
            h = (
                h
                    &
                    0xFFFFFFFFFFFFFFFF000000000000000000000000000000000000000000000000
            )
                | (
                    (
                        h
                            &
                            0x00000000000000000000000000000000FFFFFFFFFFFFFFFF0000000000000000
                    ) << 64
                );
            // Trim to 16 bytes
            b = bytes16(h);
        }
    }

    function hexToAddress(
        string memory s
    ) internal pure returns (address) {
        if (bytes(s).length != 42) {
            revert ErrInvalidHexAddress();
        }
        bytes2 prefix;
        bytes32 leftHex;
        bytes32 rightHex;
        assembly {
            prefix := mload(add(s, 0x20))
            leftHex := mload(add(s, 0x22))
            rightHex := mload(add(s, 0x2A))
        }
        if (prefix != "0x") {
            revert ErrInvalidHexAddress();
        }
        bytes16 left = hexToBytes16(leftHex);
        bytes16 right = hexToBytes16(rightHex);
        return address(bytes20(left) | (bytes20(right) >> 32));
    }

    function atoi(
        bytes1 b
    ) internal pure returns (uint8 res) {
        if (b >= "0" && b <= "9") {
            return uint8(b) - uint8(bytes1("0"));
        } else if (b >= "A" && b <= "F") {
            return 10 + uint8(b) - uint8(bytes1("A"));
        } else if (b >= "a" && b <= "f") {
            return 10 + uint8(b) - uint8(bytes1("a"));
        }
        return uint8(b);
    }

    function hexToUint256(
        string memory s
    ) internal pure returns (uint256) {
        bytes memory b = bytes(s);
        uint256 number;
        for (uint256 i = 2; i < b.length; i++) {
            number = number << 4;
            number |= atoi(b[i]);
        }
        return number;
    }
}
