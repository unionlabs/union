// Copyright 2017 The go-ethereum Authors
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

package hexutil_test

import (
	"encoding/json"
	"fmt"

	"github.com/ethereum/go-ethereum/common/hexutil"
)

type MyType [5]byte

func (v *MyType) UnmarshalText(input []byte) error {
	return hexutil.UnmarshalFixedText("MyType", input, v[:])
}

func (v MyType) String() string {
	return hexutil.Bytes(v[:]).String()
}

func ExampleUnmarshalFixedText() {
	var v1, v2 MyType
	fmt.Println("v1 error:", json.Unmarshal([]byte(`"0x01"`), &v1))
	fmt.Println("v2 error:", json.Unmarshal([]byte(`"0x0101010101"`), &v2))
	fmt.Println("v2:", v2)
	// Output:
	// v1 error: hex string has length 2, want 10 for MyType
	// v2 error: <nil>
	// v2: 0x0101010101
}
