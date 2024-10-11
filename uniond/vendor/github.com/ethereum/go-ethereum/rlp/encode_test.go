// Copyright 2014 The go-ethereum Authors
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
	"errors"
	"fmt"
	"io"
	"math/big"
	"runtime"
	"sync"
	"testing"

	"github.com/ethereum/go-ethereum/common/math"
	"github.com/holiman/uint256"
)

type testEncoder struct {
	err error
}

func (e *testEncoder) EncodeRLP(w io.Writer) error {
	if e == nil {
		panic("EncodeRLP called on nil value")
	}
	if e.err != nil {
		return e.err
	}
	w.Write([]byte{0, 1, 0, 1, 0, 1, 0, 1, 0, 1})
	return nil
}

type testEncoderValueMethod struct{}

func (e testEncoderValueMethod) EncodeRLP(w io.Writer) error {
	w.Write([]byte{0xFA, 0xFE, 0xF0})
	return nil
}

type byteEncoder byte

func (e byteEncoder) EncodeRLP(w io.Writer) error {
	w.Write(EmptyList)
	return nil
}

type undecodableEncoder func()

func (f undecodableEncoder) EncodeRLP(w io.Writer) error {
	w.Write([]byte{0xF5, 0xF5, 0xF5})
	return nil
}

type encodableReader struct {
	A, B uint
}

func (e *encodableReader) Read(b []byte) (int, error) {
	panic("called")
}

type namedByteType byte

var (
	_ = Encoder(&testEncoder{})
	_ = Encoder(byteEncoder(0))

	reader io.Reader = &encodableReader{1, 2}
)

type encTest struct {
	val           interface{}
	output, error string
}

