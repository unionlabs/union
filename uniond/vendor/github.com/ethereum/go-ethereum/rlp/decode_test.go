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
	"encoding/hex"
	"errors"
	"fmt"
	"io"
	"math/big"
	"reflect"
	"strings"
	"testing"

	"github.com/ethereum/go-ethereum/common/math"
	"github.com/holiman/uint256"
)

func TestStreamKind(t *testing.T) {
	tests := []struct {
		input    string
		wantKind Kind
		wantLen  uint64
	}{
		{"00", Byte, 0},
		{"01", Byte, 0},
		{"7F", Byte, 0},
		{"80", String, 0},
		{"B7", String, 55},
		{"B90400", String, 1024},
		{"BFFFFFFFFFFFFFFFFF", String, ^uint64(0)},
		{"C0", List, 0},
		{"C8", List, 8},
		{"F7", List, 55},
		{"F90400", List, 1024},
		{"FFFFFFFFFFFFFFFFFF", List, ^uint64(0)},
	}

	for i, test := range tests {
		// using plainReader to inhibit input limit errors.
		s := NewStream(newPlainReader(unhex(test.input)), 0)
		kind, len, err := s.Kind()
		if err != nil {
			t.Errorf("test %d: Kind returned error: %v", i, err)
			continue
		}
		if kind != test.wantKind {
			t.Errorf("test %d: kind mismatch: got %d, want %d", i, kind, test.wantKind)
		}
		if len != test.wantLen {
			t.Errorf("test %d: len mismatch: got %d, want %d", i, len, test.wantLen)
		}
	}
}

func TestNewListStream(t *testing.T) {
	ls := NewListStream(bytes.NewReader(unhex("0101010101")), 3)
	if k, size, err := ls.Kind(); k != List || size != 3 || err != nil {
		t.Errorf("Kind() returned (%v, %d, %v), expected (List, 3, nil)", k, size, err)
	}
	if size, err := ls.List(); size != 3 || err != nil {
		t.Errorf("List() returned (%d, %v), expected (3, nil)", size, err)
	}
	for i := 0; i < 3; i++ {
		if val, err := ls.Uint(); val != 1 || err != nil {
			t.Errorf("Uint() returned (%d, %v), expected (1, nil)", val, err)
		}
	}
	if err := ls.ListEnd(); err != nil {
		t.Errorf("ListEnd() returned %v, expected (3, nil)", err)
	}
}

