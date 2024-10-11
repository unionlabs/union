// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"fmt"

	"github.com/tunabay/go-bitarray"
)

func ExampleBitArray_SHA1() {
	ba1 := bitarray.MustParse("01")
	ba2 := bitarray.MustParse("01001")

	fmt.Printf("%x\n", ba1.SHA1())
	fmt.Printf("%x\n", ba2.SHA1())

	// Output:
	// ec6b39952e1a3ec3ab3507185cf756181c84bbe2
	// 3320540d1c28b96ddd03eee1b186a8f2ae883fbe
}
