// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

import (
	"fmt"
	"regexp"
	"strings"
	"unicode"
)

var (
	parsePSepRE = regexp.MustCompile(`\s*[+]\s*`)
	parseScanRE = regexp.MustCompile(
		`^\s*(0([box]))?([-_:0-9a-fA-F]+)( ?[(](pad=|!)([0-3])[)])?\s*(.*)$`,
	)
	parseBaseExpr = map[byte]byte{'b': 2, 'o': 8, 'x': 16}
	parseDigits   = map[rune]byte{
		'0': 0, '1': 1, '2': 2, '3': 3, '4': 4,
		'5': 5, '6': 6, '7': 7, '8': 8, '9': 9,
		'a': 0xa, 'b': 0xb, 'c': 0xc, 'd': 0xd, 'e': 0xe, 'f': 0xf,
		'A': 0xa, 'B': 0xb, 'C': 0xc, 'D': 0xd, 'E': 0xe, 'F': 0xf,
	}
)

// MustParse is like Parse but panics if the expression can not be parsed. It
// simplifies safe initialization of global variables holding bit arrays.
func MustParse(s string) *BitArray {
	ba, err := Parse(s)
	if err != nil {
		panicf("MustParse(%q): %d", s, err)
	}
	return ba
}

// Parse parses a string as a bit array representation, like "01010".
//
// Multiple tokens can be presented, which are separated by one or more spaces
// and/or a single "+" sign. All bits contained in tokens will be simply
// concatenated. Each token can be binary, octal, or hexademical, and the type
// is specified by the prefixes "0b", "0o" or "0x". Tokens without a prefix are
// always parsed as binary representation. Each token also can contain any
// number of separators "-", "_", and ":". These separators are safely ignored.
//
// Note that spaces between digits are parsed as token delimiters, not
// separators within tokens. This is not a problem for binary representations,
// but "0o" and "0x" prefixes have no effect beyond the spaces. For example,
// "0b0000 1111" is legal, but "0x0000 ffff" is illegal. Because the "ffff" is
// interpreted as a second token without a prefix, so "f" cannot be parsed as
// binary representation. Use other separators instead: e.g. "0x0000_ffff".
//
//     bitarray  = *WSP [ token *( token-sep token ) ] *WSP
//     token-sep = *WSP ( WSP / "+" ) *WSP
//     token     = bin-token / oct-token / hex-token
//     bin-token = [ "0b" [ char-sep ] ] bin-char *( [ char-sep ] bin-char )
//     oct-token = "0o" 1*( [ char-sep ] oct-char ) [ oct-pad ]
//     hex-token = "0x" 1*( [ char-sep ] hex-char ) [ hex-pad ]
//     char-sep  = "-" / "_" / ":"
//     bin-char  = "0" / "1"
//     oct-char  = bin-char / "2" / "3" / "4" / "5" / "6" / "7"
//     hex-char  = oct-char / "8" / "9"
//               / "a" / "b" / "c" / "d" / "e" / "f"
//               / "A" / "B" / "C" / "D" / "E" / "F"
//     oct-pad   = [ " " ] "(" pad-ind ( "0" / "1" / "2" )       ")"
//     hex-pad   = [ " " ] "(" pad-ind ( "0" / "1" / "2" / "3" ) ")"
//     pad-ind   = "pad=" / "!"
func Parse(s string) (*BitArray, error) {
	s = strings.Map(parseMapSpaces, s)
	zf := true
	bb := NewBuilder()
	lines := parsePSepRE.Split(s, -1)
	for _, line := range lines {
		line = strings.TrimSpace(line)
		if len(line) == 0 && 1 < len(lines) {
			return nil, fmt.Errorf("%q: %w: empty token", s, ErrIllegalExpression)
		}
		for 0 < len(line) {
			m := parseScanRE.FindStringSubmatch(line)
			if len(m) != parseScanRE.NumSubexp()+1 {
				return nil, fmt.Errorf("%q: %w: malformed input", line, ErrIllegalExpression)
			}
			tzf, err := parseToken(bb, m[2], m[3], m[6])
			if err != nil {
				return nil, fmt.Errorf("%q: malformed token: %w", m[0], err)
			}
			zf = zf && tzf
			line = m[7]
		}
	}
	if zf {
		return &BitArray{nBits: bb.nBits}, nil
	}

	return bb.BitArray(), nil
}

func parseMapSpaces(r rune) rune {
	if unicode.IsSpace(r) {
		return ' '
	}
	return r
}

func parseToken(bb *Builder, baseStr, bodyStr, npadStr string) (bool, error) {
	base := byte(2)
	if len(baseStr) != 0 {
		b, ok := parseBaseExpr[baseStr[0]]
		if !ok {
			return false, fmt.Errorf(`%w: base %q`, ErrIllegalExpression, baseStr)
		}
		base = b
	}

	// digits
	var zfb byte
	digits := make([]byte, 0, len(bodyStr))
	allowSep := baseStr != ""
	var lastSep rune
	for _, r := range bodyStr {
		if dv, ok := parseDigits[r]; ok {
			if base <= dv {
				return false, fmt.Errorf(`%w: digit '%x' for base %d`, ErrIllegalExpression, dv, base)
			}
			zfb |= dv
			digits = append(digits, dv)
			allowSep = true
			continue
		}
		if strings.ContainsRune("-_:", r) {
			if !allowSep {
				return false, fmt.Errorf(`%w: separator '%c'`, ErrIllegalExpression, r)
			}
			allowSep = false
			lastSep = r
			continue
		}
		return false, fmt.Errorf("%w: unexpected '%c'", ErrIllegalExpression, r)
	}
	if !allowSep {
		return false, fmt.Errorf(`%w: token ends with a separator '%c'`, ErrIllegalExpression, lastSep)
	}

	// padding
	npad := 0
	if npadStr != "" {
		npad = int(npadStr[0]) - int('0')
	}
	switch base {
	case 2:
		if npadStr != "" {
			return false, fmt.Errorf("%w: pad=%s for bin token", ErrIllegalExpression, npadStr)
		}
		bb.WriteByteBits(digits)
	case 8:
		if 2 < npad {
			return false, fmt.Errorf("%w: pad=%s for oct token", ErrIllegalExpression, npadStr)
		}
		for i, digit := range digits {
			switch {
			case i+1 < len(digits) || npad == 0:
				bb.WriteByteBits([]byte{
					digit >> 2,
					digit >> 1,
					digit,
				})
			case npad == 1:
				bb.WriteByteBits([]byte{
					digit >> 2,
					digit >> 1,
				})
			case npad == 2:
				bb.WriteByteBits([]byte{
					digit >> 2,
				})
			}
		}
	case 16:
		if 3 < npad { // this case should have been eliminated by regex
			return false, fmt.Errorf(
				"%w: pad=%s for hex token",
				ErrIllegalExpression, npadStr,
			)
		}
		for i, digit := range digits {
			switch {
			case i+1 < len(digits) || npad == 0:
				bb.WriteByteBits([]byte{
					digit >> 3,
					digit >> 2,
					digit >> 1,
					digit,
				})
			case npad == 1:
				bb.WriteByteBits([]byte{
					digit >> 3,
					digit >> 2,
					digit >> 1,
				})
			case npad == 2:
				bb.WriteByteBits([]byte{
					digit >> 3,
					digit >> 2,
				})
			case npad == 3:
				bb.WriteByteBits([]byte{
					digit >> 3,
				})
			}
		}
	default:
		// this should never happen
		return false, fmt.Errorf(
			"%w: base %d(%s)",
			ErrIllegalExpression, base, baseStr,
		)
	}

	return zfb == 0, nil
}
