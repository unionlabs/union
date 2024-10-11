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
	"errors"
	"io"
	"testing"
	"testing/quick"
)

func TestCountValues(t *testing.T) {
	tests := []struct {
		input string // note: spaces in input are stripped by unhex
		count int
		err   error
	}{
		// simple cases
		{"", 0, nil},
		{"00", 1, nil},
		{"80", 1, nil},
		{"C0", 1, nil},
		{"01 02 03", 3, nil},
		{"01 C406070809 02", 3, nil},
		{"820101 820202 8403030303 04", 4, nil},

		// size errors
		{"8142", 0, ErrCanonSize},
		{"01 01 8142", 0, ErrCanonSize},
		{"02 84020202", 0, ErrValueTooLarge},

		{
			input: "A12000BF49F440A1CD0527E4D06E2765654C0F56452257516D793A9B8D604DCFDF2AB853F851808D10000000000000000000000000A056E81F171BCC55A6FF8345E692C0F86E5B48E01B996CADC001622FB5E363B421A0C5D2460186F7233C927E7DB2DCC703C0E500B653CA82273B7BFAD8045D85A470",
			count: 2,
		},
	}
	for i, test := range tests {
		count, err := CountValues(unhex(test.input))
		if count != test.count {
			t.Errorf("test %d: count mismatch, got %d want %d\ninput: %s", i, count, test.count, test.input)
		}
		if !errors.Is(err, test.err) {
			t.Errorf("test %d: err mismatch, got %q want %q\ninput: %s", i, err, test.err, test.input)
		}
	}
}

func TestSplitString(t *testing.T) {
	for i, test := range []string{
		"C0",
		"C100",
		"C3010203",
		"C88363617483646F67",
		"F8384C6F72656D20697073756D20646F6C6F722073697420616D65742C20636F6E7365637465747572206164697069736963696E6720656C6974",
	} {
		if _, _, err := SplitString(unhex(test)); !errors.Is(err, ErrExpectedString) {
			t.Errorf("test %d: error mismatch: have %q, want %q", i, err, ErrExpectedString)
		}
	}
}

func TestSplitList(t *testing.T) {
	for i, test := range []string{
		"80",
		"00",
		"01",
		"8180",
		"81FF",
		"820400",
		"83636174",
		"83646F67",
		"B8384C6F72656D20697073756D20646F6C6F722073697420616D65742C20636F6E7365637465747572206164697069736963696E6720656C6974",
	} {
		if _, _, err := SplitList(unhex(test)); !errors.Is(err, ErrExpectedList) {
			t.Errorf("test %d: error mismatch: have %q, want %q", i, err, ErrExpectedList)
		}
	}
}

func TestSplitUint64(t *testing.T) {
	tests := []struct {
		input string
		val   uint64
		rest  string
		err   error
	}{
		{"01", 1, "", nil},
		{"7FFF", 0x7F, "FF", nil},
		{"80FF", 0, "FF", nil},
		{"81FAFF", 0xFA, "FF", nil},
		{"82FAFAFF", 0xFAFA, "FF", nil},
		{"83FAFAFAFF", 0xFAFAFA, "FF", nil},
		{"84FAFAFAFAFF", 0xFAFAFAFA, "FF", nil},
		{"85FAFAFAFAFAFF", 0xFAFAFAFAFA, "FF", nil},
		{"86FAFAFAFAFAFAFF", 0xFAFAFAFAFAFA, "FF", nil},
		{"87FAFAFAFAFAFAFAFF", 0xFAFAFAFAFAFAFA, "FF", nil},
		{"88FAFAFAFAFAFAFAFAFF", 0xFAFAFAFAFAFAFAFA, "FF", nil},

		// errors
		{"", 0, "", io.ErrUnexpectedEOF},
		{"00", 0, "00", ErrCanonInt},
		{"81", 0, "81", ErrValueTooLarge},
		{"8100", 0, "8100", ErrCanonSize},
		{"8200FF", 0, "8200FF", ErrCanonInt},
		{"8103FF", 0, "8103FF", ErrCanonSize},
		{"89FAFAFAFAFAFAFAFAFAFF", 0, "89FAFAFAFAFAFAFAFAFAFF", errUintOverflow},
	}

	for i, test := range tests {
		val, rest, err := SplitUint64(unhex(test.input))
		if val != test.val {
			t.Errorf("test %d: val mismatch: got %x, want %x (input %q)", i, val, test.val, test.input)
		}
		if !bytes.Equal(rest, unhex(test.rest)) {
			t.Errorf("test %d: rest mismatch: got %x, want %s (input %q)", i, rest, test.rest, test.input)
		}
		if err != test.err {
			t.Errorf("test %d: error mismatch: got %q, want %q", i, err, test.err)
		}
	}
}

