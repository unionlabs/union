// uint256: Fixed size 256-bit math library
// Copyright 2020 uint256 Authors
// SPDX-License-Identifier: BSD-3-Clause

package uint256

import (
	"errors"
	"fmt"
	"math/big"
	"testing"
)

// Test SetFromDecimal
func testSetFromDec(tc string) error {
	a := new(Int).SetAllOne()
	err := a.SetFromDecimal(tc)
	{ // Check the FromDecimal too
		b, err2 := FromDecimal(tc)
		if (err == nil) != (err2 == nil) {
			return fmt.Errorf("err != err2: %v %v", err, err2)
		}
		// Test the MustFromDecimal too
		if err != nil {
			if !causesPanic(func() { MustFromDecimal(tc) }) {
				return errors.New("expected panic")
			}
		} else {
			MustFromDecimal(tc) // must not manic
		}
		if err == nil {
			if a.Cmp(b) != 0 {
				return fmt.Errorf("a != b: %v %v", a, b)
			}
		}
	}
	// If input is negative, we should eror
	if len(tc) > 0 && tc[0] == '-' {
		if err == nil {
			return fmt.Errorf("want error on negative input")
		}
		return nil
	}
	// Need to compare with big.Int
	bigA, ok := big.NewInt(0).SetString(tc, 10)
	if !ok {
		if err == nil {
			return fmt.Errorf("want error")
		}
		return nil // both agree that input is bad
	}
	if bigA.BitLen() > 256 {
		if err == nil {
			return fmt.Errorf("want error (bitlen > 256)")
		}
		return nil
	}
	want := bigA.String()
	have := a.Dec()
	if want != have {
		return fmt.Errorf("want %v, have %v", want, have)
	}
	return nil
}

var cases = []string{
	"0000000000000000000000000000000000000000000000000000000000000000000000000000000",
	"0000000000000000000000000000000000000000000000000000000000000000000000000000097",
	"-000000000000000000000000000000000000000000000000000000000000000000000000000000",
	"1157920892373161954235709850086879078532699846656405640394575840079131296399351",
	"215792089237316195423570985008687907853269984665640564039457584007913129639935",
	"115792089237316195423570985008687907853269984665640564039457584007913129639935",
	"15792089237316195423570985008687907853269984665640564039457584007913129639935",
	"+115792089237316195423570985008687907853269984665640564039457584007913129639935",
	"115792089237316195423570985008687907853269984665640564039457584007913129639936",
	"115792089237316195423570985008687907853269984665640564039457584007913129639935",
	"+0b00000000000000000000000000000000000000000000000000000000000000010",
	"340282366920938463463374607431768211456",
	"3402823669209384634633746074317682114561",
	"+3402823669209384634633746074317682114561",
	"+-3402823669209384634633746074317682114561",
	"40282366920938463463374607431768211456",
	"00000000000000000000000097",
	"184467440737095516161",
	"8446744073709551616",
	"banana",
	"+0x10",
	"000",
	"+000",
	"010",
	"0xab",
	"-10",
	"01",
	"ab",
	"0",
	"-0",
	"+0",
	"",
	"熊熊熊熊熊熊熊熊",
	"0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
	"-0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
	"0x10101011010",
	"熊熊熊熊熊熊熊熊",
	"0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
	"-0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
	"-0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
	"0x10000000000000000000000000000000000000000000000000000000000000000",
	"+0x10000000000000000000000000000000000000000000000000000000000000000",
	"+0x00000000000000000000000000000000000000000000000000000000000000000",
	"-0x00000000000000000000000000000000000000000000000000000000000000000",
}

func TestStringScan(t *testing.T) {
	for i, tc := range cases {
		if err := testSetFromDec(tc); err != nil {
			t.Errorf("test %d, input '%s', SetFromDecimal err: %v", i, tc, err)
		}
		// TODO testSetFromHex(tc)
	}
}

func FuzzBase10StringCompare(f *testing.F) {
	for _, tc := range cases {
		f.Add(tc)
	}
	f.Fuzz(func(t *testing.T, tc string) {
		if err := testSetFromDec(tc); err != nil {
			t.Errorf("input '%s', SetFromDecimal err: %v", tc, err)
		}
		// TODO testSetFromHex(tc)
	})
}

func BenchmarkFromDecimalString(b *testing.B) {
	input := twoPow256Sub1

	b.Run("big", func(b *testing.B) {
		val := new(big.Int)
		b.ReportAllocs()
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			for j := 1; j < len(input); j++ {
				if _, ok := val.SetString(input[:j], 10); !ok {
					b.Fatalf("Error on %v", string(input[:j]))
				}
			}
		}
	})

	b.Run("uint256", func(b *testing.B) {
		val := new(Int)
		b.ReportAllocs()
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			for j := 1; j < len(input); j++ {
				if err := val.SetFromDecimal(input[:j]); err != nil {
					b.Fatalf("%v: %v", err, string(input[:j]))
				}
			}
		}
	})
}
