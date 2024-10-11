// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"fmt"

	"github.com/tunabay/go-bitarray"
)

func ExampleBuffer_BitAt() {
	ba := bitarray.MustParse("1100-10")
	buf := bitarray.NewBufferFromBitArray(ba)

	fmt.Println(buf.BitAt(0))
	fmt.Println(buf.BitAt(3))
	fmt.Println(buf.BitAt(5))

	// Output:
	// 1
	// 0
	// 0
}

func ExampleBuffer_PutBitAt() {
	ba := bitarray.MustParse("11110000")
	buf := bitarray.NewBufferFromBitArray(ba)

	fmt.Println(buf)
	buf.PutBitAt(0, 0)
	buf.PutBitAt(1, 1)
	buf.PutBitAt(7, 1)
	fmt.Println(buf)

	// Output:
	// 11110000
	// 01110001
}

func ExampleBuffer_BitArrayAt() {
	ba := bitarray.MustParse("1100-1010 0000-1111")
	buf := bitarray.NewBufferFromBitArray(ba)

	fmt.Println(buf.BitArrayAt(0, 6))
	fmt.Println(buf.BitArrayAt(4, 10))

	// Output:
	// 110010
	// 1010000011
}

func ExampleBuffer_PutBitArrayAt() {
	ba := bitarray.MustParse("1111-0000 0000-1111")
	buf := bitarray.NewBufferFromBitArray(ba)

	fmt.Println(buf)
	buf.PutBitArrayAt(2, bitarray.MustParse("0101"))
	buf.PutBitArrayAt(10, bitarray.MustParse("0000"))
	buf.PutBitArrayAt(6, bitarray.MustParse("1111"))
	fmt.Println(buf)

	// Output:
	// 1111000000001111
	// 1101011111000011
}

func ExampleBuffer_ByteAt() {
	ba := bitarray.MustParse("1100-1010 0011-1111")
	buf := bitarray.NewBufferFromBitArray(ba)

	fmt.Printf("%08b\n", buf.ByteAt(2))
	fmt.Printf("%08b\n", buf.ByteAt(6))

	// Output:
	// 00101000
	// 10001111
}

func ExampleBuffer_PutByteAt() {
	ba := bitarray.MustParse("0000-1010 0011-11")
	buf := bitarray.NewBufferFromBitArray(ba)

	fmt.Println(buf)
	buf.PutByteAt(2, 0xff)
	fmt.Println(buf)
	buf.PutByteAt(6, 0x00)
	fmt.Println(buf)

	// Output:
	// 00001010001111
	// 00111111111111
	// 00111100000000
}

func ExampleBuffer_RawBytes() {
	ba := bitarray.MustParse("1111-0000 1010-1010 1111-1111 11")
	buf := bitarray.NewBufferFromBitArray(ba)

	// not byte-aligned, copied, 0-padded
	fmt.Printf("%08b\n", buf.Slice(3, 24).RawBytes())
	// byte-aligned, not copied, not 0-padded
	fmt.Printf("%08b\n", buf.Slice(3, 24).Slice(5, 16).RawBytes())

	// Output:
	// [10000101 01010111 11111000]
	// [10101010 11111111]
}

func ExampleBuffer_Bytes() {
	ba := bitarray.MustParse("1100-1010 1010-1111 0101-11")
	buf := bitarray.NewBufferFromBitArray(ba)

	fmt.Printf("%08b\n", buf.Bytes())
	fmt.Printf("%08b\n", buf.Slice(4, 20).Bytes())

	// Output:
	// [11001010 10101111 01011100]
	// [10101010 11110101]
}

func ExampleBuffer_BytesAt() {
	ba := bitarray.MustParse("1100-1010 0011-1111 1010-0000 0111-1000")
	buf := bitarray.NewBufferFromBitArray(ba)

	fmt.Printf("%08b\n", buf.BytesAt(2, 3))
	fmt.Printf("%08b\n", buf.BytesAt(12, 2))

	// Output:
	// [00101000 11111110 10000001]
	// [11111010 00000111]
}

func ExampleBuffer_PutBytesAt() {
	ba := bitarray.MustParse("1100-1010 0011-1111 1010-0000 0111-1000")
	buf := bitarray.NewBufferFromBitArray(ba)

	fmt.Println(buf)
	buf.PutBytesAt(2, []byte{0x00, 0x00, 0x00})
	fmt.Println(buf)
	buf.PutBytesAt(6, []byte{0xAA, 0xFF, 0xAA})
	fmt.Println(buf)

	// Output:
	// 11001010001111111010000001111000
	// 11000000000000000000000000111000
	// 11000010101010111111111010101000
}
