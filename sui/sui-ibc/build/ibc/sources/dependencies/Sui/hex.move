// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// HEX (Base16) encoding utility.
module sui::hex;

const EInvalidHexLength: u64 = 0;
const ENotValidHexCharacter: u64 = 1;

// prettier-ignore
/// Vector of Base16 values from `00` to `FF`
const HEX: vector<vector<u8>> = vector[
    b"00", b"01", b"02", b"03", b"04", b"05", b"06", b"07", b"08", b"09", b"0a", b"0b", b"0c", b"0d", b"0e", b"0f",
    b"10", b"11", b"12", b"13", b"14", b"15", b"16", b"17", b"18", b"19", b"1a", b"1b", b"1c", b"1d", b"1e", b"1f",
    b"20", b"21", b"22", b"23", b"24", b"25", b"26", b"27", b"28", b"29", b"2a", b"2b", b"2c", b"2d", b"2e", b"2f",
    b"30", b"31", b"32", b"33", b"34", b"35", b"36", b"37", b"38", b"39", b"3a", b"3b", b"3c", b"3d", b"3e", b"3f",
    b"40", b"41", b"42", b"43", b"44", b"45", b"46", b"47", b"48", b"49", b"4a", b"4b", b"4c", b"4d", b"4e", b"4f",
    b"50", b"51", b"52", b"53", b"54", b"55", b"56", b"57", b"58", b"59", b"5a", b"5b", b"5c", b"5d", b"5e", b"5f",
    b"60", b"61", b"62", b"63", b"64", b"65", b"66", b"67", b"68", b"69", b"6a", b"6b", b"6c", b"6d", b"6e", b"6f",
    b"70", b"71", b"72", b"73", b"74", b"75", b"76", b"77", b"78", b"79", b"7a", b"7b", b"7c", b"7d", b"7e", b"7f",
    b"80", b"81", b"82", b"83", b"84", b"85", b"86", b"87", b"88", b"89", b"8a", b"8b", b"8c", b"8d", b"8e", b"8f",
    b"90", b"91", b"92", b"93", b"94", b"95", b"96", b"97", b"98", b"99", b"9a", b"9b", b"9c", b"9d", b"9e", b"9f",
    b"a0", b"a1", b"a2", b"a3", b"a4", b"a5", b"a6", b"a7", b"a8", b"a9", b"aa", b"ab", b"ac", b"ad", b"ae", b"af",
    b"b0", b"b1", b"b2", b"b3", b"b4", b"b5", b"b6", b"b7", b"b8", b"b9", b"ba", b"bb", b"bc", b"bd", b"be", b"bf",
    b"c0", b"c1", b"c2", b"c3", b"c4", b"c5", b"c6", b"c7", b"c8", b"c9", b"ca", b"cb", b"cc", b"cd", b"ce", b"cf",
    b"d0", b"d1", b"d2", b"d3", b"d4", b"d5", b"d6", b"d7", b"d8", b"d9", b"da", b"db", b"dc", b"dd", b"de", b"df",
    b"e0", b"e1", b"e2", b"e3", b"e4", b"e5", b"e6", b"e7", b"e8", b"e9", b"ea", b"eb", b"ec", b"ed", b"ee", b"ef",
    b"f0", b"f1", b"f2", b"f3", b"f4", b"f5", b"f6", b"f7", b"f8", b"f9", b"fa", b"fb", b"fc", b"fd", b"fe", b"ff",
];

/// Encode `bytes` in lowercase hex
public fun encode(bytes: vector<u8>): vector<u8> {
    let (mut i, mut r, l) = (0, vector[], bytes.length());
    let hex_vector = HEX;
    while (i < l) {
        r.append(hex_vector[bytes[i] as u64]);
        i = i + 1;
    };
    r
}

/// Decode hex into `bytes`
/// Takes a hex string (no 0x prefix) (e.g. b"0f3a")
/// Returns vector of `bytes` that represents the hex string (e.g. x"0f3a")
/// Hex string can be case insensitive (e.g. b"0F3A" and b"0f3a" both return x"0f3a")
/// Aborts if the hex string does not have an even number of characters (as each hex character is 2 characters long)
/// Aborts if the hex string contains non-valid hex characters (valid characters are 0 - 9, a - f, A - F)
public fun decode(hex: vector<u8>): vector<u8> {
    let (mut i, mut r, l) = (0, vector[], hex.length());
    assert!(l % 2 == 0, EInvalidHexLength);
    while (i < l) {
        let decimal = decode_byte(hex[i]) * 16 + decode_byte(hex[i + 1]);
        r.push_back(decimal);
        i = i + 2;
    };
    r
}

fun decode_byte(hex: u8): u8 {
    if (48 <= hex && hex < 58) {
        hex - 48
    } else if (65 <= hex && hex < 71) {
        10 + hex - 65
    } else if (97 <= hex && hex < 103) {
        10 + hex - 97
    } else {
        abort ENotValidHexCharacter
    }
}
