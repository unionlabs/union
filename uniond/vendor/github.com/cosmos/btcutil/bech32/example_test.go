// Copyright (c) 2017 The btcsuite developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package bech32_test

import (
	"encoding/hex"
	"fmt"

	"github.com/cosmos/btcutil/bech32"
)

// This example demonstrates how to decode a bech32 encoded string.
func ExampleDecode() {
	encoded := "bc1pw508d6qejxtdg4y5r3zarvary0c5xw7kw508d6qejxtdg4y5r3zarvary0c5xw7k7grplx"
	hrp, decoded, err := bech32.Decode(encoded, bech32.MaxLengthBIP173)
	if err != nil {
		fmt.Println("Error:", err)
	}

	// Show the decoded data.
	fmt.Println("Decoded human-readable part:", hrp)
	fmt.Println("Decoded Data:", hex.EncodeToString(decoded))

	// Output:
	// Decoded human-readable part: bc
	// Decoded Data: 010e140f070d1a001912060b0d081504140311021d030c1d03040f1814060e1e160e140f070d1a001912060b0d081504140311021d030c1d03040f1814060e1e16
}

// This example demonstrates how to encode data into a bech32 string.
func ExampleEncode() {
	data := []byte("Test data")
	// Convert test data to base32:
	conv, err := bech32.ConvertBits(data, 8, 5, true)
	if err != nil {
		fmt.Println("Error:", err)
	}
	encoded, err := bech32.Encode("customHrp!11111q", conv)
	if err != nil {
		fmt.Println("Error:", err)
	}

	// Show the encoded data.
	fmt.Println("Encoded Data:", encoded)

	// Output:
	// Encoded Data: customhrp!11111q123jhxapqv3shgcgkxpuhe
}
