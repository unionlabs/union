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

// Package strobe implements enough of STROBE-128/1600 to implement
// merlin.
package strobe

import "fmt"

const (
	constN   = 1600 / 8
	constSec = 128
)

type flags uint8

const (
	flagI flags = 1 << 0 // inbound
	flagA flags = 1 << 1 // application
	flagC flags = 1 << 2 // cipher
	flagM flags = 1 << 4 // meta
	// T, K left undefined due to not being needed.
)

type Strobe struct {
	st       [constN]byte
	pos      int
	posBegin int

	initialized bool
	curFlags    flags
	r           int
}

func (s *Strobe) Clone() *Strobe {
	// All the sub-fields are naively copy-able.
	sCopy := *s

	return &sCopy
}

func (s *Strobe) AD(data []byte, more bool) {
	s.operate(flagA, data, more)
}

func (s *Strobe) MetaAD(data []byte, more bool) {
	s.operate(flagA|flagM, data, more)
}

func (s *Strobe) KEY(data []byte) {
	// See the comment in operate (TLDR: work on a copy, side-effects
	// are rude.
	keyCopy := make([]byte, len(data))
	copy(keyCopy, data)

	s.operate(flagA|flagC, keyCopy, false)
}

func (s *Strobe) PRF(dest []byte) {
	// Clear out the destination buffer.
	for i := range dest {
		dest[i] = 0
	}

	s.operate(flagI|flagA|flagC, dest, false)
}

func (s *Strobe) duplex(data []byte, cBefore, forceF bool) {
	dataIdx, dataLen := 0, len(data)

	// TODO/perf: This does the simple thing, and always keeps the
	// canonical view of the state (s.st) as a byte array.  This is
	// not ideal for performance, and the alternative approach as
	// done by mimoo/StrobeGo of keeping the state as a uint64 array
	// and XORing in data 8 bytes at a time as part of runF would
	// be faster.
	//
	// This is not done as:
	//  * This is significantly easier to read.
	//  * The naive thing is sufficiently fast/faster on amd64,
	//    particularly because we can just cast s.st and call
	//    keccakf1600 (StrobeGo pulls ahead on a trivial `AD`
	//    benchmark somewhere at the 2 MiB data size mark).
	//  * People should be using sr25519.NewTranscriptHash or
	//    sr25519.NewTranscriptXOF instead of huge messages
	//    anyway.
	//
	// Notes:
	//  * On non-amd64 targets StrobeGo's approach is expected
	//    to be faster at significantly smaller (~64 bytes)
	//    data sizes.  PRs welcome.
	//  * Add https://github.com/golang/go/issues/30553 to the
	//    list of things that would have been useful, that have
	//    been rejected by the Go developers.
	for remaining := dataLen; remaining > 0; {
		n := remaining
		if bytesAvailable := s.r - s.pos; n > bytesAvailable {
			n = bytesAvailable
		}

		dataTodo := data[dataIdx : dataIdx+n]
		stTodo := s.st[s.pos : s.pos+n]

		// Force the compiler to elide bounds checks in the loops.
		_ = dataTodo[n-1]
		_ = stTodo[n-1]

		if cBefore {
			// This could be merged with the next loop, but KEY
			// and PRF aren't called that often, and shouldn't be
			// called with very large data sizes.
			for i := 0; i < n; i++ {
				dataTodo[i] ^= stTodo[i]
			}
		}
		for i := 0; i < n; i++ {
			stTodo[i] ^= dataTodo[i]
		}
		// cAfter handling omitted due to not being needed.

		s.pos += n
		dataIdx += n
		remaining -= n

		if s.pos == s.r {
			s.runF()
		}
	}

	if forceF && s.pos != 0 {
		s.runF()
	}
}

func (s *Strobe) runF() {
	if s.initialized {
		s.st[s.pos] ^= byte(s.posBegin)
		s.st[s.pos+1] ^= 0x04
		s.st[s.r+1] ^= 0x80
	}

	keccakF1600Bytes(&s.st)

	s.pos, s.posBegin = 0, 0
}

func (s *Strobe) beginOp(f flags) {
	// No need to adjust direction, T flag not supported.

	oldBegin := s.posBegin
	s.posBegin = s.pos + 1

	s.duplex([]byte{byte(oldBegin), byte(f)}, false, f&flagC != 0)
}

func (s *Strobe) operate(f flags, data []byte, more bool) {
	if !s.initialized {
		panic("internal/strobe: operate called on uninitialzed state")
	}

	switch more {
	case true:
		if f != s.curFlags {
			panic(fmt.Sprintf("internal/strobe: flag mismatch on more: %x, expected %x", f, s.curFlags))
		}
	case false:
		s.beginOp(f)
		s.curFlags = f
	}

	// So cBefore causes s.duplex to trample over data.  This is what
	// we want in the case of `PRF` since that is the only operation
	// that is implemented in our subset that has `I` and `A` set.
	// The caller handles ensuring that data is zero-ed out before
	// calling operate.
	//
	// We explicitly do not want to write over data in the case of
	// `KEY`, but we handle that by passing in a copy of the caller
	// provided key material.

	// No need for `cafter`, T flag not supported.
	cBefore := (f & flagC) != 0
	s.duplex(data, cBefore, false)
}

func New(proto string) Strobe {
	s := Strobe{
		r: constN - constSec/4,
	}

	domain := []byte{
		1, byte(s.r), 1, 0, 1, 12 * 8,
		'S', 'T', 'R', 'O', 'B', 'E', 'v', '1', '.', '0', '.', '2',
	}
	s.duplex(domain, false, true)

	s.r = s.r - 2
	s.initialized = true
	s.operate(flagA|flagM, []byte(proto), false)

	return s
}
