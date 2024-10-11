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

package abi

import (
	"bytes"
	"encoding/hex"
	"errors"
	"fmt"
	"math/big"
	"reflect"
	"strings"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/math"
	"github.com/ethereum/go-ethereum/crypto"
)

const jsondata = `
[
	{ "type" : "function", "name" : ""},
	{ "type" : "function", "name" : "balance", "stateMutability" : "view" },
	{ "type" : "function", "name" : "send", "inputs" : [ { "name" : "amount", "type" : "uint256" } ] },
	{ "type" : "function", "name" : "test", "inputs" : [ { "name" : "number", "type" : "uint32" } ] },
	{ "type" : "function", "name" : "string", "inputs" : [ { "name" : "inputs", "type" : "string" } ] },
	{ "type" : "function", "name" : "bool", "inputs" : [ { "name" : "inputs", "type" : "bool" } ] },
	{ "type" : "function", "name" : "address", "inputs" : [ { "name" : "inputs", "type" : "address" } ] },
	{ "type" : "function", "name" : "uint64[2]", "inputs" : [ { "name" : "inputs", "type" : "uint64[2]" } ] },
	{ "type" : "function", "name" : "uint64[]", "inputs" : [ { "name" : "inputs", "type" : "uint64[]" } ] },
	{ "type" : "function", "name" : "int8", "inputs" : [ { "name" : "inputs", "type" : "int8" } ] },
	{ "type" : "function", "name" : "bytes32", "inputs" : [ { "name" : "inputs", "type" : "bytes32" } ] },
	{ "type" : "function", "name" : "foo", "inputs" : [ { "name" : "inputs", "type" : "uint32" } ] },
	{ "type" : "function", "name" : "bar", "inputs" : [ { "name" : "inputs", "type" : "uint32" }, { "name" : "string", "type" : "uint16" } ] },
	{ "type" : "function", "name" : "slice", "inputs" : [ { "name" : "inputs", "type" : "uint32[2]" } ] },
	{ "type" : "function", "name" : "slice256", "inputs" : [ { "name" : "inputs", "type" : "uint256[2]" } ] },
	{ "type" : "function", "name" : "sliceAddress", "inputs" : [ { "name" : "inputs", "type" : "address[]" } ] },
	{ "type" : "function", "name" : "sliceMultiAddress", "inputs" : [ { "name" : "a", "type" : "address[]" }, { "name" : "b", "type" : "address[]" } ] },
	{ "type" : "function", "name" : "nestedArray", "inputs" : [ { "name" : "a", "type" : "uint256[2][2]" }, { "name" : "b", "type" : "address[]" } ] },
	{ "type" : "function", "name" : "nestedArray2", "inputs" : [ { "name" : "a", "type" : "uint8[][2]" } ] },
	{ "type" : "function", "name" : "nestedSlice", "inputs" : [ { "name" : "a", "type" : "uint8[][]" } ] },
	{ "type" : "function", "name" : "receive", "inputs" : [ { "name" : "memo", "type" : "bytes" }], "outputs" : [], "payable" : true, "stateMutability" : "payable" },
	{ "type" : "function", "name" : "fixedArrStr", "stateMutability" : "view", "inputs" : [ { "name" : "str", "type" : "string" }, { "name" : "fixedArr", "type" : "uint256[2]" } ] },
	{ "type" : "function", "name" : "fixedArrBytes", "stateMutability" : "view", "inputs" : [ { "name" : "bytes", "type" : "bytes" }, { "name" : "fixedArr", "type" : "uint256[2]" } ] },
	{ "type" : "function", "name" : "mixedArrStr", "stateMutability" : "view", "inputs" : [ { "name" : "str", "type" : "string" }, { "name" : "fixedArr", "type" : "uint256[2]" }, { "name" : "dynArr", "type" : "uint256[]" } ] },
	{ "type" : "function", "name" : "doubleFixedArrStr", "stateMutability" : "view", "inputs" : [ { "name" : "str", "type" : "string" }, { "name" : "fixedArr1", "type" : "uint256[2]" }, { "name" : "fixedArr2", "type" : "uint256[3]" } ] },
	{ "type" : "function", "name" : "multipleMixedArrStr", "stateMutability" : "view", "inputs" : [ { "name" : "str", "type" : "string" }, { "name" : "fixedArr1", "type" : "uint256[2]" }, { "name" : "dynArr", "type" : "uint256[]" }, { "name" : "fixedArr2", "type" : "uint256[3]" } ] },
	{ "type" : "function", "name" : "overloadedNames", "stateMutability" : "view", "inputs": [ { "components": [ { "internalType": "uint256", "name": "_f",	"type": "uint256" }, { "internalType": "uint256", "name": "__f", "type": "uint256"}, { "internalType": "uint256", "name": "f", "type": "uint256"}],"internalType": "struct Overloader.F", "name": "f","type": "tuple"}]}
]`

var (
	Uint256, _    = NewType("uint256", "", nil)
	Uint32, _     = NewType("uint32", "", nil)
	Uint16, _     = NewType("uint16", "", nil)
	String, _     = NewType("string", "", nil)
	Bool, _       = NewType("bool", "", nil)
	Bytes, _      = NewType("bytes", "", nil)
	Bytes32, _    = NewType("bytes32", "", nil)
	Address, _    = NewType("address", "", nil)
	Uint64Arr, _  = NewType("uint64[]", "", nil)
	AddressArr, _ = NewType("address[]", "", nil)
	Int8, _       = NewType("int8", "", nil)
	// Special types for testing
	Uint32Arr2, _       = NewType("uint32[2]", "", nil)
	Uint64Arr2, _       = NewType("uint64[2]", "", nil)
	Uint256Arr, _       = NewType("uint256[]", "", nil)
	Uint256Arr2, _      = NewType("uint256[2]", "", nil)
	Uint256Arr3, _      = NewType("uint256[3]", "", nil)
	Uint256ArrNested, _ = NewType("uint256[2][2]", "", nil)
	Uint8ArrNested, _   = NewType("uint8[][2]", "", nil)
	Uint8SliceNested, _ = NewType("uint8[][]", "", nil)
	TupleF, _           = NewType("tuple", "struct Overloader.F", []ArgumentMarshaling{
		{Name: "_f", Type: "uint256"},
		{Name: "__f", Type: "uint256"},
		{Name: "f", Type: "uint256"}})
)

