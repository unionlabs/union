// Copyright 2019 Gregory Petrosyan <gregory.petrosyan@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package rapid

import (
	"fmt"
	"math/bits"
	"math/rand"
	"testing"
)

func TestJsfRand(t *testing.T) {
	t.Parallel()

	// using https://gist.github.com/imneme/85cff47d4bad8de6bdeb671f9c76c814
	golden := [10]uint64{
		0xe7ac7348cb3c6182,
		0xe20e62c321f18c3f,
		0x592927f9846891ae,
		0xda5c2b6e56ace47a,
		0x3c5987be726a7740,
		0x1463137b89c7292a,
		0xd118e05a46bc8156,
		0xeb72c3391969bc15,
		0xe94f306afee04198,
		0x0f57e93805e22a54,
	}

	ctx := &jsf64ctx{}
	ctx.init(0xcafe5eed00000001)

	for _, g := range golden {
		u := ctx.rand()
		if u != g {
			t.Errorf("0x%x instead of golden 0x%x", u, g)
		}
	}
}

func BenchmarkJsfRand(b *testing.B) {
	ctx := &jsf64ctx{}
	ctx.init(1)
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		ctx.rand()
	}
}

func BenchmarkMathRand(b *testing.B) {
	s := rand.NewSource(1).(rand.Source64)
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		s.Uint64()
	}
}

func TestRandomBitSteam_DrawBits(t *testing.T) {
	t.Parallel()

	s := createRandomBitStream(t)

	for n := 1; n <= 64; n++ {
		for i := 0; i < 100; i++ {
			t.Run(fmt.Sprintf("%v bits #%v", n, i), func(t *testing.T) {
				b := s.drawBits(n)
				if bits.Len64(b) > n {
					t.Errorf("%v: bitlen too big for %v bits", b, n)
				}
				if bits.OnesCount64(b) > n {
					t.Errorf("%v: too much ones for %v bits", b, n)
				}
			})
		}
	}
}

func BenchmarkBaseSeed(b *testing.B) {
	for i := 0; i < b.N; i++ {
		baseSeed()
	}
}

func BenchmarkRandomBitStream_DrawBits1(b *testing.B) {
	s := newRandomBitStream(baseSeed(), false)
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		s.drawBits(1)
	}
}

func BenchmarkRandomBitStream_DrawBits64(b *testing.B) {
	s := newRandomBitStream(baseSeed(), false)
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		s.drawBits(64)
	}
}
