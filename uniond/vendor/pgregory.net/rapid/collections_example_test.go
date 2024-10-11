// Copyright 2020 Gregory Petrosyan <gregory.petrosyan@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package rapid_test

import (
	"fmt"

	"pgregory.net/rapid"
)

func ExampleSliceOf() {
	gen := rapid.SliceOf(rapid.Int())

	for i := 0; i < 5; i++ {
		fmt.Println(gen.Example(i))
	}
	// Output:
	// [1 -1902 7 -236 14 -433 -1572631 -1 4219826 -50 1414 -3890044391133 -9223372036854775808 5755498240 -10 680558 10 -80458281 0 -27]
	// [-3 -2 -1 -3 -2172865589 -5 -2 -2503553836720]
	// [4 308 -2 21 -5843 3 1 78 6129321692 -59]
	// [590 -131 -15 -769 16 -1 14668 14 -1 -58784]
	// []
}

func ExampleSliceOfN() {
	gen := rapid.SliceOfN(rapid.Int(), 5, 5)

	for i := 0; i < 5; i++ {
		fmt.Println(gen.Example(i))
	}
	// Output:
	// [1 -1902 7 -236 14]
	// [-3 -2 -1 -3 -2172865589]
	// [4 308 -2 21 -5843]
	// [590 -131 -15 -769 16]
	// [4629136912 270 141395 -129322425838843911 -7]
}

func ExampleSliceOfDistinct() {
	gen := rapid.SliceOfDistinct(rapid.IntMin(0), func(i int) int { return i % 2 })

	for i := 0; i < 5; i++ {
		fmt.Println(gen.Example(i))
	}
	// Output:
	// [1]
	// [2 1]
	// [4 1]
	// [590]
	// []
}

func ExampleSliceOfNDistinct() {
	gen := rapid.SliceOfNDistinct(rapid.IntMin(0), 2, 2, func(i int) int { return i % 2 })

	for i := 0; i < 5; i++ {
		fmt.Println(gen.Example(i))
	}
	// Output:
	// [4219826 49]
	// [2 1]
	// [4 1]
	// [0 58783]
	// [4629136912 141395]
}

func ExampleMapOf() {
	gen := rapid.MapOf(rapid.Int(), rapid.StringMatching(`[a-z]+`))

	for i := 0; i < 5; i++ {
		fmt.Println(gen.Example(i))
	}
	// Output:
	// map[1:nhlgqwasbggbaociac 561860:r]
	// map[-3752:pizpv -3:bacuabp 0:bi]
	// map[-33086515648293:gewf -264276:b -1313:a -258:v -4:b -2:fdhbzcz 4:ubfsdbowrja 1775:tcozav 8334:lvcprss 376914:braigey]
	// map[-350:h 590:coaaamcasnapgaad]
	// map[]
}

func ExampleMapOfN() {
	gen := rapid.MapOfN(rapid.Int(), rapid.StringMatching(`[a-z]+`), 5, 5)

	for i := 0; i < 5; i++ {
		fmt.Println(gen.Example(i))
	}
	// Output:
	// map[-130450326583:bd -2983:bbdbcs 1:nhlgqwasbggbaociac 31:kmdnpmcbuagzr 561860:r]
	// map[-82024404:d -3752:pizpv -3:bacuabp 0:bi 179745:rzkneb]
	// map[-33086515648293:gewf -258:v 4:ubfsdbowrja 1775:tcozav 8334:lvcprss]
	// map[-4280678227:j -25651:aafmd -3308:o -350:h 590:coaaamcasnapgaad]
	// map[-9614404661322:gsb -378:y 2:paai 4629136912:otg 1476419818092:qign]
}

func ExampleMapOfValues() {
	gen := rapid.MapOfValues(rapid.StringMatching(`[a-z]+`), func(s string) int { return len(s) })

	for i := 0; i < 5; i++ {
		fmt.Println(gen.Example(i))
	}
	// Output:
	// map[2:dr 7:xguehfc 11:sbggbaociac]
	// map[2:bp 5:jarxz 6:ebzkwa]
	// map[1:j 2:aj 3:gjl 4:vayt 5:eeeqa 6:riacaa 7:stcozav 8:mfdhbzcz 9:fxmcadagf 10:bgsbraigey 15:gxongygnxqlovib]
	// map[2:ub 8:waraafmd 10:bfiqcaxazu 16:rjgqimcasnapgaad 17:gckfbljafcedhcvfc]
	// map[]
}

func ExampleMapOfNValues() {
	gen := rapid.MapOfNValues(rapid.StringMatching(`[a-z]+`), 5, 5, func(s string) int { return len(s) })

	for i := 0; i < 5; i++ {
		fmt.Println(gen.Example(i))
	}
	// Output:
	// map[1:s 2:dr 3:anc 7:xguehfc 11:sbggbaociac]
	// map[1:b 2:bp 4:ydag 5:jarxz 6:ebzkwa]
	// map[1:j 3:gjl 5:eeeqa 7:stcozav 9:fxmcadagf]
	// map[2:ub 8:waraafmd 10:bfiqcaxazu 16:rjgqimcasnapgaad 17:gckfbljafcedhcvfc]
	// map[1:k 2:ay 3:wzb 4:dign 7:faabhcb]
}