func TestStreamErrors(t *testing.T) {
	withoutInputLimit := func(b []byte) *Stream {
		return NewStream(newPlainReader(b), 0)
	}
	withCustomInputLimit := func(limit uint64) func([]byte) *Stream {
		return func(b []byte) *Stream {
			return NewStream(bytes.NewReader(b), limit)
		}
	}

	type calls []string
	tests := []struct {
		string
		calls
		newStream func([]byte) *Stream // uses bytes.Reader if nil
		error     error
	}{
		{"C0", calls{"Bytes"}, nil, ErrExpectedString},
		{"C0", calls{"Uint"}, nil, ErrExpectedString},
		{"89000000000000000001", calls{"Uint"}, nil, errUintOverflow},
		{"00", calls{"List"}, nil, ErrExpectedList},
		{"80", calls{"List"}, nil, ErrExpectedList},
		{"C0", calls{"List", "Uint"}, nil, EOL},
		{"C8C9010101010101010101", calls{"List", "Kind"}, nil, ErrElemTooLarge},
		{"C3C2010201", calls{"List", "List", "Uint", "Uint", "ListEnd", "Uint"}, nil, EOL},
		{"00", calls{"ListEnd"}, nil, errNotInList},
		{"C401020304", calls{"List", "Uint", "ListEnd"}, nil, errNotAtEOL},

		// Non-canonical integers (e.g. leading zero bytes).
		{"00", calls{"Uint"}, nil, ErrCanonInt},
		{"820002", calls{"Uint"}, nil, ErrCanonInt},
		{"8133", calls{"Uint"}, nil, ErrCanonSize},
		{"817F", calls{"Uint"}, nil, ErrCanonSize},
		{"8180", calls{"Uint"}, nil, nil},

		// Non-valid boolean
		{"02", calls{"Bool"}, nil, errors.New("rlp: invalid boolean value: 2")},

		// Size tags must use the smallest possible encoding.
		// Leading zero bytes in the size tag are also rejected.
		{"8100", calls{"Uint"}, nil, ErrCanonSize},
		{"8100", calls{"Bytes"}, nil, ErrCanonSize},
		{"8101", calls{"Bytes"}, nil, ErrCanonSize},
		{"817F", calls{"Bytes"}, nil, ErrCanonSize},
		{"8180", calls{"Bytes"}, nil, nil},
		{"B800", calls{"Kind"}, withoutInputLimit, ErrCanonSize},
		{"B90000", calls{"Kind"}, withoutInputLimit, ErrCanonSize},
		{"B90055", calls{"Kind"}, withoutInputLimit, ErrCanonSize},
		{"BA0002FFFF", calls{"Bytes"}, withoutInputLimit, ErrCanonSize},
		{"F800", calls{"Kind"}, withoutInputLimit, ErrCanonSize},
		{"F90000", calls{"Kind"}, withoutInputLimit, ErrCanonSize},
		{"F90055", calls{"Kind"}, withoutInputLimit, ErrCanonSize},
		{"FA0002FFFF", calls{"List"}, withoutInputLimit, ErrCanonSize},

		// Expected EOF
		{"", calls{"Kind"}, nil, io.EOF},
		{"", calls{"Uint"}, nil, io.EOF},
		{"", calls{"List"}, nil, io.EOF},
		{"8180", calls{"Uint", "Uint"}, nil, io.EOF},
		{"C0", calls{"List", "ListEnd", "List"}, nil, io.EOF},

		{"", calls{"List"}, withoutInputLimit, io.EOF},
		{"8180", calls{"Uint", "Uint"}, withoutInputLimit, io.EOF},
		{"C0", calls{"List", "ListEnd", "List"}, withoutInputLimit, io.EOF},

		// Input limit errors.
		{"81", calls{"Bytes"}, nil, ErrValueTooLarge},
		{"81", calls{"Uint"}, nil, ErrValueTooLarge},
		{"81", calls{"Raw"}, nil, ErrValueTooLarge},
		{"BFFFFFFFFFFFFFFFFFFF", calls{"Bytes"}, nil, ErrValueTooLarge},
		{"C801", calls{"List"}, nil, ErrValueTooLarge},

		// Test for list element size check overflow.
		{"CD04040404FFFFFFFFFFFFFFFFFF0303", calls{"List", "Uint", "Uint", "Uint", "Uint", "List"}, nil, ErrElemTooLarge},

		// Test for input limit overflow. Since we are counting the limit
		// down toward zero in Stream.remaining, reading too far can overflow
		// remaining to a large value, effectively disabling the limit.
		{"C40102030401", calls{"Raw", "Uint"}, withCustomInputLimit(5), io.EOF},
		{"C4010203048180", calls{"Raw", "Uint"}, withCustomInputLimit(6), ErrValueTooLarge},

		// Check that the same calls are fine without a limit.
		{"C40102030401", calls{"Raw", "Uint"}, withoutInputLimit, nil},
		{"C4010203048180", calls{"Raw", "Uint"}, withoutInputLimit, nil},

		// Unexpected EOF. This only happens when there is
		// no input limit, so the reader needs to be 'dumbed down'.
		{"81", calls{"Bytes"}, withoutInputLimit, io.ErrUnexpectedEOF},
		{"81", calls{"Uint"}, withoutInputLimit, io.ErrUnexpectedEOF},
		{"BFFFFFFFFFFFFFFF", calls{"Bytes"}, withoutInputLimit, io.ErrUnexpectedEOF},
		{"C801", calls{"List", "Uint", "Uint"}, withoutInputLimit, io.ErrUnexpectedEOF},

		// This test verifies that the input position is advanced
		// correctly when calling Bytes for empty strings. Kind can be called
		// any number of times in between and doesn't advance.
		{"C3808080", calls{
			"List",  // enter the list
			"Bytes", // past first element

			"Kind", "Kind", "Kind", // this shouldn't advance

			"Bytes", // past second element

			"Kind", "Kind", // can't hurt to try

			"Bytes", // past final element
			"Bytes", // this one should fail
		}, nil, EOL},
	}

testfor:
	for i, test := range tests {
		if test.newStream == nil {
			test.newStream = func(b []byte) *Stream { return NewStream(bytes.NewReader(b), 0) }
		}
		s := test.newStream(unhex(test.string))
		rs := reflect.ValueOf(s)
		for j, call := range test.calls {
			fval := rs.MethodByName(call)
			ret := fval.Call(nil)
			err := "<nil>"
			if lastret := ret[len(ret)-1].Interface(); lastret != nil {
				err = lastret.(error).Error()
			}
			if j == len(test.calls)-1 {
				want := "<nil>"
				if test.error != nil {
					want = test.error.Error()
				}
				if err != want {
					t.Log(test)
					t.Errorf("test %d: last call (%s) error mismatch\ngot:  %s\nwant: %s",
						i, call, err, test.error)
				}
			} else if err != "<nil>" {
				t.Log(test)
				t.Errorf("test %d: call %d (%s) unexpected error: %q", i, j, call, err)
				continue testfor
			}
		}
	}
}

func TestStreamList(t *testing.T) {
	s := NewStream(bytes.NewReader(unhex("C80102030405060708")), 0)

	len, err := s.List()
	if err != nil {
		t.Fatalf("List error: %v", err)
	}
	if len != 8 {
		t.Fatalf("List returned invalid length, got %d, want 8", len)
	}

	for i := uint64(1); i <= 8; i++ {
		v, err := s.Uint()
		if err != nil {
			t.Fatalf("Uint error: %v", err)
		}
		if i != v {
			t.Errorf("Uint returned wrong value, got %d, want %d", v, i)
		}
	}

	if _, err := s.Uint(); err != EOL {
		t.Errorf("Uint error mismatch, got %v, want %v", err, EOL)
	}
	if err = s.ListEnd(); err != nil {
		t.Fatalf("ListEnd error: %v", err)
	}
}

func TestStreamRaw(t *testing.T) {
	tests := []struct {
		input  string
		output string
	}{
		{
			"C58401010101",
			"8401010101",
		},
		{
			"F842B84001010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101",
			"B84001010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101",
		},
	}
	for i, tt := range tests {
		s := NewStream(bytes.NewReader(unhex(tt.input)), 0)
		s.List()

		want := unhex(tt.output)
		raw, err := s.Raw()
		if err != nil {
			t.Fatal(err)
		}
		if !bytes.Equal(want, raw) {
			t.Errorf("test %d: raw mismatch: got %x, want %x", i, raw, want)
		}
	}
}

