// Copyright 2015 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package rlp

import (
	"bytes"
	"fmt"
)

type structWithTail struct {
	A, B uint
	C    []uint `rlp:"tail"`
}

func ExampleDecode_structTagTail() {
	// In this example, the "tail" struct tag is used to decode lists of
	// differing length into a struct.
	var val structWithTail

	err := Decode(bytes.NewReader([]byte{0xC4, 0x01, 0x02, 0x03, 0x04}), &val)
	fmt.Printf("with 4 elements: err=%v val=%v\n", err, val)

	err = Decode(bytes.NewReader([]byte{0xC6, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06}), &val)
	fmt.Printf("with 6 elements: err=%v val=%v\n", err, val)

	// Note that at least two list elements must be present to
	// fill fields A and B:
	err = Decode(bytes.NewReader([]byte{0xC1, 0x01}), &val)
	fmt.Printf("with 1 element: err=%q\n", err)

	// Output:
	// with 4 elements: err=<nil> val={1 2 [3 4]}
	// with 6 elements: err=<nil> val={1 2 [3 4 5 6]}
	// with 1 element: err="rlp: too few elements for rlp.structWithTail"
}
