// Copyright 2011 The Go Authors.  All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package coding

import (
	"bytes"
	"testing"

	"rsc.io/qr/gf256"
	"rsc.io/qr/libqrencode"
)

func test(t *testing.T, v Version, l Level, text ...Encoding) bool {
	s := ""
	ty := libqrencode.EightBit
	switch x := text[0].(type) {
	case String:
		s = string(x)
	case Alpha:
		s = string(x)
		ty = libqrencode.Alphanumeric
	case Num:
		s = string(x)
		ty = libqrencode.Numeric
	}
	key, err := libqrencode.Encode(libqrencode.Version(v), libqrencode.Level(l), ty, s)
	if err != nil {
		t.Errorf("libqrencode.Encode(%v, %v, %d, %#q): %v", v, l, ty, s, err)
		return false
	}
	mask := (^key.Pixel[8][2]&1)<<2 | (key.Pixel[8][3]&1)<<1 | (^key.Pixel[8][4] & 1)
	p, err := NewPlan(v, l, Mask(mask))
	if err != nil {
		t.Errorf("NewPlan(%v, L, %d): %v", v, err, mask)
		return false
	}
	if len(p.Pixel) != len(key.Pixel) {
		t.Errorf("%v: NewPlan uses %dx%d, libqrencode uses %dx%d", v, len(p.Pixel), len(p.Pixel), len(key.Pixel), len(key.Pixel))
		return false
	}
	c, err := p.Encode(text...)
	if err != nil {
		t.Errorf("Encode: %v", err)
		return false
	}
	badpix := 0
Pixel:
	for y, prow := range p.Pixel {
		for x, pix := range prow {
			pix &^= Black
			if c.Black(x, y) {
				pix |= Black
			}

			keypix := key.Pixel[y][x]
			want := Pixel(0)
			switch {
			case keypix&libqrencode.Finder != 0:
				want = Position.Pixel()
			case keypix&libqrencode.Alignment != 0:
				want = Alignment.Pixel()
			case keypix&libqrencode.Timing != 0:
				want = Timing.Pixel()
			case keypix&libqrencode.Format != 0:
				want = Format.Pixel()
				want |= OffsetPixel(pix.Offset()) // sic
				want |= pix & Invert
			case keypix&libqrencode.PVersion != 0:
				want = PVersion.Pixel()
			case keypix&libqrencode.DataECC != 0:
				if pix.Role() == Check || pix.Role() == Extra {
					want = pix.Role().Pixel()
				} else {
					want = Data.Pixel()
				}
				want |= OffsetPixel(pix.Offset())
				want |= pix & Invert
			default:
				want = Unused.Pixel()
			}
			if keypix&libqrencode.Black != 0 {
				want |= Black
			}
			if pix != want {
				t.Errorf("%v/%v: Pixel[%d][%d] = %v, want %v %#x", v, mask, y, x, pix, want, keypix)
				if badpix++; badpix >= 100 {
					t.Errorf("stopping after %d bad pixels", badpix)
					break Pixel
				}
			}
		}
	}
	return badpix == 0
}

var input = []Encoding{
	String("hello"),
	Num("1"),
	Num("12"),
	Num("123"),
	Alpha("AB"),
	Alpha("ABC"),
}

func TestVersion(t *testing.T) {
	badvers := 0
Version:
	for v := Version(1); v <= 40; v++ {
		for l := L; l <= H; l++ {
			for _, in := range input {
				if !test(t, v, l, in) {
					if badvers++; badvers >= 10 {
						t.Errorf("stopping after %d bad versions", badvers)
						break Version
					}
				}
			}
		}
	}
}

func TestEncode(t *testing.T) {
	data := []byte{0x10, 0x20, 0x0c, 0x56, 0x61, 0x80, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11}
	check := []byte{0xa5, 0x24, 0xd4, 0xc1, 0xed, 0x36, 0xc7, 0x87, 0x2c, 0x55}
	rs := gf256.NewRSEncoder(Field, len(check))
	out := make([]byte, len(check))
	rs.ECC(data, out)
	if !bytes.Equal(out, check) {
		t.Errorf("have %x want %x", out, check)
	}
}
