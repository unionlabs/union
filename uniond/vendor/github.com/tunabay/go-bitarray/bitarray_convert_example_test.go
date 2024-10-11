// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"fmt"
	"math/big"

	"github.com/tunabay/go-bitarray"
)

func ExampleNewFromInt() {
	v := big.NewInt(1234567890123456)
	ba := bitarray.NewFromInt(v)
	fmt.Printf("% s\n", ba)

	// Output:
	// 10001100 01011010 10100111 10010001 01010111 01011000 000
}

func ExampleBitArray_ToInt() {
	ba := bitarray.MustParse("00001001 00110010 11000000 01011010 010")
	fmt.Println(ba.ToInt())

	// Output:
	// 1234567890
}

func ExampleBitArray_ToUint64() {
	ba := bitarray.MustParse("10010011 00101100 00000101 10111010 10")
	fmt.Println(ba.ToUint64())

	// Output:
	// 9876543210
}
