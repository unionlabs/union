// Copyright (c) 2013-2016 The btcsuite developers
// Copyright (c) 2013-2016 Dave Collins
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package btcec

import (
	"math/rand"

	"encoding/hex"
	"testing"
)

// TestIsZero ensures that checking if a field IsZero works as expected.
func TestIsZero(t *testing.T) {
	f := new(FieldVal)
	if !f.IsZero() {
		t.Errorf("new field value is not zero - got %v (rawints %x)", f,
			f.String())
	}

	f.SetInt(1)
	if f.IsZero() {
		t.Errorf("field claims it's zero when it's not - got %v "+
			"(raw rawints %x)", f, f.String())
	}

	f.Zero()
	if !f.IsZero() {
		t.Errorf("field claims it's not zero when it is - got %v "+
			"(raw rawints %x)", f, f.String())
	}
}

// TestStringer ensures the stringer returns the appropriate hex string.
func TestStringer(t *testing.T) {
	tests := []struct {
		in       string
		expected string
	}{
		{"0", "0000000000000000000000000000000000000000000000000000000000000000"},
		{"1", "0000000000000000000000000000000000000000000000000000000000000001"},
		{"a", "000000000000000000000000000000000000000000000000000000000000000a"},
		{"b", "000000000000000000000000000000000000000000000000000000000000000b"},
		{"c", "000000000000000000000000000000000000000000000000000000000000000c"},
		{"d", "000000000000000000000000000000000000000000000000000000000000000d"},
		{"e", "000000000000000000000000000000000000000000000000000000000000000e"},
		{"f", "000000000000000000000000000000000000000000000000000000000000000f"},
		{"f0", "00000000000000000000000000000000000000000000000000000000000000f0"},
		// 2^26-1
		{
			"3ffffff",
			"0000000000000000000000000000000000000000000000000000000003ffffff",
		},
		// 2^32-1
		{
			"ffffffff",
			"00000000000000000000000000000000000000000000000000000000ffffffff",
		},
		// 2^64-1
		{
			"ffffffffffffffff",
			"000000000000000000000000000000000000000000000000ffffffffffffffff",
		},
		// 2^96-1
		{
			"ffffffffffffffffffffffff",
			"0000000000000000000000000000000000000000ffffffffffffffffffffffff",
		},
		// 2^128-1
		{
			"ffffffffffffffffffffffffffffffff",
			"00000000000000000000000000000000ffffffffffffffffffffffffffffffff",
		},
		// 2^160-1
		{
			"ffffffffffffffffffffffffffffffffffffffff",
			"000000000000000000000000ffffffffffffffffffffffffffffffffffffffff",
		},
		// 2^192-1
		{
			"ffffffffffffffffffffffffffffffffffffffffffffffff",
			"0000000000000000ffffffffffffffffffffffffffffffffffffffffffffffff",
		},
		// 2^224-1
		{
			"ffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
			"00000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
		},
		// 2^256-4294968273 (the btcec prime, so should result in 0)
		{
			"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f",
			"0000000000000000000000000000000000000000000000000000000000000000",
		},
		// 2^256-4294968274 (the secp256k1 prime+1, so should result in 1)
		{
			"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc30",
			"0000000000000000000000000000000000000000000000000000000000000001",
		},

		// Invalid hex
		{"g", "0000000000000000000000000000000000000000000000000000000000000000"},
		{"1h", "0000000000000000000000000000000000000000000000000000000000000000"},
		{"i1", "0000000000000000000000000000000000000000000000000000000000000000"},
	}

	t.Logf("Running %d tests", len(tests))
	for i, test := range tests {
		f := setHex(test.in)
		result := f.String()
		if result != test.expected {
			t.Errorf("FieldVal.String #%d wrong result\ngot: %v\n"+
				"want: %v", i, result, test.expected)
			continue
		}
	}
}

