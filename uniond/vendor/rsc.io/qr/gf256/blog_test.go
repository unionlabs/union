// Copyright 2012 The Go Authors.  All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// This file contains a straightforward implementation of
// Reed-Solomon encoding, along with a benchmark.
// It goes with http://research.swtch.com/field.
//
// For an optimized implementation, see gf256.go.

package gf256

import (
	"bytes"
	"fmt"
	"testing"
)

// BlogECC writes to check the error correcting code bytes
// for data using the given Reed-Solomon parameters.
func BlogECC(rs *RSEncoder, m []byte, check []byte) {
	if len(check) < rs.c {
		panic("gf256: invalid check byte length")
	}
	if rs.c == 0 {
		return
	}

	// The check bytes are the remainder after dividing
	// data padded with c zeros by the generator polynomial.

	// p = data padded with c zeros.
	var p []byte
	n := len(m) + rs.c
	if len(rs.p) >= n {
		p = rs.p
	} else {
		p = make([]byte, n)
	}
	copy(p, m)
	for i := len(m); i < len(p); i++ {
		p[i] = 0
	}

	gen := rs.gen

	// Divide p by gen, leaving the remainder in p[len(data):].
	// p[0] is the most significant term in p, and
	// gen[0] is the most significant term in the generator.
	for i := 0; i < len(m); i++ {
		k := f.Mul(p[i], f.Inv(gen[0])) // k = pi / g0
		// p -= k·g
		for j, g := range gen {
			p[i+j] = f.Add(p[i+j], f.Mul(k, g))
		}
	}

	copy(check, p[len(m):])
	rs.p = p
}

func BenchmarkBlogECC(b *testing.B) {
	data := []byte{0x10, 0x20, 0x0c, 0x56, 0x61, 0x80, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11, 0x10, 0x20, 0x0c, 0x56, 0x61, 0x80, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11}
	check := []byte{0x29, 0x41, 0xb3, 0x93, 0x8, 0xe8, 0xa3, 0xe7, 0x63, 0x8f}
	out := make([]byte, len(check))
	rs := NewRSEncoder(f, len(check))
	for i := 0; i < b.N; i++ {
		BlogECC(rs, data, out)
	}
	b.SetBytes(int64(len(data)))
	if !bytes.Equal(out, check) {
		fmt.Printf("have %#v want %#v\n", out, check)
	}
}

func TestBlogECC(t *testing.T) {
	data := []byte{0x10, 0x20, 0x0c, 0x56, 0x61, 0x80, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11, 0xec, 0x11}
	check := []byte{0xa5, 0x24, 0xd4, 0xc1, 0xed, 0x36, 0xc7, 0x87, 0x2c, 0x55}
	out := make([]byte, len(check))
	rs := NewRSEncoder(f, len(check))
	BlogECC(rs, data, out)
	if !bytes.Equal(out, check) {
		t.Errorf("have %x want %x", out, check)
	}
}
