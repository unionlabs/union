// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"fmt"

	"github.com/tunabay/go-bitarray"
)

func ExampleBitArray_LeadingZeros() {
	ba1 := bitarray.MustParse("1000-0111 0100")
	ba2 := bitarray.MustParse("0000-0000 0000-1100 0110-1")
	ba3 := bitarray.MustParse("0000-0")

	fmt.Println(ba1.LeadingZeros())
	fmt.Println(ba2.LeadingZeros())
	fmt.Println(ba3.LeadingZeros())

	// Output:
	// 0
	// 12
	// 5
}

func ExampleBitArray_TrailingZeros() {
	ba1 := bitarray.MustParse("1100-1111 1100-0000 0000-0")
	ba2 := bitarray.MustParse("0000-0001 1111")
	ba3 := bitarray.MustParse("000")

	fmt.Println(ba1.TrailingZeros())
	fmt.Println(ba2.TrailingZeros())
	fmt.Println(ba3.TrailingZeros())

	// Output:
	// 11
	// 0
	// 3
}

func ExampleBitArray_OnesCount() {
	ba1 := bitarray.MustParse("1100-1100 11")
	ba2 := bitarray.MustParse("0000-0000 0000-10")
	ba3 := bitarray.MustParse("0000")
	fmt.Println(ba1.OnesCount())
	fmt.Println(ba2.OnesCount())
	fmt.Println(ba3.OnesCount())

	// Output:
	// 6
	// 1
	// 0
}

func ExampleBitArray_And() {
	ba1 := bitarray.MustParse("1111-0000 0011-01")
	ba2 := bitarray.MustParse("1010-1010 1010-10")
	fmt.Printf("% b\n", ba1.And(ba2))

	// Output:
	// 10100000 001000
}

func ExampleBitArray_Or() {
	ba1 := bitarray.MustParse("1111-0000 0011-01")
	ba2 := bitarray.MustParse("1010-1010 1010-10")
	fmt.Printf("% b\n", ba1.Or(ba2))

	// Output:
	// 11111010 101111
}

func ExampleBitArray_Xor() {
	ba1 := bitarray.MustParse("1111-0000 0011-01")
	ba2 := bitarray.MustParse("1010-1010 1010-10")
	fmt.Printf("% b\n", ba1.Xor(ba2))

	// Output:
	// 01011010 100111
}

func ExampleBitArray_Not() {
	ba := bitarray.MustParse("1111-0000 1010")
	fmt.Printf("% b\n", ba.Not())

	// Output:
	// 00001111 0101
}

func ExampleBitArray_AndAt() {
	ba1 := bitarray.MustParse("1010-1010 1010-1010 10")
	ba2 := bitarray.MustParse("1111-0000")
	fmt.Printf("% b\n", ba1.AndAt(0, ba2))
	fmt.Printf("% b\n", ba1.AndAt(4, ba2))
	fmt.Printf("% b\n", ba1.AndAt(10, ba2))

	// Output:
	// 10100000 10101010 10
	// 10101010 00001010 10
	// 10101010 10101000 00
}

func ExampleBitArray_OrAt() {
	ba1 := bitarray.MustParse("1010-1010 1010-1010 10")
	ba2 := bitarray.MustParse("1111-0000")
	fmt.Printf("% b\n", ba1.OrAt(0, ba2))
	fmt.Printf("% b\n", ba1.OrAt(4, ba2))
	fmt.Printf("% b\n", ba1.OrAt(10, ba2))

	// Output:
	// 11111010 10101010 10
	// 10101111 10101010 10
	// 10101010 10111110 10
}

func ExampleBitArray_XorAt() {
	ba1 := bitarray.MustParse("1010-1010 1010-1010 10")
	ba2 := bitarray.MustParse("1111-0000")
	fmt.Printf("% b\n", ba1.XorAt(0, ba2))
	fmt.Printf("% b\n", ba1.XorAt(4, ba2))
	fmt.Printf("% b\n", ba1.XorAt(10, ba2))

	// Output:
	// 01011010 10101010 10
	// 10100101 10101010 10
	// 10101010 10010110 10
}