var methods = map[string]Method{
	"":                    NewMethod("", "", Function, "", false, false, nil, nil),
	"balance":             NewMethod("balance", "balance", Function, "view", false, false, nil, nil),
	"send":                NewMethod("send", "send", Function, "", false, false, []Argument{{"amount", Uint256, false}}, nil),
	"test":                NewMethod("test", "test", Function, "", false, false, []Argument{{"number", Uint32, false}}, nil),
	"string":              NewMethod("string", "string", Function, "", false, false, []Argument{{"inputs", String, false}}, nil),
	"bool":                NewMethod("bool", "bool", Function, "", false, false, []Argument{{"inputs", Bool, false}}, nil),
	"address":             NewMethod("address", "address", Function, "", false, false, []Argument{{"inputs", Address, false}}, nil),
	"uint64[]":            NewMethod("uint64[]", "uint64[]", Function, "", false, false, []Argument{{"inputs", Uint64Arr, false}}, nil),
	"uint64[2]":           NewMethod("uint64[2]", "uint64[2]", Function, "", false, false, []Argument{{"inputs", Uint64Arr2, false}}, nil),
	"int8":                NewMethod("int8", "int8", Function, "", false, false, []Argument{{"inputs", Int8, false}}, nil),
	"bytes32":             NewMethod("bytes32", "bytes32", Function, "", false, false, []Argument{{"inputs", Bytes32, false}}, nil),
	"foo":                 NewMethod("foo", "foo", Function, "", false, false, []Argument{{"inputs", Uint32, false}}, nil),
	"bar":                 NewMethod("bar", "bar", Function, "", false, false, []Argument{{"inputs", Uint32, false}, {"string", Uint16, false}}, nil),
	"slice":               NewMethod("slice", "slice", Function, "", false, false, []Argument{{"inputs", Uint32Arr2, false}}, nil),
	"slice256":            NewMethod("slice256", "slice256", Function, "", false, false, []Argument{{"inputs", Uint256Arr2, false}}, nil),
	"sliceAddress":        NewMethod("sliceAddress", "sliceAddress", Function, "", false, false, []Argument{{"inputs", AddressArr, false}}, nil),
	"sliceMultiAddress":   NewMethod("sliceMultiAddress", "sliceMultiAddress", Function, "", false, false, []Argument{{"a", AddressArr, false}, {"b", AddressArr, false}}, nil),
	"nestedArray":         NewMethod("nestedArray", "nestedArray", Function, "", false, false, []Argument{{"a", Uint256ArrNested, false}, {"b", AddressArr, false}}, nil),
	"nestedArray2":        NewMethod("nestedArray2", "nestedArray2", Function, "", false, false, []Argument{{"a", Uint8ArrNested, false}}, nil),
	"nestedSlice":         NewMethod("nestedSlice", "nestedSlice", Function, "", false, false, []Argument{{"a", Uint8SliceNested, false}}, nil),
	"receive":             NewMethod("receive", "receive", Function, "payable", false, true, []Argument{{"memo", Bytes, false}}, []Argument{}),
	"fixedArrStr":         NewMethod("fixedArrStr", "fixedArrStr", Function, "view", false, false, []Argument{{"str", String, false}, {"fixedArr", Uint256Arr2, false}}, nil),
	"fixedArrBytes":       NewMethod("fixedArrBytes", "fixedArrBytes", Function, "view", false, false, []Argument{{"bytes", Bytes, false}, {"fixedArr", Uint256Arr2, false}}, nil),
	"mixedArrStr":         NewMethod("mixedArrStr", "mixedArrStr", Function, "view", false, false, []Argument{{"str", String, false}, {"fixedArr", Uint256Arr2, false}, {"dynArr", Uint256Arr, false}}, nil),
	"doubleFixedArrStr":   NewMethod("doubleFixedArrStr", "doubleFixedArrStr", Function, "view", false, false, []Argument{{"str", String, false}, {"fixedArr1", Uint256Arr2, false}, {"fixedArr2", Uint256Arr3, false}}, nil),
	"multipleMixedArrStr": NewMethod("multipleMixedArrStr", "multipleMixedArrStr", Function, "view", false, false, []Argument{{"str", String, false}, {"fixedArr1", Uint256Arr2, false}, {"dynArr", Uint256Arr, false}, {"fixedArr2", Uint256Arr3, false}}, nil),
	"overloadedNames":     NewMethod("overloadedNames", "overloadedNames", Function, "view", false, false, []Argument{{"f", TupleF, false}}, nil),
}

func TestReader(t *testing.T) {
	abi := ABI{
		Methods: methods,
	}

	exp, err := JSON(strings.NewReader(jsondata))
	if err != nil {
		t.Fatal(err)
	}

	for name, expM := range exp.Methods {
		gotM, exist := abi.Methods[name]
		if !exist {
			t.Errorf("Missing expected method %v", name)
		}
		if !reflect.DeepEqual(gotM, expM) {
			t.Errorf("\nGot abi method: \n%v\ndoes not match expected method\n%v", gotM, expM)
		}
	}

	for name, gotM := range abi.Methods {
		expM, exist := exp.Methods[name]
		if !exist {
			t.Errorf("Found extra method %v", name)
		}
		if !reflect.DeepEqual(gotM, expM) {
			t.Errorf("\nGot abi method: \n%v\ndoes not match expected method\n%v", gotM, expM)
		}
	}
}

func TestInvalidABI(t *testing.T) {
	json := `[{ "type" : "function", "name" : "", "constant" : fals }]`
	_, err := JSON(strings.NewReader(json))
	if err == nil {
		t.Fatal("invalid json should produce error")
	}
	json2 := `[{ "type" : "function", "name" : "send", "constant" : false, "inputs" : [ { "name" : "amount", "typ" : "uint256" } ] }]`
	_, err = JSON(strings.NewReader(json2))
	if err == nil {
		t.Fatal("invalid json should produce error")
	}
}

// TestConstructor tests a constructor function.
// The test is based on the following contract:
//
//	contract TestConstructor {
//		constructor(uint256 a, uint256 b) public{}
//	}
func TestConstructor(t *testing.T) {
	json := `[{	"inputs": [{"internalType": "uint256","name": "a","type": "uint256"	},{	"internalType": "uint256","name": "b","type": "uint256"}],"stateMutability": "nonpayable","type": "constructor"}]`
	method := NewMethod("", "", Constructor, "nonpayable", false, false, []Argument{{"a", Uint256, false}, {"b", Uint256, false}}, nil)
	// Test from JSON
	abi, err := JSON(strings.NewReader(json))
	if err != nil {
		t.Fatal(err)
	}
	if !reflect.DeepEqual(abi.Constructor, method) {
		t.Error("Missing expected constructor")
	}
	// Test pack/unpack
	packed, err := abi.Pack("", big.NewInt(1), big.NewInt(2))
	if err != nil {
		t.Error(err)
	}
	unpacked, err := abi.Constructor.Inputs.Unpack(packed)
	if err != nil {
		t.Error(err)
	}

	if !reflect.DeepEqual(unpacked[0], big.NewInt(1)) {
		t.Error("Unable to pack/unpack from constructor")
	}
	if !reflect.DeepEqual(unpacked[1], big.NewInt(2)) {
		t.Error("Unable to pack/unpack from constructor")
	}
}

func TestTestNumbers(t *testing.T) {
	abi, err := JSON(strings.NewReader(jsondata))
	if err != nil {
		t.Fatal(err)
	}

	if _, err := abi.Pack("balance"); err != nil {
		t.Error(err)
	}

	if _, err := abi.Pack("balance", 1); err == nil {
		t.Error("expected error for balance(1)")
	}

	if _, err := abi.Pack("doesntexist", nil); err == nil {
		t.Errorf("doesntexist shouldn't exist")
	}

	if _, err := abi.Pack("doesntexist", 1); err == nil {
		t.Errorf("doesntexist(1) shouldn't exist")
	}

	if _, err := abi.Pack("send", big.NewInt(1000)); err != nil {
		t.Error(err)
	}

	i := new(int)
	*i = 1000
	if _, err := abi.Pack("send", i); err == nil {
		t.Errorf("expected send( ptr ) to throw, requires *big.Int instead of *int")
	}

	if _, err := abi.Pack("test", uint32(1000)); err != nil {
		t.Error(err)
	}
}

