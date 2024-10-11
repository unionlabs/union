// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"fmt"

	"github.com/tunabay/go-bitarray"
)

func ExampleCompare() {
	x := bitarray.MustParse("11100010 1100111")
	y := bitarray.MustParse("11100010 100")
	fmt.Println(bitarray.Compare(x, y))
	fmt.Println(bitarray.Compare(y, x))
	fmt.Println(bitarray.Compare(x, x))

	// Output:
	// 1
	// -1
	// 0
}

func ExampleBitArray_Equal() {
	ba1 := bitarray.MustParse("1100-0010 001")
	ba2 := bitarray.MustParse("0011-1101 110")
	fmt.Println(ba1.Equal(ba2))
	fmt.Println(ba1.Equal(ba2.Not()))

	// Output:
	// false
	// true
}