// TestNormalize ensures that normalizing the internal field words works as
// expected.
func TestNormalize(t *testing.T) {
	tests := []struct {
		raw        [10]uint32 // Intentionally denormalized value
		normalized [10]uint32 // Normalized form of the raw value
	}{
		{
			[10]uint32{0x00000005, 0, 0, 0, 0, 0, 0, 0, 0, 0},
			[10]uint32{0x00000005, 0, 0, 0, 0, 0, 0, 0, 0, 0},
		},
		// 2^26
		{
			[10]uint32{0x04000000, 0x0, 0, 0, 0, 0, 0, 0, 0, 0},
			[10]uint32{0x00000000, 0x1, 0, 0, 0, 0, 0, 0, 0, 0},
		},
		// 2^26 + 1
		{
			[10]uint32{0x04000001, 0x0, 0, 0, 0, 0, 0, 0, 0, 0},
			[10]uint32{0x00000001, 0x1, 0, 0, 0, 0, 0, 0, 0, 0},
		},
		// 2^32 - 1
		{
			[10]uint32{0xffffffff, 0x00, 0, 0, 0, 0, 0, 0, 0, 0},
			[10]uint32{0x03ffffff, 0x3f, 0, 0, 0, 0, 0, 0, 0, 0},
		},
		// 2^32
		{
			[10]uint32{0x04000000, 0x3f, 0, 0, 0, 0, 0, 0, 0, 0},
			[10]uint32{0x00000000, 0x40, 0, 0, 0, 0, 0, 0, 0, 0},
		},
		// 2^32 + 1
		{
			[10]uint32{0x04000001, 0x3f, 0, 0, 0, 0, 0, 0, 0, 0},
			[10]uint32{0x00000001, 0x40, 0, 0, 0, 0, 0, 0, 0, 0},
		},
		// 2^64 - 1
		{
			[10]uint32{0xffffffff, 0xffffffc0, 0xfc0, 0, 0, 0, 0, 0, 0, 0},
			[10]uint32{0x03ffffff, 0x03ffffff, 0xfff, 0, 0, 0, 0, 0, 0, 0},
		},
		// 2^64
		{
			[10]uint32{0x04000000, 0x03ffffff, 0x0fff, 0, 0, 0, 0, 0, 0, 0},
			[10]uint32{0x00000000, 0x00000000, 0x1000, 0, 0, 0, 0, 0, 0, 0},
		},
		// 2^64 + 1
		{
			[10]uint32{0x04000001, 0x03ffffff, 0x0fff, 0, 0, 0, 0, 0, 0, 0},
			[10]uint32{0x00000001, 0x00000000, 0x1000, 0, 0, 0, 0, 0, 0, 0},
		},
		// 2^96 - 1
		{
			[10]uint32{0xffffffff, 0xffffffc0, 0xffffffc0, 0x3ffc0, 0, 0, 0, 0, 0, 0},
			[10]uint32{0x03ffffff, 0x03ffffff, 0x03ffffff, 0x3ffff, 0, 0, 0, 0, 0, 0},
		},
		// 2^96
		{
			[10]uint32{0x04000000, 0x03ffffff, 0x03ffffff, 0x3ffff, 0, 0, 0, 0, 0, 0},
			[10]uint32{0x00000000, 0x00000000, 0x00000000, 0x40000, 0, 0, 0, 0, 0, 0},
		},
		// 2^128 - 1
		{
			[10]uint32{0xffffffff, 0xffffffc0, 0xffffffc0, 0xffffffc0, 0xffffc0, 0, 0, 0, 0, 0},
			[10]uint32{0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0xffffff, 0, 0, 0, 0, 0},
		},
		// 2^128
		{
			[10]uint32{0x04000000, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x0ffffff, 0, 0, 0, 0, 0},
			[10]uint32{0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x1000000, 0, 0, 0, 0, 0},
		},
		// 2^256 - 4294968273 (secp256k1 prime)
		{
			[10]uint32{0xfffffc2f, 0xffffff80, 0xffffffc0, 0xffffffc0, 0xffffffc0, 0xffffffc0, 0xffffffc0, 0xffffffc0, 0xffffffc0, 0x3fffc0},
			[10]uint32{0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x000000},
		},
		// Prime larger than P where both first and second words are larger
		// than P's first and second words
		{
			[10]uint32{0xfffffc30, 0xffffff86, 0xffffffc0, 0xffffffc0, 0xffffffc0, 0xffffffc0, 0xffffffc0, 0xffffffc0, 0xffffffc0, 0x3fffc0},
			[10]uint32{0x00000001, 0x00000006, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x000000},
		},
		// Prime larger than P where only the second word is larger
		// than P's second words.
		{
			[10]uint32{0xfffffc2a, 0xffffff87, 0xffffffc0, 0xffffffc0, 0xffffffc0, 0xffffffc0, 0xffffffc0, 0xffffffc0, 0xffffffc0, 0x3fffc0},
			[10]uint32{0x03fffffb, 0x00000006, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x000000},
		},
		// 2^256 - 1
		{
			[10]uint32{0xffffffff, 0xffffffc0, 0xffffffc0, 0xffffffc0, 0xffffffc0, 0xffffffc0, 0xffffffc0, 0xffffffc0, 0xffffffc0, 0x3fffc0},
			[10]uint32{0x000003d0, 0x00000040, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x000000},
		},
		// Prime with field representation such that the initial
		// reduction does not result in a carry to bit 256.
		//
		// 2^256 - 4294968273 (secp256k1 prime)
		{
			[10]uint32{0x03fffc2f, 0x03ffffbf, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x003fffff},
			[10]uint32{0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000},
		},
		// Prime larger than P that reduces to a value which is still
		// larger than P when it has a magnitude of 1 due to its first
		// word and does not result in a carry to bit 256.
		//
		// 2^256 - 4294968272 (secp256k1 prime + 1)
		{
			[10]uint32{0x03fffc30, 0x03ffffbf, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x003fffff},
			[10]uint32{0x00000001, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000},
		},
		// Prime larger than P that reduces to a value which is still
		// larger than P when it has a magnitude of 1 due to its second
		// word and does not result in a carry to bit 256.
		//
		// 2^256 - 4227859409 (secp256k1 prime + 0x4000000)
		{
			[10]uint32{0x03fffc2f, 0x03ffffc0, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x003fffff},
			[10]uint32{0x00000000, 0x00000001, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000},
		},
		// Prime larger than P that reduces to a value which is still
		// larger than P when it has a magnitude of 1 due to a carry to
		// bit 256, but would not be without the carry.  These values
		// come from the fact that P is 2^256 - 4294968273 and 977 is
		// the low order word in the internal field representation.
		//
		// 2^256 * 5 - ((4294968273 - (977+1)) * 4)
		{
			[10]uint32{0x03ffffff, 0x03fffeff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x0013fffff},
			[10]uint32{0x00001314, 0x00000040, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x000000000},
		},
		// Prime larger than P that reduces to a value which is still
		// larger than P when it has a magnitude of 1 due to both a
		// carry to bit 256 and the first word.
		{
			[10]uint32{0x03fffc30, 0x03ffffbf, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x07ffffff, 0x003fffff},
			[10]uint32{0x00000001, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000001},
		},
		// Prime larger than P that reduces to a value which is still
		// larger than P when it has a magnitude of 1 due to both a
		// carry to bit 256 and the second word.
		//
		{
			[10]uint32{0x03fffc2f, 0x03ffffc0, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x3ffffff, 0x07ffffff, 0x003fffff},
			[10]uint32{0x00000000, 0x00000001, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x0000000, 0x00000000, 0x00000001},
		},
		// Prime larger than P that reduces to a value which is still
		// larger than P when it has a magnitude of 1 due to a carry to
		// bit 256 and the first and second words.
		//
		{
			[10]uint32{0x03fffc30, 0x03ffffc0, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x03ffffff, 0x07ffffff, 0x003fffff},
			[10]uint32{0x00000001, 0x00000001, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000001},
		},
	}

	t.Logf("Running %d tests", len(tests))
	for range tests {
		// TODO(roasbeef): can't access internal state
		/*f := new(FieldVal)
		f.n = test.raw
		f.Normalize()
		if !reflect.DeepEqual(f.n, test.normalized) {
			t.Errorf("FieldVal.Normalize #%d wrong result\n"+
				"got: %x\nwant: %x", i, f.n, test.normalized)
			continue
		}*/
	}
}

