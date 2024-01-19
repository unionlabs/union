// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

import (
	"bytes"
	"fmt"
	"io"
)

// Format implements the fmt.Formatter interface to format BitArray values using
// the standard fmt.Printf family functions.
//
// Verbs:
// 	%b, %s  binary, 1 digit represents 1 bit.
// 	%q      quoted version of %b.
// 	%v      default format, same as %b.
// 	%o      octal, 1 digit represents 3 bits.
// 	%x, %X  hexadecimal, 1 digit represents 4 bits.
//
// Flags:
// 	' '     (space) print a separator space every 8 digits.
// 	#       more separators, ' ' and/or '-' every 4 digits.
// 	-       left-justify
// 	+       print the number of padding bits at the end, for %o, %x, %X
func (ba BitArray) Format(s fmt.State, verb rune) {
	switch verb {
	case 'b', 's', 'q':
		wid, widOK := s.Width()
		if !widOK {
			wid = 0
		}
		if s.Flag(int('0')) {
			fmt.Fprintf(s, "%%!%c(ERROR: unsupported flag 0)", verb)
			break
		}
		if err := ba.writeBinStr(
			s, wid, s.Flag(int(' ')), s.Flag(int('#')),
			verb == 'q', s.Flag(int('-')),
		); err != nil {
			fmt.Fprintf(s, "%%!%c(ERROR: %v)", verb, err)
			break
		}

	case 'v':
		if s.Flag(int('#')) {
			fmt.Fprintf(s, "%%!%c(ERROR: unsupported flag #)", verb)
			break
		}
		if err := ba.writeBinStr(s, 0, false, false, false, false); err != nil {
			fmt.Fprintf(s, "%%!%c(ERROR: %v)", verb, err)
			break
		}

	case 'o':
		wid, widOK := s.Width()
		if !widOK {
			wid = 0
		}
		if s.Flag(int('0')) {
			fmt.Fprintf(s, "%%!%c(ERROR: unsupported flag 0)", verb)
			break
		}
		if err := ba.writeOctStr(
			s, wid,
			s.Flag(int(' ')), s.Flag(int('#')),
			s.Flag(int('+')), s.Flag(int('-')),
		); err != nil {
			fmt.Fprintf(s, "%%!%c(ERROR: %v)", verb, err)
			break
		}

	case 'x', 'X':
		wid, widOK := s.Width()
		if !widOK {
			wid = 0
		}
		if s.Flag(int('0')) {
			fmt.Fprintf(s, "%%!%c(ERROR: unsupported flag 0)", verb)
			break
		}
		if err := ba.writeHexStr(
			s, wid, verb == 'X',
			s.Flag(int(' ')), s.Flag(int('#')),
			s.Flag(int('+')), s.Flag(int('-')),
		); err != nil {
			fmt.Fprintf(s, "%%!%c(ERROR: %v)", verb, err)
			break
		}

	default:
		fmt.Fprintf(s, "%%!%c(BitArray=", verb)
		if err := ba.writeBinStr(s, 0, true, true, true, false); err != nil {
			fmt.Fprintf(s, "(ERROR: %v)", err)
		}
		fmt.Fprint(s, ")")
	}
}

func (ba *BitArray) writeBinStr(wr io.Writer, wid int, sep8, sep4, quote, left bool) error {
	sep8 = sep8 || sep4
	sLen := ba.Len()
	if 0 < ba.nBits && sep8 {
		n4d := (ba.nBits + 3) >> 2 // num of 1111
		n8d := (n4d + 1) >> 1      // num of 11111111
		if sep4 {
			sLen += n4d - 1
		} else {
			sLen += n8d - 1
		}
	}
	if quote {
		sLen += 2
	}
	if wid < sLen {
		wid = sLen
	}
	if !left {
		if pad := wid - sLen; 0 < pad {
			if _, err := wr.Write(bytes.Repeat([]byte{' '}, pad)); err != nil {
				return fmt.Errorf("pad-l: %w", err)
			}
		}
	}
	if quote {
		if _, err := wr.Write([]byte{'"'}); err != nil {
			return fmt.Errorf("quote-l: %w", err)
		}
	}
	octbuf := make([]byte, 9)
	for i := 0; i < (ba.nBits+7)>>3; i++ {
		var wb byte
		if ba.b != nil {
			wb = ba.b[i]
		}
		if sep8 && i != 0 {
			if _, err := wr.Write([]byte{' '}); err != nil {
				return fmt.Errorf("oct-sep: %w", err)
			}
		}
		cLSB := 0                   // num of LSBs to be cleared
		if i+1 == (ba.nBits+7)>>3 { // last byte
			cLSB = ba.NumPadding()
		}
		bi := 0
		for j := 7; cLSB <= j; j-- {
			if sep4 && j == 3 {
				octbuf[bi] = byte('-')
				bi++
			}
			octbuf[bi] = '0' + wb>>j&1
			bi++
		}
		if _, err := wr.Write(octbuf[:bi]); err != nil {
			return fmt.Errorf("byte: %d(%d): %w", i, cLSB, err)
		}
	}
	if quote {
		if _, err := wr.Write([]byte{'"'}); err != nil {
			return fmt.Errorf("quote-r: %w", err)
		}
	}
	if left {
		if pad := wid - sLen; 0 < pad {
			if _, err := wr.Write(bytes.Repeat([]byte{' '}, pad)); err != nil {
				return fmt.Errorf("pad-r: %w", err)
			}
		}
	}

	return nil
}

