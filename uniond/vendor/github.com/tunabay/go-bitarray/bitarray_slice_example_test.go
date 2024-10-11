// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"fmt"

	"github.com/tunabay/go-bitarray"
)

func ExampleBitArray_Slice() {
	ba := bitarray.MustParse("0011-1010 0110-1111 110")
	fmt.Printf("% s\n", ba.Slice(4, 14))
	fmt.Printf("% s\n", ba.Slice(9, 13))

	// Output:
	// 10100110 11
	// 1101
}

func ExampleBitArray_SliceToEnd() {
	ba := bitarray.MustParse("0011-1010 01101")
	fmt.Printf("% b\n", ba.SliceToEnd(4))
	fmt.Printf("% b\n", ba.SliceToEnd(9))

	// Output:
	// 10100110 1
	// 1101
}

func ExampleBitArray_ToWidth() {
	ba := bitarray.MustParse("1010-1111 0000-11")
	fmt.Printf("% s\n", ba.ToWidth(7, bitarray.AlignLeft))
	fmt.Printf("% s\n", ba.ToWidth(7, bitarray.AlignRight))
	fmt.Printf("% s\n", ba.ToWidth(20, bitarray.AlignLeft))
	fmt.Printf("% s\n", ba.ToWidth(20, bitarray.AlignRight))

	// Output:
	// 1010111
	// 1000011
	// 10101111 00001100 0000
	// 00000010 10111100 0011
}

func ExampleBitArray_TrimPrefix() {
	ba1 := bitarray.MustParse("1010-1011 0011-0011 01")
	ba2 := bitarray.MustParse("1010-1111 1111")
	ba3 := bitarray.MustParse("1010-10")
	prefix := bitarray.MustParse("1010-10")
	fmt.Printf("% s\n", ba1.TrimPrefix(prefix))
	fmt.Printf("% s\n", ba2.TrimPrefix(prefix))
	fmt.Printf("[% s]\n", ba3.TrimPrefix(prefix))

	// Output:
	// 11001100 1101
	// 10101111 1111
	// []
}

func ExampleBitArray_TrimSuffix() {
	ba1 := bitarray.MustParse("1010-1011 0011-00")
	ba2 := bitarray.MustParse("1111-1000 11")
	ba3 := bitarray.MustParse("1100")
	suffix := bitarray.MustParse("1100")
	fmt.Printf("% s\n", ba1.TrimSuffix(suffix))
	fmt.Printf("% s\n", ba2.TrimSuffix(suffix))
	fmt.Printf("[% s]\n", ba3.TrimSuffix(suffix))

	// Output:
	// 10101011 00
	// 11111000 11
	// []
}

func ExampleBitArray_TrimLeadingZeros() {
	ba1 := bitarray.MustParse("0000-0010 1100-11")
	ba2 := bitarray.MustParse("1000-0000 0")
	ba3 := bitarray.MustParse("0000-0000 0000-0000 0000")
	fmt.Printf("% s\n", ba1.TrimLeadingZeros())
	fmt.Printf("% s\n", ba2.TrimLeadingZeros())
	fmt.Printf("[% s]\n", ba3.TrimLeadingZeros())

	// Output:
	// 10110011
	// 10000000 0
	// []
}

func ExampleBitArray_TrimTrailingZeros() {
	ba1 := bitarray.MustParse("1010-0110 000")
	ba2 := bitarray.MustParse("0000-0000 01")
	ba3 := bitarray.MustParse("0000-0000 0000-0000 0000")
	fmt.Printf("% s\n", ba1.TrimTrailingZeros())
	fmt.Printf("% s\n", ba2.TrimTrailingZeros())
	fmt.Printf("[% s]\n", ba3.TrimTrailingZeros())

	// Output:
	// 1010011
	// 00000000 01
	// []
}
