// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"bytes"
	"fmt"
	"io"

	"github.com/tunabay/go-bitarray"
)

func ExampleBuilder() {
	b := bitarray.NewBuilder()

	// Add bits with various methods.
	b.WriteBitsFromBytes([]byte{0b_1100_0111}, 2, 3) // 000
	b.WriteByte(0xFF)                                // 11111111
	b.WriteBit(0)                                    // 0
	b.WriteBitArray(bitarray.MustParse("111000111")) // 111000111
	b.WriteByteBits([]byte{0, 1, 0, 1})              // 0101

	// Build a BitArray containing the accumulated bits.
	ba := b.BitArray()
	fmt.Printf("% b\n", ba)

	// Output:
	// 00011111 11101110 00111010 1
}

func ExampleNewBuilder() {
	// Create an empty builder and add bitarrays.
	b1 := bitarray.NewBuilder()
	b1.WriteBitArray(bitarray.MustParse("1111"))
	b1.WriteBitArray(bitarray.MustParse("000"))
	fmt.Printf("%b\n", b1.BitArray())

	// Same as above.
	b2 := bitarray.NewBuilder(
		bitarray.MustParse("1111"),
		bitarray.MustParse("000"),
	)
	fmt.Printf("%b\n", b2.BitArray())

	// Output:
	// 1111000
	// 1111000
}

func ExampleBuilder_BitArray() {
	b := bitarray.NewBuilder()

	b.WriteBitArray(bitarray.MustParse("1111-00"))
	b.WriteBitArray(bitarray.MustParse("1100-11"))

	ba := b.BitArray()
	fmt.Printf("% b\n", ba)

	// Output:
	// 11110011 0011
}

func ExampleBuilder_String() {
	b := bitarray.NewBuilder()

	b.WriteBitArray(bitarray.MustParse("1111-00"))
	b.WriteBitArray(bitarray.MustParse("1100-11"))

	fmt.Println(b.String())
	fmt.Println(b.BitArray().String())

	// Output:
	// 111100110011
	// 111100110011
}

func ExampleBuilder_Reset() {
	b := bitarray.NewBuilder()

	b.WriteBitArray(bitarray.MustParse("1111-00"))
	b.WriteBitArray(bitarray.MustParse("1100-11"))

	b.Reset()

	b.WriteBitArray(bitarray.MustParse("111"))

	fmt.Println(b)

	// Output:
	// 111
}

func ExampleBuilder_Len() {
	b := bitarray.NewBuilder()
	b.WriteBitArray(bitarray.MustParse("1111-00"))
	b.WriteBitArray(bitarray.MustParse("1100-11"))

	fmt.Println(b.Len())

	// Output:
	// 12
}

func ExampleBuilder_WriteBitsFromBytes() {
	b := bitarray.NewBuilder()

	src := []byte{0xF3, 0x50} // 1111-0011 0101-0000

	b.WriteBitsFromBytes(src, 3, 4)  // 1001
	b.WriteBitsFromBytes(src, 8, 0)  // (empty)
	b.WriteBitsFromBytes(src, 5, 6)  // 011010
	b.WriteBitsFromBytes(src, 11, 3) // 100

	fmt.Println(b)

	// Output:
	// 1001011010100
}

func ExampleBuilder_Write() {
	b := bitarray.NewBuilder()

	src := []byte{0xF0, 0xF0} // 1111-0000 1111-0000
	n, err := b.Write(src)
	if err != nil {
		panic(err)
	}

	fmt.Println(n)
	fmt.Println(b)

	// Output:
	// 2
	// 1111000011110000
}

func ExampleBuilder_Write_copy() {
	b := bitarray.NewBuilder(bitarray.MustParse("0000"))

	src := bytes.NewReader([]byte{0xFF, 0x00, 0xAA})
	_, err := io.Copy(b, src) // uses b.Write() via src.WriteTo()
	if err != nil {
		panic(err)
	}

	ba := b.BitArray()
	fmt.Printf("% b\n", ba)

	// Output:
	// 00001111 11110000 00001010 1010
}

func ExampleBuilder_ReadFrom() {
	b := bitarray.NewBuilder(bitarray.MustParse("0000"))

	src := bytes.NewReader([]byte{0xFF, 0x00, 0xAA})
	n, err := b.ReadFrom(src)
	if err != nil {
		panic(err)
	}
	fmt.Println(n)

	ba := b.BitArray()
	fmt.Printf("% b\n", ba)

	// Output:
	// 3
	// 00001111 11110000 00001010 1010
}

func ExampleBuilder_WriteBit() {
	b := bitarray.NewBuilder()

	b.WriteBit(0)
	b.WriteBit(1)
	b.WriteBit(0)
	b.WriteBit(1)

	fmt.Println(b)

	// Output:
	// 0101
}

func ExampleBuilder_WriteBitArray() {
	b := bitarray.NewBuilder()

	b.WriteBitArray(bitarray.MustParse("111"))
	b.WriteBitArray(bitarray.MustParse("0000"))

	fmt.Println(b)

	// Output:
	// 1110000
}

func ExampleBuilder_WriteByteBits() {
	b := bitarray.NewBuilder()

	src := []byte{1, 0, 1, 0, 1, 0}
	b.WriteByteBits(src)

	fmt.Println(b)

	// Output:
	// 101010
}
