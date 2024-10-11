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

package abi

import (
	"bytes"
	"encoding/hex"
	"fmt"
	"math"
	"math/big"
	"reflect"
	"strconv"
	"strings"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/stretchr/testify/require"
)

// TestUnpack tests the general pack/unpack tests in packing_test.go
func TestUnpack(t *testing.T) {
	for i, test := range packUnpackTests {
		t.Run(strconv.Itoa(i)+" "+test.def, func(t *testing.T) {
			//Unpack
			def := fmt.Sprintf(`[{ "name" : "method", "type": "function", "outputs": %s}]`, test.def)
			abi, err := JSON(strings.NewReader(def))
			if err != nil {
				t.Fatalf("invalid ABI definition %s: %v", def, err)
			}
			encb, err := hex.DecodeString(test.packed)
			if err != nil {
				t.Fatalf("invalid hex %s: %v", test.packed, err)
			}
			out, err := abi.Unpack("method", encb)
			if err != nil {
				t.Errorf("test %d (%v) failed: %v", i, test.def, err)
				return
			}
			if !reflect.DeepEqual(test.unpacked, ConvertType(out[0], test.unpacked)) {
				t.Errorf("test %d (%v) failed: expected %v, got %v", i, test.def, test.unpacked, out[0])
			}
		})
	}
}

type unpackTest struct {
	def  string      // ABI definition JSON
	enc  string      // evm return data
	want interface{} // the expected output
	err  string      // empty or error if expected
}

func (test unpackTest) checkError(err error) error {
	if err != nil {
		if len(test.err) == 0 {
			return fmt.Errorf("expected no err but got: %v", err)
		} else if err.Error() != test.err {
			return fmt.Errorf("expected err: '%v' got err: %q", test.err, err)
		}
	} else if len(test.err) > 0 {
		return fmt.Errorf("expected err: %v but got none", test.err)
	}
	return nil
}

var unpackTests = []unpackTest{
	// Bools
	{
		def:  `[{ "type": "bool" }]`,
		enc:  "0000000000000000000000000000000000000000000000000001000000000001",
		want: false,
		err:  "abi: improperly encoded boolean value",
	},
	{
		def:  `[{ "type": "bool" }]`,
		enc:  "0000000000000000000000000000000000000000000000000000000000000003",
		want: false,
		err:  "abi: improperly encoded boolean value",
	},
	// Integers
	{
		def:  `[{"type": "uint32"}]`,
		enc:  "0000000000000000000000000000000000000000000000000000000000000001",
		want: uint16(0),
		err:  "abi: cannot unmarshal uint32 in to uint16",
	},
	{
		def:  `[{"type": "uint17"}]`,
		enc:  "0000000000000000000000000000000000000000000000000000000000000001",
		want: uint16(0),
		err:  "abi: cannot unmarshal *big.Int in to uint16",
	},
	{
		def:  `[{"type": "int32"}]`,
		enc:  "0000000000000000000000000000000000000000000000000000000000000001",
		want: int16(0),
		err:  "abi: cannot unmarshal int32 in to int16",
	},
	{
		def:  `[{"type": "int17"}]`,
		enc:  "0000000000000000000000000000000000000000000000000000000000000001",
		want: int16(0),
		err:  "abi: cannot unmarshal *big.Int in to int16",
	},
	{
		def:  `[{"type": "bytes"}]`,
		enc:  "000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000200100000000000000000000000000000000000000000000000000000000000000",
		want: [32]byte{1},
	},
	{
		def:  `[{"type": "bytes32"}]`,
		enc:  "000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000200100000000000000000000000000000000000000000000000000000000000000",
		want: []byte(nil),
		err:  "abi: cannot unmarshal [32]uint8 in to []uint8",
	},
	{
		def: `[{"name":"___","type":"int256"}]`,
		enc: "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002",
		want: struct {
			IntOne *big.Int
			Intone *big.Int
		}{IntOne: big.NewInt(1)},
	},
	{
		def: `[{"name":"int_one","type":"int256"},{"name":"IntOne","type":"int256"}]`,
		enc: "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002",
		want: struct {
			Int1 *big.Int
			Int2 *big.Int
		}{},
		err: "abi: multiple outputs mapping to the same struct field 'IntOne'",
	},
	{
		def: `[{"name":"int","type":"int256"},{"name":"Int","type":"int256"}]`,
		enc: "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002",
		want: struct {
			Int1 *big.Int
			Int2 *big.Int
		}{},
		err: "abi: multiple outputs mapping to the same struct field 'Int'",
	},
	{
		def: `[{"name":"int","type":"int256"},{"name":"_int","type":"int256"}]`,
		enc: "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002",
		want: struct {
			Int1 *big.Int
			Int2 *big.Int
		}{},
		err: "abi: multiple outputs mapping to the same struct field 'Int'",
	},
	{
		def: `[{"name":"Int","type":"int256"},{"name":"_int","type":"int256"}]`,
		enc: "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002",
		want: struct {
			Int1 *big.Int
			Int2 *big.Int
		}{},
		err: "abi: multiple outputs mapping to the same struct field 'Int'",
	},
	{
		def: `[{"name":"Int","type":"int256"},{"name":"_","type":"int256"}]`,
		enc: "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002",
		want: struct {
			Int1 *big.Int
			Int2 *big.Int
		}{},
		err: "abi: purely underscored output cannot unpack to struct",
	},
	// Make sure only the first argument is consumed
	{
		def: `[{"name":"int_one","type":"int256"}]`,
		enc: "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002",
		want: struct {
			IntOne *big.Int
		}{big.NewInt(1)},
	},
	{
		def: `[{"name":"int__one","type":"int256"}]`,
		enc: "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002",
		want: struct {
			IntOne *big.Int
		}{big.NewInt(1)},
	},
	{
		def: `[{"name":"int_one_","type":"int256"}]`,
		enc: "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000002",
		want: struct {
			IntOne *big.Int
		}{big.NewInt(1)},
	},
	{
		def:  `[{"type":"bool"}]`,
		enc:  "",
		want: false,
		err:  "abi: attempting to unmarshall an empty string while arguments are expected",
	},
	{
		def:  `[{"type":"bytes32","indexed":true},{"type":"uint256","indexed":false}]`,
		enc:  "",
		want: false,
		err:  "abi: attempting to unmarshall an empty string while arguments are expected",
	},
	{
		def:  `[{"type":"bool","indexed":true},{"type":"uint64","indexed":true}]`,
		enc:  "",
		want: false,
	},
}