func TestMethodSignature(t *testing.T) {
	m := NewMethod("foo", "foo", Function, "", false, false, []Argument{{"bar", String, false}, {"baz", String, false}}, nil)
	exp := "foo(string,string)"
	if m.Sig != exp {
		t.Error("signature mismatch", exp, "!=", m.Sig)
	}

	idexp := crypto.Keccak256([]byte(exp))[:4]
	if !bytes.Equal(m.ID, idexp) {
		t.Errorf("expected ids to match %x != %x", m.ID, idexp)
	}

	m = NewMethod("foo", "foo", Function, "", false, false, []Argument{{"bar", Uint256, false}}, nil)
	exp = "foo(uint256)"
	if m.Sig != exp {
		t.Error("signature mismatch", exp, "!=", m.Sig)
	}

	// Method with tuple arguments
	s, _ := NewType("tuple", "", []ArgumentMarshaling{
		{Name: "a", Type: "int256"},
		{Name: "b", Type: "int256[]"},
		{Name: "c", Type: "tuple[]", Components: []ArgumentMarshaling{
			{Name: "x", Type: "int256"},
			{Name: "y", Type: "int256"},
		}},
		{Name: "d", Type: "tuple[2]", Components: []ArgumentMarshaling{
			{Name: "x", Type: "int256"},
			{Name: "y", Type: "int256"},
		}},
	})
	m = NewMethod("foo", "foo", Function, "", false, false, []Argument{{"s", s, false}, {"bar", String, false}}, nil)
	exp = "foo((int256,int256[],(int256,int256)[],(int256,int256)[2]),string)"
	if m.Sig != exp {
		t.Error("signature mismatch", exp, "!=", m.Sig)
	}
}

func TestOverloadedMethodSignature(t *testing.T) {
	json := `[{"constant":true,"inputs":[{"name":"i","type":"uint256"},{"name":"j","type":"uint256"}],"name":"foo","outputs":[],"payable":false,"stateMutability":"pure","type":"function"},{"constant":true,"inputs":[{"name":"i","type":"uint256"}],"name":"foo","outputs":[],"payable":false,"stateMutability":"pure","type":"function"},{"anonymous":false,"inputs":[{"indexed":false,"name":"i","type":"uint256"}],"name":"bar","type":"event"},{"anonymous":false,"inputs":[{"indexed":false,"name":"i","type":"uint256"},{"indexed":false,"name":"j","type":"uint256"}],"name":"bar","type":"event"}]`
	abi, err := JSON(strings.NewReader(json))
	if err != nil {
		t.Fatal(err)
	}
	check := func(name string, expect string, method bool) {
		if method {
			if abi.Methods[name].Sig != expect {
				t.Fatalf("The signature of overloaded method mismatch, want %s, have %s", expect, abi.Methods[name].Sig)
			}
		} else {
			if abi.Events[name].Sig != expect {
				t.Fatalf("The signature of overloaded event mismatch, want %s, have %s", expect, abi.Events[name].Sig)
			}
		}
	}
	check("foo", "foo(uint256,uint256)", true)
	check("foo0", "foo(uint256)", true)
	check("bar", "bar(uint256)", false)
	check("bar0", "bar(uint256,uint256)", false)
}

func TestCustomErrors(t *testing.T) {
	json := `[{ "inputs": [	{ "internalType": "uint256", "name": "", "type": "uint256" } ],"name": "MyError", "type": "error"} ]`
	abi, err := JSON(strings.NewReader(json))
	if err != nil {
		t.Fatal(err)
	}
	check := func(name string, expect string) {
		if abi.Errors[name].Sig != expect {
			t.Fatalf("The signature of overloaded method mismatch, want %s, have %s", expect, abi.Methods[name].Sig)
		}
	}
	check("MyError", "MyError(uint256)")
}

func TestMultiPack(t *testing.T) {
	abi, err := JSON(strings.NewReader(jsondata))
	if err != nil {
		t.Fatal(err)
	}

	sig := crypto.Keccak256([]byte("bar(uint32,uint16)"))[:4]
	sig = append(sig, make([]byte, 64)...)
	sig[35] = 10
	sig[67] = 11

	packed, err := abi.Pack("bar", uint32(10), uint16(11))
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(packed, sig) {
		t.Errorf("expected %x got %x", sig, packed)
	}
}

func ExampleJSON() {
	const definition = `[{"constant":true,"inputs":[{"name":"","type":"address"}],"name":"isBar","outputs":[{"name":"","type":"bool"}],"type":"function"}]`

	abi, err := JSON(strings.NewReader(definition))
	if err != nil {
		panic(err)
	}
	out, err := abi.Pack("isBar", common.HexToAddress("01"))
	if err != nil {
		panic(err)
	}

	fmt.Printf("%x\n", out)
	// Output:
	// 1f2c40920000000000000000000000000000000000000000000000000000000000000001
}

func TestInputVariableInputLength(t *testing.T) {
	const definition = `[
	{ "type" : "function", "name" : "strOne", "constant" : true, "inputs" : [ { "name" : "str", "type" : "string" } ] },
	{ "type" : "function", "name" : "bytesOne", "constant" : true, "inputs" : [ { "name" : "str", "type" : "bytes" } ] },
	{ "type" : "function", "name" : "strTwo", "constant" : true, "inputs" : [ { "name" : "str", "type" : "string" }, { "name" : "str1", "type" : "string" } ] }
	]`

	abi, err := JSON(strings.NewReader(definition))
	if err != nil {
		t.Fatal(err)
	}

	// test one string
	strin := "hello world"
	strpack, err := abi.Pack("strOne", strin)
	if err != nil {
		t.Error(err)
	}

	offset := make([]byte, 32)
	offset[31] = 32
	length := make([]byte, 32)
	length[31] = byte(len(strin))
	value := common.RightPadBytes([]byte(strin), 32)
	exp := append(offset, append(length, value...)...)

	// ignore first 4 bytes of the output. This is the function identifier
	strpack = strpack[4:]
	if !bytes.Equal(strpack, exp) {
		t.Errorf("expected %x, got %x\n", exp, strpack)
	}

	// test one bytes
	btspack, err := abi.Pack("bytesOne", []byte(strin))
	if err != nil {
		t.Error(err)
	}
	// ignore first 4 bytes of the output. This is the function identifier
	btspack = btspack[4:]
	if !bytes.Equal(btspack, exp) {
		t.Errorf("expected %x, got %x\n", exp, btspack)
	}

	//  test two strings
	str1 := "hello"
	str2 := "world"
	str2pack, err := abi.Pack("strTwo", str1, str2)
	if err != nil {
		t.Error(err)
	}

	offset1 := make([]byte, 32)
	offset1[31] = 64
	length1 := make([]byte, 32)
	length1[31] = byte(len(str1))
	value1 := common.RightPadBytes([]byte(str1), 32)

	offset2 := make([]byte, 32)
	offset2[31] = 128
	length2 := make([]byte, 32)
	length2[31] = byte(len(str2))
	value2 := common.RightPadBytes([]byte(str2), 32)

	exp2 := append(offset1, offset2...)
	exp2 = append(exp2, append(length1, value1...)...)
	exp2 = append(exp2, append(length2, value2...)...)

	// ignore first 4 bytes of the output. This is the function identifier
	str2pack = str2pack[4:]
	if !bytes.Equal(str2pack, exp2) {
		t.Errorf("expected %x, got %x\n", exp, str2pack)
	}

	// test two strings, first > 32, second < 32
	str1 = strings.Repeat("a", 33)
	str2pack, err = abi.Pack("strTwo", str1, str2)
	if err != nil {
		t.Error(err)
	}

	offset1 = make([]byte, 32)
	offset1[31] = 64
	length1 = make([]byte, 32)
	length1[31] = byte(len(str1))
	value1 = common.RightPadBytes([]byte(str1), 64)
	offset2[31] = 160

	exp2 = append(offset1, offset2...)
	exp2 = append(exp2, append(length1, value1...)...)
	exp2 = append(exp2, append(length2, value2...)...)

	// ignore first 4 bytes of the output. This is the function identifier
	str2pack = str2pack[4:]
	if !bytes.Equal(str2pack, exp2) {
		t.Errorf("expected %x, got %x\n", exp, str2pack)
	}

	// test two strings, first > 32, second >32
	str1 = strings.Repeat("a", 33)
	str2 = strings.Repeat("a", 33)
	str2pack, err = abi.Pack("strTwo", str1, str2)
	if err != nil {
		t.Error(err)
	}

	offset1 = make([]byte, 32)
	offset1[31] = 64
	length1 = make([]byte, 32)
	length1[31] = byte(len(str1))
	value1 = common.RightPadBytes([]byte(str1), 64)

	offset2 = make([]byte, 32)
	offset2[31] = 160
	length2 = make([]byte, 32)
	length2[31] = byte(len(str2))
	value2 = common.RightPadBytes([]byte(str2), 64)

	exp2 = append(offset1, offset2...)
	exp2 = append(exp2, append(length1, value1...)...)
	exp2 = append(exp2, append(length2, value2...)...)

	// ignore first 4 bytes of the output. This is the function identifier
	str2pack = str2pack[4:]
	if !bytes.Equal(str2pack, exp2) {
		t.Errorf("expected %x, got %x\n", exp, str2pack)
	}
}