var encTests = []encTest{
	// booleans
	{val: true, output: "01"},
	{val: false, output: "80"},

	// integers
	{val: uint32(0), output: "80"},
	{val: uint32(127), output: "7F"},
	{val: uint32(128), output: "8180"},
	{val: uint32(256), output: "820100"},
	{val: uint32(1024), output: "820400"},
	{val: uint32(0xFFFFFF), output: "83FFFFFF"},
	{val: uint32(0xFFFFFFFF), output: "84FFFFFFFF"},
	{val: uint64(0xFFFFFFFF), output: "84FFFFFFFF"},
	{val: uint64(0xFFFFFFFFFF), output: "85FFFFFFFFFF"},
	{val: uint64(0xFFFFFFFFFFFF), output: "86FFFFFFFFFFFF"},
	{val: uint64(0xFFFFFFFFFFFFFF), output: "87FFFFFFFFFFFFFF"},
	{val: uint64(0xFFFFFFFFFFFFFFFF), output: "88FFFFFFFFFFFFFFFF"},

	// big integers (should match uint for small values)
	{val: big.NewInt(0), output: "80"},
	{val: big.NewInt(1), output: "01"},
	{val: big.NewInt(127), output: "7F"},
	{val: big.NewInt(128), output: "8180"},
	{val: big.NewInt(256), output: "820100"},
	{val: big.NewInt(1024), output: "820400"},
	{val: big.NewInt(0xFFFFFF), output: "83FFFFFF"},
	{val: big.NewInt(0xFFFFFFFF), output: "84FFFFFFFF"},
	{val: big.NewInt(0xFFFFFFFFFF), output: "85FFFFFFFFFF"},
	{val: big.NewInt(0xFFFFFFFFFFFF), output: "86FFFFFFFFFFFF"},
	{val: big.NewInt(0xFFFFFFFFFFFFFF), output: "87FFFFFFFFFFFFFF"},
	{
		val:    new(big.Int).SetBytes(unhex("102030405060708090A0B0C0D0E0F2")),
		output: "8F102030405060708090A0B0C0D0E0F2",
	},
	{
		val:    new(big.Int).SetBytes(unhex("0100020003000400050006000700080009000A000B000C000D000E01")),
		output: "9C0100020003000400050006000700080009000A000B000C000D000E01",
	},
	{
		val:    new(big.Int).SetBytes(unhex("010000000000000000000000000000000000000000000000000000000000000000")),
		output: "A1010000000000000000000000000000000000000000000000000000000000000000",
	},
	{
		val:    veryBigInt,
		output: "89FFFFFFFFFFFFFFFFFF",
	},
	{
		val:    veryVeryBigInt,
		output: "B848FFFFFFFFFFFFFFFFF800000000000000001BFFFFFFFFFFFFFFFFC8000000000000000045FFFFFFFFFFFFFFFFC800000000000000001BFFFFFFFFFFFFFFFFF8000000000000000001",
	},

	// non-pointer big.Int
	{val: *big.NewInt(0), output: "80"},
	{val: *big.NewInt(0xFFFFFF), output: "83FFFFFF"},

	// negative ints are not supported
	{val: big.NewInt(-1), error: "rlp: cannot encode negative big.Int"},
	{val: *big.NewInt(-1), error: "rlp: cannot encode negative big.Int"},

	// uint256
	{val: uint256.NewInt(0), output: "80"},
	{val: uint256.NewInt(1), output: "01"},
	{val: uint256.NewInt(127), output: "7F"},
	{val: uint256.NewInt(128), output: "8180"},
	{val: uint256.NewInt(256), output: "820100"},
	{val: uint256.NewInt(1024), output: "820400"},
	{val: uint256.NewInt(0xFFFFFF), output: "83FFFFFF"},
	{val: uint256.NewInt(0xFFFFFFFF), output: "84FFFFFFFF"},
	{val: uint256.NewInt(0xFFFFFFFFFF), output: "85FFFFFFFFFF"},
	{val: uint256.NewInt(0xFFFFFFFFFFFF), output: "86FFFFFFFFFFFF"},
	{val: uint256.NewInt(0xFFFFFFFFFFFFFF), output: "87FFFFFFFFFFFFFF"},
	{
		val:    new(uint256.Int).SetBytes(unhex("102030405060708090A0B0C0D0E0F2")),
		output: "8F102030405060708090A0B0C0D0E0F2",
	},
	{
		val:    new(uint256.Int).SetBytes(unhex("0100020003000400050006000700080009000A000B000C000D000E01")),
		output: "9C0100020003000400050006000700080009000A000B000C000D000E01",
	},
	// non-pointer uint256.Int
	{val: *uint256.NewInt(0), output: "80"},
	{val: *uint256.NewInt(0xFFFFFF), output: "83FFFFFF"},

	// byte arrays
	{val: [0]byte{}, output: "80"},
	{val: [1]byte{0}, output: "00"},
	{val: [1]byte{1}, output: "01"},
	{val: [1]byte{0x7F}, output: "7F"},
	{val: [1]byte{0x80}, output: "8180"},
	{val: [1]byte{0xFF}, output: "81FF"},
	{val: [3]byte{1, 2, 3}, output: "83010203"},
	{val: [57]byte{1, 2, 3}, output: "B839010203000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"},

	// named byte type arrays
	{val: [0]namedByteType{}, output: "80"},
	{val: [1]namedByteType{0}, output: "00"},
	{val: [1]namedByteType{1}, output: "01"},
	{val: [1]namedByteType{0x7F}, output: "7F"},
	{val: [1]namedByteType{0x80}, output: "8180"},
	{val: [1]namedByteType{0xFF}, output: "81FF"},
	{val: [3]namedByteType{1, 2, 3}, output: "83010203"},
	{val: [57]namedByteType{1, 2, 3}, output: "B839010203000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"},

	// byte slices
	{val: []byte{}, output: "80"},
	{val: []byte{0}, output: "00"},
	{val: []byte{0x7E}, output: "7E"},
	{val: []byte{0x7F}, output: "7F"},
	{val: []byte{0x80}, output: "8180"},
	{val: []byte{1, 2, 3}, output: "83010203"},

	// named byte type slices
	{val: []namedByteType{}, output: "80"},
	{val: []namedByteType{0}, output: "00"},
	{val: []namedByteType{0x7E}, output: "7E"},
	{val: []namedByteType{0x7F}, output: "7F"},
	{val: []namedByteType{0x80}, output: "8180"},
	{val: []namedByteType{1, 2, 3}, output: "83010203"},

	// strings
	{val: "", output: "80"},
	{val: "\x7E", output: "7E"},
	{val: "\x7F", output: "7F"},
	{val: "\x80", output: "8180"},
	{val: "dog", output: "83646F67"},
	{
		val:    "Lorem ipsum dolor sit amet, consectetur adipisicing eli",
		output: "B74C6F72656D20697073756D20646F6C6F722073697420616D65742C20636F6E7365637465747572206164697069736963696E6720656C69",
	},
	{
		val:    "Lorem ipsum dolor sit amet, consectetur adipisicing elit",
		output: "B8384C6F72656D20697073756D20646F6C6F722073697420616D65742C20636F6E7365637465747572206164697069736963696E6720656C6974",
	},
	{
		val:    "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Curabitur mauris magna, suscipit sed vehicula non, iaculis faucibus tortor. Proin suscipit ultricies malesuada. Duis tortor elit, dictum quis tristique eu, ultrices at risus. Morbi a est imperdiet mi ullamcorper aliquet suscipit nec lorem. Aenean quis leo mollis, vulputate elit varius, consequat enim. Nulla ultrices turpis justo, et posuere urna consectetur nec. Proin non convallis metus. Donec tempor ipsum in mauris congue sollicitudin. Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia Curae; Suspendisse convallis sem vel massa faucibus, eget lacinia lacus tempor. Nulla quis ultricies purus. Proin auctor rhoncus nibh condimentum mollis. Aliquam consequat enim at metus luctus, a eleifend purus egestas. Curabitur at nibh metus. Nam bibendum, neque at auctor tristique, lorem libero aliquet arcu, non interdum tellus lectus sit amet eros. Cras rhoncus, metus ac ornare cursus, dolor justo ultrices metus, at ullamcorper volutpat",
		output: "B904004C6F72656D20697073756D20646F6C6F722073697420616D65742C20636F6E73656374657475722061646970697363696E6720656C69742E20437572616269747572206D6175726973206D61676E612C20737573636970697420736564207665686963756C61206E6F6E2C20696163756C697320666175636962757320746F72746F722E2050726F696E20737573636970697420756C74726963696573206D616C6573756164612E204475697320746F72746F7220656C69742C2064696374756D2071756973207472697374697175652065752C20756C7472696365732061742072697375732E204D6F72626920612065737420696D70657264696574206D6920756C6C616D636F7270657220616C6971756574207375736369706974206E6563206C6F72656D2E2041656E65616E2071756973206C656F206D6F6C6C69732C2076756C70757461746520656C6974207661726975732C20636F6E73657175617420656E696D2E204E756C6C6120756C74726963657320747572706973206A7573746F2C20657420706F73756572652075726E6120636F6E7365637465747572206E65632E2050726F696E206E6F6E20636F6E76616C6C6973206D657475732E20446F6E65632074656D706F7220697073756D20696E206D617572697320636F6E67756520736F6C6C696369747564696E2E20566573746962756C756D20616E746520697073756D207072696D697320696E206661756369627573206F726369206C756374757320657420756C74726963657320706F737565726520637562696C69612043757261653B2053757370656E646973736520636F6E76616C6C69732073656D2076656C206D617373612066617563696275732C2065676574206C6163696E6961206C616375732074656D706F722E204E756C6C61207175697320756C747269636965732070757275732E2050726F696E20617563746F722072686F6E637573206E69626820636F6E64696D656E74756D206D6F6C6C69732E20416C697175616D20636F6E73657175617420656E696D206174206D65747573206C75637475732C206120656C656966656E6420707572757320656765737461732E20437572616269747572206174206E696268206D657475732E204E616D20626962656E64756D2C206E6571756520617420617563746F72207472697374697175652C206C6F72656D206C696265726F20616C697175657420617263752C206E6F6E20696E74657264756D2074656C6C7573206C65637475732073697420616D65742065726F732E20437261732072686F6E6375732C206D65747573206163206F726E617265206375727375732C20646F6C6F72206A7573746F20756C747269636573206D657475732C20617420756C6C616D636F7270657220766F6C7574706174",
	},

	// slices
	{val: []uint{}, output: "C0"},
	{val: []uint{1, 2, 3}, output: "C3010203"},
	{
		// [ [], [[]], [ [], [[]] ] ]
		val:    []interface{}{[]interface{}{}, [][]interface{}{{}}, []interface{}{[]interface{}{}, [][]interface{}{{}}}},
		output: "C7C0C1C0C3C0C1C0",
	},
	{
		val:    []string{"aaa", "bbb", "ccc", "ddd", "eee", "fff", "ggg", "hhh", "iii", "jjj", "kkk", "lll", "mmm", "nnn", "ooo"},
		output: "F83C836161618362626283636363836464648365656583666666836767678368686883696969836A6A6A836B6B6B836C6C6C836D6D6D836E6E6E836F6F6F",
	},
	{
		val:    []interface{}{uint(1), uint(0xFFFFFF), []interface{}{[]uint{4, 5, 5}}, "abc"},
		output: "CE0183FFFFFFC4C304050583616263",
	},
	{
		val: [][]string{
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
			{"asdf", "qwer", "zxcv"},
		},
		output: "F90200CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376CF84617364668471776572847A786376",
	},

	// Non-byte arrays are encoded as lists.
	// Note that it is important to test [4]uint64 specifically,
	// because that's the underlying type of uint256.Int.
	{val: [4]uint32{1, 2, 3, 4}, output: "C401020304"},
	{val: [4]uint64{1, 2, 3, 4}, output: "C401020304"},

	// RawValue
	{val: RawValue(unhex("01")), output: "01"},
	{val: RawValue(unhex("82FFFF")), output: "82FFFF"},
	{val: []RawValue{unhex("01"), unhex("02")}, output: "C20102"},

	// structs
	{val: simplestruct{}, output: "C28080"},
	{val: simplestruct{A: 3, B: "foo"}, output: "C50383666F6F"},
	{val: &recstruct{5, nil}, output: "C205C0"},
	{val: &recstruct{5, &recstruct{4, &recstruct{3, nil}}}, output: "C605C404C203C0"},
	{val: &intField{X: 3}, error: "rlp: type int is not RLP-serializable (struct field rlp.intField.X)"},

	// struct tag "-"
	{val: &ignoredField{A: 1, B: 2, C: 3}, output: "C20103"},

	// struct tag "tail"
	{val: &tailRaw{A: 1, Tail: []RawValue{unhex("02"), unhex("03")}}, output: "C3010203"},
	{val: &tailRaw{A: 1, Tail: []RawValue{unhex("02")}}, output: "C20102"},
	{val: &tailRaw{A: 1, Tail: []RawValue{}}, output: "C101"},
	{val: &tailRaw{A: 1, Tail: nil}, output: "C101"},

	// struct tag "optional"
	{val: &optionalFields{}, output: "C180"},
	{val: &optionalFields{A: 1}, output: "C101"},
	{val: &optionalFields{A: 1, B: 2}, output: "C20102"},
	{val: &optionalFields{A: 1, B: 2, C: 3}, output: "C3010203"},
	{val: &optionalFields{A: 1, B: 0, C: 3}, output: "C3018003"},
	{val: &optionalAndTailField{A: 1}, output: "C101"},
	{val: &optionalAndTailField{A: 1, B: 2}, output: "C20102"},
	{val: &optionalAndTailField{A: 1, Tail: []uint{5, 6}}, output: "C401800506"},
	{val: &optionalAndTailField{A: 1, Tail: []uint{5, 6}}, output: "C401800506"},
	{val: &optionalBigIntField{A: 1}, output: "C101"},
	{val: &optionalPtrField{A: 1}, output: "C101"},
	{val: &optionalPtrFieldNil{A: 1}, output: "C101"},
	{val: &multipleOptionalFields{A: nil, B: nil}, output: "C0"},
	{val: &multipleOptionalFields{A: &[3]byte{1, 2, 3}, B: &[3]byte{1, 2, 3}}, output: "C88301020383010203"},
	{val: &multipleOptionalFields{A: nil, B: &[3]byte{1, 2, 3}}, output: "C58083010203"}, // encodes without error but decode will fail
	{val: &nonOptionalPtrField{A: 1}, output: "C20180"},                                  // encodes without error but decode will fail

	// nil
	{val: (*uint)(nil), output: "80"},
	{val: (*string)(nil), output: "80"},
	{val: (*[]byte)(nil), output: "80"},
	{val: (*[10]byte)(nil), output: "80"},
	{val: (*big.Int)(nil), output: "80"},
	{val: (*uint256.Int)(nil), output: "80"},
	{val: (*[]string)(nil), output: "C0"},
	{val: (*[10]string)(nil), output: "C0"},
	{val: (*[]interface{})(nil), output: "C0"},
	{val: (*[]struct{ uint })(nil), output: "C0"},
	{val: (*interface{})(nil), output: "C0"},

	// nil struct fields
	{
		val: struct {
			X *[]byte
		}{},
		output: "C180",
	},
	{
		val: struct {
			X *[2]byte
		}{},
		output: "C180",
	},
	{
		val: struct {
			X *uint64
		}{},
		output: "C180",
	},
	{
		val: struct {
			X *uint64 `rlp:"nilList"`
		}{},
		output: "C1C0",
	},
	{
		val: struct {
			X *[]uint64
		}{},
		output: "C1C0",
	},
	{
		val: struct {
			X *[]uint64 `rlp:"nilString"`
		}{},
		output: "C180",
	},

	// interfaces
	{val: []io.Reader{reader}, output: "C3C20102"}, // the contained value is a struct

	// Encoder
	{val: (*testEncoder)(nil), output: "C0"},
	{val: &testEncoder{}, output: "00010001000100010001"},
	{val: &testEncoder{errors.New("test error")}, error: "test error"},
	{val: struct{ E testEncoderValueMethod }{}, output: "C3FAFEF0"},
	{val: struct{ E *testEncoderValueMethod }{}, output: "C1C0"},

	// Verify that the Encoder interface works for unsupported types like func().
	{val: undecodableEncoder(func() {}), output: "F5F5F5"},

	// Verify that pointer method testEncoder.EncodeRLP is called for
	// addressable non-pointer values.
	{val: &struct{ TE testEncoder }{testEncoder{}}, output: "CA00010001000100010001"},
	{val: &struct{ TE testEncoder }{testEncoder{errors.New("test error")}}, error: "test error"},

	// Verify the error for non-addressable non-pointer Encoder.
	{val: testEncoder{}, error: "rlp: unaddressable value of type rlp.testEncoder, EncodeRLP is pointer method"},

	// Verify Encoder takes precedence over []byte.
	{val: []byteEncoder{0, 1, 2, 3, 4}, output: "C5C0C0C0C0C0"},
}

