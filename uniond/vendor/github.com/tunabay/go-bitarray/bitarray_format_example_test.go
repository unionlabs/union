// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"fmt"

	"github.com/tunabay/go-bitarray"
)

func ExampleBitArray_Format_printf() {
	ba := bitarray.MustParse("1111-0000 1010-0011 111")

	fmt.Printf("%b\n", ba)      // 1111000010100011111
	fmt.Printf("% b\n", ba)     // 11110000 10100011 111
	fmt.Printf("%#b\n", ba)     // 1111-0000 1010-0011 111
	fmt.Printf("%o\n", ba)      // 7412174
	fmt.Printf("%+x\n", ba)     // f0a3e(pad=1)
	fmt.Printf("%q\n", ba)      // "1111000010100011111"
	fmt.Printf("[%24b]\n", ba)  // [     1111000010100011111]
	fmt.Printf("[%-24b]\n", ba) // [1111000010100011111     ]

	// Output:
	// 1111000010100011111
	// 11110000 10100011 111
	// 1111-0000 1010-0011 111
	// 7412174
	// f0a3e(pad=1)
	// "1111000010100011111"
	// [     1111000010100011111]
	// [1111000010100011111     ]
}
