// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"fmt"

	"github.com/tunabay/go-bitarray"
)

func ExampleParse() {
	baB, err := bitarray.Parse("0101-0101 1111-1111 1101-11")
	if err != nil {
		fmt.Println(err)
	}
	fmt.Printf("% s\n", baB)

	baH, err := bitarray.Parse("0x_ff88_7")
	if err != nil {
		fmt.Println(err)
	}
	fmt.Printf("% s\n", baH)

	baO, err := bitarray.Parse("0o0777")
	if err != nil {
		fmt.Println(err)
	}
	fmt.Printf("% s\n", baO)

	// Output:
	// 01010101 11111111 110111
	// 11111111 10001000 0111
	// 00011111 1111
}

func ExampleParse_multipleTokens() {
	ba, err := bitarray.Parse("00000 + 0xfff + 0b_000 + 0o775")
	if err != nil {
		fmt.Println(err)
	}
	fmt.Printf("% s\n", ba)

	// Output:
	// 00000111 11111111 10001111 11101
}

func ExampleParse_paddingBits() {
	ba0, _ := bitarray.Parse("0xfc0")
	ba1, _ := bitarray.Parse("0xfc0 (pad=1)")
	ba2, _ := bitarray.Parse("0xfc0 (pad=2)")
	ba3, _ := bitarray.Parse("0xfc0 (pad=3)")
	fmt.Printf("% s\n", ba0)
	fmt.Printf("% s\n", ba1)
	fmt.Printf("% s\n", ba2)
	fmt.Printf("% s\n", ba3)

	// Output:
	// 11111100 0000
	// 11111100 000
	// 11111100 00
	// 11111100 0
}

func ExampleMustParse() {
	ba := bitarray.MustParse("1100-1111 1010-0000 1111")
	fmt.Println(ba)

	// Output:
	// 11001111101000001111
}