func runEncTests(t *testing.T, f func(val interface{}) ([]byte, error)) {
	for i, test := range encTests {
		output, err := f(test.val)
		if err != nil && test.error == "" {
			t.Errorf("test %d: unexpected error: %v\nvalue %#v\ntype %T",
				i, err, test.val, test.val)
			continue
		}
		if test.error != "" && fmt.Sprint(err) != test.error {
			t.Errorf("test %d: error mismatch\ngot   %v\nwant  %v\nvalue %#v\ntype  %T",
				i, err, test.error, test.val, test.val)
			continue
		}
		if err == nil && !bytes.Equal(output, unhex(test.output)) {
			t.Errorf("test %d: output mismatch:\ngot   %X\nwant  %s\nvalue %#v\ntype  %T",
				i, output, test.output, test.val, test.val)
		}
	}
}

func TestEncode(t *testing.T) {
	runEncTests(t, func(val interface{}) ([]byte, error) {
		b := new(bytes.Buffer)
		err := Encode(b, val)
		return b.Bytes(), err
	})
}

func TestEncodeToBytes(t *testing.T) {
	runEncTests(t, EncodeToBytes)
}

func TestEncodeAppendToBytes(t *testing.T) {
	buffer := make([]byte, 20)
	runEncTests(t, func(val interface{}) ([]byte, error) {
		w := NewEncoderBuffer(nil)
		defer w.Flush()

		err := Encode(w, val)
		if err != nil {
			return nil, err
		}
		output := w.AppendToBytes(buffer[:0])
		return output, nil
	})
}

