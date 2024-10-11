// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"fmt"

	"github.com/tunabay/go-bitarray"
)

func ExampleBuffer_Slice() {
	ba := bitarray.MustParse("0011-1010 0110-1111 110")
	buf := bitarray.NewBufferFromBitArray(ba)

	buf2 := buf.Slice(4, 14)
	fmt.Println(buf2)
	buf3 := buf.Slice(9, 13)
	fmt.Println(buf3)

	// Output:
	// 1010011011
	// 1101
}

func ExampleBuffer_SliceToEnd() {
	ba := bitarray.MustParse("0011-1010 0110-1")
	buf := bitarray.NewBufferFromBitArray(ba)

	fmt.Println(buf.SliceToEnd(4))
	fmt.Println(buf.SliceToEnd(9))

	// Output:
	// 101001101
	// 1101
}

func ExampleBuffer_Slice_update() {
	ba := bitarray.MustParse("1000-0000 0000-01")
	buf := bitarray.NewBufferFromBitArray(ba)

	sub := buf.Slice(6, 10)
	fmt.Println(buf, sub)

	sub.FillBitsAt(0, 4, 1)
	fmt.Println(buf, sub)

	buf.FillBitsAt(7, 2, 0)
	fmt.Println(buf, sub)

	// Output:
	// 10000000000001 0000
	// 10000011110001 1111
	// 10000010010001 1001
}
