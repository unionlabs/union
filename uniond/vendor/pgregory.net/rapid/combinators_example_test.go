// Copyright 2020 Gregory Petrosyan <gregory.petrosyan@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package rapid_test

import (
	"fmt"
	"strconv"

	"pgregory.net/rapid"
)

func ExampleCustom() {
	type point struct {
		x int
		y int
	}

	gen := rapid.Custom(func(t *rapid.T) point {
		return point{
			x: rapid.IntRange(-100, 100).Draw(t, "x"),
			y: rapid.IntRange(-100, 100).Draw(t, "y"),
		}
	})

	for i := 0; i < 5; i++ {
		fmt.Println(gen.Example(i))
	}
	// Output:
	// {-1 23}
	// {-3 -50}
	// {0 94}
	// {-2 -50}
	// {11 -57}
}

func recursive() *rapid.Generator[any] {
	return rapid.OneOf(
		rapid.Bool().AsAny(),
		rapid.SliceOfN(rapid.Deferred(recursive), 1, 2).AsAny(),
	)
}

func ExampleDeferred() {
	gen := recursive()
	for i := 0; i < 5; i++ {
		fmt.Println(gen.Example(i))
	}
	// Output:
	// [[[[false] false]]]
	// false
	// [[true [[[true]]]]]
	// true
	// true
}

func ExampleMap() {
	gen := rapid.Map(rapid.Int(), strconv.Itoa)
	for i := 0; i < 5; i++ {
		fmt.Printf("%#v\n", gen.Example(i))
	}
	// Output:
	// "-3"
	// "-186981"
	// "4"
	// "-2"
	// "43"
}

func ExampleJust() {
	gen := rapid.Just(42)

	for i := 0; i < 5; i++ {
		fmt.Println(gen.Example(i))
	}
	// Output:
	// 42
	// 42
	// 42
	// 42
	// 42
}

func ExampleSampledFrom() {
	gen := rapid.SampledFrom([]int{1, 2, 3})

	for i := 0; i < 5; i++ {
		fmt.Println(gen.Example(i))
	}
	// Output:
	// 2
	// 3
	// 2
	// 3
	// 1
}

func ExamplePermutation() {
	gen := rapid.Permutation([]int{1, 2, 3})

	for i := 0; i < 5; i++ {
		fmt.Println(gen.Example(i))
	}
	// Output:
	// [2 3 1]
	// [3 2 1]
	// [2 1 3]
	// [3 2 1]
	// [1 2 3]
}

func ExampleOneOf() {
	gen := rapid.OneOf(rapid.Int32Range(1, 10).AsAny(), rapid.Float32Range(100, 1000).AsAny())

	for i := 0; i < 5; i++ {
		fmt.Println(gen.Example(i))
	}
	// Output:
	// 997.0737
	// 10
	// 475.3125
	// 2
	// 9
}

func ExamplePtr() {
	gen := rapid.Ptr(rapid.Int(), true)

	for i := 0; i < 5; i++ {
		v := gen.Example(i)
		if v == nil {
			fmt.Println("<nil>")
		} else {
			fmt.Println("(*int)", *v)
		}
	}
	// Output:
	// (*int) 1
	// (*int) -3
	// <nil>
	// (*int) 590
	// <nil>
}