func TestEncodeToReader(t *testing.T) {
	runEncTests(t, func(val interface{}) ([]byte, error) {
		_, r, err := EncodeToReader(val)
		if err != nil {
			return nil, err
		}
		return io.ReadAll(r)
	})
}

func TestEncodeToReaderPiecewise(t *testing.T) {
	runEncTests(t, func(val interface{}) ([]byte, error) {
		size, r, err := EncodeToReader(val)
		if err != nil {
			return nil, err
		}

		// read output piecewise
		output := make([]byte, size)
		for start, end := 0, 0; start < size; start = end {
			if remaining := size - start; remaining < 3 {
				end += remaining
			} else {
				end = start + 3
			}
			n, err := r.Read(output[start:end])
			end = start + n
			if err == io.EOF {
				break
			} else if err != nil {
				return nil, err
			}
		}
		return output, nil
	})
}

// This is a regression test verifying that encReader
// returns its encbuf to the pool only once.
func TestEncodeToReaderReturnToPool(t *testing.T) {
	buf := make([]byte, 50)
	wg := new(sync.WaitGroup)
	for i := 0; i < 5; i++ {
		wg.Add(1)
		go func() {
			for i := 0; i < 1000; i++ {
				_, r, _ := EncodeToReader("foo")
				io.ReadAll(r)
				r.Read(buf)
				r.Read(buf)
				r.Read(buf)
				r.Read(buf)
			}
			wg.Done()
		}()
	}
	wg.Wait()
}