// TestIsOdd ensures that checking if a field value IsOdd works as expected.
func TestIsOdd(t *testing.T) {
	tests := []struct {
		in       string // hex encoded value
		expected bool   // expected oddness
	}{
		{"0", false},
		{"1", true},
		{"2", false},
		// 2^32 - 1
		{"ffffffff", true},
		// 2^64 - 2
		{"fffffffffffffffe", false},
		// secp256k1 prime
		{"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f", true},
	}

	t.Logf("Running %d tests", len(tests))
	for i, test := range tests {
		f := setHex(test.in)
		result := f.IsOdd()
		if result != test.expected {
			t.Errorf("FieldVal.IsOdd #%d wrong result\n"+
				"got: %v\nwant: %v", i, result, test.expected)
			continue
		}
	}
}

// TestEquals ensures that checking two field values for equality via Equals
// works as expected.
func TestEquals(t *testing.T) {
	tests := []struct {
		in1      string // hex encoded value
		in2      string // hex encoded value
		expected bool   // expected equality
	}{
		{"0", "0", true},
		{"0", "1", false},
		{"1", "0", false},
		// 2^32 - 1 == 2^32 - 1?
		{"ffffffff", "ffffffff", true},
		// 2^64 - 1 == 2^64 - 2?
		{"ffffffffffffffff", "fffffffffffffffe", false},
		// 0 == prime (mod prime)?
		{"0", "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f", true},
		// 1 == prime+1 (mod prime)?
		{"1", "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc30", true},
	}

	t.Logf("Running %d tests", len(tests))
	for i, test := range tests {
		f := setHex(test.in1).Normalize()
		f2 := setHex(test.in2).Normalize()
		result := f.Equals(f2)
		if result != test.expected {
			t.Errorf("FieldVal.Equals #%d wrong result\n"+
				"got: %v\nwant: %v", i, result, test.expected)
			continue
		}
	}
}

