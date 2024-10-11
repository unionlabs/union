// Copyright 2019 Gregory Petrosyan <gregory.petrosyan@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package rapid

import "testing"

type trivialGenImpl struct{}

func (trivialGenImpl) String() string    { return "" }
func (trivialGenImpl) value(t *T) uint64 { return t.s.drawBits(64) }

func BenchmarkTrivialGenImplValue(b *testing.B) {
	t := newT(nil, newRandomBitStream(baseSeed(), false), false, nil)
	g := trivialGenImpl{}
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		g.value(t)
	}
}

func BenchmarkGenerator_Value(b *testing.B) {
	t := newT(nil, newRandomBitStream(baseSeed(), false), false, nil)
	g := newGenerator[uint64](trivialGenImpl{})
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		g.value(t)
	}
}

func TestExampleHelper(t *testing.T) {
	g := Custom(func(t *T) int {
		t.Helper()
		return Int().Draw(t, t.Name())
	})

	g.Example(0)
}