var sink interface{}

func BenchmarkIntsize(b *testing.B) {
	for i := 0; i < b.N; i++ {
		sink = intsize(0x12345678)
	}
}

func BenchmarkPutint(b *testing.B) {
	buf := make([]byte, 8)
	for i := 0; i < b.N; i++ {
		putint(buf, 0x12345678)
		sink = buf
	}
}

func BenchmarkEncodeBigInts(b *testing.B) {
	ints := make([]*big.Int, 200)
	for i := range ints {
		ints[i] = math.BigPow(2, int64(i))
	}
	out := bytes.NewBuffer(make([]byte, 0, 4096))
	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		out.Reset()
		if err := Encode(out, ints); err != nil {
			b.Fatal(err)
		}
	}
}

func BenchmarkEncodeU256Ints(b *testing.B) {
	ints := make([]*uint256.Int, 200)
	for i := range ints {
		ints[i], _ = uint256.FromBig(math.BigPow(2, int64(i)))
	}
	out := bytes.NewBuffer(make([]byte, 0, 4096))
	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		out.Reset()
		if err := Encode(out, ints); err != nil {
			b.Fatal(err)
		}
	}
}

func BenchmarkEncodeConcurrentInterface(b *testing.B) {
	type struct1 struct {
		A string
		B *big.Int
		C [20]byte
	}
	value := []interface{}{
		uint(999),
		&struct1{A: "hello", B: big.NewInt(0xFFFFFFFF)},
		[10]byte{1, 2, 3, 4, 5, 6},
		[]string{"yeah", "yeah", "yeah"},
	}

	var wg sync.WaitGroup
	for cpu := 0; cpu < runtime.NumCPU(); cpu++ {
		wg.Add(1)
		go func() {
			defer wg.Done()

			var buffer bytes.Buffer
			for i := 0; i < b.N; i++ {
				buffer.Reset()
				err := Encode(&buffer, value)
				if err != nil {
					panic(err)
				}
			}
		}()
	}
	wg.Wait()
}

type byteArrayStruct struct {
	A [20]byte
	B [32]byte
	C [32]byte
}

func BenchmarkEncodeByteArrayStruct(b *testing.B) {
	var out bytes.Buffer
	var value byteArrayStruct

	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		out.Reset()
		if err := Encode(&out, &value); err != nil {
			b.Fatal(err)
		}
	}
}

type structSliceElem struct {
	X uint64
	Y uint64
	Z uint64
}

type structPtrSlice []*structSliceElem

func BenchmarkEncodeStructPtrSlice(b *testing.B) {
	var out bytes.Buffer
	var value = structPtrSlice{
		&structSliceElem{1, 1, 1},
		&structSliceElem{2, 2, 2},
		&structSliceElem{3, 3, 3},
		&structSliceElem{5, 5, 5},
		&structSliceElem{6, 6, 6},
		&structSliceElem{7, 7, 7},
	}

	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		out.Reset()
		if err := Encode(&out, &value); err != nil {
			b.Fatal(err)
		}
	}
}