func TestInputFixedArrayAndVariableInputLength(t *testing.T) {
	abi, err := JSON(strings.NewReader(jsondata))
	if err != nil {
		t.Error(err)
	}

	// test string, fixed array uint256[2]
	strin := "hello world"
	arrin := [2]*big.Int{big.NewInt(1), big.NewInt(2)}
	fixedArrStrPack, err := abi.Pack("fixedArrStr", strin, arrin)
	if err != nil {
		t.Error(err)
	}

	// generate expected output
	offset := make([]byte, 32)
	offset[31] = 96
	length := make([]byte, 32)
	length[31] = byte(len(strin))
	strvalue := common.RightPadBytes([]byte(strin), 32)
	arrinvalue1 := common.LeftPadBytes(arrin[0].Bytes(), 32)
	arrinvalue2 := common.LeftPadBytes(arrin[1].Bytes(), 32)
	exp := append(offset, arrinvalue1...)
	exp = append(exp, arrinvalue2...)
	exp = append(exp, append(length, strvalue...)...)

	// ignore first 4 bytes of the output. This is the function identifier
	fixedArrStrPack = fixedArrStrPack[4:]
	if !bytes.Equal(fixedArrStrPack, exp) {
		t.Errorf("expected %x, got %x\n", exp, fixedArrStrPack)
	}

	// test byte array, fixed array uint256[2]
	bytesin := []byte(strin)
	arrin = [2]*big.Int{big.NewInt(1), big.NewInt(2)}
	fixedArrBytesPack, err := abi.Pack("fixedArrBytes", bytesin, arrin)
	if err != nil {
		t.Error(err)
	}

	// generate expected output
	offset = make([]byte, 32)
	offset[31] = 96
	length = make([]byte, 32)
	length[31] = byte(len(strin))
	strvalue = common.RightPadBytes([]byte(strin), 32)
	arrinvalue1 = common.LeftPadBytes(arrin[0].Bytes(), 32)
	arrinvalue2 = common.LeftPadBytes(arrin[1].Bytes(), 32)
	exp = append(offset, arrinvalue1...)
	exp = append(exp, arrinvalue2...)
	exp = append(exp, append(length, strvalue...)...)

	// ignore first 4 bytes of the output. This is the function identifier
	fixedArrBytesPack = fixedArrBytesPack[4:]
	if !bytes.Equal(fixedArrBytesPack, exp) {
		t.Errorf("expected %x, got %x\n", exp, fixedArrBytesPack)
	}

	// test string, fixed array uint256[2], dynamic array uint256[]
	strin = "hello world"
	fixedarrin := [2]*big.Int{big.NewInt(1), big.NewInt(2)}
	dynarrin := []*big.Int{big.NewInt(1), big.NewInt(2), big.NewInt(3)}
	mixedArrStrPack, err := abi.Pack("mixedArrStr", strin, fixedarrin, dynarrin)
	if err != nil {
		t.Error(err)
	}

	// generate expected output
	stroffset := make([]byte, 32)
	stroffset[31] = 128
	strlength := make([]byte, 32)
	strlength[31] = byte(len(strin))
	strvalue = common.RightPadBytes([]byte(strin), 32)
	fixedarrinvalue1 := common.LeftPadBytes(fixedarrin[0].Bytes(), 32)
	fixedarrinvalue2 := common.LeftPadBytes(fixedarrin[1].Bytes(), 32)
	dynarroffset := make([]byte, 32)
	dynarroffset[31] = byte(160 + ((len(strin)/32)+1)*32)
	dynarrlength := make([]byte, 32)
	dynarrlength[31] = byte(len(dynarrin))
	dynarrinvalue1 := common.LeftPadBytes(dynarrin[0].Bytes(), 32)
	dynarrinvalue2 := common.LeftPadBytes(dynarrin[1].Bytes(), 32)
	dynarrinvalue3 := common.LeftPadBytes(dynarrin[2].Bytes(), 32)
	exp = append(stroffset, fixedarrinvalue1...)
	exp = append(exp, fixedarrinvalue2...)
	exp = append(exp, dynarroffset...)
	exp = append(exp, append(strlength, strvalue...)...)
	dynarrarg := append(dynarrlength, dynarrinvalue1...)
	dynarrarg = append(dynarrarg, dynarrinvalue2...)
	dynarrarg = append(dynarrarg, dynarrinvalue3...)
	exp = append(exp, dynarrarg...)

	// ignore first 4 bytes of the output. This is the function identifier
	mixedArrStrPack = mixedArrStrPack[4:]
	if !bytes.Equal(mixedArrStrPack, exp) {
		t.Errorf("expected %x, got %x\n", exp, mixedArrStrPack)
	}

	// test string, fixed array uint256[2], fixed array uint256[3]
	strin = "hello world"
	fixedarrin1 := [2]*big.Int{big.NewInt(1), big.NewInt(2)}
	fixedarrin2 := [3]*big.Int{big.NewInt(1), big.NewInt(2), big.NewInt(3)}
	doubleFixedArrStrPack, err := abi.Pack("doubleFixedArrStr", strin, fixedarrin1, fixedarrin2)
	if err != nil {
		t.Error(err)
	}

	// generate expected output
	stroffset = make([]byte, 32)
	stroffset[31] = 192
	strlength = make([]byte, 32)
	strlength[31] = byte(len(strin))
	strvalue = common.RightPadBytes([]byte(strin), 32)
	fixedarrin1value1 := common.LeftPadBytes(fixedarrin1[0].Bytes(), 32)
	fixedarrin1value2 := common.LeftPadBytes(fixedarrin1[1].Bytes(), 32)
	fixedarrin2value1 := common.LeftPadBytes(fixedarrin2[0].Bytes(), 32)
	fixedarrin2value2 := common.LeftPadBytes(fixedarrin2[1].Bytes(), 32)
	fixedarrin2value3 := common.LeftPadBytes(fixedarrin2[2].Bytes(), 32)
	exp = append(stroffset, fixedarrin1value1...)
	exp = append(exp, fixedarrin1value2...)
	exp = append(exp, fixedarrin2value1...)
	exp = append(exp, fixedarrin2value2...)
	exp = append(exp, fixedarrin2value3...)
	exp = append(exp, append(strlength, strvalue...)...)

	// ignore first 4 bytes of the output. This is the function identifier
	doubleFixedArrStrPack = doubleFixedArrStrPack[4:]
	if !bytes.Equal(doubleFixedArrStrPack, exp) {
		t.Errorf("expected %x, got %x\n", exp, doubleFixedArrStrPack)
	}

	// test string, fixed array uint256[2], dynamic array uint256[], fixed array uint256[3]
	strin = "hello world"
	fixedarrin1 = [2]*big.Int{big.NewInt(1), big.NewInt(2)}
	dynarrin = []*big.Int{big.NewInt(1), big.NewInt(2)}
	fixedarrin2 = [3]*big.Int{big.NewInt(1), big.NewInt(2), big.NewInt(3)}
	multipleMixedArrStrPack, err := abi.Pack("multipleMixedArrStr", strin, fixedarrin1, dynarrin, fixedarrin2)
	if err != nil {
		t.Error(err)
	}

	// generate expected output
	stroffset = make([]byte, 32)
	stroffset[31] = 224
	strlength = make([]byte, 32)
	strlength[31] = byte(len(strin))
	strvalue = common.RightPadBytes([]byte(strin), 32)
	fixedarrin1value1 = common.LeftPadBytes(fixedarrin1[0].Bytes(), 32)
	fixedarrin1value2 = common.LeftPadBytes(fixedarrin1[1].Bytes(), 32)
	dynarroffset = math.U256Bytes(big.NewInt(int64(256 + ((len(strin)/32)+1)*32)))
	dynarrlength = make([]byte, 32)
	dynarrlength[31] = byte(len(dynarrin))
	dynarrinvalue1 = common.LeftPadBytes(dynarrin[0].Bytes(), 32)
	dynarrinvalue2 = common.LeftPadBytes(dynarrin[1].Bytes(), 32)
	fixedarrin2value1 = common.LeftPadBytes(fixedarrin2[0].Bytes(), 32)
	fixedarrin2value2 = common.LeftPadBytes(fixedarrin2[1].Bytes(), 32)
	fixedarrin2value3 = common.LeftPadBytes(fixedarrin2[2].Bytes(), 32)
	exp = append(stroffset, fixedarrin1value1...)
	exp = append(exp, fixedarrin1value2...)
	exp = append(exp, dynarroffset...)
	exp = append(exp, fixedarrin2value1...)
	exp = append(exp, fixedarrin2value2...)
	exp = append(exp, fixedarrin2value3...)
	exp = append(exp, append(strlength, strvalue...)...)
	dynarrarg = append(dynarrlength, dynarrinvalue1...)
	dynarrarg = append(dynarrarg, dynarrinvalue2...)
	exp = append(exp, dynarrarg...)

	// ignore first 4 bytes of the output. This is the function identifier
	multipleMixedArrStrPack = multipleMixedArrStrPack[4:]
	if !bytes.Equal(multipleMixedArrStrPack, exp) {
		t.Errorf("expected %x, got %x\n", exp, multipleMixedArrStrPack)
	}
}

