// Copyright 2021 The go-ethereum Authors
// This file is part of go-ethereum.
//
// go-ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// go-ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with go-ethereum. If not, see <http://www.gnu.org/licenses/>.

package main

import (
	"bytes"
	"fmt"
	"strings"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
)

func TestRoundtrip(t *testing.T) {
	for i, want := range []string{
		"0xf880806482520894d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0a1010000000000000000000000000000000000000000000000000000000000000001801ba0c16787a8e25e941d67691954642876c08f00996163ae7dfadbbfd6cd436f549da06180e5626cae31590f40641fe8f63734316c4bfeb4cdfab6714198c1044d2e28",
		"0xd5c0d3cb84746573742a2a808213378667617a6f6e6b",
		"0xc780c0c1c0825208",
	} {
		var out strings.Builder
		err := rlpToText(bytes.NewReader(common.FromHex(want)), &out)
		if err != nil {
			t.Fatal(err)
		}
		text := out.String()
		rlpBytes, err := textToRlp(strings.NewReader(text))
		if err != nil {
			t.Errorf("test %d: error %v", i, err)
			continue
		}
		have := fmt.Sprintf("%#x", rlpBytes)
		if have != want {
			t.Errorf("test %d: have\n%v\nwant:\n%v\n", i, have, want)
		}
	}
}

func TestTextToRlp(t *testing.T) {
	type tc struct {
		text string
		want string
	}
	cases := []tc{
		{
			text: `[
  "",
  [],
[     
 [],
    ],
  5208,
]`,
			want: "0xc780c0c1c0825208",
		},
	}
	for i, tc := range cases {
		have, err := textToRlp(strings.NewReader(tc.text))
		if err != nil {
			t.Errorf("test %d: error %v", i, err)
			continue
		}
		if hexutil.Encode(have) != tc.want {
			t.Errorf("test %d:\nhave %v\nwant %v", i, hexutil.Encode(have), tc.want)
		}
	}
}