// TestNegate ensures that negating field values via Negate works as expected.
func TestNegate(t *testing.T) {
	tests := []struct {
		in       string // hex encoded value
		expected string // expected hex encoded value
	}{
		// secp256k1 prime (aka 0)
		{"0", "0"},
		{"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f", "0"},
		{"0", "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f"},
		// secp256k1 prime-1
		{"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e", "1"},
		{"1", "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e"},
		// secp256k1 prime-2
		{"2", "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2d"},
		{"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2d", "2"},
		// Random sampling
		{
			"b3d9aac9c5e43910b4385b53c7e78c21d4cd5f8e683c633aed04c233efc2e120",
			"4c2655363a1bc6ef4bc7a4ac381873de2b32a07197c39cc512fb3dcb103d1b0f",
		},
		{
			"f8a85984fee5a12a7c8dd08830d83423c937d77c379e4a958e447a25f407733f",
			"757a67b011a5ed583722f77cf27cbdc36c82883c861b56a71bb85d90bf888f0",
		},
		{
			"45ee6142a7fda884211e93352ed6cb2807800e419533be723a9548823ece8312",
			"ba119ebd5802577bdee16ccad12934d7f87ff1be6acc418dc56ab77cc131791d",
		},
		{
			"53c2a668f07e411a2e473e1c3b6dcb495dec1227af27673761d44afe5b43d22b",
			"ac3d59970f81bee5d1b8c1e3c49234b6a213edd850d898c89e2bb500a4bc2a04",
		},
	}

	t.Logf("Running %d tests", len(tests))
	for i, test := range tests {
		f := setHex(test.in).Normalize()
		expected := setHex(test.expected).Normalize()
		result := f.Negate(1).Normalize()
		if !result.Equals(expected) {
			t.Errorf("FieldVal.Negate #%d wrong result\n"+
				"got: %v\nwant: %v", i, result, expected)
			continue
		}
	}
}

// TestFieldAddInt ensures that adding an integer to field values via AddInt
// works as expected.
func TestFieldAddInt(t *testing.T) {
	tests := []struct {
		name     string // test description
		in1      string // hex encoded value
		in2      uint16 // unsigned integer to add to the value above
		expected string // expected hex encoded value
	}{{
		name:     "zero + one",
		in1:      "0",
		in2:      1,
		expected: "1",
	}, {
		name:     "one + zero",
		in1:      "1",
		in2:      0,
		expected: "1",
	}, {
		name:     "one + one",
		in1:      "1",
		in2:      1,
		expected: "2",
	}, {
		name:     "secp256k1 prime-1 + 1",
		in1:      "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e",
		in2:      1,
		expected: "0",
	}, {
		name:     "secp256k1 prime + 1",
		in1:      "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f",
		in2:      1,
		expected: "1",
	}, {
		name:     "random sampling #1",
		in1:      "ff95ad9315aff04ab4af0ce673620c7145dc85d03bab5ba4b09ca2c4dec2d6c1",
		in2:      0x10f,
		expected: "ff95ad9315aff04ab4af0ce673620c7145dc85d03bab5ba4b09ca2c4dec2d7d0",
	}, {
		name:     "random sampling #2",
		in1:      "44bdae6b772e7987941f1ba314e6a5b7804a4c12c00961b57d20f41deea9cecf",
		in2:      0x3196,
		expected: "44bdae6b772e7987941f1ba314e6a5b7804a4c12c00961b57d20f41deeaa0065",
	}, {
		name:     "random sampling #3",
		in1:      "88c3ecae67b591935fb1f6a9499c35315ffad766adca665c50b55f7105122c9c",
		in2:      0x966f,
		expected: "88c3ecae67b591935fb1f6a9499c35315ffad766adca665c50b55f710512c30b",
	}, {
		name:     "random sampling #4",
		in1:      "8523e9edf360ca32a95aae4e57fcde5a542b471d08a974d94ea0ee09a015e2a6",
		in2:      0xc54,
		expected: "8523e9edf360ca32a95aae4e57fcde5a542b471d08a974d94ea0ee09a015eefa",
	}}

	for _, test := range tests {
		f := setHex(test.in1).Normalize()
		expected := setHex(test.expected).Normalize()
		result := f.AddInt(test.in2).Normalize()
		if !result.Equals(expected) {
			t.Errorf("%s: wrong result -- got: %v -- want: %v", test.name,
				result, expected)
			continue
		}
	}
}