func TestDefaultFunctionParsing(t *testing.T) {
	const definition = `[{ "name" : "balance", "type" : "function" }]`

	abi, err := JSON(strings.NewReader(definition))
	if err != nil {
		t.Fatal(err)
	}

	if _, ok := abi.Methods["balance"]; !ok {
		t.Error("expected 'balance' to be present")
	}
}

func TestBareEvents(t *testing.T) {
	const definition = `[
	{ "type" : "event", "name" : "balance" },
	{ "type" : "event", "name" : "anon", "anonymous" : true},
	{ "type" : "event", "name" : "args", "inputs" : [{ "indexed":false, "name":"arg0", "type":"uint256" }, { "indexed":true, "name":"arg1", "type":"address" }] },
	{ "type" : "event", "name" : "tuple", "inputs" : [{ "indexed":false, "name":"t", "type":"tuple", "components":[{"name":"a", "type":"uint256"}] }, { "indexed":true, "name":"arg1", "type":"address" }] }
	]`

	tuple, _ := NewType("tuple", "", []ArgumentMarshaling{{Name: "a", Type: "uint256"}})

	expectedEvents := map[string]struct {
		Anonymous bool
		Args      []Argument
	}{
		"balance": {false, nil},
		"anon":    {true, nil},
		"args": {false, []Argument{
			{Name: "arg0", Type: Uint256, Indexed: false},
			{Name: "arg1", Type: Address, Indexed: true},
		}},
		"tuple": {false, []Argument{
			{Name: "t", Type: tuple, Indexed: false},
			{Name: "arg1", Type: Address, Indexed: true},
		}},
	}

	abi, err := JSON(strings.NewReader(definition))
	if err != nil {
		t.Fatal(err)
	}

	if len(abi.Events) != len(expectedEvents) {
		t.Fatalf("invalid number of events after parsing, want %d, got %d", len(expectedEvents), len(abi.Events))
	}

	for name, exp := range expectedEvents {
		got, ok := abi.Events[name]
		if !ok {
			t.Errorf("could not found event %s", name)
			continue
		}
		if got.Anonymous != exp.Anonymous {
			t.Errorf("invalid anonymous indication for event %s, want %v, got %v", name, exp.Anonymous, got.Anonymous)
		}
		if len(got.Inputs) != len(exp.Args) {
			t.Errorf("invalid number of args, want %d, got %d", len(exp.Args), len(got.Inputs))
			continue
		}
		for i, arg := range exp.Args {
			if arg.Name != got.Inputs[i].Name {
				t.Errorf("events[%s].Input[%d] has an invalid name, want %s, got %s", name, i, arg.Name, got.Inputs[i].Name)
			}
			if arg.Indexed != got.Inputs[i].Indexed {
				t.Errorf("events[%s].Input[%d] has an invalid indexed indication, want %v, got %v", name, i, arg.Indexed, got.Inputs[i].Indexed)
			}
			if arg.Type.T != got.Inputs[i].Type.T {
				t.Errorf("events[%s].Input[%d] has an invalid type, want %x, got %x", name, i, arg.Type.T, got.Inputs[i].Type.T)
			}
		}
	}
}

// TestUnpackEvent is based on this contract:
//
//	contract T {
//		event received(address sender, uint amount, bytes memo);
//		event receivedAddr(address sender);
//		function receive(bytes memo) external payable {
//			received(msg.sender, msg.value, memo);
//			receivedAddr(msg.sender);
//		}
//	}
//
// When receive("X") is called with sender 0x00... and value 1, it produces this tx receipt:
//
//	receipt{status=1 cgas=23949 bloom=00000000004000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000040200000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 logs=[log: b6818c8064f645cd82d99b59a1a267d6d61117ef [75fd880d39c1daf53b6547ab6cb59451fc6452d27caa90e5b6649dd8293b9eed] 000000000000000000000000376c47978271565f56deb45495afa69e59c16ab200000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000158 9ae378b6d4409eada347a5dc0c180f186cb62dc68fcc0f043425eb917335aa28 0 95d429d309bb9d753954195fe2d69bd140b4ae731b9b5b605c34323de162cf00 0]}
func TestUnpackEvent(t *testing.T) {
	const abiJSON = `[{"constant":false,"inputs":[{"name":"memo","type":"bytes"}],"name":"receive","outputs":[],"payable":true,"stateMutability":"payable","type":"function"},{"anonymous":false,"inputs":[{"indexed":false,"name":"sender","type":"address"},{"indexed":false,"name":"amount","type":"uint256"},{"indexed":false,"name":"memo","type":"bytes"}],"name":"received","type":"event"},{"anonymous":false,"inputs":[{"indexed":false,"name":"sender","type":"address"}],"name":"receivedAddr","type":"event"}]`
	abi, err := JSON(strings.NewReader(abiJSON))
	if err != nil {
		t.Fatal(err)
	}

	const hexdata = `000000000000000000000000376c47978271565f56deb45495afa69e59c16ab200000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000158`
	data, err := hex.DecodeString(hexdata)
	if err != nil {
		t.Fatal(err)
	}
	if len(data)%32 == 0 {
		t.Errorf("len(data) is %d, want a non-multiple of 32", len(data))
	}

	type ReceivedEvent struct {
		Sender common.Address
		Amount *big.Int
		Memo   []byte
	}
	var ev ReceivedEvent

	err = abi.UnpackIntoInterface(&ev, "received", data)
	if err != nil {
		t.Error(err)
	}

	type ReceivedAddrEvent struct {
		Sender common.Address
	}
	var receivedAddrEv ReceivedAddrEvent
	err = abi.UnpackIntoInterface(&receivedAddrEv, "receivedAddr", data)
	if err != nil {
		t.Error(err)
	}
}