func (ba *BitArray) writeOctStr(wr io.Writer, wid int, sep8, sep4, rpad, left bool) error {
	sep8 = sep8 || sep4
	ntri, nfrc := ba.nBits/3, ba.nBits%3 // num of 777, and remaining bits
	if nfrc != 0 {
		ntri++
	}
	npad := (3 - nfrc) % 3 // num of last padding bits

	sLen := ntri
	if 0 < ba.nBits && sep8 {
		n4d := (ntri + 3) >> 2 // num of 7777
		n8d := (n4d + 1) >> 1  // num of 77777777
		if sep4 {
			sLen += n4d - 1
		} else {
			sLen += n8d - 1
		}
	}
	if rpad && npad != 0 { // (pad=2)
		if sep8 {
			sLen++
		}
		sLen += 7
	}
	if wid < sLen {
		wid = sLen
	}
	if !left {
		if pad := wid - sLen; 0 < pad {
			if _, err := wr.Write(bytes.Repeat([]byte{' '}, pad)); err != nil {
				return fmt.Errorf("pad-l: %w", err)
			}
		}
	}

	iby, ibi := 0, 0 // source cursor on ba.b, byte and bit
	for i := 0; i < ntri; i++ {
		if sep8 && i != 0 && i&3 == 0 {
			if i&7 == 0 {
				if _, err := wr.Write([]byte{' '}); err != nil {
					return fmt.Errorf("sep8: %w", err)
				}
			} else if sep4 {
				if _, err := wr.Write([]byte{'-'}); err != nil {
					return fmt.Errorf("sep4: %w", err)
				}
			}
		}
		var b byte
		if ba.b != nil {
			b = ba.b[iby]
		}
		rsf := (5 - ibi)
		if 0 < rsf {
			b >>= rsf
		} else {
			b <<= -rsf
		}
		b &= 7
		ibi += 3
		if 7 < ibi {
			iby++
			if iby < (ba.nBits+7)>>3 {
				ibi &= 7
				if ba.b != nil {
					b |= ba.b[iby] >> (8 - ibi) & byte(ibi|1)
				}
			}
		}
		if _, err := wr.Write([]byte{'0' + b}); err != nil {
			return fmt.Errorf("oct: %w", err)
		}
	}
	if rpad && npad != 0 { // (pad=2)
		b := []byte{' ', '(', 'p', 'a', 'd', '=', '0' + byte(npad), ')'}
		if !sep8 {
			b = b[1:]
		}
		if _, err := wr.Write(b); err != nil {
			return fmt.Errorf("npad: %w", err)
		}
	}
	if left {
		if pad := wid - sLen; 0 < pad {
			if _, err := wr.Write(bytes.Repeat([]byte{' '}, pad)); err != nil {
				return fmt.Errorf("pad-r: %w", err)
			}
		}
	}

	return nil
}

func (ba *BitArray) writeHexStr(wr io.Writer, wid int, upper, sep8, sep4, rpad, left bool) error {
	const (
		hexCharsL = "0123456789abcdef"
		hexCharsU = "0123456789ABCDEF"
	)
	sep8 = sep8 || sep4
	nnbl := (ba.nBits + 3) >> 2 // num of f
	npad := ba.NumPadding() & 3
	sLen := nnbl
	if 0 < ba.nBits && sep8 {
		n4d := (nnbl + 3) >> 2 // num of ffff
		n8d := (n4d + 1) >> 1  // num of ffffffff
		if sep4 {
			sLen += n4d - 1
		} else {
			sLen += n8d - 1
		}
	}
	if rpad && npad != 0 { // (pad=3)
		if sep8 {
			sLen++
		}
		sLen += 7
	}
	if wid < sLen {
		wid = sLen
	}
	if !left {
		if pad := wid - sLen; 0 < pad {
			if _, err := wr.Write(bytes.Repeat([]byte{' '}, pad)); err != nil {
				return fmt.Errorf("pad-l: %w", err)
			}
		}
	}
	hexc := hexCharsL
	if upper {
		hexc = hexCharsU
	}
	for i := 0; i < nnbl; i++ {
		if i != 0 && sep8 && i&3 == 0 {
			if sep4 || i&7 == 0 {
				if _, err := wr.Write([]byte{' '}); err != nil {
					return fmt.Errorf("sep: %w", err)
				}
			}
		}
		boff := 4 ^ i&1<<2
		var b byte
		if ba.b != nil {
			b = ba.b[i>>1]
		}
		if _, err := wr.Write([]byte{hexc[b>>boff&0xf]}); err != nil {
			return fmt.Errorf("nibble: %w", err)
		}
	}
	if rpad && npad != 0 { // (pad=3)
		b := []byte{' ', '(', 'p', 'a', 'd', '=', '0' + byte(npad), ')'}
		if !sep8 {
			b = b[1:]
		}
		if _, err := wr.Write(b); err != nil {
			return fmt.Errorf("npad: %w", err)
		}
	}
	if left {
		if pad := wid - sLen; 0 < pad {
			if _, err := wr.Write(bytes.Repeat([]byte{' '}, pad)); err != nil {
				return fmt.Errorf("pad-r: %w", err)
			}
		}
	}

	return nil
}
