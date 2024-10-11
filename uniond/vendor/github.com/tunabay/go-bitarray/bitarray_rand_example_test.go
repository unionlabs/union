// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"fmt"
	"math/rand"

	"github.com/tunabay/go-bitarray"
)

func ExampleRand() {
	ba, err := bitarray.Rand(42)
	if err != nil {
		panic(err)
	}

	fmt.Printf("% s\n", ba)
}

func ExamplePseudoRand() {
	rand.Seed(1234)

	ba := bitarray.PseudoRand(42, nil)
	fmt.Printf("% s\n", ba)

	// Output:
	// 11000000 00001110 01011101 01100111 11000010 01
}

func ExamplePseudoRand_customSource() {
	myRand := rand.New(rand.NewSource(1234))

	ba0 := bitarray.PseudoRand(42, myRand)
	ba1 := bitarray.PseudoRand(13, myRand)
	fmt.Printf("% s\n", ba0)
	fmt.Printf("% s\n", ba1)

	// Output:
	// 11000000 00001110 01011101 01100111 11000010 01
	// 01010011 10001
}
