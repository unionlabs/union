// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"fmt"

	"github.com/tunabay/go-bitarray"
)

func ExampleBuffer() {
	buf := bitarray.NewBuffer(32)
	fmt.Println(buf)
	buf.PutBitAt(0, 1)
	buf.PutBitAt(1, 1)
	fmt.Println(buf)
	buf.PutBitArrayAt(8, bitarray.MustParse("1010101"))
	fmt.Println(buf)
	buf.FillBitsAt(16, 4, 1)
	fmt.Println(buf)
	buf.PutBitArrayAt(24, bitarray.MustParse("1111-0000"))
	fmt.Println(buf)
	buf.ToggleBitsAt(24, 8)
	fmt.Println(buf)

	fmt.Printf("% b\n", buf.BitArray())

	// Output:
	// 00000000000000000000000000000000
	// 11000000000000000000000000000000
	// 11000000101010100000000000000000
	// 11000000101010101111000000000000
	// 11000000101010101111000011110000
	// 11000000101010101111000000001111
	// 11000000 10101010 11110000 00001111
}

func ExampleNewBuffer() {
	buf0 := bitarray.NewBuffer(0)
	buf64 := bitarray.NewBuffer(64)

	fmt.Println(buf0)
	fmt.Println(buf64)

	// Output:
	// 0000000000000000000000000000000000000000000000000000000000000000
}

func ExampleNewBufferFromBitArray() {
	ba := bitarray.MustParse("1111-1111 0000-0000 1111-1111")
	buf := bitarray.NewBufferFromBitArray(ba)

	fmt.Println(buf)

	// Output:
	// 111111110000000011111111
}

func ExampleNewBufferFromByteSlice() {
	b := []byte{0b_0000_1111, 0b_1111_0000}
	buf := bitarray.NewBufferFromByteSlice(b)

	fmt.Printf("b=%08b, buf=% b\n", b, buf)
	buf.PutBitAt(0, 1)
	buf.PutBitAt(15, 1)
	fmt.Printf("b=%08b, buf=% b\n", b, buf)

	// Output:
	// b=[00001111 11110000], buf=00001111 11110000
	// b=[10001111 11110001], buf=10001111 11110001
}

func ExampleNewBufferFromByteSlicePartial() {
	b := []byte{0b_0000_1111, 0b_1111_0000}
	buf := bitarray.NewBufferFromByteSlicePartial(b, 4, 6)

	fmt.Printf("b=%08b, buf=%b\n", b, buf)
	buf.PutBitAt(0, 0)
	buf.PutBitAt(1, 0)
	buf.PutBitAt(5, 0)
	fmt.Printf("b=%08b, buf=%b\n", b, buf)

	// Output:
	// b=[00001111 11110000], buf=111111
	// b=[00000011 10110000], buf=001110
}

func ExampleBuffer_Len() {
	buf := bitarray.NewBuffer(4096)

	fmt.Println(buf.Len())

	// Output:
	// 4096
}

func ExampleBuffer_FillBits() {
	buf := bitarray.NewBuffer(12)

	buf.Slice(5, 10).FillBits(1)
	fmt.Printf("% b\n", buf)

	// Output:
	// 00000111 1100
}