func TestStreamReadBytes(t *testing.T) {
	tests := []struct {
		input string
		size  int
		err   string
	}{
		// kind List
		{input: "C0", size: 1, err: "rlp: expected String or Byte"},
		// kind Byte
		{input: "04", size: 0, err: "input value has wrong size 1, want 0"},
		{input: "04", size: 1},
		{input: "04", size: 2, err: "input value has wrong size 1, want 2"},
		// kind String
		{input: "820102", size: 0, err: "input value has wrong size 2, want 0"},
		{input: "820102", size: 1, err: "input value has wrong size 2, want 1"},
		{input: "820102", size: 2},
		{input: "820102", size: 3, err: "input value has wrong size 2, want 3"},
	}

	for _, test := range tests {
		test := test
		name := fmt.Sprintf("input_%s/size_%d", test.input, test.size)
		t.Run(name, func(t *testing.T) {
			s := NewStream(bytes.NewReader(unhex(test.input)), 0)
			b := make([]byte, test.size)
			err := s.ReadBytes(b)
			if test.err == "" {
				if err != nil {
					t.Errorf("unexpected error %q", err)
				}
			} else {
				if err == nil {
					t.Errorf("expected error, got nil")
				} else if err.Error() != test.err {
					t.Errorf("wrong error %q", err)
				}
			}
		})
	}
}

func TestDecodeErrors(t *testing.T) {
	r := bytes.NewReader(nil)

	if err := Decode(r, nil); err != errDecodeIntoNil {
		t.Errorf("Decode(r, nil) error mismatch, got %q, want %q", err, errDecodeIntoNil)
	}

	var nilptr *struct{}
	if err := Decode(r, nilptr); err != errDecodeIntoNil {
		t.Errorf("Decode(r, nilptr) error mismatch, got %q, want %q", err, errDecodeIntoNil)
	}

	if err := Decode(r, struct{}{}); err != errNoPointer {
		t.Errorf("Decode(r, struct{}{}) error mismatch, got %q, want %q", err, errNoPointer)
	}

	expectErr := "rlp: type chan bool is not RLP-serializable"
	if err := Decode(r, new(chan bool)); err == nil || err.Error() != expectErr {
		t.Errorf("Decode(r, new(chan bool)) error mismatch, got %q, want %q", err, expectErr)
	}

	if err := Decode(r, new(uint)); err != io.EOF {
		t.Errorf("Decode(r, new(int)) error mismatch, got %q, want %q", err, io.EOF)
	}
}

type decodeTest struct {
	input string
	ptr   interface{}
	value interface{}
	error string
}

type simplestruct struct {
	A uint
	B string
}

type recstruct struct {
	I     uint
	Child *recstruct `rlp:"nil"`
}

type bigIntStruct struct {
	I *big.Int
	B string
}

type invalidNilTag struct {
	X []byte `rlp:"nil"`
}

type invalidTail1 struct {
	A uint `rlp:"tail"`
	B string
}

type invalidTail2 struct {
	A uint
	B string `rlp:"tail"`
}

type tailRaw struct {
	A    uint
	Tail []RawValue `rlp:"tail"`
}

type tailUint struct {
	A    uint
	Tail []uint `rlp:"tail"`
}

type tailPrivateFields struct {
	A    uint
	Tail []uint `rlp:"tail"`
	x, y bool   //lint:ignore U1000 unused fields required for testing purposes.
}

type nilListUint struct {
	X *uint `rlp:"nilList"`
}

type nilStringSlice struct {
	X *[]uint `rlp:"nilString"`
}

type intField struct {
	X int
}

type optionalFields struct {
	A uint
	B uint `rlp:"optional"`
	C uint `rlp:"optional"`
}

type optionalAndTailField struct {
	A    uint
	B    uint   `rlp:"optional"`
	Tail []uint `rlp:"tail"`
}

type optionalBigIntField struct {
	A uint
	B *big.Int `rlp:"optional"`
}

type optionalPtrField struct {
	A uint
	B *[3]byte `rlp:"optional"`
}

type nonOptionalPtrField struct {
	A uint
	B *[3]byte
}

type multipleOptionalFields struct {
	A *[3]byte `rlp:"optional"`
	B *[3]byte `rlp:"optional"`
}

type optionalPtrFieldNil struct {
	A uint
	B *[3]byte `rlp:"optional,nil"`
}

type ignoredField struct {
	A uint
	B uint `rlp:"-"`
	C uint
}

var (
	veryBigInt = new(big.Int).Add(
		new(big.Int).Lsh(big.NewInt(0xFFFFFFFFFFFFFF), 16),
		big.NewInt(0xFFFF),
	)
	veryVeryBigInt = new(big.Int).Exp(veryBigInt, big.NewInt(8), nil)
)

var (
	veryBigInt256, _ = uint256.FromBig(veryBigInt)
)

