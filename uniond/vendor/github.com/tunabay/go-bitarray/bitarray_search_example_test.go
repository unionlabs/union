// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"fmt"

	"github.com/tunabay/go-bitarray"
)

func ExampleBitArray_HasPrefix() {
	ba := bitarray.MustParse("1010-1111 0000-1111 10")

	fmt.Println(ba.HasPrefix(bitarray.MustParse("10101")))
	fmt.Println(ba.HasPrefix(bitarray.MustParse("111")))

	// Output:
	// true
	// false
}

func ExampleBitArray_HasSuffix() {
	ba := bitarray.MustParse("1010-1111 0000-1111 10")

	fmt.Println(ba.HasSuffix(bitarray.MustParse("11110")))
	fmt.Println(ba.HasSuffix(bitarray.MustParse("111")))

	// Output:
	// true
	// false
}

func ExampleBitArray_Index() {
	haystack := bitarray.MustParse("0010-1011 0001-0101 0101-0111 111")
	needle := bitarray.MustParse("1010")

	fmt.Println(haystack.Index(needle))

	// Output:
	// 2
}

func ExampleBitArray_LastIndex() {
	haystack := bitarray.MustParse("0010-1011 0001-0101 0101-0111 111")
	needle := bitarray.MustParse("1010")

	fmt.Println(haystack.LastIndex(needle))

	// Output:
	// 17
}

func ExampleBitArray_AllIndex() {
	haystack := bitarray.MustParse("0010-1011 0001-0101 0101-0111 111")
	needle := bitarray.MustParse("1010")

	fmt.Println(haystack.AllIndex(needle))

	// Output:
	// [2 11 13 15 17]
}