// TestFieldAdd ensures that adding two field values together via Add and Add2
// works as expected.
func TestFieldAdd(t *testing.T) {
	tests := []struct {
		name     string // test description
		in1      string // first hex encoded value
		in2      string // second hex encoded value to add
		expected string // expected hex encoded value
	}{{
		name:     "zero + one",
		in1:      "0",
		in2:      "1",
		expected: "1",
	}, {
		name:     "one + zero",
		in1:      "1",
		in2:      "0",
		expected: "1",
	}, {
		name:     "secp256k1 prime-1 + 1",
		in1:      "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e",
		in2:      "1",
		expected: "0",
	}, {
		name:     "secp256k1 prime + 1",
		in1:      "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f",
		in2:      "1",
		expected: "1",
	}, {
		name:     "random sampling #1",
		in1:      "2b2012f975404e5065b4292fb8bed0a5d315eacf24c74d8b27e73bcc5430edcc",
		in2:      "2c3cefa4e4753e8aeec6ac4c12d99da4d78accefda3b7885d4c6bab46c86db92",
		expected: "575d029e59b58cdb547ad57bcb986e4aaaa0b7beff02c610fcadf680c0b7c95e",
	}, {
		name:     "random sampling #2",
		in1:      "8131e8722fe59bb189692b96c9f38de92885730f1dd39ab025daffb94c97f79c",
		in2:      "ff5454b765f0aab5f0977dcc629becc84cabeb9def48e79c6aadb2622c490fa9",
		expected: "80863d2995d646677a00a9632c8f7ab175315ead0d1c824c9088b21c78e10b16",
	}, {
		name:     "random sampling #3",
		in1:      "c7c95e93d0892b2b2cdd77e80eb646ea61be7a30ac7e097e9f843af73fad5c22",
		in2:      "3afe6f91a74dfc1c7f15c34907ee981656c37236d946767dd53ccad9190e437c",
		expected: "2c7ce2577d72747abf33b3116a4df00b881ec6785c47ffc74c105d158bba36f",
	}, {
		name:     "random sampling #4",
		in1:      "fd1c26f6a23381e5d785ba889494ec059369b888ad8431cd67d8c934b580dbe1",
		in2:      "a475aa5a31dcca90ef5b53c097d9133d6b7117474b41e7877bb199590fc0489c",
		expected: "a191d150d4104c76c6e10e492c6dff42fedacfcff8c61954e38a628ec541284e",
	}, {
		name:     "random sampling #5",
		in1:      "ad82b8d1cc136e23e9fd77fe2c7db1fe5a2ecbfcbde59ab3529758334f862d28",
		in2:      "4d6a4e95d6d61f4f46b528bebe152d408fd741157a28f415639347a84f6f574b",
		expected: "faed0767a2e98d7330b2a0bcea92df3eea060d12380e8ec8b62a9fdb9ef58473",
	}, {
		name:     "random sampling #6",
		in1:      "f3f43a2540054a86e1df98547ec1c0e157b193e5350fb4a3c3ea214b228ac5e7",
		in2:      "25706572592690ea3ddc951a1b48b504a4c83dc253756e1b96d56fdfb3199522",
		expected: "19649f97992bdb711fbc2d6e9a0a75e5fc79d1a7888522bf5abf912bd5a45eda",
	}, {
		name:     "random sampling #7",
		in1:      "6915bb94eef13ff1bb9b2633d997e13b9b1157c713363cc0e891416d6734f5b8",
		in2:      "11f90d6ac6fe1c4e8900b1c85fb575c251ec31b9bc34b35ada0aea1c21eded22",
		expected: "7b0ec8ffb5ef5c40449bd7fc394d56fdecfd8980cf6af01bc29c2b898922e2da",
	}, {
		name:     "random sampling #8",
		in1:      "48b0c9eae622eed9335b747968544eb3e75cb2dc8128388f948aa30f88cabde4",
		in2:      "0989882b52f85f9d524a3a3061a0e01f46d597839d2ba637320f4b9510c8d2d5",
		expected: "523a5216391b4e7685a5aea9c9f52ed32e324a601e53dec6c699eea4999390b9",
	}}

	for _, test := range tests {
		// Parse test hex.
		f1 := setHex(test.in1).Normalize()
		f2 := setHex(test.in2).Normalize()
		expected := setHex(test.expected).Normalize()

		// Ensure adding the two values with the result going to another
		// variable produces the expected result.
		result := new(FieldVal).Add2(f1, f2).Normalize()
		if !result.Equals(expected) {
			t.Errorf("%s: unexpected result\ngot: %v\nwant: %v", test.name,
				result, expected)
			continue
		}

		// Ensure adding the value to an existing field value produces the
		// expected result.
		f1.Add(f2).Normalize()
		if !f1.Equals(expected) {
			t.Errorf("%s: unexpected result\ngot: %v\nwant: %v", test.name,
				f1, expected)
			continue
		}
	}
}

