// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"fmt"

	"github.com/tunabay/go-bitarray"
)

func ExampleBuffer_CopyBitsFromBytes() {
	ba := bitarray.MustParse("1100-1010 0001-0011 1010-00")
	buf := bitarray.NewBufferFromBitArray(ba)

	fmt.Println(buf)
	buf.CopyBitsFromBytes(2, []byte{0xff, 0x00}, 6, 4)
	fmt.Println(buf)
	buf.CopyBitsFromBytes(6, []byte{0xAA, 0xFF, 0xAA}, 4, 16)
	fmt.Println(buf)

	// Output:
	// 1100101000010011101000
	// 1111001000010011101000
	// 1111001010111111111010
}

func ExampleBuffer_CopyBitsToBytes() {
	ba := bitarray.MustParse("1100-1010 0001")
	buf := bitarray.NewBufferFromBitArray(ba)

	b := make([]byte, 3)

	buf.CopyBitsToBytes(0, b, 0, 12)
	fmt.Printf("%08b\n", b)

	buf.CopyBitsToBytes(4, b, 16, 4)
	buf.CopyBitsToBytes(4, b, 20, 4)
	fmt.Printf("%08b\n", b)

	// Output:
	// [11001010 00010000 00000000]
	// [11001010 00010000 10101010]
}

func ExampleCopyBits() {
	ba := bitarray.MustParse("1100-1010 0001")
	bufSrc := bitarray.NewBufferFromBitArray(ba)
	bufDstL := bitarray.NewBuffer(18)
	bufDstS := bitarray.NewBuffer(7)

	bitarray.CopyBits(bufDstL, bufSrc)
	fmt.Println(bufDstL)
	bitarray.CopyBits(bufDstS, bufSrc)
	fmt.Println(bufDstS)

	// Output:
	// 110010100001000000
	// 1100101
}

func ExampleCopyBitsN() {
	ba := bitarray.MustParse("1100-1010 1111")
	bufSrc := bitarray.NewBufferFromBitArray(ba)
	bufDstL := bitarray.NewBuffer(18)
	bufDstS := bitarray.NewBuffer(7)

	bitarray.CopyBitsN(bufDstL, bufSrc, 10)
	fmt.Println(bufDstL)
	bitarray.CopyBitsN(bufDstS, bufSrc, 10)
	fmt.Println(bufDstS)

	// Output:
	// 110010101100000000
	// 1100101
}

func ExampleCopyBitsPartial() {
	ba := bitarray.MustParse("1101-1110 0011")
	bufSrc := bitarray.NewBufferFromBitArray(ba)
	bufDst := bitarray.NewBuffer(18)

	bitarray.CopyBitsPartial(bufDst, bufSrc, 10, 3, 4)
	fmt.Println(bufDst)

	// Output:
	// 000000000011110000
}