var decodeTests = []decodeTest{
	// booleans
	{input: "01", ptr: new(bool), value: true},
	{input: "80", ptr: new(bool), value: false},
	{input: "02", ptr: new(bool), error: "rlp: invalid boolean value: 2"},

	// integers
	{input: "05", ptr: new(uint32), value: uint32(5)},
	{input: "80", ptr: new(uint32), value: uint32(0)},
	{input: "820505", ptr: new(uint32), value: uint32(0x0505)},
	{input: "83050505", ptr: new(uint32), value: uint32(0x050505)},
	{input: "8405050505", ptr: new(uint32), value: uint32(0x05050505)},
	{input: "850505050505", ptr: new(uint32), error: "rlp: input string too long for uint32"},
	{input: "C0", ptr: new(uint32), error: "rlp: expected input string or byte for uint32"},
	{input: "00", ptr: new(uint32), error: "rlp: non-canonical integer (leading zero bytes) for uint32"},
	{input: "8105", ptr: new(uint32), error: "rlp: non-canonical size information for uint32"},
	{input: "820004", ptr: new(uint32), error: "rlp: non-canonical integer (leading zero bytes) for uint32"},
	{input: "B8020004", ptr: new(uint32), error: "rlp: non-canonical size information for uint32"},

	// slices
	{input: "C0", ptr: new([]uint), value: []uint{}},
	{input: "C80102030405060708", ptr: new([]uint), value: []uint{1, 2, 3, 4, 5, 6, 7, 8}},
	{input: "F8020004", ptr: new([]uint), error: "rlp: non-canonical size information for []uint"},

	// arrays
	{input: "C50102030405", ptr: new([5]uint), value: [5]uint{1, 2, 3, 4, 5}},
	{input: "C0", ptr: new([5]uint), error: "rlp: input list has too few elements for [5]uint"},
	{input: "C102", ptr: new([5]uint), error: "rlp: input list has too few elements for [5]uint"},
	{input: "C6010203040506", ptr: new([5]uint), error: "rlp: input list has too many elements for [5]uint"},
	{input: "F8020004", ptr: new([5]uint), error: "rlp: non-canonical size information for [5]uint"},

	// zero sized arrays
	{input: "C0", ptr: new([0]uint), value: [0]uint{}},
	{input: "C101", ptr: new([0]uint), error: "rlp: input list has too many elements for [0]uint"},

	// byte slices
	{input: "01", ptr: new([]byte), value: []byte{1}},
	{input: "80", ptr: new([]byte), value: []byte{}},
	{input: "8D6162636465666768696A6B6C6D", ptr: new([]byte), value: []byte("abcdefghijklm")},
	{input: "C0", ptr: new([]byte), error: "rlp: expected input string or byte for []uint8"},
	{input: "8105", ptr: new([]byte), error: "rlp: non-canonical size information for []uint8"},

	// byte arrays
	{input: "02", ptr: new([1]byte), value: [1]byte{2}},
	{input: "8180", ptr: new([1]byte), value: [1]byte{128}},
	{input: "850102030405", ptr: new([5]byte), value: [5]byte{1, 2, 3, 4, 5}},

	// byte array errors
	{input: "02", ptr: new([5]byte), error: "rlp: input string too short for [5]uint8"},
	{input: "80", ptr: new([5]byte), error: "rlp: input string too short for [5]uint8"},
	{input: "820000", ptr: new([5]byte), error: "rlp: input string too short for [5]uint8"},
	{input: "C0", ptr: new([5]byte), error: "rlp: expected input string or byte for [5]uint8"},
	{input: "C3010203", ptr: new([5]byte), error: "rlp: expected input string or byte for [5]uint8"},
	{input: "86010203040506", ptr: new([5]byte), error: "rlp: input string too long for [5]uint8"},
	{input: "8105", ptr: new([1]byte), error: "rlp: non-canonical size information for [1]uint8"},
	{input: "817F", ptr: new([1]byte), error: "rlp: non-canonical size information for [1]uint8"},

	// zero sized byte arrays
	{input: "80", ptr: new([0]byte), value: [0]byte{}},
	{input: "01", ptr: new([0]byte), error: "rlp: input string too long for [0]uint8"},
	{input: "8101", ptr: new([0]byte), error: "rlp: input string too long for [0]uint8"},

	// strings
	{input: "00", ptr: new(string), value: "\000"},
	{input: "8D6162636465666768696A6B6C6D", ptr: new(string), value: "abcdefghijklm"},
	{input: "C0", ptr: new(string), error: "rlp: expected input string or byte for string"},

	// big ints
	{input: "80", ptr: new(*big.Int), value: big.NewInt(0)},
	{input: "01", ptr: new(*big.Int), value: big.NewInt(1)},
	{input: "89FFFFFFFFFFFFFFFFFF", ptr: new(*big.Int), value: veryBigInt},
	{input: "B848FFFFFFFFFFFFFFFFF800000000000000001BFFFFFFFFFFFFFFFFC8000000000000000045FFFFFFFFFFFFFFFFC800000000000000001BFFFFFFFFFFFFFFFFF8000000000000000001", ptr: new(*big.Int), value: veryVeryBigInt},
	{input: "10", ptr: new(big.Int), value: *big.NewInt(16)}, // non-pointer also works

	// big int errors
	{input: "C0", ptr: new(*big.Int), error: "rlp: expected input string or byte for *big.Int"},
	{input: "00", ptr: new(*big.Int), error: "rlp: non-canonical integer (leading zero bytes) for *big.Int"},
	{input: "820001", ptr: new(*big.Int), error: "rlp: non-canonical integer (leading zero bytes) for *big.Int"},
	{input: "8105", ptr: new(*big.Int), error: "rlp: non-canonical size information for *big.Int"},

	// uint256
	{input: "80", ptr: new(*uint256.Int), value: uint256.NewInt(0)},
	{input: "01", ptr: new(*uint256.Int), value: uint256.NewInt(1)},
	{input: "88FFFFFFFFFFFFFFFF", ptr: new(*uint256.Int), value: uint256.NewInt(math.MaxUint64)},
	{input: "89FFFFFFFFFFFFFFFFFF", ptr: new(*uint256.Int), value: veryBigInt256},
	{input: "10", ptr: new(uint256.Int), value: *uint256.NewInt(16)}, // non-pointer also works

	// uint256 errors
	{input: "C0", ptr: new(*uint256.Int), error: "rlp: expected input string or byte for *uint256.Int"},
	{input: "00", ptr: new(*uint256.Int), error: "rlp: non-canonical integer (leading zero bytes) for *uint256.Int"},
	{input: "820001", ptr: new(*uint256.Int), error: "rlp: non-canonical integer (leading zero bytes) for *uint256.Int"},
	{input: "8105", ptr: new(*uint256.Int), error: "rlp: non-canonical size information for *uint256.Int"},
	{input: "A1FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF00", ptr: new(*uint256.Int), error: "rlp: value too large for uint256"},

	// structs
	{
		input: "C50583343434",
		ptr:   new(simplestruct),
		value: simplestruct{5, "444"},
	},
	{
		input: "C601C402C203C0",
		ptr:   new(recstruct),
		value: recstruct{1, &recstruct{2, &recstruct{3, nil}}},
	},
	{
		// This checks that empty big.Int works correctly in struct context. It's easy to
		// miss the update of s.kind for this case, so it needs its own test.
		input: "C58083343434",
		ptr:   new(bigIntStruct),
		value: bigIntStruct{new(big.Int), "444"},
	},

	// struct errors
	{
		input: "C0",
		ptr:   new(simplestruct),
		error: "rlp: too few elements for rlp.simplestruct",
	},
	{
		input: "C105",
		ptr:   new(simplestruct),
		error: "rlp: too few elements for rlp.simplestruct",
	},
	{
		input: "C7C50583343434C0",
		ptr:   new([]*simplestruct),
		error: "rlp: too few elements for rlp.simplestruct, decoding into ([]*rlp.simplestruct)[1]",
	},
	{
		input: "83222222",
		ptr:   new(simplestruct),
		error: "rlp: expected input list for rlp.simplestruct",
	},
	{
		input: "C3010101",
		ptr:   new(simplestruct),
		error: "rlp: input list has too many elements for rlp.simplestruct",
	},
	{
		input: "C501C3C00000",
		ptr:   new(recstruct),
		error: "rlp: expected input string or byte for uint, decoding into (rlp.recstruct).Child.I",
	},
	{
		input: "C103",
		ptr:   new(intField),
		error: "rlp: type int is not RLP-serializable (struct field rlp.intField.X)",
	},
	{
		input: "C50102C20102",
		ptr:   new(tailUint),
		error: "rlp: expected input string or byte for uint, decoding into (rlp.tailUint).Tail[1]",
	},
	{
		input: "C0",
		ptr:   new(invalidNilTag),
		error: `rlp: invalid struct tag "nil" for rlp.invalidNilTag.X (field is not a pointer)`,
	},

	// struct tag "tail"
	{
		input: "C3010203",
		ptr:   new(tailRaw),
		value: tailRaw{A: 1, Tail: []RawValue{unhex("02"), unhex("03")}},
	},
	{
		input: "C20102",
		ptr:   new(tailRaw),
		value: tailRaw{A: 1, Tail: []RawValue{unhex("02")}},
	},
	{
		input: "C101",
		ptr:   new(tailRaw),
		value: tailRaw{A: 1, Tail: []RawValue{}},
	},
	{
		input: "C3010203",
		ptr:   new(tailPrivateFields),
		value: tailPrivateFields{A: 1, Tail: []uint{2, 3}},
	},
	{
		input: "C0",
		ptr:   new(invalidTail1),
		error: `rlp: invalid struct tag "tail" for rlp.invalidTail1.A (must be on last field)`,
	},
	{
		input: "C0",
		ptr:   new(invalidTail2),
		error: `rlp: invalid struct tag "tail" for rlp.invalidTail2.B (field type is not slice)`,
	},

	// struct tag "-"
	{
		input: "C20102",
		ptr:   new(ignoredField),
		value: ignoredField{A: 1, C: 2},
	},

	// struct tag "nilList"
	{
		input: "C180",
		ptr:   new(nilListUint),
		error: "rlp: wrong kind of empty value (got String, want List) for *uint, decoding into (rlp.nilListUint).X",
	},
	{
		input: "C1C0",
		ptr:   new(nilListUint),
		value: nilListUint{},
	},
	{
		input: "C103",
		ptr:   new(nilListUint),
		value: func() interface{} {
			v := uint(3)
			return nilListUint{X: &v}
		}(),
	},

	// struct tag "nilString"
	{
		input: "C1C0",
		ptr:   new(nilStringSlice),
		error: "rlp: wrong kind of empty value (got List, want String) for *[]uint, decoding into (rlp.nilStringSlice).X",
	},
	{
		input: "C180",
		ptr:   new(nilStringSlice),
		value: nilStringSlice{},
	},
	{
		input: "C2C103",
		ptr:   new(nilStringSlice),
		value: nilStringSlice{X: &[]uint{3}},
	},

	// struct tag "optional"
	{
		input: "C101",
		ptr:   new(optionalFields),
		value: optionalFields{1, 0, 0},
	},
	{
		input: "C20102",
		ptr:   new(optionalFields),
		value: optionalFields{1, 2, 0},
	},
	{
		input: "C3010203",
		ptr:   new(optionalFields),
		value: optionalFields{1, 2, 3},
	},
	{
		input: "C401020304",
		ptr:   new(optionalFields),
		error: "rlp: input list has too many elements for rlp.optionalFields",
	},
	{
		input: "C101",
		ptr:   new(optionalAndTailField),
		value: optionalAndTailField{A: 1},
	},
	{
		input: "C20102",
		ptr:   new(optionalAndTailField),
		value: optionalAndTailField{A: 1, B: 2, Tail: []uint{}},
	},
	{
		input: "C401020304",
		ptr:   new(optionalAndTailField),
		value: optionalAndTailField{A: 1, B: 2, Tail: []uint{3, 4}},
	},
	{
		input: "C101",
		ptr:   new(optionalBigIntField),
		value: optionalBigIntField{A: 1, B: nil},
	},
	{
		input: "C20102",
		ptr:   new(optionalBigIntField),
		value: optionalBigIntField{A: 1, B: big.NewInt(2)},
	},
	{
		input: "C101",
		ptr:   new(optionalPtrField),
		value: optionalPtrField{A: 1},
	},
	{
		input: "C20180", // not accepted because "optional" doesn't enable "nil"
		ptr:   new(optionalPtrField),
		error: "rlp: input string too short for [3]uint8, decoding into (rlp.optionalPtrField).B",
	},
	{
		input: "C20102",
		ptr:   new(optionalPtrField),
		error: "rlp: input string too short for [3]uint8, decoding into (rlp.optionalPtrField).B",
	},
	{
		input: "C50183010203",
		ptr:   new(optionalPtrField),
		value: optionalPtrField{A: 1, B: &[3]byte{1, 2, 3}},
	},
	{
		// all optional fields nil
		input: "C0",
		ptr:   new(multipleOptionalFields),
		value: multipleOptionalFields{A: nil, B: nil},
	},
	{
		// all optional fields set
		input: "C88301020383010203",
		ptr:   new(multipleOptionalFields),
		value: multipleOptionalFields{A: &[3]byte{1, 2, 3}, B: &[3]byte{1, 2, 3}},
	},
	{
		// nil optional field appears before a non-nil one
		input: "C58083010203",
		ptr:   new(multipleOptionalFields),
		error: "rlp: input string too short for [3]uint8, decoding into (rlp.multipleOptionalFields).A",
	},
	{
		// decode a nil ptr into a ptr that is not nil or not optional
		input: "C20180",
		ptr:   new(nonOptionalPtrField),
		error: "rlp: input string too short for [3]uint8, decoding into (rlp.nonOptionalPtrField).B",
	},
	{
		input: "C101",
		ptr:   new(optionalPtrFieldNil),
		value: optionalPtrFieldNil{A: 1},
	},
	{
		input: "C20180", // accepted because "nil" tag allows empty input
		ptr:   new(optionalPtrFieldNil),
		value: optionalPtrFieldNil{A: 1},
	},
	{
		input: "C20102",
		ptr:   new(optionalPtrFieldNil),
		error: "rlp: input string too short for [3]uint8, decoding into (rlp.optionalPtrFieldNil).B",
	},

	// struct tag "optional" field clearing
	{
		input: "C101",
		ptr:   &optionalFields{A: 9, B: 8, C: 7},
		value: optionalFields{A: 1, B: 0, C: 0},
	},
	{
		input: "C20102",
		ptr:   &optionalFields{A: 9, B: 8, C: 7},
		value: optionalFields{A: 1, B: 2, C: 0},
	},
	{
		input: "C20102",
		ptr:   &optionalAndTailField{A: 9, B: 8, Tail: []uint{7, 6, 5}},
		value: optionalAndTailField{A: 1, B: 2, Tail: []uint{}},
	},
	{
		input: "C101",
		ptr:   &optionalPtrField{A: 9, B: &[3]byte{8, 7, 6}},
		value: optionalPtrField{A: 1},
	},

	// RawValue
	{input: "01", ptr: new(RawValue), value: RawValue(unhex("01"))},
	{input: "82FFFF", ptr: new(RawValue), value: RawValue(unhex("82FFFF"))},
	{input: "C20102", ptr: new([]RawValue), value: []RawValue{unhex("01"), unhex("02")}},

	// pointers
	{input: "00", ptr: new(*[]byte), value: &[]byte{0}},
	{input: "80", ptr: new(*uint), value: uintp(0)},
	{input: "C0", ptr: new(*uint), error: "rlp: expected input string or byte for uint"},
	{input: "07", ptr: new(*uint), value: uintp(7)},
	{input: "817F", ptr: new(*uint), error: "rlp: non-canonical size information for uint"},
	{input: "8180", ptr: new(*uint), value: uintp(0x80)},
	{input: "C109", ptr: new(*[]uint), value: &[]uint{9}},
	{input: "C58403030303", ptr: new(*[][]byte), value: &[][]byte{{3, 3, 3, 3}}},

	// check that input position is advanced also for empty values.
	{input: "C3808005", ptr: new([]*uint), value: []*uint{uintp(0), uintp(0), uintp(5)}},

	// interface{}
	{input: "00", ptr: new(interface{}), value: []byte{0}},
	{input: "01", ptr: new(interface{}), value: []byte{1}},
	{input: "80", ptr: new(interface{}), value: []byte{}},
	{input: "850505050505", ptr: new(interface{}), value: []byte{5, 5, 5, 5, 5}},
	{input: "C0", ptr: new(interface{}), value: []interface{}{}},
	{input: "C50183040404", ptr: new(interface{}), value: []interface{}{[]byte{1}, []byte{4, 4, 4}}},
	{
		input: "C3010203",
		ptr:   new([]io.Reader),
		error: "rlp: type io.Reader is not RLP-serializable",
	},

	// fuzzer crashes
	{
		input: "c330f9c030f93030ce3030303030303030bd303030303030",
		ptr:   new(interface{}),
		error: "rlp: element is larger than containing list",
	},
}

