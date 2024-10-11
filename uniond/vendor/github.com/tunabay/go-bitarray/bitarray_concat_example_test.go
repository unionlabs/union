// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"fmt"

	"github.com/tunabay/go-bitarray"
)

func ExampleJoin() {
	elems := []*bitarray.BitArray{
		bitarray.MustParse("10101"),
		bitarray.MustParse("111"),
		bitarray.MustParse("1"),
		bitarray.MustParse("1111111111"),
	}
	sep := bitarray.MustParse("00000")
	ba := bitarray.Join(elems, sep)
	fmt.Printf("% s\n", ba)

	// Output:
	// 10101000 00111000 00100000 11111111 11
}

func ExampleJoinBitArrayer() {
	elems := []bitarray.BitArrayer{
		bitarray.MustParse("10101"),
		bitarray.MustParse("111"),
		bitarray.MustParse("1"),
		bitarray.MustParse("1111111111"),
	}
	sep := bitarray.MustParse("00000")
	ba := bitarray.JoinBitArrayer(elems, sep)
	fmt.Printf("% s\n", ba)

	// Output:
	// 10101000 00111000 00100000 11111111 11
}

func ExampleBitArray_Append() {
	ba1 := bitarray.MustParse("110011")
	ba2 := bitarray.MustParse("00000000 0000")
	ba3 := bitarray.MustParse("111")
	fmt.Printf("% s\n", ba1.Append(ba2, ba3))
	fmt.Printf("% s\n", ba2.Append(nil))
	fmt.Printf("% s\n", ba3.Append(ba3, ba3, ba3))

	// Output:
	// 11001100 00000000 00111
	// 00000000 0000
	// 11111111 1111
}

func ExampleBitArray_Repeat() {
	ba := bitarray.MustParse("111000")
	fmt.Printf("% s\n", ba.Repeat(5))
	fmt.Printf("% s\n", ba.Repeat(1))
	fmt.Printf("% q\n", ba.Repeat(0))

	// Output:
	// 11100011 10001110 00111000 111000
	// 111000
	// ""
}
