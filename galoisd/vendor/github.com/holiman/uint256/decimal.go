// uint256: Fixed size 256-bit math library
// Copyright 2020 uint256 Authors
// SPDX-License-Identifier: BSD-3-Clause

package uint256

import (
	"io"
	"strconv"
)

const twoPow256Sub1 = "115792089237316195423570985008687907853269984665640564039457584007913129639935"

// Dec returns the decimal representation of z.
func (z *Int) Dec() string {
	if z.IsZero() {
		return "0"
	}
	if z.IsUint64() {
		return strconv.FormatUint(z.Uint64(), 10)
	}
	// The max uint64 value being 18446744073709551615, the largest
	// power-of-ten below that is 10000000000000000000.
	// When we do a DivMod using that number, the remainder that we
	// get back is the lower part of the output.
	//
	// The ascii-output of remainder will never exceed 19 bytes (since it will be
	// below 10000000000000000000).
	//
	// Algorithm example using 100 as divisor
	//
	// 12345 % 100 = 45   (rem)
	// 12345 / 100 = 123  (quo)
	// -> output '45', continue iterate on 123
	var (
		// out is 98 bytes long: 78 (max size of a string without leading zeroes,
		// plus slack so we can copy 19 bytes every iteration).
		// We init it with zeroes, because when strconv appends the ascii representations,
		// it will omit leading zeroes.
		out     = []byte("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000")
		divisor = NewInt(10000000000000000000) // 20 digits
		y       = new(Int).Set(z)              // copy to avoid modifying z
		pos     = len(out)                     // position to write to
		buf     = make([]byte, 0, 19)          // buffer to write uint64:s to
	)
	for {
		// Obtain Q and R for divisor
		var quot, rem Int
		udivrem(quot[:], y[:], divisor, &rem)
		y.Set(&quot) // Set Q for next loop
		// Convert the R to ascii representation
		buf = strconv.AppendUint(buf[:0], rem.Uint64(), 10)
		// Copy in the ascii digits
		copy(out[pos-len(buf):], buf)
		if y.IsZero() {
			break
		}
		// Move 19 digits left
		pos -= 19
	}
	// skip leading zeroes by only using the 'used size' of buf
	return string(out[pos-len(buf):])
}

// PrettyDec returns the decimal representation of z, with thousands-separators.
func (z *Int) PrettyDec(separator byte) string {
	if z.IsZero() {
		return "0"
	}
	// See algorithm-description in Dec()
	// This just also inserts comma while copying byte-for-byte instead
	// of using copy().
	var (
		out     = []byte("0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000")
		divisor = NewInt(10000000000000000000)
		y       = new(Int).Set(z)     // copy to avoid modifying z
		pos     = len(out) - 1        // position to write to
		buf     = make([]byte, 0, 19) // buffer to write uint64:s to
		comma   = 0
	)
	for {
		var quot, rem Int
		udivrem(quot[:], y[:], divisor, &rem)
		y.Set(&quot) // Set Q for next loop
		buf = strconv.AppendUint(buf[:0], rem.Uint64(), 10)
		for j := len(buf) - 1; j >= 0; j-- {
			if comma == 3 {
				out[pos] = separator
				pos--
				comma = 0
			}
			out[pos] = buf[j]
			comma++
			pos--
		}
		if y.IsZero() {
			break
		}
		// Need to do zero-padding if we have more iterations coming
		for j := 0; j < 19-len(buf); j++ {
			if comma == 3 {
				out[pos] = separator
				pos--
				comma = 0
			}
			comma++
			pos--
		}
	}
	return string(out[pos+1:])
}

// FromDecimal is a convenience-constructor to create an Int from a
// decimal (base 10) string. Numbers larger than 256 bits are not accepted.
func FromDecimal(decimal string) (*Int, error) {
	var z Int
	if err := z.SetFromDecimal(decimal); err != nil {
		return nil, err
	}
	return &z, nil
}

// MustFromDecimal is a convenience-constructor to create an Int from a
// decimal (base 10) string.
// Returns a new Int and panics if any error occurred.
func MustFromDecimal(decimal string) *Int {
	var z Int
	if err := z.SetFromDecimal(decimal); err != nil {
		panic(err)
	}
	return &z
}

// SetFromDecimal sets z from the given string, interpreted as a decimal number.
// OBS! This method is _not_ strictly identical to the (*big.Int).SetString(..., 10) method.
// Notable differences:
// - This method does not accept underscore input, e.g. "100_000",
// - This method does not accept negative zero as valid, e.g "-0",
//   - (this method does not accept any negative input as valid))
func (z *Int) SetFromDecimal(s string) (err error) {
	// Remove max one leading +
	if len(s) > 0 && s[0] == '+' {
		s = s[1:]
	}
	// Remove any number of leading zeroes
	if len(s) > 0 && s[0] == '0' {
		var i int
		var c rune
		for i, c = range s {
			if c != '0' {
				break
			}
		}
		s = s[i:]
	}
	if len(s) < len(twoPow256Sub1) {
		return z.fromDecimal(s)
	}
	if len(s) == len(twoPow256Sub1) {
		if s > twoPow256Sub1 {
			return ErrBig256Range
		}
		return z.fromDecimal(s)
	}
	return ErrBig256Range
}

// multipliers holds the values that are needed for fromDecimal
var multipliers = [5]*Int{
	nil,                             // represents first round, no multiplication needed
	{10000000000000000000, 0, 0, 0}, // 10 ^ 19
	{687399551400673280, 5421010862427522170, 0, 0},                     // 10 ^ 38
	{5332261958806667264, 17004971331911604867, 2938735877055718769, 0}, // 10 ^ 57
	{0, 8607968719199866880, 532749306367912313, 1593091911132452277},   // 10 ^ 76
}

// fromDecimal is a helper function to only ever be called via SetFromDecimal
// this function takes a string and chunks it up, calling ParseUint on it up to 5 times
// these chunks are then multiplied by the proper power of 10, then added together.
func (z *Int) fromDecimal(bs string) error {
	// first clear the input
	z.Clear()
	// the maximum value of uint64 is 18446744073709551615, which is 20 characters
	// one less means that a string of 19 9's is always within the uint64 limit
	var (
		num       uint64
		err       error
		remaining = len(bs)
	)
	if remaining == 0 {
		return io.EOF
	}
	// We proceed in steps of 19 characters (nibbles), from least significant to most significant.
	// This means that the first (up to) 19 characters do not need to be multiplied.
	// In the second iteration, our slice of 19 characters needs to be multipleied
	// by a factor of 10^19. Et cetera.
	for i, mult := range multipliers {
		if remaining <= 0 {
			return nil // Done
		} else if remaining > 19 {
			num, err = strconv.ParseUint(bs[remaining-19:remaining], 10, 64)
		} else {
			// Final round
			num, err = strconv.ParseUint(bs, 10, 64)
		}
		if err != nil {
			return err
		}
		// add that number to our running total
		if i == 0 {
			z.SetUint64(num)
		} else {
			base := NewInt(num)
			z.Add(z, base.Mul(base, mult))
		}
		// Chop off another 19 characters
		if remaining > 19 {
			bs = bs[0 : remaining-19]
		}
		remaining -= 19
	}
	return nil
}
