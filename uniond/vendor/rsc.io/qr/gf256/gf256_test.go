// Copyright 2010 The Go Authors.  All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package gf256

import (
	"bytes"
	"fmt"
	"testing"
)

var f = NewField(0x11d, 2) // x^8 + x^4 + x^3 + x^2 + 1

func TestBasic(t *testing.T) {
	if f.Exp(0) != 1 || f.Exp(1) != 2 || f.Exp(255) != 1 {
		panic("bad Exp")
	}
}

func TestECC(t *testing.T) {
	data := []byte{0x10, 0x20, 0x0c, 0x56, 0x61, 0x80, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11}
	check := []byte{0xa5, 0x24, 0xd4, 0xc1, 0xed, 0x36, 0xc7, 0x87, 0x2c, 0x55}
	out := make([]byte, len(check))
	rs := NewRSEncoder(f, len(check))
	rs.ECC(data, out)
	if !bytes.Equal(out, check) {
		t.Errorf("have %x want %x", out, check)
	}
}

func TestLinear(t *testing.T) {
	d1 := []byte{0x00, 0x00}
	c1 := []byte{0x00, 0x00}
	out := make([]byte, len(c1))
	rs := NewRSEncoder(f, len(c1))
	if rs.ECC(d1, out); !bytes.Equal(out, c1) {
		t.Errorf("ECBytes(%x, %d) = %x, want 0", d1, len(c1), out)
	}
	d2 := []byte{0x00, 0x01}
	c2 := make([]byte, 2)
	rs.ECC(d2, c2)
	d3 := []byte{0x00, 0x02}
	c3 := make([]byte, 2)
	rs.ECC(d3, c3)
	cx := make([]byte, 2)
	for i := range cx {
		cx[i] = c2[i] ^ c3[i]
	}
	d4 := []byte{0x00, 0x03}
	c4 := make([]byte, 2)
	rs.ECC(d4, c4)
	if !bytes.Equal(cx, c4) {
		t.Errorf("ECBytes(%x, 2) = %x\nECBytes(%x, 2) = %x\nxor = %x\nECBytes(%x, 2) = %x",
			d2, c2, d3, c3, cx, d4, c4)
	}
}

func TestGaussJordan(t *testing.T) {
	rs := NewRSEncoder(f, 2)
	m := make([][]byte, 16)
	for i := range m {
		m[i] = make([]byte, 4)
		m[i][i/8] = 1 << uint(i%8)
		rs.ECC(m[i][:2], m[i][2:])
	}
	if false {
		fmt.Printf("---\n")
		for _, row := range m {
			fmt.Printf("%x\n", row)
		}
	}
	b := []uint{0, 1, 2, 3, 12, 13, 14, 15, 20, 21, 22, 23, 24, 25, 26, 27}
	for i := 0; i < 16; i++ {
		bi := b[i]
		if m[i][bi/8]&(1<<(7-bi%8)) == 0 {
			for j := i + 1; ; j++ {
				if j >= len(m) {
					t.Errorf("lost track for %d", bi)
					break
				}
				if m[j][bi/8]&(1<<(7-bi%8)) != 0 {
					m[i], m[j] = m[j], m[i]
					break
				}
			}
		}
		for j := i + 1; j < len(m); j++ {
			if m[j][bi/8]&(1<<(7-bi%8)) != 0 {
				for k := range m[j] {
					m[j][k] ^= m[i][k]
				}
			}
		}
	}
	if false {
		fmt.Printf("---\n")
		for _, row := range m {
			fmt.Printf("%x\n", row)
		}
	}
	for i := 15; i >= 0; i-- {
		bi := b[i]
		for j := i - 1; j >= 0; j-- {
			if m[j][bi/8]&(1<<(7-bi%8)) != 0 {
				for k := range m[j] {
					m[j][k] ^= m[i][k]
				}
			}
		}
	}
	if false {
		fmt.Printf("---\n")
		for _, row := range m {
			fmt.Printf("%x", row)
			out := make([]byte, 2)
			if rs.ECC(row[:2], out); !bytes.Equal(out, row[2:]) {
				fmt.Printf(" - want %x", out)
			}
			fmt.Printf("\n")
		}
	}
}

func BenchmarkECC(b *testing.B) {
	data := []byte{0x10, 0x20, 0x0c, 0x56, 0x61, 0x80, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11, 0x10, 0x20, 0x0c, 0x56, 0x61, 0x80, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11}
	check := []byte{0x29, 0x41, 0xb3, 0x93, 0x8, 0xe8, 0xa3, 0xe7, 0x63, 0x8f}
	out := make([]byte, len(check))
	rs := NewRSEncoder(f, len(check))
	for i := 0; i < b.N; i++ {
		rs.ECC(data, out)
	}
	b.SetBytes(int64(len(data)))
	if !bytes.Equal(out, check) {
		fmt.Printf("have %#v want %#v\n", out, check)
	}
}

func TestGen(t *testing.T) {
	for i := 0; i < 256; i++ {
		_, lg := f.gen(i)
		if lg[0] != 0 {
			t.Errorf("#%d: %x", i, lg)
		}
	}
}

func TestReducible(t *testing.T) {
	var count = []int{1, 2, 3, 6, 9, 18, 30, 56, 99, 186} // oeis.org/A1037
	for i, want := range count {
		n := 0
		for p := 1 << uint(i+2); p < 1<<uint(i+3); p++ {
			if !reducible(p) {
				n++
			}
		}
		if n != want {
			t.Errorf("#reducible(%d-bit) = %d, want %d", i+2, n, want)
		}
	}
}

func TestExhaustive(t *testing.T) {
	for poly := 0x100; poly < 0x200; poly++ {
		if reducible(poly) {
			continue
		}
		α := 2
		for !generates(α, poly) {
			α++
		}
		f := NewField(poly, α)
		for p := 0; p < 256; p++ {
			for q := 0; q < 256; q++ {
				fm := int(f.Mul(byte(p), byte(q)))
				pm := mul(p, q, poly)
				if fm != pm {
					t.Errorf("NewField(%#x).Mul(%#x, %#x) = %#x, want %#x", poly, p, q, fm, pm)
				}
			}
		}
	}
}

func generates(α, poly int) bool {
	x := α
	for i := 0; i < 254; i++ {
		if x == 1 {
			return false
		}
		x = mul(x, α, poly)
	}
	return true
}