func uintp(i uint) *uint { return &i }

func runTests(t *testing.T, decode func([]byte, interface{}) error) {
	for i, test := range decodeTests {
		input, err := hex.DecodeString(test.input)
		if err != nil {
			t.Errorf("test %d: invalid hex input %q", i, test.input)
			continue
		}
		err = decode(input, test.ptr)
		if err != nil && test.error == "" {
			t.Errorf("test %d: unexpected Decode error: %v\ndecoding into %T\ninput %q",
				i, err, test.ptr, test.input)
			continue
		}
		if test.error != "" && fmt.Sprint(err) != test.error {
			t.Errorf("test %d: Decode error mismatch\ngot  %v\nwant %v\ndecoding into %T\ninput %q",
				i, err, test.error, test.ptr, test.input)
			continue
		}
		deref := reflect.ValueOf(test.ptr).Elem().Interface()
		if err == nil && !reflect.DeepEqual(deref, test.value) {
			t.Errorf("test %d: value mismatch\ngot  %#v\nwant %#v\ndecoding into %T\ninput %q",
				i, deref, test.value, test.ptr, test.input)
		}
	}
}

func TestDecodeWithByteReader(t *testing.T) {
	runTests(t, func(input []byte, into interface{}) error {
		return Decode(bytes.NewReader(input), into)
	})
}