// TestFieldMulInt ensures that multiplying an integer to field values via
// MulInt works as expected.
func TestFieldMulInt(t *testing.T) {
	tests := []struct {
		name     string // test description
		in1      string // hex encoded value
		in2      uint8  // unsigned integer to multiply with value above
		expected string // expected hex encoded value
	}{{
		name:     "zero * zero",
		in1:      "0",
		in2:      0,
		expected: "0",
	}, {
		name:     "one * zero",
		in1:      "1",
		in2:      0,
		expected: "0",
	}, {
		name:     "zero * one",
		in1:      "0",
		in2:      1,
		expected: "0",
	}, {
		name:     "one * one",
		in1:      "1",
		in2:      1,
		expected: "1",
	}, {
		name:     "secp256k1 prime-1 * 2",
		in1:      "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e",
		in2:      2,
		expected: "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2d",
	}, {
		name:     "secp256k1 prime * 3",
		in1:      "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f",
		in2:      3,
		expected: "0",
	}, {
		name:     "secp256k1 prime-1 * 8",
		in1:      "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e",
		in2:      8,
		expected: "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc27",
	}, {
		// Random samples for first value.  The second value is limited
		// to 8 since that is the maximum int used in the elliptic curve
		// calculations.
		name:     "random sampling #1",
		in1:      "b75674dc9180d306c692163ac5e089f7cef166af99645c0c23568ab6d967288a",
		in2:      6,
		expected: "4c06bd2b6904f228a76c8560a3433bced9a8681d985a2848d407404d186b0280",
	}, {
		name:     "random sampling #2",
		in1:      "54873298ac2b5ba8591c125ae54931f5ea72040aee07b208d6135476fb5b9c0e",
		in2:      3,
		expected: "fd9597ca048212f90b543710afdb95e1bf560c20ca17161a8239fd64f212d42a",
	}, {
		name:     "random sampling #3",
		in1:      "7c30fbd363a74c17e1198f56b090b59bbb6c8755a74927a6cba7a54843506401",
		in2:      5,
		expected: "6cf4eb20f2447c77657fccb172d38c0aa91ea4ac446dc641fa463a6b5091fba7",
	}, {
		name:     "random sampling #3",
		in1:      "fb4529be3e027a3d1587d8a500b72f2d312e3577340ef5175f96d113be4c2ceb",
		in2:      8,
		expected: "da294df1f013d1e8ac3ec52805b979698971abb9a077a8bafcb688a4f261820f",
	}}

	for _, test := range tests {
		f := setHex(test.in1).Normalize()
		expected := setHex(test.expected).Normalize()
		result := f.MulInt(test.in2).Normalize()
		if !result.Equals(expected) {
			t.Errorf("%s: wrong result -- got: %v -- want: %v", test.name,
				result, expected)
			continue
		}
	}
}

// TestFieldMul ensures that multiplying two field values via Mul and Mul2 works
// as expected.
func TestFieldMul(t *testing.T) {
	tests := []struct {
		name     string // test description
		in1      string // first hex encoded value
		in2      string // second hex encoded value to multiply with
		expected string // expected hex encoded value
	}{{
		name:     "zero * zero",
		in1:      "0",
		in2:      "0",
		expected: "0",
	}, {
		name:     "one * zero",
		in1:      "1",
		in2:      "0",
		expected: "0",
	}, {
		name:     "zero * one",
		in1:      "0",
		in2:      "1",
		expected: "0",
	}, {
		name:     "one * one",
		in1:      "1",
		in2:      "1",
		expected: "1",
	}, {
		name:     "slightly over prime",
		in1:      "ffffffffffffffffffffffffffffffffffffffffffffffffffffffff1ffff",
		in2:      "1000",
		expected: "1ffff3d1",
	}, {
		name:     "secp256k1 prime-1 * 2",
		in1:      "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e",
		in2:      "2",
		expected: "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2d",
	}, {
		name:     "secp256k1 prime * 3",
		in1:      "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f",
		in2:      "3",
		expected: "0",
	}, {
		name:     "secp256k1 prime * 3",
		in1:      "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e",
		in2:      "8",
		expected: "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc27",
	}, {
		name:     "random sampling #1",
		in1:      "cfb81753d5ef499a98ecc04c62cb7768c2e4f1740032946db1c12e405248137e",
		in2:      "58f355ad27b4d75fb7db0442452e732c436c1f7c5a7c4e214fa9cc031426a7d3",
		expected: "1018cd2d7c2535235b71e18db9cd98027386328d2fa6a14b36ec663c4c87282b",
	}, {
		name:     "random sampling #2",
		in1:      "26e9d61d1cdf3920e9928e85fa3df3e7556ef9ab1d14ec56d8b4fc8ed37235bf",
		in2:      "2dfc4bbe537afee979c644f8c97b31e58be5296d6dbc460091eae630c98511cf",
		expected: "da85f48da2dc371e223a1ae63bd30b7e7ee45ae9b189ac43ff357e9ef8cf107a",
	}, {
		name:     "random sampling #3",
		in1:      "5db64ed5afb71646c8b231585d5b2bf7e628590154e0854c4c29920b999ff351",
		in2:      "279cfae5eea5d09ade8e6a7409182f9de40981bc31c84c3d3dfe1d933f152e9a",
		expected: "2c78fbae91792dd0b157abe3054920049b1879a7cc9d98cfda927d83be411b37",
	}, {
		name:     "random sampling #4",
		in1:      "b66dfc1f96820b07d2bdbd559c19319a3a73c97ceb7b3d662f4fe75ecb6819e6",
		in2:      "bf774aba43e3e49eb63a6e18037d1118152568f1a3ac4ec8b89aeb6ff8008ae1",
		expected: "c4f016558ca8e950c21c3f7fc15f640293a979c7b01754ee7f8b3340d4902ebb",
	}}

	for _, test := range tests {
		f1 := setHex(test.in1).Normalize()
		f2 := setHex(test.in2).Normalize()
		expected := setHex(test.expected).Normalize()

		// Ensure multiplying the two values with the result going to another
		// variable produces the expected result.
		result := new(FieldVal).Mul2(f1, f2).Normalize()
		if !result.Equals(expected) {
			t.Errorf("%s: unexpected result\ngot: %v\nwant: %v", test.name,
				result, expected)
			continue
		}

		// Ensure multiplying the value to an existing field value produces the
		// expected result.
		f1.Mul(f2).Normalize()
		if !f1.Equals(expected) {
			t.Errorf("%s: unexpected result\ngot: %v\nwant: %v", test.name,
				f1, expected)
			continue
		}
	}
}

