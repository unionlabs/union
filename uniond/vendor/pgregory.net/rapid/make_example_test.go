// Copyright 2022 Gregory Petrosyan <gregory.petrosyan@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package rapid_test

import (
	"fmt"

	"pgregory.net/rapid"
)

func ExampleMake() {
	gen := rapid.Make[map[int]bool]()

	for i := 0; i < 5; i++ {
		fmt.Println(gen.Example(i))
	}
	// Output:
	// map[-433:true -261:false -53:false -23:false 1:true 184:false]
	// map[-3:true 0:true]
	// map[4:true]
	// map[-359:true -154:true -71:true -17:false -1:false 590:false 22973756520:true]
	// map[]
}

type nodeValue int

type tree struct {
	Value       nodeValue
	Left, Right *tree
}

func (t *tree) String() string {
	if t == nil {
		return "nil"
	}
	return fmt.Sprintf("(%s %v %s)", t.Left.String(), t.Value, t.Right.String())
}

func ExampleMake_tree() {
	gen := rapid.Make[*tree]()

	for i := 0; i < 5; i++ {
		fmt.Println(gen.Example(i))
	}
	// Output:
	// (nil 1 (nil 184 nil))
	// (((nil -1 (((((nil -485 ((nil -2 ((((nil -5 nil) -9898554875447 nil) -34709387 ((nil 50440 nil) 113 (((((nil -442 nil) -66090341586 nil) 179745 nil) 494 (((nil -2 nil) 543360606020 nil) 15261837 nil)) -1778 nil))) -21034573818 nil)) -5 nil)) 15606609 nil) 882666 (nil 3 nil)) -12 (nil -2 ((nil 1 nil) -2 (((nil 11 nil) -187307 ((nil -198 (nil -6895 nil)) 12027 (nil -539313 nil))) 1532 (nil 6 nil))))) 1745354 nil)) -2 nil) -3 nil)
	// nil
	// (((nil -15 (nil 6598 nil)) -131 (nil 317121006373596 ((nil 14 ((nil -9223372036854775808 nil) 1 nil)) 14668 nil))) 590 nil)
	// nil
}
