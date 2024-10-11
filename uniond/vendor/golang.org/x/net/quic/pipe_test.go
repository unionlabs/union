// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"bytes"
	"math/rand"
	"testing"
)

func TestPipeWrites(t *testing.T) {
	type writeOp struct {
		start, end int64
	}
	type discardBeforeOp struct {
		off int64
	}
	type op any
	src := make([]byte, 65536)
	rand.New(rand.NewSource(0)).Read(src)
	for _, test := range []struct {
		desc string
		ops  []op
	}{{
		desc: "sequential writes",
		ops: []op{
			writeOp{0, 1024},
			writeOp{1024, 4096},
			writeOp{4096, 65536},
		},
	}, {
		desc: "disordered overlapping writes",
		ops: []op{
			writeOp{2000, 8000},
			writeOp{0, 3000},
			writeOp{7000, 12000},
		},
	}, {
		desc: "write to discarded region",
		ops: []op{
			writeOp{0, 65536},
			discardBeforeOp{32768},
			writeOp{0, 1000},
			writeOp{3000, 5000},
			writeOp{0, 32768},
		},
	}, {
		desc: "write overlaps discarded region",
		ops: []op{
			discardBeforeOp{10000},
			writeOp{0, 20000},
		},
	}, {
		desc: "discard everything",
		ops: []op{
			writeOp{0, 10000},
			discardBeforeOp{10000},
			writeOp{10000, 20000},
		},
	}, {
		desc: "discard before writing",
		ops: []op{
			discardBeforeOp{1000},
			writeOp{0, 1},
		},
	}} {
		var p pipe
		var wantset rangeset[int64]
		var wantStart, wantEnd int64
		for i, o := range test.ops {
			switch o := o.(type) {
			case writeOp:
				p.writeAt(src[o.start:o.end], o.start)
				wantset.add(o.start, o.end)
				wantset.sub(0, wantStart)
				if o.end > wantEnd {
					wantEnd = o.end
				}
			case discardBeforeOp:
				p.discardBefore(o.off)
				wantset.sub(0, o.off)
				wantStart = o.off
				if o.off > wantEnd {
					wantEnd = o.off
				}
			}
			if p.start != wantStart || p.end != wantEnd {
				t.Errorf("%v: after %#v p contains [%v,%v), want [%v,%v)", test.desc, test.ops[:i+1], p.start, p.end, wantStart, wantEnd)
			}
			for _, r := range wantset {
				want := src[r.start:][:r.size()]
				got := make([]byte, r.size())
				p.copy(r.start, got)
				if !bytes.Equal(got, want) {
					t.Errorf("%v after %#v, mismatch in data in %v", test.desc, test.ops[:i+1], r)
				}
			}
		}
	}
}