func TestUnpackEventIntoMap(t *testing.T) {
	const abiJSON = `[{"constant":false,"inputs":[{"name":"memo","type":"bytes"}],"name":"receive","outputs":[],"payable":true,"stateMutability":"payable","type":"function"},{"anonymous":false,"inputs":[{"indexed":false,"name":"sender","type":"address"},{"indexed":false,"name":"amount","type":"uint256"},{"indexed":false,"name":"memo","type":"bytes"}],"name":"received","type":"event"},{"anonymous":false,"inputs":[{"indexed":false,"name":"sender","type":"address"}],"name":"receivedAddr","type":"event"}]`
	abi, err := JSON(strings.NewReader(abiJSON))
	if err != nil {
		t.Fatal(err)
	}

	const hexdata = `000000000000000000000000376c47978271565f56deb45495afa69e59c16ab200000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000158`
	data, err := hex.DecodeString(hexdata)
	if err != nil {
		t.Fatal(err)
	}
	if len(data)%32 == 0 {
		t.Errorf("len(data) is %d, want a non-multiple of 32", len(data))
	}

	receivedMap := map[string]interface{}{}
	expectedReceivedMap := map[string]interface{}{
		"sender": common.HexToAddress("0x376c47978271565f56DEB45495afa69E59c16Ab2"),
		"amount": big.NewInt(1),
		"memo":   []byte{88},
	}
	if err := abi.UnpackIntoMap(receivedMap, "received", data); err != nil {
		t.Error(err)
	}
	if len(receivedMap) != 3 {
		t.Error("unpacked `received` map expected to have length 3")
	}
	if receivedMap["sender"] != expectedReceivedMap["sender"] {
		t.Error("unpacked `received` map does not match expected map")
	}
	if receivedMap["amount"].(*big.Int).Cmp(expectedReceivedMap["amount"].(*big.Int)) != 0 {
		t.Error("unpacked `received` map does not match expected map")
	}
	if !bytes.Equal(receivedMap["memo"].([]byte), expectedReceivedMap["memo"].([]byte)) {
		t.Error("unpacked `received` map does not match expected map")
	}

	receivedAddrMap := map[string]interface{}{}
	if err = abi.UnpackIntoMap(receivedAddrMap, "receivedAddr", data); err != nil {
		t.Error(err)
	}
	if len(receivedAddrMap) != 1 {
		t.Error("unpacked `receivedAddr` map expected to have length 1")
	}
	if receivedAddrMap["sender"] != expectedReceivedMap["sender"] {
		t.Error("unpacked `receivedAddr` map does not match expected map")
	}
}

func TestUnpackMethodIntoMap(t *testing.T) {
	const abiJSON = `[{"constant":false,"inputs":[{"name":"memo","type":"bytes"}],"name":"receive","outputs":[],"payable":true,"stateMutability":"payable","type":"function"},{"constant":false,"inputs":[],"name":"send","outputs":[{"name":"amount","type":"uint256"}],"payable":true,"stateMutability":"payable","type":"function"},{"constant":false,"inputs":[{"name":"addr","type":"address"}],"name":"get","outputs":[{"name":"hash","type":"bytes"}],"payable":true,"stateMutability":"payable","type":"function"}]`
	abi, err := JSON(strings.NewReader(abiJSON))
	if err != nil {
		t.Fatal(err)
	}
	const hexdata = `00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000015800000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000158000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000001580000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000015800000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000158`
	data, err := hex.DecodeString(hexdata)
	if err != nil {
		t.Fatal(err)
	}
	if len(data)%32 != 0 {
		t.Errorf("len(data) is %d, want a multiple of 32", len(data))
	}

	// Tests a method with no outputs
	receiveMap := map[string]interface{}{}
	if err = abi.UnpackIntoMap(receiveMap, "receive", data); err != nil {
		t.Error(err)
	}
	if len(receiveMap) > 0 {
		t.Error("unpacked `receive` map expected to have length 0")
	}

	// Tests a method with only outputs
	sendMap := map[string]interface{}{}
	if err = abi.UnpackIntoMap(sendMap, "send", data); err != nil {
		t.Error(err)
	}
	if len(sendMap) != 1 {
		t.Error("unpacked `send` map expected to have length 1")
	}
	if sendMap["amount"].(*big.Int).Cmp(big.NewInt(1)) != 0 {
		t.Error("unpacked `send` map expected `amount` value of 1")
	}

	// Tests a method with outputs and inputs
	getMap := map[string]interface{}{}
	if err = abi.UnpackIntoMap(getMap, "get", data); err != nil {
		t.Error(err)
	}
	if len(getMap) != 1 {
		t.Error("unpacked `get` map expected to have length 1")
	}
	expectedBytes := []byte{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 96, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 88, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 96, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 88, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 96, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 88, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 96, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 88, 0}
	if !bytes.Equal(getMap["hash"].([]byte), expectedBytes) {
		t.Errorf("unpacked `get` map expected `hash` value of %v", expectedBytes)
	}
}