func testDecodeWithEncReader(t *testing.T, n int) {
	s := strings.Repeat("0", n)
	_, r, _ := EncodeToReader(s)
	var decoded string
	err := Decode(r, &decoded)
	if err != nil {
		t.Errorf("Unexpected decode error with n=%v: %v", n, err)
	}
	if decoded != s {
		t.Errorf("Decode mismatch with n=%v", n)
	}
}

// This is a regression test checking that decoding from encReader
// works for RLP values of size 8192 bytes or more.
func TestDecodeWithEncReader(t *testing.T) {
	testDecodeWithEncReader(t, 8188) // length with header is 8191
	testDecodeWithEncReader(t, 8189) // length with header is 8192
}

// plainReader reads from a byte slice but does not
// implement ReadByte. It is also not recognized by the
// size validation. This is useful to test how the decoder
// behaves on a non-buffered input stream.
type plainReader []byte

func newPlainReader(b []byte) io.Reader {
	return (*plainReader)(&b)
}

func (r *plainReader) Read(buf []byte) (n int, err error) {
	if len(*r) == 0 {
		return 0, io.EOF
	}
	n = copy(buf, *r)
	*r = (*r)[n:]
	return n, nil
}

func TestDecodeWithNonByteReader(t *testing.T) {
	runTests(t, func(input []byte, into interface{}) error {
		return Decode(newPlainReader(input), into)
	})
}

