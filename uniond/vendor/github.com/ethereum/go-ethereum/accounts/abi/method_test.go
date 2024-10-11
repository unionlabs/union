// Copyright 2018 The go-ethereum Authors
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

package abi

import (
	"strings"
	"testing"
)

const methoddata = `
[
	{"type": "function", "name": "balance", "stateMutability": "view"},
	{"type": "function", "name": "send", "inputs": [{ "name": "amount", "type": "uint256" }]},
	{"type": "function", "name": "transfer", "inputs": [{"name": "from", "type": "address"}, {"name": "to", "type": "address"}, {"name": "value", "type": "uint256"}], "outputs": [{"name": "success", "type": "bool"}]},
	{"constant":false,"inputs":[{"components":[{"name":"x","type":"uint256"},{"name":"y","type":"uint256"}],"name":"a","type":"tuple"}],"name":"tuple","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},
	{"constant":false,"inputs":[{"components":[{"name":"x","type":"uint256"},{"name":"y","type":"uint256"}],"name":"a","type":"tuple[]"}],"name":"tupleSlice","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},
	{"constant":false,"inputs":[{"components":[{"name":"x","type":"uint256"},{"name":"y","type":"uint256"}],"name":"a","type":"tuple[5]"}],"name":"tupleArray","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},
	{"constant":false,"inputs":[{"components":[{"name":"x","type":"uint256"},{"name":"y","type":"uint256"}],"name":"a","type":"tuple[5][]"}],"name":"complexTuple","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},
	{"stateMutability":"nonpayable","type":"fallback"},
	{"stateMutability":"payable","type":"receive"}
]`

func TestMethodString(t *testing.T) {
	var table = []struct {
		method      string
		expectation string
	}{
		{
			method:      "balance",
			expectation: "function balance() view returns()",
		},
		{
			method:      "send",
			expectation: "function send(uint256 amount) returns()",
		},
		{
			method:      "transfer",
			expectation: "function transfer(address from, address to, uint256 value) returns(bool success)",
		},
		{
			method:      "tuple",
			expectation: "function tuple((uint256,uint256) a) returns()",
		},
		{
			method:      "tupleArray",
			expectation: "function tupleArray((uint256,uint256)[5] a) returns()",
		},
		{
			method:      "tupleSlice",
			expectation: "function tupleSlice((uint256,uint256)[] a) returns()",
		},
		{
			method:      "complexTuple",
			expectation: "function complexTuple((uint256,uint256)[5][] a) returns()",
		},
		{
			method:      "fallback",
			expectation: "fallback() returns()",
		},
		{
			method:      "receive",
			expectation: "receive() payable returns()",
		},
	}

	abi, err := JSON(strings.NewReader(methoddata))
	if err != nil {
		t.Fatal(err)
	}

	for _, test := range table {
		var got string
		switch test.method {
		case "fallback":
			got = abi.Fallback.String()
		case "receive":
			got = abi.Receive.String()
		default:
			got = abi.Methods[test.method].String()
		}
		if got != test.expectation {
			t.Errorf("expected string to be %s, got %s", test.expectation, got)
		}
	}
}

func TestMethodSig(t *testing.T) {
	var cases = []struct {
		method string
		expect string
	}{
		{
			method: "balance",
			expect: "balance()",
		},
		{
			method: "send",
			expect: "send(uint256)",
		},
		{
			method: "transfer",
			expect: "transfer(address,address,uint256)",
		},
		{
			method: "tuple",
			expect: "tuple((uint256,uint256))",
		},
		{
			method: "tupleArray",
			expect: "tupleArray((uint256,uint256)[5])",
		},
		{
			method: "tupleSlice",
			expect: "tupleSlice((uint256,uint256)[])",
		},
		{
			method: "complexTuple",
			expect: "complexTuple((uint256,uint256)[5][])",
		},
	}
	abi, err := JSON(strings.NewReader(methoddata))
	if err != nil {
		t.Fatal(err)
	}

	for _, test := range cases {
		got := abi.Methods[test.method].Sig
		if got != test.expect {
			t.Errorf("expected string to be %s, got %s", test.expect, got)
		}
	}
}