// TestLocalUnpackTests runs test specially designed only for unpacking.
// All test cases that can be used to test packing and unpacking should move to packing_test.go
func TestLocalUnpackTests(t *testing.T) {
	for i, test := range unpackTests {
		t.Run(strconv.Itoa(i), func(t *testing.T) {
			//Unpack
			def := fmt.Sprintf(`[{ "name" : "method", "type": "function", "outputs": %s}]`, test.def)
			abi, err := JSON(strings.NewReader(def))
			if err != nil {
				t.Fatalf("invalid ABI definition %s: %v", def, err)
			}
			encb, err := hex.DecodeString(test.enc)
			if err != nil {
				t.Fatalf("invalid hex %s: %v", test.enc, err)
			}
			outptr := reflect.New(reflect.TypeOf(test.want))
			err = abi.UnpackIntoInterface(outptr.Interface(), "method", encb)
			if err := test.checkError(err); err != nil {
				t.Errorf("test %d (%v) failed: %v", i, test.def, err)
				return
			}
			out := outptr.Elem().Interface()
			if !reflect.DeepEqual(test.want, out) {
				t.Errorf("test %d (%v) failed: expected %v, got %v", i, test.def, test.want, out)
			}
		})
	}
}

func TestUnpackIntoInterfaceSetDynamicArrayOutput(t *testing.T) {
	abi, err := JSON(strings.NewReader(`[{"constant":true,"inputs":[],"name":"testDynamicFixedBytes15","outputs":[{"name":"","type":"bytes15[]"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"testDynamicFixedBytes32","outputs":[{"name":"","type":"bytes32[]"}],"payable":false,"stateMutability":"view","type":"function"}]`))
	if err != nil {
		t.Fatal(err)
	}

	var (
		marshalledReturn32 = common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000230783132333435363738393000000000000000000000000000000000000000003078303938373635343332310000000000000000000000000000000000000000")
		marshalledReturn15 = common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000230783031323334350000000000000000000000000000000000000000000000003078393837363534000000000000000000000000000000000000000000000000")

		out32 [][32]byte
		out15 [][15]byte
	)

	// test 32
	err = abi.UnpackIntoInterface(&out32, "testDynamicFixedBytes32", marshalledReturn32)
	if err != nil {
		t.Fatal(err)
	}
	if len(out32) != 2 {
		t.Fatalf("expected array with 2 values, got %d", len(out32))
	}
	expected := common.Hex2Bytes("3078313233343536373839300000000000000000000000000000000000000000")
	if !bytes.Equal(out32[0][:], expected) {
		t.Errorf("expected %x, got %x\n", expected, out32[0])
	}
	expected = common.Hex2Bytes("3078303938373635343332310000000000000000000000000000000000000000")
	if !bytes.Equal(out32[1][:], expected) {
		t.Errorf("expected %x, got %x\n", expected, out32[1])
	}

	// test 15
	err = abi.UnpackIntoInterface(&out15, "testDynamicFixedBytes32", marshalledReturn15)
	if err != nil {
		t.Fatal(err)
	}
	if len(out15) != 2 {
		t.Fatalf("expected array with 2 values, got %d", len(out15))
	}
	expected = common.Hex2Bytes("307830313233343500000000000000")
	if !bytes.Equal(out15[0][:], expected) {
		t.Errorf("expected %x, got %x\n", expected, out15[0])
	}
	expected = common.Hex2Bytes("307839383736353400000000000000")
	if !bytes.Equal(out15[1][:], expected) {
		t.Errorf("expected %x, got %x\n", expected, out15[1])
	}
}

type methodMultiOutput struct {
	Int    *big.Int
	String string
}