func TestSplit(t *testing.T) {
	tests := []struct {
		input     string
		kind      Kind
		val, rest string
		err       error
	}{
		{input: "00FFFF", kind: Byte, val: "00", rest: "FFFF"},
		{input: "01FFFF", kind: Byte, val: "01", rest: "FFFF"},
		{input: "7FFFFF", kind: Byte, val: "7F", rest: "FFFF"},
		{input: "80FFFF", kind: String, val: "", rest: "FFFF"},
		{input: "C3010203", kind: List, val: "010203"},

		// errors
		{input: "", err: io.ErrUnexpectedEOF},

		{input: "8141", err: ErrCanonSize, rest: "8141"},
		{input: "B800", err: ErrCanonSize, rest: "B800"},
		{input: "B802FFFF", err: ErrCanonSize, rest: "B802FFFF"},
		{input: "B90000", err: ErrCanonSize, rest: "B90000"},
		{input: "B90055", err: ErrCanonSize, rest: "B90055"},
		{input: "BA0002FFFF", err: ErrCanonSize, rest: "BA0002FFFF"},
		{input: "F800", err: ErrCanonSize, rest: "F800"},
		{input: "F90000", err: ErrCanonSize, rest: "F90000"},
		{input: "F90055", err: ErrCanonSize, rest: "F90055"},
		{input: "FA0002FFFF", err: ErrCanonSize, rest: "FA0002FFFF"},

		{input: "81", err: ErrValueTooLarge, rest: "81"},
		{input: "8501010101", err: ErrValueTooLarge, rest: "8501010101"},
		{input: "C60607080902", err: ErrValueTooLarge, rest: "C60607080902"},

		// size check overflow
		{input: "BFFFFFFFFFFFFFFFFF", err: ErrValueTooLarge, rest: "BFFFFFFFFFFFFFFFFF"},
		{input: "FFFFFFFFFFFFFFFFFF", err: ErrValueTooLarge, rest: "FFFFFFFFFFFFFFFFFF"},

		{
			input: "B838FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
			err:   ErrValueTooLarge,
			rest:  "B838FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
		},
		{
			input: "F838FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
			err:   ErrValueTooLarge,
			rest:  "F838FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
		},

		// a few bigger values, just for kicks
		{
			input: "F839FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
			kind:  List,
			val:   "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
			rest:  "",
		},
		{
			input: "F90211A060EF29F20CC1007AE6E9530AEE16F4B31F8F1769A2D1264EC995C6D1241868D6A07C62AB8AC9838F5F5877B20BB37B387BC2106E97A3D52172CBEDB5EE17C36008A00EAB6B7324AADC0F6047C6AFC8229F09F7CF451B51D67C8DFB08D49BA8C3C626A04453343B2F3A6E42FCF87948F88AF7C8FC16D0C2735CBA7F026836239AB2C15FA024635C7291C882CE4C0763760C1A362DFC3FFCD802A55722236DE058D74202ACA0A220C808DE10F55E40AB25255201CFF009EA181D3906638E944EE2BF34049984A08D325AB26796F1CCB470F69C0F842501DC35D368A0C2575B2D243CFD1E8AB0FDA0B5298FF60DA5069463D610513C9F04F24051348391A143AFFAB7197DFACDEA72A02D2A7058A4463F8FB69378369E11EF33AE3252E2DB86CB545B36D3C26DDECE5AA0888F97BCA8E0BD83DC5B3B91CFF5FAF2F66F9501010682D67EF4A3B4E66115FBA0E8175A60C93BE9ED02921958F0EA55DA0FB5E4802AF5846147BAD92BC2D8AF26A08B3376FF433F3A4250FA64B7F804004CAC5807877D91C4427BD1CD05CF912ED8A09B32EF0F03BD13C37FF950C0CCCEFCCDD6669F2E7F2AA5CB859928E84E29763EA09BBA5E46610C8C8B1F8E921E5691BF8C7E40D75825D5EA3217AA9C3A8A355F39A0EEB95BC78251CCCEC54A97F19755C4A59A293544EEE6119AFA50531211E53C4FA00B6E86FE150BF4A9E0FEEE9C90F5465E617A861BB5E357F942881EE762212E2580",
			kind:  List,
			val:   "A060EF29F20CC1007AE6E9530AEE16F4B31F8F1769A2D1264EC995C6D1241868D6A07C62AB8AC9838F5F5877B20BB37B387BC2106E97A3D52172CBEDB5EE17C36008A00EAB6B7324AADC0F6047C6AFC8229F09F7CF451B51D67C8DFB08D49BA8C3C626A04453343B2F3A6E42FCF87948F88AF7C8FC16D0C2735CBA7F026836239AB2C15FA024635C7291C882CE4C0763760C1A362DFC3FFCD802A55722236DE058D74202ACA0A220C808DE10F55E40AB25255201CFF009EA181D3906638E944EE2BF34049984A08D325AB26796F1CCB470F69C0F842501DC35D368A0C2575B2D243CFD1E8AB0FDA0B5298FF60DA5069463D610513C9F04F24051348391A143AFFAB7197DFACDEA72A02D2A7058A4463F8FB69378369E11EF33AE3252E2DB86CB545B36D3C26DDECE5AA0888F97BCA8E0BD83DC5B3B91CFF5FAF2F66F9501010682D67EF4A3B4E66115FBA0E8175A60C93BE9ED02921958F0EA55DA0FB5E4802AF5846147BAD92BC2D8AF26A08B3376FF433F3A4250FA64B7F804004CAC5807877D91C4427BD1CD05CF912ED8A09B32EF0F03BD13C37FF950C0CCCEFCCDD6669F2E7F2AA5CB859928E84E29763EA09BBA5E46610C8C8B1F8E921E5691BF8C7E40D75825D5EA3217AA9C3A8A355F39A0EEB95BC78251CCCEC54A97F19755C4A59A293544EEE6119AFA50531211E53C4FA00B6E86FE150BF4A9E0FEEE9C90F5465E617A861BB5E357F942881EE762212E2580",
			rest:  "",
		},
		{
			input: "F877A12000BF49F440A1CD0527E4D06E2765654C0F56452257516D793A9B8D604DCFDF2AB853F851808D10000000000000000000000000A056E81F171BCC55A6FF8345E692C0F86E5B48E01B996CADC001622FB5E363B421A0C5D2460186F7233C927E7DB2DCC703C0E500B653CA82273B7BFAD8045D85A470",
			kind:  List,
			val:   "A12000BF49F440A1CD0527E4D06E2765654C0F56452257516D793A9B8D604DCFDF2AB853F851808D10000000000000000000000000A056E81F171BCC55A6FF8345E692C0F86E5B48E01B996CADC001622FB5E363B421A0C5D2460186F7233C927E7DB2DCC703C0E500B653CA82273B7BFAD8045D85A470",
			rest:  "",
		},
	}

	for i, test := range tests {
		kind, val, rest, err := Split(unhex(test.input))
		if kind != test.kind {
			t.Errorf("test %d: kind mismatch: got %v, want %v", i, kind, test.kind)
		}
		if !bytes.Equal(val, unhex(test.val)) {
			t.Errorf("test %d: val mismatch: got %x, want %s", i, val, test.val)
		}
		if !bytes.Equal(rest, unhex(test.rest)) {
			t.Errorf("test %d: rest mismatch: got %x, want %s", i, rest, test.rest)
		}
		if err != test.err {
			t.Errorf("test %d: error mismatch: got %q, want %q", i, err, test.err)
		}
	}
}