func TestDecodeStreamReset(t *testing.T) {
	s := NewStream(nil, 0)
	runTests(t, func(input []byte, into interface{}) error {
		s.Reset(bytes.NewReader(input), 0)
		return s.Decode(into)
	})
}

type testDecoder struct{ called bool }

func (t *testDecoder) DecodeRLP(s *Stream) error {
	if _, err := s.Uint(); err != nil {
		return err
	}
	t.called = true
	return nil
}

func TestDecodeDecoder(t *testing.T) {
	var s struct {
		T1 testDecoder
		T2 *testDecoder
		T3 **testDecoder
	}
	if err := Decode(bytes.NewReader(unhex("C3010203")), &s); err != nil {
		t.Fatalf("Decode error: %v", err)
	}

	if !s.T1.called {
		t.Errorf("DecodeRLP was not called for (non-pointer) testDecoder")
	}

	if s.T2 == nil {
		t.Errorf("*testDecoder has not been allocated")
	} else if !s.T2.called {
		t.Errorf("DecodeRLP was not called for *testDecoder")
	}

	if s.T3 == nil || *s.T3 == nil {
		t.Errorf("**testDecoder has not been allocated")
	} else if !(*s.T3).called {
		t.Errorf("DecodeRLP was not called for **testDecoder")
	}
}

func TestDecodeDecoderNilPointer(t *testing.T) {
	var s struct {
		T1 *testDecoder `rlp:"nil"`
		T2 *testDecoder
	}
	if err := Decode(bytes.NewReader(unhex("C2C002")), &s); err != nil {
		t.Fatalf("Decode error: %v", err)
	}
	if s.T1 != nil {
		t.Errorf("decoder T1 allocated for empty input (called: %v)", s.T1.called)
	}
	if s.T2 == nil || !s.T2.called {
		t.Errorf("decoder T2 not allocated/called")
	}
}

type byteDecoder byte

func (bd *byteDecoder) DecodeRLP(s *Stream) error {
	_, err := s.Uint()
	*bd = 255
	return err
}

func (bd byteDecoder) called() bool {
	return bd == 255
}

// This test verifies that the byte slice/byte array logic
// does not kick in for element types implementing Decoder.
func TestDecoderInByteSlice(t *testing.T) {
	var slice []byteDecoder
	if err := Decode(bytes.NewReader(unhex("C101")), &slice); err != nil {
		t.Errorf("unexpected Decode error %v", err)
	} else if !slice[0].called() {
		t.Errorf("DecodeRLP not called for slice element")
	}

	var array [1]byteDecoder
	if err := Decode(bytes.NewReader(unhex("C101")), &array); err != nil {
		t.Errorf("unexpected Decode error %v", err)
	} else if !array[0].called() {
		t.Errorf("DecodeRLP not called for array element")
	}
}

type unencodableDecoder func()

func (f *unencodableDecoder) DecodeRLP(s *Stream) error {
	if _, err := s.List(); err != nil {
		return err
	}
	if err := s.ListEnd(); err != nil {
		return err
	}
	*f = func() {}
	return nil
}

func TestDecoderFunc(t *testing.T) {
	var x func()
	if err := DecodeBytes([]byte{0xC0}, (*unencodableDecoder)(&x)); err != nil {
		t.Fatal(err)
	}
	x()
}

// This tests the validity checks for fields with struct tag "optional".
func TestInvalidOptionalField(t *testing.T) {
	type (
		invalid1 struct {
			A uint `rlp:"optional"`
			B uint
		}
		invalid2 struct {
			T []uint `rlp:"tail,optional"`
		}
		invalid3 struct {
			T []uint `rlp:"optional,tail"`
		}
	)

	tests := []struct {
		v   interface{}
		err string
	}{
		{v: new(invalid1), err: `rlp: invalid struct tag "" for rlp.invalid1.B (must be optional because preceding field "A" is optional)`},
		{v: new(invalid2), err: `rlp: invalid struct tag "optional" for rlp.invalid2.T (also has "tail" tag)`},
		{v: new(invalid3), err: `rlp: invalid struct tag "tail" for rlp.invalid3.T (also has "optional" tag)`},
	}
	for _, test := range tests {
		err := DecodeBytes(unhex("C20102"), test.v)
		if err == nil {
			t.Errorf("no error for %T", test.v)
		} else if err.Error() != test.err {
			t.Errorf("wrong error for %T: %v", test.v, err.Error())
		}
	}
}

