// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"fmt"

	"github.com/tunabay/go-bitarray"
)

func ExampleBitArray_SHA512() {
	ba1 := bitarray.MustParse("01")
	ba2 := bitarray.MustParse("0100000")

	fmt.Printf("%x\n", ba1.SHA512())
	fmt.Printf("%x\n", ba2.SHA512())

	// Output:
	// a726c0deb12ba0c375cc75ec974f567c08c8d921d78fc8d0a05bfc644d0730ea5716970f2006b4599264d4145dc579b118113ffa1690040e4d98ed2d3450e923
	// acec0655565de641ff3185c686798c1428026673fe2b5deef309987bc991df2b5dadcccfc4eeafe99ff57c97188427e98edafe30bca3f4e4139fd33a9dd6bf79
}

func ExampleBitArray_SHA384() {
	ba1 := bitarray.MustParse("0")
	ba2 := bitarray.MustParse("00")

	fmt.Printf("%x\n", ba1.SHA384())
	fmt.Printf("%x\n", ba2.SHA384())

	// Output:
	// 634aa63038a164ae6c7d48b319f2aca0a107908e548519204c6d72dbeac0fdc3c9246674f98e8fd30221ba986e737d61
	// c6b08368812f4f02aaf84c1b8fcd549f53099816b212fe68cb32f6d73563fae8cec52b96051ade12ba8f3c6a6e98a616
}

func ExampleBitArray_SHA512_256() { //nolint: govet // false positive for _256 suffix
	ba1 := bitarray.MustParse("0000")
	ba2 := bitarray.MustParse("00000")

	fmt.Printf("%x\n", ba1.SHA512_256())
	fmt.Printf("%x\n", ba2.SHA512_256())

	// Output:
	// d164616e829e534122e34d48492b66ce4d4fb39fff688880a67e646c0a98f6ab
	// 849eb77bcea6b85ab720b3788ff4b0c04896e26b902d9040edeebfbd190fc8dd
}