func TestReadSize(t *testing.T) {
	tests := []struct {
		input string
		slen  byte
		size  uint64
		err   error
	}{
		{input: "", slen: 1, err: io.ErrUnexpectedEOF},
		{input: "FF", slen: 2, err: io.ErrUnexpectedEOF},
		{input: "00", slen: 1, err: ErrCanonSize},
		{input: "36", slen: 1, err: ErrCanonSize},
		{input: "37", slen: 1, err: ErrCanonSize},
		{input: "38", slen: 1, size: 0x38},
		{input: "FF", slen: 1, size: 0xFF},
		{input: "FFFF", slen: 2, size: 0xFFFF},
		{input: "FFFFFF", slen: 3, size: 0xFFFFFF},
		{input: "FFFFFFFF", slen: 4, size: 0xFFFFFFFF},
		{input: "FFFFFFFFFF", slen: 5, size: 0xFFFFFFFFFF},
		{input: "FFFFFFFFFFFF", slen: 6, size: 0xFFFFFFFFFFFF},
		{input: "FFFFFFFFFFFFFF", slen: 7, size: 0xFFFFFFFFFFFFFF},
		{input: "FFFFFFFFFFFFFFFF", slen: 8, size: 0xFFFFFFFFFFFFFFFF},
		{input: "0102", slen: 2, size: 0x0102},
		{input: "010203", slen: 3, size: 0x010203},
		{input: "01020304", slen: 4, size: 0x01020304},
		{input: "0102030405", slen: 5, size: 0x0102030405},
		{input: "010203040506", slen: 6, size: 0x010203040506},
		{input: "01020304050607", slen: 7, size: 0x01020304050607},
		{input: "0102030405060708", slen: 8, size: 0x0102030405060708},
	}

	for _, test := range tests {
		size, err := readSize(unhex(test.input), test.slen)
		if err != test.err {
			t.Errorf("readSize(%s, %d): error mismatch: got %q, want %q", test.input, test.slen, err, test.err)
			continue
		}
		if size != test.size {
			t.Errorf("readSize(%s, %d): size mismatch: got %#x, want %#x", test.input, test.slen, size, test.size)
		}
	}
}