func TestUnpackIntoMapNamingConflict(t *testing.T) {
	// Two methods have the same name
	var abiJSON = `[{"constant":false,"inputs":[{"name":"memo","type":"bytes"}],"name":"get","outputs":[],"payable":true,"stateMutability":"payable","type":"function"},{"constant":false,"inputs":[],"name":"send","outputs":[{"name":"amount","type":"uint256"}],"payable":true,"stateMutability":"payable","type":"function"},{"constant":false,"inputs":[{"name":"addr","type":"address"}],"name":"get","outputs":[{"name":"hash","type":"bytes"}],"payable":true,"stateMutability":"payable","type":"function"}]`
	abi, err := JSON(strings.NewReader(abiJSON))
	if err != nil {
		t.Fatal(err)
	}
	var hexdata = `00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000158`
	data, err := hex.DecodeString(hexdata)
	if err != nil {
		t.Fatal(err)
	}
	if len(data)%32 == 0 {
		t.Errorf("len(data) is %d, want a non-multiple of 32", len(data))
	}
	getMap := map[string]interface{}{}
	if err = abi.UnpackIntoMap(getMap, "get", data); err == nil {
		t.Error("naming conflict between two methods; error expected")
	}

	// Two events have the same name
	abiJSON = `[{"constant":false,"inputs":[{"name":"memo","type":"bytes"}],"name":"receive","outputs":[],"payable":true,"stateMutability":"payable","type":"function"},{"anonymous":false,"inputs":[{"indexed":false,"name":"sender","type":"address"},{"indexed":false,"name":"amount","type":"uint256"},{"indexed":false,"name":"memo","type":"bytes"}],"name":"received","type":"event"},{"anonymous":false,"inputs":[{"indexed":false,"name":"sender","type":"address"}],"name":"received","type":"event"}]`
	abi, err = JSON(strings.NewReader(abiJSON))
	if err != nil {
		t.Fatal(err)
	}
	hexdata = `000000000000000000000000376c47978271565f56deb45495afa69e59c16ab200000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000158`
	data, err = hex.DecodeString(hexdata)
	if err != nil {
		t.Fatal(err)
	}
	if len(data)%32 == 0 {
		t.Errorf("len(data) is %d, want a non-multiple of 32", len(data))
	}
	receivedMap := map[string]interface{}{}
	if err = abi.UnpackIntoMap(receivedMap, "received", data); err != nil {
		t.Error("naming conflict between two events; no error expected")
	}

	// Method and event have the same name
	abiJSON = `[{"constant":false,"inputs":[{"name":"memo","type":"bytes"}],"name":"received","outputs":[],"payable":true,"stateMutability":"payable","type":"function"},{"anonymous":false,"inputs":[{"indexed":false,"name":"sender","type":"address"},{"indexed":false,"name":"amount","type":"uint256"},{"indexed":false,"name":"memo","type":"bytes"}],"name":"received","type":"event"},{"anonymous":false,"inputs":[{"indexed":false,"name":"sender","type":"address"}],"name":"receivedAddr","type":"event"}]`
	abi, err = JSON(strings.NewReader(abiJSON))
	if err != nil {
		t.Fatal(err)
	}
	if len(data)%32 == 0 {
		t.Errorf("len(data) is %d, want a non-multiple of 32", len(data))
	}
	if err = abi.UnpackIntoMap(receivedMap, "received", data); err == nil {
		t.Error("naming conflict between an event and a method; error expected")
	}

	// Conflict is case sensitive
	abiJSON = `[{"constant":false,"inputs":[{"name":"memo","type":"bytes"}],"name":"received","outputs":[],"payable":true,"stateMutability":"payable","type":"function"},{"anonymous":false,"inputs":[{"indexed":false,"name":"sender","type":"address"},{"indexed":false,"name":"amount","type":"uint256"},{"indexed":false,"name":"memo","type":"bytes"}],"name":"Received","type":"event"},{"anonymous":false,"inputs":[{"indexed":false,"name":"sender","type":"address"}],"name":"receivedAddr","type":"event"}]`
	abi, err = JSON(strings.NewReader(abiJSON))
	if err != nil {
		t.Fatal(err)
	}
	if len(data)%32 == 0 {
		t.Errorf("len(data) is %d, want a non-multiple of 32", len(data))
	}
	expectedReceivedMap := map[string]interface{}{
		"sender": common.HexToAddress("0x376c47978271565f56DEB45495afa69E59c16Ab2"),
		"amount": big.NewInt(1),
		"memo":   []byte{88},
	}
	if err = abi.UnpackIntoMap(receivedMap, "Received", data); err != nil {
		t.Error(err)
	}
	if len(receivedMap) != 3 {
		t.Error("unpacked `received` map expected to have length 3")
	}
	if receivedMap["sender"] != expectedReceivedMap["sender"] {
		t.Error("unpacked `received` map does not match expected map")
	}
	if receivedMap["amount"].(*big.Int).Cmp(expectedReceivedMap["amount"].(*big.Int)) != 0 {
		t.Error("unpacked `received` map does not match expected map")
	}
	if !bytes.Equal(receivedMap["memo"].([]byte), expectedReceivedMap["memo"].([]byte)) {
		t.Error("unpacked `received` map does not match expected map")
	}
}

func TestABI_MethodById(t *testing.T) {
	abi, err := JSON(strings.NewReader(jsondata))
	if err != nil {
		t.Fatal(err)
	}
	for name, m := range abi.Methods {
		a := fmt.Sprintf("%v", m)
		m2, err := abi.MethodById(m.ID)
		if err != nil {
			t.Fatalf("Failed to look up ABI method: %v", err)
		}
		b := fmt.Sprintf("%v", m2)
		if a != b {
			t.Errorf("Method %v (id %x) not 'findable' by id in ABI", name, m.ID)
		}
	}
	// test unsuccessful lookups
	if _, err = abi.MethodById(crypto.Keccak256()); err == nil {
		t.Error("Expected error: no method with this id")
	}
	// Also test empty
	if _, err := abi.MethodById([]byte{0x00}); err == nil {
		t.Errorf("Expected error, too short to decode data")
	}
	if _, err := abi.MethodById([]byte{}); err == nil {
		t.Errorf("Expected error, too short to decode data")
	}
	if _, err := abi.MethodById(nil); err == nil {
		t.Errorf("Expected error, nil is short to decode data")
	}
}

func TestABI_EventById(t *testing.T) {
	tests := []struct {
		name  string
		json  string
		event string
	}{
		{
			name: "",
			json: `[
			{"type":"event","name":"received","anonymous":false,"inputs":[
				{"indexed":false,"name":"sender","type":"address"},
				{"indexed":false,"name":"amount","type":"uint256"},
				{"indexed":false,"name":"memo","type":"bytes"}
				]
			}]`,
			event: "received(address,uint256,bytes)",
		}, {
			name: "",
			json: `[
				{ "constant": true, "inputs": [], "name": "name", "outputs": [ { "name": "", "type": "string" } ], "payable": false, "stateMutability": "view", "type": "function" },
				{ "constant": false, "inputs": [ { "name": "_spender", "type": "address" }, { "name": "_value", "type": "uint256" } ], "name": "approve", "outputs": [ { "name": "", "type": "bool" } ], "payable": false, "stateMutability": "nonpayable", "type": "function" },
				{ "constant": true, "inputs": [], "name": "totalSupply", "outputs": [ { "name": "", "type": "uint256" } ], "payable": false, "stateMutability": "view", "type": "function" },
				{ "constant": false, "inputs": [ { "name": "_from", "type": "address" }, { "name": "_to", "type": "address" }, { "name": "_value", "type": "uint256" } ], "name": "transferFrom", "outputs": [ { "name": "", "type": "bool" } ], "payable": false, "stateMutability": "nonpayable", "type": "function" },
				{ "constant": true, "inputs": [], "name": "decimals", "outputs": [ { "name": "", "type": "uint8" } ], "payable": false, "stateMutability": "view", "type": "function" },
				{ "constant": true, "inputs": [ { "name": "_owner", "type": "address" } ], "name": "balanceOf", "outputs": [ { "name": "balance", "type": "uint256" } ], "payable": false, "stateMutability": "view", "type": "function" },
				{ "constant": true, "inputs": [], "name": "symbol", "outputs": [ { "name": "", "type": "string" } ], "payable": false, "stateMutability": "view", "type": "function" },
				{ "constant": false, "inputs": [ { "name": "_to", "type": "address" }, { "name": "_value", "type": "uint256" } ], "name": "transfer", "outputs": [ { "name": "", "type": "bool" } ], "payable": false, "stateMutability": "nonpayable", "type": "function" },
				{ "constant": true, "inputs": [ { "name": "_owner", "type": "address" }, { "name": "_spender", "type": "address" } ], "name": "allowance", "outputs": [ { "name": "", "type": "uint256" } ], "payable": false, "stateMutability": "view", "type": "function" },
				{ "payable": true, "stateMutability": "payable", "type": "fallback" },
				{ "anonymous": false, "inputs": [ { "indexed": true, "name": "owner", "type": "address" }, { "indexed": true, "name": "spender", "type": "address" }, { "indexed": false, "name": "value", "type": "uint256" } ], "name": "Approval", "type": "event" },
				{ "anonymous": false, "inputs": [ { "indexed": true, "name": "from", "type": "address" }, { "indexed": true, "name": "to", "type": "address" }, { "indexed": false, "name": "value", "type": "uint256" } ], "name": "Transfer", "type": "event" }
			]`,
			event: "Transfer(address,address,uint256)",
		},
	}

	for testnum, test := range tests {
		abi, err := JSON(strings.NewReader(test.json))
		if err != nil {
			t.Error(err)
		}

		topic := test.event
		topicID := crypto.Keccak256Hash([]byte(topic))

		event, err := abi.EventByID(topicID)
		if err != nil {
			t.Fatalf("Failed to look up ABI method: %v, test #%d", err, testnum)
		}
		if event == nil {
			t.Errorf("We should find a event for topic %s, test #%d", topicID.Hex(), testnum)
		} else if event.ID != topicID {
			t.Errorf("Event id %s does not match topic %s, test #%d", event.ID.Hex(), topicID.Hex(), testnum)
		}

		unknowntopicID := crypto.Keccak256Hash([]byte("unknownEvent"))
		unknownEvent, err := abi.EventByID(unknowntopicID)
		if err == nil {
			t.Errorf("EventByID should return an error if a topic is not found, test #%d", testnum)
		}
		if unknownEvent != nil {
			t.Errorf("We should not find any event for topic %s, test #%d", unknowntopicID.Hex(), testnum)
		}
	}
}

