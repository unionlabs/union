// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"math/rand"
	"testing"

	"github.com/tunabay/go-bitarray"
)

func TestParse(t *testing.T) {
	tdt := []string{
		"   ", "",
		"0", "0",
		"1", "1",
		"1+1", "11",
		"0x_f0:f0:37:37", "11110000111100000011011100110111",
		"0110110", "0110110",
		" 1010-1111 \t 0011-1100 \r\n", "1010111100111100",
		" 0xf  \n +01", "111101",
		" 000 + 0o0753 + 111 ", "000000111101011111",

		"10101010\n0b_1111_1111  \n  \n  0101010 + 111111111   \t\n+01010\t\n",
		"1010101011111111010101011111111101010",

		// 84 bits
		"0o_7777_7777_7777_7777_7777_7777_7777",
		"111111111111111111111111111111111111111111111111111111111111111111111111111111111111",

		// 128 bits
		"0x_0000_0000_0000_0000_0000_0000_0000_0000",
		"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
		"0x_ffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff",
		"11111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111",

		// 96 bits
		"0o_0000_0000_0000_0000_0000_0000_0000_0000",
		"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",

		// 132 bits
		"0x_0000_0000_0000_0000_0000_0000_0000_0000_0",
		"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
	}
	for i := 0; i < len(tdt); i += 2 {
		ba, err := bitarray.Parse(tdt[i])
		if err != nil {
			t.Errorf("%q: unexpected error: %s", tdt[i], err)
			continue
		}
		ba.V()
		if ba.String() != tdt[i+1] {
			t.Errorf("%q: unexpected result.", tdt[i])
			t.Logf("  got: %#b", ba)
			t.Logf("  got: %s", ba.D())
			t.Logf(" want: %q", tdt[i+1])
		}
	}
}

func TestParse_rand(t *testing.T) {
	tokens := []struct {
		s, d string
		e    bool
	}{
		{"0", "-", false},
		{"10", "-", false},
		{"01", "-", false},
		{"101", "-", false},
		{"1010", "-", false},
		{"10101", "-", false},
		{"01010", "-", false},
		{"101010", "-", false},
		{"0101010", "-", false},
		{"1010101", "-", false},
		{"1111111", "-", false},
		{"10101010", "-", false},
		{"01010101", "-", false},
		{"000000000", "-", false},
		{"111111111", "-", false},
		{"0000-0000 0000-0000 0000-0000", "000000000000000000000000", false},
		{"0000-0000 0000-0000 0000-0000 0", "0000000000000000000000000", false},
		{"1111-1111 1111-1111 1111-1111", "111111111111111111111111", false},
		{"1111-1111 1111-1111 1111-1111 1", "1111111111111111111111111", false},
		{"10_10", "1010", false},
		{"0b_1111_1111", "11111111", false},
		{"0b_1111_0101_01", "1111010101", false},
		{"0x5", "0101", false},
		{"0xF77", "111101110111", false},
		{"0o_0755", "000111101101", false},
		{"0o00", "000000", false},
		{"0o1234", "001010011100", false},
		{"0o567", "101110111", false},
		{"0xbeef", "1011111011101111", false},
		{"0xDEAD", "1101111010101101", false},
		{"0x333333333", "001100110011001100110011001100110011", false},
		{"0x:ff:33:88", "111111110011001110001000", false},
		{"0x000", "000000000000", false},
		{"0x_123", "000100100011", false},
		{"0x_45678", "01000101011001111000", false},
		{"0x_9a-bc:de-f", "1001101010111100110111101111", false},
		{"0x98", "10011000", false},
		{"0xa8 (pad=1)", "1010100", false},
		{"0xb8 (pad=2)", "101110", false},
		{"0xc8 (pad=3)", "11001", false},
		{"0o54", "101100", false},
		{"0o64 (pad=1)", "11010", false},
		{"0o74 (pad=2)", "1111", false},
	}
	seps := []string{
		"+", " ", "\n", "\r\n", " + ", "\t", "\t+\t", "   \t\n+",
		"+\n", "\n\n   ", "\n+\n", "  \n  \n  ", "\n\n\n",
	}
	sps := []string{
		"", "     ", "\r\n", "  \n\t\n  \n", "\t", "\r\n\r\n",
		"  \n", "\t\n", "\t\n",
	}
	for i := 0; i < 50000; i++ {
		var src, want string
		iserr := false
		ntokens := rand.Intn(20)
		for i := 0; i < ntokens; i++ {
			if i != 0 {
				src += seps[rand.Intn(len(seps))]
			}
			tok := tokens[rand.Intn(len(tokens))]
			src += tok.s
			exp := tok.d
			if exp == "-" {
				exp = tok.s
			}
			want += exp
			iserr = iserr || tok.e
		}
		src = sps[rand.Intn(len(sps))] + src + sps[rand.Intn(len(sps))]
		if ba, err := bitarray.Parse(src); err != nil {
			if !iserr {
				t.Errorf("%q: unexpected error.", src)
				t.Logf("  err: %v", err)
			} else {
				// t.Logf(" pass: %v", err)
			}
		} else {
			if iserr {
				t.Errorf("%q: error expected but no error.", src)
				t.Logf("  got: %q", ba)
			} else {
				s := ba.String()
				if s != want {
					t.Errorf("%q: unexpected result.", src)
					t.Logf("  got: %q", s)
					t.Logf(" want: %q", want)
				} else {
					// t.Logf(" pass: %q", s)
				}
			}
		}
	}
}

func TestParse_error(t *testing.T) {
	tcs := []string{
		"invalid data",
		"0bx00",
		"0x0101x",
		"0x_",
		"0x",
		"0q0",             // invalid base specifier
		"0b2",             // invalid bin
		"0o8",             // invalid oct
		"0xX",             // invalid hex
		"0b1111(pad=1)",   // invalid pad
		"0xffff(pad=4)",   // invalid pad
		"0xfffff(pad=-1)", // invalid pad
		"0o777(pad=3)",    // invalid pad
		"0o7777(pad=-1)",  // invalid pad
		"0b__100",         // invalid separator
		"0000--1111",      // invalid separator
		"-00001111",       // invalid separator
		"00001111-",       // invalid separator
		"0x::0",           // invalid separator
		"0000 ++ 0000",
		"+ 0000",
		"0000 + ",
	}
	for _, tc := range tcs {
		ba, err := bitarray.Parse(tc)
		if err == nil {
			t.Error("error expected:")
			t.Logf("data: %q", tc)
			t.Logf(" got: %#b", ba)
			t.Logf(" got: %s", ba.D())
			// continue
		}
		// t.Logf("pass: %s", err)
	}
}

func TestMustParse_panic(t *testing.T) {
	func() {
		var ba *bitarray.BitArray
		defer func() {
			if recover() == nil {
				t.Errorf("panic expected: got %#b", ba)
			}
		}()
		ba = bitarray.MustParse("invalid input")
	}()
}