func ExampleDecode() {
	input, _ := hex.DecodeString("C90A1486666F6F626172")

	type example struct {
		A, B   uint
		String string
	}

	var s example
	err := Decode(bytes.NewReader(input), &s)
	if err != nil {
		fmt.Printf("Error: %v\n", err)
	} else {
		fmt.Printf("Decoded value: %#v\n", s)
	}
	// Output:
	// Decoded value: rlp.example{A:0xa, B:0x14, String:"foobar"}
}

func ExampleDecode_structTagNil() {
	// In this example, we'll use the "nil" struct tag to change
	// how a pointer-typed field is decoded. The input contains an RLP
	// list of one element, an empty string.
	input := []byte{0xC1, 0x80}

	// This type uses the normal rules.
	// The empty input string is decoded as a pointer to an empty Go string.
	var normalRules struct {
		String *string
	}
	Decode(bytes.NewReader(input), &normalRules)
	fmt.Printf("normal: String = %q\n", *normalRules.String)

	// This type uses the struct tag.
	// The empty input string is decoded as a nil pointer.
	var withEmptyOK struct {
		String *string `rlp:"nil"`
	}
	Decode(bytes.NewReader(input), &withEmptyOK)
	fmt.Printf("with nil tag: String = %v\n", withEmptyOK.String)

	// Output:
	// normal: String = ""
	// with nil tag: String = <nil>
}

func ExampleStream() {
	input, _ := hex.DecodeString("C90A1486666F6F626172")
	s := NewStream(bytes.NewReader(input), 0)

	// Check what kind of value lies ahead
	kind, size, _ := s.Kind()
	fmt.Printf("Kind: %v size:%d\n", kind, size)

	// Enter the list
	if _, err := s.List(); err != nil {
		fmt.Printf("List error: %v\n", err)
		return
	}

	// Decode elements
	fmt.Println(s.Uint())
	fmt.Println(s.Uint())
	fmt.Println(s.Bytes())

	// Acknowledge end of list
	if err := s.ListEnd(); err != nil {
		fmt.Printf("ListEnd error: %v\n", err)
	}
	// Output:
	// Kind: List size:9
	// 10 <nil>
	// 20 <nil>
	// [102 111 111 98 97 114] <nil>
}

func BenchmarkDecodeUints(b *testing.B) {
	enc := encodeTestSlice(90000)
	b.SetBytes(int64(len(enc)))
	b.ReportAllocs()
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		var s []uint
		r := bytes.NewReader(enc)
		if err := Decode(r, &s); err != nil {
			b.Fatalf("Decode error: %v", err)
		}
	}
}

func BenchmarkDecodeUintsReused(b *testing.B) {
	enc := encodeTestSlice(100000)
	b.SetBytes(int64(len(enc)))
	b.ReportAllocs()
	b.ResetTimer()

	var s []uint
	for i := 0; i < b.N; i++ {
		r := bytes.NewReader(enc)
		if err := Decode(r, &s); err != nil {
			b.Fatalf("Decode error: %v", err)
		}
	}
}

func BenchmarkDecodeByteArrayStruct(b *testing.B) {
	enc, err := EncodeToBytes(&byteArrayStruct{})
	if err != nil {
		b.Fatal(err)
	}
	b.SetBytes(int64(len(enc)))
	b.ReportAllocs()
	b.ResetTimer()

	var out byteArrayStruct
	for i := 0; i < b.N; i++ {
		if err := DecodeBytes(enc, &out); err != nil {
			b.Fatal(err)
		}
	}
}

func BenchmarkDecodeBigInts(b *testing.B) {
	ints := make([]*big.Int, 200)
	for i := range ints {
		ints[i] = math.BigPow(2, int64(i))
	}
	enc, err := EncodeToBytes(ints)
	if err != nil {
		b.Fatal(err)
	}
	b.SetBytes(int64(len(enc)))
	b.ReportAllocs()
	b.ResetTimer()

	var out []*big.Int
	for i := 0; i < b.N; i++ {
		if err := DecodeBytes(enc, &out); err != nil {
			b.Fatal(err)
		}
	}
}

func BenchmarkDecodeU256Ints(b *testing.B) {
	ints := make([]*uint256.Int, 200)
	for i := range ints {
		ints[i], _ = uint256.FromBig(math.BigPow(2, int64(i)))
	}
	enc, err := EncodeToBytes(ints)
	if err != nil {
		b.Fatal(err)
	}
	b.SetBytes(int64(len(enc)))
	b.ReportAllocs()
	b.ResetTimer()

	var out []*uint256.Int
	for i := 0; i < b.N; i++ {
		if err := DecodeBytes(enc, &out); err != nil {
			b.Fatal(err)
		}
	}
}

func encodeTestSlice(n uint) []byte {
	s := make([]uint, n)
	for i := uint(0); i < n; i++ {
		s[i] = i
	}
	b, err := EncodeToBytes(s)
	if err != nil {
		panic(fmt.Sprintf("encode error: %v", err))
	}
	return b
}

func unhex(str string) []byte {
	b, err := hex.DecodeString(strings.ReplaceAll(str, " ", ""))
	if err != nil {
		panic(fmt.Sprintf("invalid hex string: %q", str))
	}
	return b
}
