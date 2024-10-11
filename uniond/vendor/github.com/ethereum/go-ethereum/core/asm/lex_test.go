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

package asm

import (
	"reflect"
	"testing"
)

func lexAll(src string) []token {
	ch := Lex([]byte(src), false)

	var tokens []token
	for i := range ch {
		tokens = append(tokens, i)
	}
	return tokens
}

func TestLexer(t *testing.T) {
	tests := []struct {
		input  string
		tokens []token
	}{
		{
			input:  ";; this is a comment",
			tokens: []token{{typ: lineStart}, {typ: eof}},
		},
		{
			input:  "0x12345678",
			tokens: []token{{typ: lineStart}, {typ: number, text: "0x12345678"}, {typ: eof}},
		},
		{
			input:  "0x123ggg",
			tokens: []token{{typ: lineStart}, {typ: number, text: "0x123"}, {typ: element, text: "ggg"}, {typ: eof}},
		},
		{
			input:  "12345678",
			tokens: []token{{typ: lineStart}, {typ: number, text: "12345678"}, {typ: eof}},
		},
		{
			input:  "123abc",
			tokens: []token{{typ: lineStart}, {typ: number, text: "123"}, {typ: element, text: "abc"}, {typ: eof}},
		},
		{
			input:  "0123abc",
			tokens: []token{{typ: lineStart}, {typ: number, text: "0123"}, {typ: element, text: "abc"}, {typ: eof}},
		},
		{
			input:  "00123abc",
			tokens: []token{{typ: lineStart}, {typ: number, text: "00123"}, {typ: element, text: "abc"}, {typ: eof}},
		},
		{
			input:  "@foo",
			tokens: []token{{typ: lineStart}, {typ: label, text: "foo"}, {typ: eof}},
		},
		{
			input:  "@label123",
			tokens: []token{{typ: lineStart}, {typ: label, text: "label123"}, {typ: eof}},
		},
		// Comment after label
		{
			input:  "@label123 ;; comment",
			tokens: []token{{typ: lineStart}, {typ: label, text: "label123"}, {typ: eof}},
		},
		// Comment after instruction
		{
			input:  "push 3 ;; comment\nadd",
			tokens: []token{{typ: lineStart}, {typ: element, text: "push"}, {typ: number, text: "3"}, {typ: lineEnd, text: "\n"}, {typ: lineStart, lineno: 1}, {typ: element, lineno: 1, text: "add"}, {typ: eof, lineno: 1}},
		},
	}

	for _, test := range tests {
		tokens := lexAll(test.input)
		if !reflect.DeepEqual(tokens, test.tokens) {
			t.Errorf("input %q\ngot:  %+v\nwant: %+v", test.input, tokens, test.tokens)
		}
	}
}
