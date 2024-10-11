// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"fmt"

	"github.com/tunabay/go-bitarray"
)

func ExampleBitArray_SHA256() {
	ba1 := bitarray.MustParse("011")
	ba2 := bitarray.MustParse("010000")

	fmt.Printf("%x\n", ba1.SHA256())
	fmt.Printf("%x\n", ba2.SHA256())

	// Output:
	// 1f7794d4b0b67d3a6edcd17aba2144a95828032f7943ed26bf0c7c7628945f48
	// 5ef0224f79737bda30562831152184b939cc43fcd40d09f4945e081a39c6d542
}

func ExampleBitArray_SHA224() {
	ba1 := bitarray.MustParse("10")
	ba2 := bitarray.MustParse("100")

	fmt.Printf("%x\n", ba1.SHA224())
	fmt.Printf("%x\n", ba2.SHA224())

	// Output:
	// ef9c947a47bb9311a0f2b8939cfc12090554868b3b64d8f71e6442f3
	// 4f2ec61c914dce56c3fe5067aa184125ab126c39edb8bf64f58bdccd
}
