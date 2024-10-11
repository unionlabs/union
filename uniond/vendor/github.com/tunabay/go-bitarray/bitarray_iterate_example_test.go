// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"fmt"

	"github.com/tunabay/go-bitarray"
)

func ExampleBitArray_Iterate() {
	ba := bitarray.MustParse("1111 0101 0000")

	fn := func(i, b int) error {
		fmt.Printf("%d: %d\n", i, b)
		if i == 10 {
			return bitarray.BreakIteration
		}
		return nil
	}
	if err := ba.Iterate(fn); err != nil {
		panic(err)
	}

	// Output:
	// 0: 1
	// 1: 1
	// 2: 1
	// 3: 1
	// 4: 0
	// 5: 1
	// 6: 0
	// 7: 1
	// 8: 0
	// 9: 0
	// 10: 0
}

func ExampleBitArray_Iterate_error() {
	ba := bitarray.MustParse("000010")

	fn := func(i, b int) error {
		if b == 1 {
			return fmt.Errorf("unexpected bit 1 at %d", i)
		}
		fmt.Printf("%d: %d\n", i, b)
		return nil
	}
	if err := ba.Iterate(fn); err != nil {
		fmt.Printf("got error: %s\n", err)
	}

	// Output:
	// 0: 0
	// 1: 0
	// 2: 0
	// 3: 0
	// got error: unexpected bit 1 at 4
}