func methodMultiReturn(require *require.Assertions) (ABI, []byte, methodMultiOutput) {
	const definition = `[
	{ "name" : "multi", "type": "function", "outputs": [ { "name": "Int", "type": "uint256" }, { "name": "String", "type": "string" } ] }]`
	var expected = methodMultiOutput{big.NewInt(1), "hello"}

	abi, err := JSON(strings.NewReader(definition))
	require.NoError(err)
	// using buff to make the code readable
	buff := new(bytes.Buffer)
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000001"))
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000040"))
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000005"))
	buff.Write(common.RightPadBytes([]byte(expected.String), 32))
	return abi, buff.Bytes(), expected
}

func TestMethodMultiReturn(t *testing.T) {
	type reversed struct {
		String string
		Int    *big.Int
	}

	newInterfaceSlice := func(len int) interface{} {
		slice := make([]interface{}, len)
		return &slice
	}

	abi, data, expected := methodMultiReturn(require.New(t))
	bigint := new(big.Int)
	var testCases = []struct {
		dest     interface{}
		expected interface{}
		error    string
		name     string
	}{{
		&methodMultiOutput{},
		&expected,
		"",
		"Can unpack into structure",
	}, {
		&reversed{},
		&reversed{expected.String, expected.Int},
		"",
		"Can unpack into reversed structure",
	}, {
		&[]interface{}{&bigint, new(string)},
		&[]interface{}{&expected.Int, &expected.String},
		"",
		"Can unpack into a slice",
	}, {
		&[]interface{}{&bigint, ""},
		&[]interface{}{&expected.Int, expected.String},
		"",
		"Can unpack into a slice without indirection",
	}, {
		&[2]interface{}{&bigint, new(string)},
		&[2]interface{}{&expected.Int, &expected.String},
		"",
		"Can unpack into an array",
	}, {
		&[2]interface{}{},
		&[2]interface{}{expected.Int, expected.String},
		"",
		"Can unpack into interface array",
	}, {
		newInterfaceSlice(2),
		&[]interface{}{expected.Int, expected.String},
		"",
		"Can unpack into interface slice",
	}, {
		&[]interface{}{new(int), new(int)},
		&[]interface{}{&expected.Int, &expected.String},
		"abi: cannot unmarshal *big.Int in to int",
		"Can not unpack into a slice with wrong types",
	}, {
		&[]interface{}{new(int)},
		&[]interface{}{},
		"abi: insufficient number of arguments for unpack, want 2, got 1",
		"Can not unpack into a slice with wrong types",
	}}
	for _, tc := range testCases {
		tc := tc
		t.Run(tc.name, func(t *testing.T) {
			require := require.New(t)
			err := abi.UnpackIntoInterface(tc.dest, "multi", data)
			if tc.error == "" {
				require.Nil(err, "Should be able to unpack method outputs.")
				require.Equal(tc.expected, tc.dest)
			} else {
				require.EqualError(err, tc.error)
			}
		})
	}
}

func TestMultiReturnWithArray(t *testing.T) {
	const definition = `[{"name" : "multi", "type": "function", "outputs": [{"type": "uint64[3]"}, {"type": "uint64"}]}]`
	abi, err := JSON(strings.NewReader(definition))
	if err != nil {
		t.Fatal(err)
	}
	buff := new(bytes.Buffer)
	buff.Write(common.Hex2Bytes("000000000000000000000000000000000000000000000000000000000000000900000000000000000000000000000000000000000000000000000000000000090000000000000000000000000000000000000000000000000000000000000009"))
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000008"))

	ret1, ret1Exp := new([3]uint64), [3]uint64{9, 9, 9}
	ret2, ret2Exp := new(uint64), uint64(8)
	if err := abi.UnpackIntoInterface(&[]interface{}{ret1, ret2}, "multi", buff.Bytes()); err != nil {
		t.Fatal(err)
	}
	if !reflect.DeepEqual(*ret1, ret1Exp) {
		t.Error("array result", *ret1, "!= Expected", ret1Exp)
	}
	if *ret2 != ret2Exp {
		t.Error("int result", *ret2, "!= Expected", ret2Exp)
	}
}

func TestMultiReturnWithStringArray(t *testing.T) {
	const definition = `[{"name" : "multi", "type": "function", "outputs": [{"name": "","type": "uint256[3]"},{"name": "","type": "address"},{"name": "","type": "string[2]"},{"name": "","type": "bool"}]}]`
	abi, err := JSON(strings.NewReader(definition))
	if err != nil {
		t.Fatal(err)
	}
	buff := new(bytes.Buffer)
	buff.Write(common.Hex2Bytes("000000000000000000000000000000000000000000000000000000005c1b78ea0000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000001a055690d9db80000000000000000000000000000ab1257528b3782fb40d7ed5f72e624b744dffb2f00000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000008457468657265756d000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001048656c6c6f2c20457468657265756d2100000000000000000000000000000000"))
	temp, _ := new(big.Int).SetString("30000000000000000000", 10)
	ret1, ret1Exp := new([3]*big.Int), [3]*big.Int{big.NewInt(1545304298), big.NewInt(6), temp}
	ret2, ret2Exp := new(common.Address), common.HexToAddress("ab1257528b3782fb40d7ed5f72e624b744dffb2f")
	ret3, ret3Exp := new([2]string), [2]string{"Ethereum", "Hello, Ethereum!"}
	ret4, ret4Exp := new(bool), false
	if err := abi.UnpackIntoInterface(&[]interface{}{ret1, ret2, ret3, ret4}, "multi", buff.Bytes()); err != nil {
		t.Fatal(err)
	}
	if !reflect.DeepEqual(*ret1, ret1Exp) {
		t.Error("big.Int array result", *ret1, "!= Expected", ret1Exp)
	}
	if !reflect.DeepEqual(*ret2, ret2Exp) {
		t.Error("address result", *ret2, "!= Expected", ret2Exp)
	}
	if !reflect.DeepEqual(*ret3, ret3Exp) {
		t.Error("string array result", *ret3, "!= Expected", ret3Exp)
	}
	if !reflect.DeepEqual(*ret4, ret4Exp) {
		t.Error("bool result", *ret4, "!= Expected", ret4Exp)
	}
}

