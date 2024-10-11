// Copyright (c) 2021 Oasis Labs Inc. All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
// 1. Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright
// notice, this list of conditions and the following disclaimer in the
// documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS
// IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED
// TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A
// PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED
// TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

package strobe

import (
	"bytes"
	"crypto/rand"
	"fmt"
	"testing"
)

func TestStrobeSanity(t *testing.T) {
	// Generated with mimoo/StrobeGo
	const (
		expectedHex  = "c4728cdd0361684d643a44221d16dc4677c62ed74a7f103635bd9cb6f3cc11bdd8405b105cd7de36f800dda96ea52c6adab88225c44faba4281dcdf84b2f3454"
		expectedHex2 = "16671f5f3603853adaf55614387d5604"
	)

	data := make([]byte, 1024) // Considerably larger than s.r
	for i := 0; i < len(data); i++ {
		b := byte(i & 0xff)
		data[i] = b
	}

	s := New("test-strobe-sanity")

	// MetaAD
	s.MetaAD(data, false)

	// KEY
	keyStr := "test-strobe-sanity-key"
	keyBuf := []byte(keyStr)
	s.KEY(keyBuf)
	if !bytes.Equal([]byte(keyStr), keyBuf) {
		t.Fatalf("s.KEY tramples over data: %x", keyBuf)
	}

	// Clone
	s2 := s.Clone()

	// AD
	s.AD(data, false)
	s.AD(data, true) // Test s.operate with `more`

	// PRF
	dest := make([]byte, 64)
	_, _ = rand.Read(dest) // Fill dest with garbage.
	s.PRF(dest)
	if x := fmt.Sprintf("%x", dest); x != expectedHex {
		t.Fatalf("s.PRF output mismatch: %s", x)
	}

	// PRF (cloned)
	dest2 := make([]byte, 16)
	s2.PRF(dest2)
	if x := fmt.Sprintf("%x", dest2); x != expectedHex2 {
		t.Fatalf("s2.PRF output mismatch: %s", x)
	}
}

var benchSizes = []int{1, 16, 32, 64, 128, 256, 512, 1024, 1024768}

func BenchmarkStrobe(b *testing.B) {
	for _, sz := range benchSizes {
		b.Run(fmt.Sprintf("AD/%d", sz), func(b *testing.B) {
			benchAd(b, sz)
		})
	}
}

func benchAd(b *testing.B, sz int) {
	buf := make([]byte, sz)
	_, _ = rand.Read(buf)

	s := New("benchmark-strobe")
	b.ResetTimer()

	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		s.AD(buf, false)
	}
}