// TestFieldSquare ensures that squaring field values via Square and SqualVal
// works as expected.
func TestFieldSquare(t *testing.T) {
	tests := []struct {
		name     string // test description
		in       string // hex encoded value
		expected string // expected hex encoded value
	}{{
		name:     "zero",
		in:       "0",
		expected: "0",
	}, {
		name:     "secp256k1 prime (direct val in with 0 out)",
		in:       "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f",
		expected: "0",
	}, {
		name:     "secp256k1 prime (0 in with direct val out)",
		in:       "0",
		expected: "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f",
	}, {
		name:     "secp256k1 prime - 1",
		in:       "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e",
		expected: "1",
	}, {
		name:     "secp256k1 prime - 2",
		in:       "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2d",
		expected: "4",
	}, {
		name:     "random sampling #1",
		in:       "b0ba920360ea8436a216128047aab9766d8faf468895eb5090fc8241ec758896",
		expected: "133896b0b69fda8ce9f648b9a3af38f345290c9eea3cbd35bafcadf7c34653d3",
	}, {
		name:     "random sampling #2",
		in:       "c55d0d730b1d0285a1599995938b042a756e6e8857d390165ffab480af61cbd5",
		expected: "cd81758b3f5877cbe7e5b0a10cebfa73bcbf0957ca6453e63ee8954ab7780bee",
	}, {
		name:     "random sampling #3",
		in:       "e89c1f9a70d93651a1ba4bca5b78658f00de65a66014a25544d3365b0ab82324",
		expected: "39ffc7a43e5dbef78fd5d0354fb82c6d34f5a08735e34df29da14665b43aa1f",
	}, {
		name:     "random sampling #4",
		in:       "7dc26186079d22bcbe1614aa20ae627e62d72f9be7ad1e99cac0feb438956f05",
		expected: "bf86bcfc4edb3d81f916853adfda80c07c57745b008b60f560b1912f95bce8ae",
	}}

	for _, test := range tests {
		f := setHex(test.in).Normalize()
		expected := setHex(test.expected).Normalize()

		// Ensure squaring the value with the result going to another variable
		// produces the expected result.
		result := new(FieldVal).SquareVal(f).Normalize()
		if !result.Equals(expected) {
			t.Errorf("%s: unexpected result\ngot: %v\nwant: %v", test.name,
				result, expected)
			continue
		}

		// Ensure self squaring an existing field value produces the expected
		// result.
		f.Square().Normalize()
		if !f.Equals(expected) {
			t.Errorf("%s: unexpected result\ngot: %v\nwant: %v", test.name,
				f, expected)
			continue
		}
	}
}

// TestInverse ensures that finding the multiplicative inverse via Inverse works
// as expected.
func TestInverse(t *testing.T) {
	tests := []struct {
		in       string // hex encoded value
		expected string // expected hex encoded value
	}{
		// secp256k1 prime (aka 0)
		{"0", "0"},
		{"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f", "0"},
		{"0", "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f"},
		// secp256k1 prime-1
		{
			"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e",
			"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e",
		},
		// secp256k1 prime-2
		{
			"fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2d",
			"7fffffffffffffffffffffffffffffffffffffffffffffffffffffff7ffffe17",
		},
		// Random sampling
		{
			"16fb970147a9acc73654d4be233cc48b875ce20a2122d24f073d29bd28805aca",
			"987aeb257b063df0c6d1334051c47092b6d8766c4bf10c463786d93f5bc54354",
		},
		{
			"69d1323ce9f1f7b3bd3c7320b0d6311408e30281e273e39a0d8c7ee1c8257919",
			"49340981fa9b8d3dad72de470b34f547ed9179c3953797d0943af67806f4bb6",
		},
		{
			"e0debf988ae098ecda07d0b57713e97c6d213db19753e8c95aa12a2fc1cc5272",
			"64f58077b68af5b656b413ea366863f7b2819f8d27375d9c4d9804135ca220c2",
		},
		{
			"dcd394f91f74c2ba16aad74a22bb0ed47fe857774b8f2d6c09e28bfb14642878",
			"fb848ec64d0be572a63c38fe83df5e7f3d032f60bf8c969ef67d36bf4ada22a9",
		},
	}

	t.Logf("Running %d tests", len(tests))
	for i, test := range tests {
		f := setHex(test.in).Normalize()
		expected := setHex(test.expected).Normalize()
		result := f.Inverse().Normalize()
		if !result.Equals(expected) {
			t.Errorf("FieldVal.Inverse #%d wrong result\n"+
				"got: %v\nwant: %v", i, result, expected)
			continue
		}
	}
}

