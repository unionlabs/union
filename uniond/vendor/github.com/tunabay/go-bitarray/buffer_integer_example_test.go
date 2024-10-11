// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"fmt"

	"github.com/tunabay/go-bitarray"
)

func ExampleBuffer_Uint8() {
	b := []byte{0b_1100_1010, 0b_1111_0000}
	buf := bitarray.NewBufferFromByteSlice(b)

	fmt.Println(buf.Slice(4, 12).Uint8()) // 1010 1111
	fmt.Println(buf.Slice(8, 12).Uint8()) // 1111
	fmt.Println(buf.Slice(4, 10).Uint8()) // 1010 11

	// Output:
	// 175
	// 15
	// 43
}

func ExampleBuffer_PutUint8() {
	b := make([]byte, 2)
	buf := bitarray.NewBufferFromByteSlice(b)

	buf.Slice(4, 12).PutUint8(255) // 11111111
	fmt.Printf("%08b\n", b)
	buf.Slice(0, 6).PutUint8(42)   // 101010
	buf.SliceToEnd(13).PutUint8(7) // 111
	fmt.Printf("%08b\n", b)

	// Output:
	// [00001111 11110000]
	// [10101011 11110111]
}

func ExampleBuffer_Uint16() {
	b := []byte{0x12, 0x34, 0x56}
	buf := bitarray.NewBufferFromByteSlice(b)

	fmt.Printf("%04x\n", buf.Slice(4, 20).Uint16())
	fmt.Printf("%04x\n", buf.Slice(8, 20).Uint16())

	// Output:
	// 2345
	// 0345
}

func ExampleBuffer_PutUint16() {
	b := make([]byte, 3)
	buf := bitarray.NewBufferFromByteSlice(b)

	buf.Slice(4, 20).PutUint16(0xffff)
	fmt.Printf("%08b\n", b)
	buf.Slice(10, 16).PutUint16(0b1010)
	fmt.Printf("%08b\n", b)

	// Output:
	// [00001111 11111111 11110000]
	// [00001111 11001010 11110000]
}

func ExampleBuffer_Uint32() {
	b := []byte{0x12, 0x34, 0x56, 0x78, 0x9a}
	buf := bitarray.NewBufferFromByteSlice(b)

	fmt.Printf("%08x\n", buf.Slice(4, 36).Uint32())
	fmt.Printf("%08x\n", buf.Slice(8, 20).Uint32())

	// Output:
	// 23456789
	// 00000345
}

func ExampleBuffer_PutUint32() {
	b := make([]byte, 5)
	buf := bitarray.NewBufferFromByteSlice(b)

	buf.Slice(4, 36).PutUint32(0xff00ffff)
	fmt.Printf("%08b\n", b)
	buf.Slice(16, 28).PutUint32(0xf0c)
	fmt.Printf("%08b\n", b)

	// Output:
	// [00001111 11110000 00001111 11111111 11110000]
	// [00001111 11110000 11110000 11001111 11110000]
}

func ExampleBuffer_Uint64() {
	b := []byte{0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12}
	buf := bitarray.NewBufferFromByteSlice(b)

	fmt.Printf("%016x\n", buf.Slice(4, 68).Uint64())
	fmt.Printf("%016x\n", buf.Slice(12, 24).Uint64())

	// Output:
	// 23456789abcdef01
	// 0000000000000456
}

func ExampleBuffer_PutUint64() {
	b := make([]byte, 9)
	buf := bitarray.NewBufferFromByteSlice(b)

	buf.Slice(4, 68).PutUint64(0x1234567812345678)
	fmt.Printf("%x\n", b)
	buf.Slice(12, 24).PutUint64(0xabc)
	fmt.Printf("%x\n", b)

	// Output:
	// 012345678123456780
	// 012abc678123456780
}