func TestABI_ErrorByID(t *testing.T) {
	abi, err := JSON(strings.NewReader(`[
		{"inputs":[{"internalType":"uint256","name":"x","type":"uint256"}],"name":"MyError1","type":"error"},
		{"inputs":[{"components":[{"internalType":"uint256","name":"a","type":"uint256"},{"internalType":"string","name":"b","type":"string"},{"internalType":"address","name":"c","type":"address"}],"internalType":"struct MyError.MyStruct","name":"x","type":"tuple"},{"internalType":"address","name":"y","type":"address"},{"components":[{"internalType":"uint256","name":"a","type":"uint256"},{"internalType":"string","name":"b","type":"string"},{"internalType":"address","name":"c","type":"address"}],"internalType":"struct MyError.MyStruct","name":"z","type":"tuple"}],"name":"MyError2","type":"error"},
		{"inputs":[{"internalType":"uint256[]","name":"x","type":"uint256[]"}],"name":"MyError3","type":"error"}
	]`))
	if err != nil {
		t.Fatal(err)
	}
	for name, m := range abi.Errors {
		a := fmt.Sprintf("%v", &m)
		var id [4]byte
		copy(id[:], m.ID[:4])
		m2, err := abi.ErrorByID(id)
		if err != nil {
			t.Fatalf("Failed to look up ABI error: %v", err)
		}
		b := fmt.Sprintf("%v", m2)
		if a != b {
			t.Errorf("Error %v (id %x) not 'findable' by id in ABI", name, id)
		}
	}
	// test unsuccessful lookups
	if _, err = abi.ErrorByID([4]byte{}); err == nil {
		t.Error("Expected error: no error with this id")
	}
}

// TestDoubleDuplicateMethodNames checks that if transfer0 already exists, there won't be a name
// conflict and that the second transfer method will be renamed transfer1.
func TestDoubleDuplicateMethodNames(t *testing.T) {
	abiJSON := `[{"constant":false,"inputs":[{"name":"to","type":"address"},{"name":"value","type":"uint256"}],"name":"transfer","outputs":[{"name":"ok","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"to","type":"address"},{"name":"value","type":"uint256"},{"name":"data","type":"bytes"}],"name":"transfer0","outputs":[{"name":"ok","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"to","type":"address"},{"name":"value","type":"uint256"},{"name":"data","type":"bytes"},{"name":"customFallback","type":"string"}],"name":"transfer","outputs":[{"name":"ok","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"}]`
	contractAbi, err := JSON(strings.NewReader(abiJSON))
	if err != nil {
		t.Fatal(err)
	}
	if _, ok := contractAbi.Methods["transfer"]; !ok {
		t.Fatalf("Could not find original method")
	}
	if _, ok := contractAbi.Methods["transfer0"]; !ok {
		t.Fatalf("Could not find duplicate method")
	}
	if _, ok := contractAbi.Methods["transfer1"]; !ok {
		t.Fatalf("Could not find duplicate method")
	}
	if _, ok := contractAbi.Methods["transfer2"]; ok {
		t.Fatalf("Should not have found extra method")
	}
}

// TestDoubleDuplicateEventNames checks that if send0 already exists, there won't be a name
// conflict and that the second send event will be renamed send1.
// The test runs the abi of the following contract.
//
//	contract DuplicateEvent {
//		event send(uint256 a);
//		event send0();
//		event send();
//	}
func TestDoubleDuplicateEventNames(t *testing.T) {
	abiJSON := `[{"anonymous": false,"inputs": [{"indexed": false,"internalType": "uint256","name": "a","type": "uint256"}],"name": "send","type": "event"},{"anonymous": false,"inputs": [],"name": "send0","type": "event"},{	"anonymous": false,	"inputs": [],"name": "send","type": "event"}]`
	contractAbi, err := JSON(strings.NewReader(abiJSON))
	if err != nil {
		t.Fatal(err)
	}
	if _, ok := contractAbi.Events["send"]; !ok {
		t.Fatalf("Could not find original event")
	}
	if _, ok := contractAbi.Events["send0"]; !ok {
		t.Fatalf("Could not find duplicate event")
	}
	if _, ok := contractAbi.Events["send1"]; !ok {
		t.Fatalf("Could not find duplicate event")
	}
	if _, ok := contractAbi.Events["send2"]; ok {
		t.Fatalf("Should not have found extra event")
	}
}

// TestUnnamedEventParam checks that an event with unnamed parameters is
// correctly handled.
// The test runs the abi of the following contract.
//
//	contract TestEvent {
//		event send(uint256, uint256);
//	}
func TestUnnamedEventParam(t *testing.T) {
	abiJSON := `[{ "anonymous": false, "inputs": [{	"indexed": false,"internalType": "uint256",	"name": "","type": "uint256"},{"indexed": false,"internalType": "uint256","name": "","type": "uint256"}],"name": "send","type": "event"}]`
	contractAbi, err := JSON(strings.NewReader(abiJSON))
	if err != nil {
		t.Fatal(err)
	}

	event, ok := contractAbi.Events["send"]
	if !ok {
		t.Fatalf("Could not find event")
	}
	if event.Inputs[0].Name != "arg0" {
		t.Fatalf("Could not find input")
	}
	if event.Inputs[1].Name != "arg1" {
		t.Fatalf("Could not find input")
	}
}

func TestUnpackRevert(t *testing.T) {
	t.Parallel()

	var cases = []struct {
		input     string
		expect    string
		expectErr error
	}{
		{"", "", errors.New("invalid data for unpacking")},
		{"08c379a1", "", errors.New("invalid data for unpacking")},
		{"08c379a00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000d72657665727420726561736f6e00000000000000000000000000000000000000", "revert reason", nil},
		{"4e487b710000000000000000000000000000000000000000000000000000000000000000", "generic panic", nil},
		{"4e487b7100000000000000000000000000000000000000000000000000000000000000ff", "unknown panic code: 0xff", nil},
	}
	for index, c := range cases {
		t.Run(fmt.Sprintf("case %d", index), func(t *testing.T) {
			got, err := UnpackRevert(common.Hex2Bytes(c.input))
			if c.expectErr != nil {
				if err == nil {
					t.Fatalf("Expected non-nil error")
				}
				if err.Error() != c.expectErr.Error() {
					t.Fatalf("Expected error mismatch, want %v, got %v", c.expectErr, err)
				}
				return
			}
			if c.expect != got {
				t.Fatalf("Output mismatch, want %v, got %v", c.expect, got)
			}
		})
	}
}