func TestMultiReturnWithStringSlice(t *testing.T) {
	const definition = `[{"name" : "multi", "type": "function", "outputs": [{"name": "","type": "string[]"},{"name": "","type": "uint256[]"}]}]`
	abi, err := JSON(strings.NewReader(definition))
	if err != nil {
		t.Fatal(err)
	}
	buff := new(bytes.Buffer)
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000040")) // output[0] offset
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000120")) // output[1] offset
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000002")) // output[0] length
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000040")) // output[0][0] offset
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000080")) // output[0][1] offset
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000008")) // output[0][0] length
	buff.Write(common.Hex2Bytes("657468657265756d000000000000000000000000000000000000000000000000")) // output[0][0] value
	buff.Write(common.Hex2Bytes("000000000000000000000000000000000000000000000000000000000000000b")) // output[0][1] length
	buff.Write(common.Hex2Bytes("676f2d657468657265756d000000000000000000000000000000000000000000")) // output[0][1] value
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000002")) // output[1] length
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000064")) // output[1][0] value
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000065")) // output[1][1] value
	ret1, ret1Exp := new([]string), []string{"ethereum", "go-ethereum"}
	ret2, ret2Exp := new([]*big.Int), []*big.Int{big.NewInt(100), big.NewInt(101)}
	if err := abi.UnpackIntoInterface(&[]interface{}{ret1, ret2}, "multi", buff.Bytes()); err != nil {
		t.Fatal(err)
	}
	if !reflect.DeepEqual(*ret1, ret1Exp) {
		t.Error("string slice result", *ret1, "!= Expected", ret1Exp)
	}
	if !reflect.DeepEqual(*ret2, ret2Exp) {
		t.Error("uint256 slice result", *ret2, "!= Expected", ret2Exp)
	}
}

func TestMultiReturnWithDeeplyNestedArray(t *testing.T) {
	// Similar to TestMultiReturnWithArray, but with a special case in mind:
	//  values of nested static arrays count towards the size as well, and any element following
	//  after such nested array argument should be read with the correct offset,
	//  so that it does not read content from the previous array argument.
	const definition = `[{"name" : "multi", "type": "function", "outputs": [{"type": "uint64[3][2][4]"}, {"type": "uint64"}]}]`
	abi, err := JSON(strings.NewReader(definition))
	if err != nil {
		t.Fatal(err)
	}
	buff := new(bytes.Buffer)
	// construct the test array, each 3 char element is joined with 61 '0' chars,
	// to from the ((3 + 61) * 0.5) = 32 byte elements in the array.
	buff.Write(common.Hex2Bytes(strings.Join([]string{
		"", //empty, to apply the 61-char separator to the first element as well.
		"111", "112", "113", "121", "122", "123",
		"211", "212", "213", "221", "222", "223",
		"311", "312", "313", "321", "322", "323",
		"411", "412", "413", "421", "422", "423",
	}, "0000000000000000000000000000000000000000000000000000000000000")))
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000009876"))

	ret1, ret1Exp := new([4][2][3]uint64), [4][2][3]uint64{
		{{0x111, 0x112, 0x113}, {0x121, 0x122, 0x123}},
		{{0x211, 0x212, 0x213}, {0x221, 0x222, 0x223}},
		{{0x311, 0x312, 0x313}, {0x321, 0x322, 0x323}},
		{{0x411, 0x412, 0x413}, {0x421, 0x422, 0x423}},
	}
	ret2, ret2Exp := new(uint64), uint64(0x9876)
	if err := abi.UnpackIntoInterface(&[]interface{}{ret1, ret2}, "multi", buff.Bytes()); err != nil {
		t.Fatal(err)
	}
	if !reflect.DeepEqual(*ret1, ret1Exp) {
		t.Error("array result", *ret1, "!= Expected", ret1Exp)
	}
	if *ret2 != ret2Exp {
		t.Error("int result", *ret2, "!= Expected", ret2Exp)
	}
}

