// Copyright 2017 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package bloombits

import (
	"bytes"
	crand "crypto/rand"
	"math/rand"
	"testing"

	"github.com/ethereum/go-ethereum/core/types"
)

// Tests that batched bloom bits are correctly rotated from the input bloom
// filters.
func TestGenerator(t *testing.T) {
	// Generate the input and the rotated output
	var input, output [types.BloomBitLength][types.BloomByteLength]byte

	for i := 0; i < types.BloomBitLength; i++ {
		for j := 0; j < types.BloomBitLength; j++ {
			bit := byte(rand.Int() % 2)

			input[i][j/8] |= bit << byte(7-j%8)
			output[types.BloomBitLength-1-j][i/8] |= bit << byte(7-i%8)
		}
	}
	// Crunch the input through the generator and verify the result
	gen, err := NewGenerator(types.BloomBitLength)
	if err != nil {
		t.Fatalf("failed to create bloombit generator: %v", err)
	}
	for i, bloom := range input {
		if err := gen.AddBloom(uint(i), bloom); err != nil {
			t.Fatalf("bloom %d: failed to add: %v", i, err)
		}
	}
	for i, want := range output {
		have, err := gen.Bitset(uint(i))
		if err != nil {
			t.Fatalf("output %d: failed to retrieve bits: %v", i, err)
		}
		if !bytes.Equal(have, want[:]) {
			t.Errorf("output %d: bit vector mismatch have %x, want %x", i, have, want)
		}
	}
}

func BenchmarkGenerator(b *testing.B) {
	var input [types.BloomBitLength][types.BloomByteLength]byte
	b.Run("empty", func(b *testing.B) {
		b.ReportAllocs()
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			// Crunch the input through the generator and verify the result
			gen, err := NewGenerator(types.BloomBitLength)
			if err != nil {
				b.Fatalf("failed to create bloombit generator: %v", err)
			}
			for j, bloom := range &input {
				if err := gen.AddBloom(uint(j), bloom); err != nil {
					b.Fatalf("bloom %d: failed to add: %v", i, err)
				}
			}
		}
	})
	for i := 0; i < types.BloomBitLength; i++ {
		crand.Read(input[i][:])
	}
	b.Run("random", func(b *testing.B) {
		b.ReportAllocs()
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			// Crunch the input through the generator and verify the result
			gen, err := NewGenerator(types.BloomBitLength)
			if err != nil {
				b.Fatalf("failed to create bloombit generator: %v", err)
			}
			for j, bloom := range &input {
				if err := gen.AddBloom(uint(j), bloom); err != nil {
					b.Fatalf("bloom %d: failed to add: %v", i, err)
				}
			}
		}
	})
}