func TestAppendUint64(t *testing.T) {
	tests := []struct {
		input  uint64
		slice  []byte
		output string
	}{
		{0, nil, "80"},
		{1, nil, "01"},
		{2, nil, "02"},
		{127, nil, "7F"},
		{128, nil, "8180"},
		{129, nil, "8181"},
		{0xFFFFFF, nil, "83FFFFFF"},
		{127, []byte{1, 2, 3}, "0102037F"},
		{0xFFFFFF, []byte{1, 2, 3}, "01020383FFFFFF"},
	}

	for _, test := range tests {
		x := AppendUint64(test.slice, test.input)
		if !bytes.Equal(x, unhex(test.output)) {
			t.Errorf("AppendUint64(%v, %d): got %x, want %s", test.slice, test.input, x, test.output)
		}

		// Check that IntSize returns the appended size.
		length := len(x) - len(test.slice)
		if s := IntSize(test.input); s != length {
			t.Errorf("IntSize(%d): got %d, want %d", test.input, s, length)
		}
	}
}

func TestAppendUint64Random(t *testing.T) {
	fn := func(i uint64) bool {
		enc, _ := EncodeToBytes(i)
		encAppend := AppendUint64(nil, i)
		return bytes.Equal(enc, encAppend)
	}
	config := quick.Config{MaxCountScale: 50}
	if err := quick.Check(fn, &config); err != nil {
		t.Fatal(err)
	}
}

func TestBytesSize(t *testing.T) {
	tests := []struct {
		v    []byte
		size uint64
	}{
		{v: []byte{}, size: 1},
		{v: []byte{0x1}, size: 1},
		{v: []byte{0x7E}, size: 1},
		{v: []byte{0x7F}, size: 1},
		{v: []byte{0x80}, size: 2},
		{v: []byte{0xFF}, size: 2},
		{v: []byte{0xFF, 0xF0}, size: 3},
		{v: make([]byte, 55), size: 56},
		{v: make([]byte, 56), size: 58},
	}

	for _, test := range tests {
		s := BytesSize(test.v)
		if s != test.size {
			t.Errorf("BytesSize(%#x) -> %d, want %d", test.v, s, test.size)
		}
		s = StringSize(string(test.v))
		if s != test.size {
			t.Errorf("StringSize(%#x) -> %d, want %d", test.v, s, test.size)
		}
		// Sanity check:
		enc, _ := EncodeToBytes(test.v)
		if uint64(len(enc)) != test.size {
			t.Errorf("len(EncodeToBytes(%#x)) -> %d, test says %d", test.v, len(enc), test.size)
		}
	}
}
