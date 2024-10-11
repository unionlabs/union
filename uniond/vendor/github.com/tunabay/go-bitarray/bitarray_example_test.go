// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"crypto/md5"
	"crypto/sha256"
	"fmt"

	"github.com/tunabay/go-bitarray"
)

func Example_bitLayout() {
	// This example assumes 8-byte data with the following bit layout, and
	// accesses the 5-bit integer X and the 50-bit integer Y in it.
	//
	//   |0              |1              |2              |3              |
	//   |0 1 2 3 4 5 6 7|0 1 2 3 4 5 6 7|0 1 2 3 4 5 6 7|0 1 2 3 4 5 6 7|
	//   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
	//   | 9-bit flag area | 5-bit X | Upper 18 bits of the 50-bit int Y |
	//   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
	//   |               Lower 32 bits of the 50-bit int Y               |
	//   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
	data := make([]byte, 8)
	buf := bitarray.NewBufferFromByteSlice(data)

	// set 9-bit flag area to 110000011
	buf.PutBitAt(0, 1)
	buf.PutBitAt(1, 1)
	buf.PutBitAt(7, 1)
	buf.PutBitAt(8, 1)

	// set 5-bit integer X
	buf.Slice(9, 14).PutUint8(25) // = 0b_11001

	// set 50-bit integer Y
	buf.Slice(14, 64).PutUint64(0x_3_f0ff_f0f0_ff0f)

	// raw bytes updated
	fmt.Printf("%08b\n%08b\n", data[:4], data[4:])

	// read fields
	fmt.Printf("F = %b\n", buf.Slice(0, 9))
	fmt.Printf("X = %d\n", buf.Slice(9, 14).Uint8())
	fmt.Printf("Y = %x\n", buf.SliceToEnd(14).Uint64())

	// Output:
	// [11000001 11100111 11110000 11111111]
	// [11110000 11110000 11111111 00001111]
	// F = 110000011
	// X = 25
	// Y = 3f0fff0f0ff0f
}

func Example_bitArray() {
	// Parse string representation
	ba1, err := bitarray.Parse("111000")
	if err != nil {
		panic(err)
	}
	fmt.Println(ba1)

	// Slice and Repeat
	ba2 := ba1.Slice(2, 5).Repeat(2)
	fmt.Println(ba2)

	// Append
	ba3 := ba2.Append(bitarray.MustParse("101011"))
	fmt.Printf("% b\n", ba3) // alternative formatting

	// Extract bits from []byte across byte boundary
	buf := []byte{0xff, 0x00}
	ba4 := bitarray.NewFromBytes(buf, 4, 7)
	fmt.Println(ba4)

	// Output:
	// 111000
	// 100100
	// 10010010 1011
	// 1111000
}

func ExampleNew() {
	ba := bitarray.New(0, 0, 1, 1, 0, 1)
	fmt.Printf("%s\n", ba)

	// Output:
	// 001101
}

func ExampleNewFromBytes() {
	b := []byte{0b_1010_1111, 0b_0101_0011}
	ba := bitarray.NewFromBytes(b, 2, 12)
	fmt.Printf("% b\n", ba)

	// Output:
	// 10111101 0100
}

func ExampleNewFromByteBits() {
	bits := []byte{0, 1, 1, 0, 0, 0, 1, 1, 1, 1}
	ba := bitarray.NewFromByteBits(bits)
	fmt.Printf("% b\n", ba)

	// Output:
	// 01100011 11
}

func ExampleNewZeroFilled() {
	ba := bitarray.NewZeroFilled(42)
	fmt.Printf("% b\n", ba)

	// Output:
	// 00000000 00000000 00000000 00000000 00000000 00
}

func ExampleNewOneFilled() {
	ba := bitarray.NewOneFilled(28)
	fmt.Printf("% b\n", ba)

	// Output:
	// 11111111 11111111 11111111 1111
}

func ExampleNewByRunLength() {
	ba1 := bitarray.NewByRunLength(1, 2, 3, 4, 5, 6)
	ba2 := bitarray.NewByRunLength(0, 1, 1, 2, 3, 5, 8, 13)
	fmt.Printf("% b\n", ba1)
	fmt.Printf("% b\n", ba2)

	// Output:
	// 01100011 11000001 11111
	// 10110001 11110000 00001111 11111111 1
}

func ExampleBitArray_IsZero() {
	ba1 := bitarray.MustParse("")
	ba2 := bitarray.MustParse("0")
	ba3 := bitarray.MustParse("00000000")
	fmt.Println(ba1.IsZero())
	fmt.Println(ba2.IsZero())
	fmt.Println(ba3.IsZero())

	// Output:
	// true
	// false
	// false
}

func ExampleBitArray_Len() {
	ba1 := bitarray.MustParse("1111-0000 1111-0000 111")
	ba2 := bitarray.MustParse("101")
	ba3 := bitarray.MustParse("")
	fmt.Println(ba1.Len())
	fmt.Println(ba2.Len())
	fmt.Println(ba3.Len())

	// Output:
	// 19
	// 3
	// 0
}

func ExampleBitArray_NumPadding() {
	ba1 := bitarray.MustParse("11110000 11110000")
	ba2 := bitarray.MustParse("11110000 11110000 000")
	ba3 := bitarray.MustParse("11110000 11110000 000000")
	fmt.Println(ba1.NumPadding())
	fmt.Println(ba2.NumPadding())
	fmt.Println(ba3.NumPadding())

	// Output:
	// 0
	// 5
	// 2
}