// randFieldVal returns a field value created from a random value generated by
// the passed rng.
func randFieldVal(t *testing.T, rng *rand.Rand) *FieldVal {
	t.Helper()

	var buf [32]byte
	if _, err := rng.Read(buf[:]); err != nil {
		t.Fatalf("failed to read random: %v", err)
	}

	// Create and return both a big integer and a field value.
	var fv FieldVal
	fv.SetBytes(&buf)
	return &fv
}

// TestFieldSquareRoot ensures that calculating the square root of field values
// via SquareRootVal works as expected for edge cases.
func TestFieldSquareRoot(t *testing.T) {
	tests := []struct {
		name  string // test description
		in    string // hex encoded value
		valid bool   // whether or not the value has a square root
		want  string // expected hex encoded value
	}{{
		name:  "secp256k1 prime (as 0 in and out)",
		in:    "0",
		valid: true,
		want:  "0",
	}, {
		name:  "secp256k1 prime (direct val with 0 out)",
		in:    "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f",
		valid: true,
		want:  "0",
	}, {
		name:  "secp256k1 prime (as 0 in direct val out)",
		in:    "0",
		valid: true,
		want:  "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f",
	}, {
		name:  "secp256k1 prime-1",
		in:    "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e",
		valid: false,
		want:  "0000000000000000000000000000000000000000000000000000000000000001",
	}, {
		name:  "secp256k1 prime-2",
		in:    "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2d",
		valid: false,
		want:  "210c790573632359b1edb4302c117d8a132654692c3feeb7de3a86ac3f3b53f7",
	}, {
		name:  "(secp256k1 prime-2)^2",
		in:    "0000000000000000000000000000000000000000000000000000000000000004",
		valid: true,
		want:  "0000000000000000000000000000000000000000000000000000000000000002",
	}, {
		name:  "value 1",
		in:    "0000000000000000000000000000000000000000000000000000000000000001",
		valid: true,
		want:  "0000000000000000000000000000000000000000000000000000000000000001",
	}, {
		name:  "value 2",
		in:    "0000000000000000000000000000000000000000000000000000000000000002",
		valid: true,
		want:  "210c790573632359b1edb4302c117d8a132654692c3feeb7de3a86ac3f3b53f7",
	}, {
		name:  "random sampling 1",
		in:    "16fb970147a9acc73654d4be233cc48b875ce20a2122d24f073d29bd28805aca",
		valid: false,
		want:  "6a27dcfca440cf7930a967be533b9620e397f122787c53958aaa7da7ad3d89a4",
	}, {
		name:  "square of random sampling 1",
		in:    "f4a8c3738ace0a1c3abf77737ae737f07687b5e24c07a643398298bd96893a18",
		valid: true,
		want:  "e90468feb8565338c9ab2b41dcc33b7478a31df5dedd2db0f8c2d641d77fa165",
	}, {
		name:  "random sampling 2",
		in:    "69d1323ce9f1f7b3bd3c7320b0d6311408e30281e273e39a0d8c7ee1c8257919",
		valid: true,
		want:  "61f4a7348274a52d75dfe176b8e3aaff61c1c833b6678260ba73def0fb2ad148",
	}, {
		name:  "random sampling 3",
		in:    "e0debf988ae098ecda07d0b57713e97c6d213db19753e8c95aa12a2fc1cc5272",
		valid: false,
		want:  "6e1cc9c311d33d901670135244f994b1ea39501f38002269b34ce231750cfbac",
	}, {
		name:  "random sampling 4",
		in:    "dcd394f91f74c2ba16aad74a22bb0ed47fe857774b8f2d6c09e28bfb14642878",
		valid: true,
		want:  "72b22fe6f173f8bcb21898806142ed4c05428601256eafce5d36c1b08fb82bab",
	}}

	for _, test := range tests {
		input := setHex(test.in).Normalize()
		want := setHex(test.want).Normalize()

		// Calculate the square root and ensure the validity flag matches the
		// expected value.
		var result FieldVal
		isValid := result.SquareRootVal(input)
		if isValid != test.valid {
			t.Errorf("%s: mismatched validity -- got %v, want %v", test.name,
				isValid, test.valid)
			continue
		}

		// Ensure the calculated result matches the expected value.
		result.Normalize()
		if !result.Equals(want) {
			t.Errorf("%s: d wrong result\ngot: %v\nwant: %v", test.name, result,
				want)
			continue
		}
	}
}

// hexToBytes converts the passed hex string into bytes and will panic if there
// is an error.  This is only provided for the hard-coded constants so errors in
// the source code can be detected. It will only (and must only) be called with
// hard-coded values.
func hexToBytes(s string) []byte {
	b, err := hex.DecodeString(s)
	if err != nil {
		panic("invalid hex in source file: " + s)
	}
	return b
}