func TestUnmarshal(t *testing.T) {
	const definition = `[
	{ "name" : "int", "type": "function", "outputs": [ { "type": "uint256" } ] },
	{ "name" : "bool", "type": "function", "outputs": [ { "type": "bool" } ] },
	{ "name" : "bytes", "type": "function", "outputs": [ { "type": "bytes" } ] },
	{ "name" : "fixed", "type": "function", "outputs": [ { "type": "bytes32" } ] },
	{ "name" : "multi", "type": "function", "outputs": [ { "type": "bytes" }, { "type": "bytes" } ] },
	{ "name" : "intArraySingle", "type": "function", "outputs": [ { "type": "uint256[3]" } ] },
	{ "name" : "addressSliceSingle", "type": "function", "outputs": [ { "type": "address[]" } ] },
	{ "name" : "addressSliceDouble", "type": "function", "outputs": [ { "name": "a", "type": "address[]" }, { "name": "b", "type": "address[]" } ] },
	{ "name" : "mixedBytes", "type": "function", "stateMutability" : "view", "outputs": [ { "name": "a", "type": "bytes" }, { "name": "b", "type": "bytes32" } ] }]`

	abi, err := JSON(strings.NewReader(definition))
	if err != nil {
		t.Fatal(err)
	}
	buff := new(bytes.Buffer)

	// marshall mixed bytes (mixedBytes)
	p0, p0Exp := []byte{}, common.Hex2Bytes("01020000000000000000")
	p1, p1Exp := [32]byte{}, common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000ddeeff")
	mixedBytes := []interface{}{&p0, &p1}

	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000040"))
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000ddeeff"))
	buff.Write(common.Hex2Bytes("000000000000000000000000000000000000000000000000000000000000000a"))
	buff.Write(common.Hex2Bytes("0102000000000000000000000000000000000000000000000000000000000000"))

	err = abi.UnpackIntoInterface(&mixedBytes, "mixedBytes", buff.Bytes())
	if err != nil {
		t.Error(err)
	} else {
		if !bytes.Equal(p0, p0Exp) {
			t.Errorf("unexpected value unpacked: want %x, got %x", p0Exp, p0)
		}

		if !bytes.Equal(p1[:], p1Exp) {
			t.Errorf("unexpected value unpacked: want %x, got %x", p1Exp, p1)
		}
	}

	// marshal int
	var Int *big.Int
	err = abi.UnpackIntoInterface(&Int, "int", common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000001"))
	if err != nil {
		t.Error(err)
	}

	if Int == nil || Int.Cmp(big.NewInt(1)) != 0 {
		t.Error("expected Int to be 1 got", Int)
	}

	// marshal bool
	var Bool bool
	err = abi.UnpackIntoInterface(&Bool, "bool", common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000001"))
	if err != nil {
		t.Error(err)
	}

	if !Bool {
		t.Error("expected Bool to be true")
	}

	// marshal dynamic bytes max length 32
	buff.Reset()
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000020"))
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000020"))
	bytesOut := common.RightPadBytes([]byte("hello"), 32)
	buff.Write(bytesOut)

	var Bytes []byte
	err = abi.UnpackIntoInterface(&Bytes, "bytes", buff.Bytes())
	if err != nil {
		t.Error(err)
	}

	if !bytes.Equal(Bytes, bytesOut) {
		t.Errorf("expected %x got %x", bytesOut, Bytes)
	}

	// marshall dynamic bytes max length 64
	buff.Reset()
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000020"))
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000040"))
	bytesOut = common.RightPadBytes([]byte("hello"), 64)
	buff.Write(bytesOut)

	err = abi.UnpackIntoInterface(&Bytes, "bytes", buff.Bytes())
	if err != nil {
		t.Error(err)
	}

	if !bytes.Equal(Bytes, bytesOut) {
		t.Errorf("expected %x got %x", bytesOut, Bytes)
	}

	// marshall dynamic bytes max length 64
	buff.Reset()
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000020"))
	buff.Write(common.Hex2Bytes("000000000000000000000000000000000000000000000000000000000000003f"))
	bytesOut = common.RightPadBytes([]byte("hello"), 64)
	buff.Write(bytesOut)

	err = abi.UnpackIntoInterface(&Bytes, "bytes", buff.Bytes())
	if err != nil {
		t.Error(err)
	}

	if !bytes.Equal(Bytes, bytesOut[:len(bytesOut)-1]) {
		t.Errorf("expected %x got %x", bytesOut[:len(bytesOut)-1], Bytes)
	}

	// marshal dynamic bytes output empty
	err = abi.UnpackIntoInterface(&Bytes, "bytes", nil)
	if err == nil {
		t.Error("expected error")
	}

	// marshal dynamic bytes length 5
	buff.Reset()
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000020"))
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000005"))
	buff.Write(common.RightPadBytes([]byte("hello"), 32))

	err = abi.UnpackIntoInterface(&Bytes, "bytes", buff.Bytes())
	if err != nil {
		t.Error(err)
	}

	if !bytes.Equal(Bytes, []byte("hello")) {
		t.Errorf("expected %x got %x", bytesOut, Bytes)
	}

	// marshal dynamic bytes length 5
	buff.Reset()
	buff.Write(common.RightPadBytes([]byte("hello"), 32))

	var hash common.Hash
	err = abi.UnpackIntoInterface(&hash, "fixed", buff.Bytes())
	if err != nil {
		t.Error(err)
	}

	helloHash := common.BytesToHash(common.RightPadBytes([]byte("hello"), 32))
	if hash != helloHash {
		t.Errorf("Expected %x to equal %x", hash, helloHash)
	}

	// marshal error
	buff.Reset()
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000020"))
	err = abi.UnpackIntoInterface(&Bytes, "bytes", buff.Bytes())
	if err == nil {
		t.Error("expected error")
	}

	err = abi.UnpackIntoInterface(&Bytes, "multi", make([]byte, 64))
	if err == nil {
		t.Error("expected error")
	}

	buff.Reset()
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000001"))
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000002"))
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000003"))
	// marshal int array
	var intArray [3]*big.Int
	err = abi.UnpackIntoInterface(&intArray, "intArraySingle", buff.Bytes())
	if err != nil {
		t.Error(err)
	}
	var testAgainstIntArray [3]*big.Int
	testAgainstIntArray[0] = big.NewInt(1)
	testAgainstIntArray[1] = big.NewInt(2)
	testAgainstIntArray[2] = big.NewInt(3)

	for i, Int := range intArray {
		if Int.Cmp(testAgainstIntArray[i]) != 0 {
			t.Errorf("expected %v, got %v", testAgainstIntArray[i], Int)
		}
	}
	// marshal address slice
	buff.Reset()
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000020")) // offset
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000001")) // size
	buff.Write(common.Hex2Bytes("0000000000000000000000000100000000000000000000000000000000000000"))

	var outAddr []common.Address
	err = abi.UnpackIntoInterface(&outAddr, "addressSliceSingle", buff.Bytes())
	if err != nil {
		t.Fatal("didn't expect error:", err)
	}

	if len(outAddr) != 1 {
		t.Fatal("expected 1 item, got", len(outAddr))
	}

	if outAddr[0] != (common.Address{1}) {
		t.Errorf("expected %x, got %x", common.Address{1}, outAddr[0])
	}

	// marshal multiple address slice
	buff.Reset()
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000040")) // offset
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000080")) // offset
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000001")) // size
	buff.Write(common.Hex2Bytes("0000000000000000000000000100000000000000000000000000000000000000"))
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000002")) // size
	buff.Write(common.Hex2Bytes("0000000000000000000000000200000000000000000000000000000000000000"))
	buff.Write(common.Hex2Bytes("0000000000000000000000000300000000000000000000000000000000000000"))

	var outAddrStruct struct {
		A []common.Address
		B []common.Address
	}
	err = abi.UnpackIntoInterface(&outAddrStruct, "addressSliceDouble", buff.Bytes())
	if err != nil {
		t.Fatal("didn't expect error:", err)
	}

	if len(outAddrStruct.A) != 1 {
		t.Fatal("expected 1 item, got", len(outAddrStruct.A))
	}

	if outAddrStruct.A[0] != (common.Address{1}) {
		t.Errorf("expected %x, got %x", common.Address{1}, outAddrStruct.A[0])
	}

	if len(outAddrStruct.B) != 2 {
		t.Fatal("expected 1 item, got", len(outAddrStruct.B))
	}

	if outAddrStruct.B[0] != (common.Address{2}) {
		t.Errorf("expected %x, got %x", common.Address{2}, outAddrStruct.B[0])
	}
	if outAddrStruct.B[1] != (common.Address{3}) {
		t.Errorf("expected %x, got %x", common.Address{3}, outAddrStruct.B[1])
	}

	// marshal invalid address slice
	buff.Reset()
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000100"))

	err = abi.UnpackIntoInterface(&outAddr, "addressSliceSingle", buff.Bytes())
	if err == nil {
		t.Fatal("expected error:", err)
	}
}

func TestUnpackTuple(t *testing.T) {
	const simpleTuple = `[{"name":"tuple","type":"function","outputs":[{"type":"tuple","name":"ret","components":[{"type":"int256","name":"a"},{"type":"int256","name":"b"}]}]}]`
	abi, err := JSON(strings.NewReader(simpleTuple))
	if err != nil {
		t.Fatal(err)
	}
	buff := new(bytes.Buffer)

	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000001")) // ret[a] = 1
	buff.Write(common.Hex2Bytes("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")) // ret[b] = -1

	// If the result is single tuple, use struct as return value container directly.
	type v struct {
		A *big.Int
		B *big.Int
	}
	type r struct {
		Result v
	}
	var ret0 = new(r)
	err = abi.UnpackIntoInterface(ret0, "tuple", buff.Bytes())

	if err != nil {
		t.Error(err)
	} else {
		if ret0.Result.A.Cmp(big.NewInt(1)) != 0 {
			t.Errorf("unexpected value unpacked: want %x, got %x", 1, ret0.Result.A)
		}
		if ret0.Result.B.Cmp(big.NewInt(-1)) != 0 {
			t.Errorf("unexpected value unpacked: want %x, got %x", -1, ret0.Result.B)
		}
	}

	// Test nested tuple
	const nestedTuple = `[{"name":"tuple","type":"function","outputs":[
		{"type":"tuple","name":"s","components":[{"type":"uint256","name":"a"},{"type":"uint256[]","name":"b"},{"type":"tuple[]","name":"c","components":[{"name":"x", "type":"uint256"},{"name":"y","type":"uint256"}]}]},
		{"type":"tuple","name":"t","components":[{"name":"x", "type":"uint256"},{"name":"y","type":"uint256"}]},
		{"type":"uint256","name":"a"}
	]}]`

	abi, err = JSON(strings.NewReader(nestedTuple))
	if err != nil {
		t.Fatal(err)
	}
	buff.Reset()
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000080")) // s offset
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000000")) // t.X = 0
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000001")) // t.Y = 1
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000001")) // a = 1
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000001")) // s.A = 1
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000060")) // s.B offset
	buff.Write(common.Hex2Bytes("00000000000000000000000000000000000000000000000000000000000000c0")) // s.C offset
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000002")) // s.B length
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000001")) // s.B[0] = 1
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000002")) // s.B[0] = 2
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000002")) // s.C length
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000001")) // s.C[0].X
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000002")) // s.C[0].Y
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000002")) // s.C[1].X
	buff.Write(common.Hex2Bytes("0000000000000000000000000000000000000000000000000000000000000001")) // s.C[1].Y

	type T struct {
		X *big.Int `abi:"x"`
		Z *big.Int `abi:"y"` // Test whether the abi tag works.
	}

	type S struct {
		A *big.Int
		B []*big.Int
		C []T
	}

	type Ret struct {
		FieldS S `abi:"s"`
		FieldT T `abi:"t"`
		A      *big.Int
	}
	var ret Ret
	var expected = Ret{
		FieldS: S{
			A: big.NewInt(1),
			B: []*big.Int{big.NewInt(1), big.NewInt(2)},
			C: []T{
				{big.NewInt(1), big.NewInt(2)},
				{big.NewInt(2), big.NewInt(1)},
			},
		},
		FieldT: T{
			big.NewInt(0), big.NewInt(1),
		},
		A: big.NewInt(1),
	}

	err = abi.UnpackIntoInterface(&ret, "tuple", buff.Bytes())
	if err != nil {
		t.Error(err)
	}
	if reflect.DeepEqual(ret, expected) {
		t.Error("unexpected unpack value")
	}
}

func TestOOMMaliciousInput(t *testing.T) {
	oomTests := []unpackTest{
		{
			def: `[{"type": "uint8[]"}]`,
			enc: "0000000000000000000000000000000000000000000000000000000000000020" + // offset
				"0000000000000000000000000000000000000000000000000000000000000003" + // num elems
				"0000000000000000000000000000000000000000000000000000000000000001" + // elem 1
				"0000000000000000000000000000000000000000000000000000000000000002", // elem 2
		},
		{ // Length larger than 64 bits
			def: `[{"type": "uint8[]"}]`,
			enc: "0000000000000000000000000000000000000000000000000000000000000020" + // offset
				"00ffffffffffffffffffffffffffffffffffffffffffffff0000000000000002" + // num elems
				"0000000000000000000000000000000000000000000000000000000000000001" + // elem 1
				"0000000000000000000000000000000000000000000000000000000000000002", // elem 2
		},
		{ // Offset very large (over 64 bits)
			def: `[{"type": "uint8[]"}]`,
			enc: "00ffffffffffffffffffffffffffffffffffffffffffffff0000000000000020" + // offset
				"0000000000000000000000000000000000000000000000000000000000000002" + // num elems
				"0000000000000000000000000000000000000000000000000000000000000001" + // elem 1
				"0000000000000000000000000000000000000000000000000000000000000002", // elem 2
		},
		{ // Offset very large (below 64 bits)
			def: `[{"type": "uint8[]"}]`,
			enc: "0000000000000000000000000000000000000000000000007ffffffffff00020" + // offset
				"0000000000000000000000000000000000000000000000000000000000000002" + // num elems
				"0000000000000000000000000000000000000000000000000000000000000001" + // elem 1
				"0000000000000000000000000000000000000000000000000000000000000002", // elem 2
		},
		{ // Offset negative (as 64 bit)
			def: `[{"type": "uint8[]"}]`,
			enc: "000000000000000000000000000000000000000000000000f000000000000020" + // offset
				"0000000000000000000000000000000000000000000000000000000000000002" + // num elems
				"0000000000000000000000000000000000000000000000000000000000000001" + // elem 1
				"0000000000000000000000000000000000000000000000000000000000000002", // elem 2
		},

		{ // Negative length
			def: `[{"type": "uint8[]"}]`,
			enc: "0000000000000000000000000000000000000000000000000000000000000020" + // offset
				"000000000000000000000000000000000000000000000000f000000000000002" + // num elems
				"0000000000000000000000000000000000000000000000000000000000000001" + // elem 1
				"0000000000000000000000000000000000000000000000000000000000000002", // elem 2
		},
		{ // Very large length
			def: `[{"type": "uint8[]"}]`,
			enc: "0000000000000000000000000000000000000000000000000000000000000020" + // offset
				"0000000000000000000000000000000000000000000000007fffffffff000002" + // num elems
				"0000000000000000000000000000000000000000000000000000000000000001" + // elem 1
				"0000000000000000000000000000000000000000000000000000000000000002", // elem 2
		},
	}
	for i, test := range oomTests {
		def := fmt.Sprintf(`[{ "name" : "method", "type": "function", "outputs": %s}]`, test.def)
		abi, err := JSON(strings.NewReader(def))
		if err != nil {
			t.Fatalf("invalid ABI definition %s: %v", def, err)
		}
		encb, err := hex.DecodeString(test.enc)
		if err != nil {
			t.Fatalf("invalid hex: %s" + test.enc)
		}
		_, err = abi.Methods["method"].Outputs.UnpackValues(encb)
		if err == nil {
			t.Fatalf("Expected error on malicious input, test %d", i)
		}
	}
}

func TestPackAndUnpackIncompatibleNumber(t *testing.T) {
	var encodeABI Arguments
	uint256Ty, err := NewType("uint256", "", nil)
	if err != nil {
		panic(err)
	}
	encodeABI = Arguments{
		{Type: uint256Ty},
	}

	maxU64, ok := new(big.Int).SetString(strconv.FormatUint(math.MaxUint64, 10), 10)
	if !ok {
		panic("bug")
	}
	maxU64Plus1 := new(big.Int).Add(maxU64, big.NewInt(1))
	cases := []struct {
		decodeType  string
		inputValue  *big.Int
		err         error
		expectValue interface{}
	}{
		{
			decodeType: "uint8",
			inputValue: big.NewInt(math.MaxUint8 + 1),
			err:        errBadUint8,
		},
		{
			decodeType:  "uint8",
			inputValue:  big.NewInt(math.MaxUint8),
			err:         nil,
			expectValue: uint8(math.MaxUint8),
		},
		{
			decodeType: "uint16",
			inputValue: big.NewInt(math.MaxUint16 + 1),
			err:        errBadUint16,
		},
		{
			decodeType:  "uint16",
			inputValue:  big.NewInt(math.MaxUint16),
			err:         nil,
			expectValue: uint16(math.MaxUint16),
		},
		{
			decodeType: "uint32",
			inputValue: big.NewInt(math.MaxUint32 + 1),
			err:        errBadUint32,
		},
		{
			decodeType:  "uint32",
			inputValue:  big.NewInt(math.MaxUint32),
			err:         nil,
			expectValue: uint32(math.MaxUint32),
		},
		{
			decodeType: "uint64",
			inputValue: maxU64Plus1,
			err:        errBadUint64,
		},
		{
			decodeType:  "uint64",
			inputValue:  maxU64,
			err:         nil,
			expectValue: uint64(math.MaxUint64),
		},
		{
			decodeType:  "uint256",
			inputValue:  maxU64Plus1,
			err:         nil,
			expectValue: maxU64Plus1,
		},
		{
			decodeType: "int8",
			inputValue: big.NewInt(math.MaxInt8 + 1),
			err:        errBadInt8,
		},
		{
			decodeType: "int8",
			inputValue: big.NewInt(math.MinInt8 - 1),
			err:        errBadInt8,
		},
		{
			decodeType:  "int8",
			inputValue:  big.NewInt(math.MaxInt8),
			err:         nil,
			expectValue: int8(math.MaxInt8),
		},
		{
			decodeType: "int16",
			inputValue: big.NewInt(math.MaxInt16 + 1),
			err:        errBadInt16,
		},
		{
			decodeType: "int16",
			inputValue: big.NewInt(math.MinInt16 - 1),
			err:        errBadInt16,
		},
		{
			decodeType:  "int16",
			inputValue:  big.NewInt(math.MaxInt16),
			err:         nil,
			expectValue: int16(math.MaxInt16),
		},
		{
			decodeType: "int32",
			inputValue: big.NewInt(math.MaxInt32 + 1),
			err:        errBadInt32,
		},
		{
			decodeType: "int32",
			inputValue: big.NewInt(math.MinInt32 - 1),
			err:        errBadInt32,
		},
		{
			decodeType:  "int32",
			inputValue:  big.NewInt(math.MaxInt32),
			err:         nil,
			expectValue: int32(math.MaxInt32),
		},
		{
			decodeType: "int64",
			inputValue: new(big.Int).Add(big.NewInt(math.MaxInt64), big.NewInt(1)),
			err:        errBadInt64,
		},
		{
			decodeType: "int64",
			inputValue: new(big.Int).Sub(big.NewInt(math.MinInt64), big.NewInt(1)),
			err:        errBadInt64,
		},
		{
			decodeType:  "int64",
			inputValue:  big.NewInt(math.MaxInt64),
			err:         nil,
			expectValue: int64(math.MaxInt64),
		},
	}
	for i, testCase := range cases {
		packed, err := encodeABI.Pack(testCase.inputValue)
		if err != nil {
			panic(err)
		}
		ty, err := NewType(testCase.decodeType, "", nil)
		if err != nil {
			panic(err)
		}
		decodeABI := Arguments{
			{Type: ty},
		}
		decoded, err := decodeABI.Unpack(packed)
		if err != testCase.err {
			t.Fatalf("Expected error %v, actual error %v. case %d", testCase.err, err, i)
		}
		if err != nil {
			continue
		}
		if !reflect.DeepEqual(decoded[0], testCase.expectValue) {
			t.Fatalf("Expected value %v, actual value %v", testCase.expectValue, decoded[0])
		}
	}
}
