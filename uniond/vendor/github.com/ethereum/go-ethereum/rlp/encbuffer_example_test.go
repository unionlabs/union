// Copyright 2022 The go-ethereum Authors
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

package rlp_test

import (
	"bytes"
	"fmt"

	"github.com/ethereum/go-ethereum/rlp"
)

func ExampleEncoderBuffer() {
	var w bytes.Buffer

	// Encode [4, [5, 6]] to w.
	buf := rlp.NewEncoderBuffer(&w)
	l1 := buf.List()
	buf.WriteUint64(4)
	l2 := buf.List()
	buf.WriteUint64(5)
	buf.WriteUint64(6)
	buf.ListEnd(l2)
	buf.ListEnd(l1)

	if err := buf.Flush(); err != nil {
		panic(err)
	}
	fmt.Printf("%X\n", w.Bytes())
	// Output:
	// C404C20506
}
