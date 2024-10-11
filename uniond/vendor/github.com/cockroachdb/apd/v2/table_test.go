// Copyright 2016 The Cockroach Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License.

package apd

import (
	"bytes"
	"math/big"
	"math/rand"
	"strings"
	"testing"
)

func BenchmarkNumDigitsLookup(b *testing.B) {
	b.StopTimer()
	runTest := func(start string, c byte) {
		buf := bytes.NewBufferString(start)
		for i := 1; i < digitsTableSize; i++ {
			buf.WriteByte(c)
			d, _, _ := NewFromString(buf.String())

			b.StartTimer()
			d.NumDigits()
			b.StopTimer()
		}
	}
	for i := 0; i < b.N; i++ {
		runTest("", '9')
		runTest("1", '0')
		runTest("-", '9')
		runTest("-1", '0')
	}
}

func BenchmarkNumDigitsFull(b *testing.B) {
	b.StopTimer()
	runTest := func(start string, c byte) {
		buf := bytes.NewBufferString(start)
		for i := 1; i < 1000; i++ {
			buf.WriteByte(c)
			d, _, _ := NewFromString(buf.String())

			b.StartTimer()
			d.NumDigits()
			b.StopTimer()
		}
	}
	for i := 0; i < b.N; i++ {
		runTest("", '9')
		runTest("1", '0')
		runTest("-", '9')
		runTest("-1", '0')
	}
}

func TestNumDigits(t *testing.T) {
	runTest := func(start string, c byte) {
		buf := bytes.NewBufferString(start)
		var offset int
		if strings.HasPrefix(start, "-") {
			offset--
		}
		for i := 1; i < 1000; i++ {
			buf.WriteByte(c)
			bs := buf.String()
			t.Run(bs, func(t *testing.T) {
				d := newDecimal(t, testCtx, bs)
				n := d.NumDigits()
				e := int64(buf.Len() + offset)
				if n != e {
					t.Fatalf("%s ('%c'): expected %d, got %d", bs, c, e, n)
				}
			})
		}
	}
	runTest("", '9')
	runTest("1", '0')
	runTest("-", '9')
	runTest("-1", '0')
}

func TestDigitsLookupTable(t *testing.T) {
	// Make sure all elements in table make sense.
	min := new(big.Int)
	prevBorder := big.NewInt(0)
	for i := 1; i <= digitsTableSize; i++ {
		elem := digitsLookupTable[i]

		min.SetInt64(2)
		min.Exp(min, big.NewInt(int64(i-1)), nil)
		if minLen := int64(len(min.String())); minLen != elem.digits {
			t.Errorf("expected 2^%d to have %d digits, found %d", i, elem.digits, minLen)
		}

		if zeros := int64(strings.Count(elem.border.String(), "0")); zeros != elem.digits {
			t.Errorf("the %d digits for digitsLookupTable[%d] does not agree with the border %v", elem.digits, i, &elem.border)
		}

		if min.Cmp(&elem.border) >= 0 {
			t.Errorf("expected 2^%d = %v to be less than the border, found %v", i-1, min, &elem.border)
		}

		if elem.border.Cmp(prevBorder) > 0 {
			if min.Cmp(prevBorder) <= 0 {
				t.Errorf("expected 2^%d = %v to be greater than or equal to the border, found %v", i-1, min, prevBorder)
			}
			prevBorder = &elem.border
		}
	}

	// Throw random big.Ints at the table and make sure the
	// digit lengths line up.
	const randomTrials = 100
	for i := 0; i < randomTrials; i++ {
		a := big.NewInt(rand.Int63())
		b := big.NewInt(rand.Int63())
		a.Mul(a, b)

		d := &Decimal{Coeff: *a}
		tableDigits := d.NumDigits()
		if actualDigits := int64(len(a.String())); actualDigits != tableDigits {
			t.Errorf("expected %d digits for %v, found %d", tableDigits, a, actualDigits)
		}
	}
}

func TestTableExp10(t *testing.T) {
	tests := []struct {
		pow int64
		str string
	}{
		{
			pow: 0,
			str: "1",
		},
		{
			pow: 1,
			str: "10",
		},
		{
			pow: 5,
			str: "100000",
		},
		{
			pow: powerTenTableSize + 1,
			str: "1000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
		},
	}

	for i, test := range tests {
		d := tableExp10(test.pow, nil)
		if s := d.String(); s != test.str {
			t.Errorf("%d: expected PowerOfTenDec(%d) to give %s, got %s", i, test.pow, test.str, s)
		}
	}
}
