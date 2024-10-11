// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"fmt"

	"github.com/tunabay/go-bitarray"
)

func ExampleSlice_Sort() {
	bas := bitarray.Slice{
		bitarray.MustParse("1111-1111 0000"),
		bitarray.MustParse("1111-1111 0"),
		bitarray.MustParse("0000-0000 0000"),
		bitarray.MustParse("0"),
		bitarray.MustParse("0101-00"),
	}

	bas.Sort()
	for i, ba := range bas {
		fmt.Printf("%d: % b\n", i, ba)
	}

	// Output:
	// 0: 0
	// 1: 00000000 0000
	// 2: 010100
	// 3: 11111111 0
	// 4: 11111111 0000
}

func ExampleSlice_Search() {
	bas := bitarray.Slice{
		bitarray.MustParse("1111-1111 0000"),
		bitarray.MustParse("1111-1111 0"),
		bitarray.MustParse("0000-0000 0000"),
		bitarray.MustParse("0"),
		bitarray.MustParse("0101-00"),
	}

	bas.Sort()

	x := bitarray.MustParse("0o776")
	idx := bas.Search(x)
	fmt.Printf("%d: %b (%o)\n", idx, bas[idx], bas[idx])

	// Output:
	// 3: 111111110 (776)
}