func ExampleBitArray_String() {
	ba := bitarray.MustParse("0000-1111 0000-1111 0000-1")
	fmt.Printf("%s\n", ba.String())
	fmt.Println(ba)

	// Output:
	// 000011110000111100001
	// 000011110000111100001
}

func ExampleBitArray_Bytes() {
	ba := bitarray.MustParse("11111111 00000000 1110")
	bs, npad := ba.Bytes()
	fmt.Printf("%x, npad=%d\n", bs, npad)

	// Output:
	// ff00e0, npad=4
}

func ExampleBitArray_BitAt() {
	ba := bitarray.MustParse("10001000 10")
	fmt.Println(ba.BitAt(0), ba.BitAt(4), ba.BitAt(8))

	// Output:
	// 1 1 1
}

func ExampleBitArray_Hash_sha256() {
	ba1 := bitarray.MustParse("11111110 11111111 0000")
	ba2 := bitarray.MustParse("11111111 11111111 0000")
	ba3 := bitarray.MustParse("11111111 11111111 00000")
	fmt.Printf("%x\n", ba1.Hash(sha256.New()))
	fmt.Printf("%x\n", ba2.Hash(sha256.New()))
	fmt.Printf("%x\n", ba3.Hash(sha256.New()))

	// Output:
	// 9c5dfdfd1abe7dc4c5018ed54f338bc1c7e9ec70769cd37bda4e826d389f8ba0
	// 42f0862649c6f51d8aaa31d810d9c2e4455917b61b0b17f13fbd66007cb6d75e
	// 074d99c983072816cbc8d980c4a2fd441f6036aca06f83ee14e05cab5afbed85
}

func ExampleBitArray_Hash_md5() {
	ba1 := bitarray.MustParse("11111110 11111111 0000")
	ba2 := bitarray.MustParse("11111111 11111111 0000")
	ba3 := bitarray.MustParse("11111111 11111111 00000")
	fmt.Printf("%x\n", ba1.Hash(md5.New()))
	fmt.Printf("%x\n", ba2.Hash(md5.New()))
	fmt.Printf("%x\n", ba3.Hash(md5.New()))

	// Output:
	// 734a8ff2ae91df5975e32c95cf83d88e
	// 2241281d24ea5e99500ff0b2f925709e
	// 9f51432b4c47d144039b8259564bb89b
}

func ExampleBitArray_MapKey() {
	ba1 := bitarray.MustParse("1010-1100 0")
	ba2 := bitarray.MustParse("1010-1100 00")
	ba3 := bitarray.MustParse("")

	m := make(map[string]string)
	m[ba1.MapKey()] = "ba1"
	m[ba2.MapKey()] = "ba2"
	m[ba3.MapKey()] = "ba3"

	for k, v := range m {
		fmt.Printf("%q -> %s\n", k, v)
	}

	// Unordered output:
	// "\x01\xac\x00" -> ba1
	// "\x02\xac\x00" -> ba2
	// "" -> ba3
}

func ExampleBitArray_ToPadded8() {
	ba1 := bitarray.MustParse("1111")
	ba2 := bitarray.MustParse("1111-1111 1")
	ba3 := bitarray.MustParse("1111-1111 1111-1111")
	fmt.Printf("% b\n", ba1.ToPadded8())
	fmt.Printf("% b\n", ba2.ToPadded8())
	fmt.Printf("% b\n", ba3.ToPadded8())

	// Output:
	// 11110000
	// 11111111 10000000
	// 11111111 11111111
}

func ExampleBitArray_ToPadded64() {
	ba1 := bitarray.MustParse("1111-11")
	ba2 := bitarray.MustParse("0x_ffff_ffff_ffff")
	fmt.Printf("% x\n", ba1.ToPadded64())
	fmt.Printf("% x\n", ba2.ToPadded64())

	// Output:
	// fc000000 00000000
	// ffffffff ffff0000
}

func ExampleBitArray_ToByteBits() {
	ba1 := bitarray.MustParse("101010")
	ba2 := bitarray.MustParse("0000-1111 1111")
	fmt.Printf("%d\n", ba1.ToByteBits())
	fmt.Printf("%d\n", ba2.ToByteBits())

	// Output:
	// [1 0 1 0 1 0]
	// [0 0 0 0 1 1 1 1 1 1 1 1]
}

func ExampleBitArray_ParityBit() {
	ba1 := bitarray.MustParse("0000-000")
	ba2 := bitarray.MustParse("1010-001")
	ba3 := bitarray.MustParse("1111-111")
	fmt.Println(ba1.ParityBit())
	fmt.Println(ba2.ParityBit())
	fmt.Println(ba3.ParityBit())

	// Output:
	// 1
	// 0
	// 0
}

func ExampleBitArray_RepeatEach() {
	ba := bitarray.MustParse("0101-0011 001")
	fmt.Printf("% b\n", ba.RepeatEach(2))
	fmt.Printf("% b\n", ba.RepeatEach(3))
	fmt.Printf("[% b]\n", ba.RepeatEach(0))

	// Output:
	// 00110011 00001111 000011
	// 00011100 01110000 00111111 00000011 1
	// []
}
