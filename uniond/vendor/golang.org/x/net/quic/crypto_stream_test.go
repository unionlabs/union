// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"crypto/rand"
	"reflect"
	"testing"
)

func TestCryptoStreamReceive(t *testing.T) {
	data := make([]byte, 1<<20)
	rand.Read(data) // doesn't need to be crypto/rand, but non-deprecated and harmless
	type frame struct {
		start int64
		end   int64
		want  int
	}
	for _, test := range []struct {
		name   string
		frames []frame
	}{{
		name: "linear",
		frames: []frame{{
			start: 0,
			end:   1000,
			want:  1000,
		}, {
			start: 1000,
			end:   2000,
			want:  2000,
		}, {
			// larger than any realistic packet can hold
			start: 2000,
			end:   1 << 20,
			want:  1 << 20,
		}},
	}, {
		name: "out of order",
		frames: []frame{{
			start: 1000,
			end:   2000,
		}, {
			start: 2000,
			end:   3000,
		}, {
			start: 0,
			end:   1000,
			want:  3000,
		}},
	}, {
		name: "resent",
		frames: []frame{{
			start: 0,
			end:   1000,
			want:  1000,
		}, {
			start: 0,
			end:   1000,
			want:  1000,
		}, {
			start: 1000,
			end:   2000,
			want:  2000,
		}, {
			start: 0,
			end:   1000,
			want:  2000,
		}, {
			start: 1000,
			end:   2000,
			want:  2000,
		}},
	}, {
		name: "overlapping",
		frames: []frame{{
			start: 0,
			end:   1000,
			want:  1000,
		}, {
			start: 3000,
			end:   4000,
			want:  1000,
		}, {
			start: 2000,
			end:   3000,
			want:  1000,
		}, {
			start: 1000,
			end:   3000,
			want:  4000,
		}},
	}, {
		name: "resent consumed data",
		frames: []frame{{
			start: 0,
			end:   1000,
			want:  1000,
		}, {
			start: 1000,
			end:   2000,
			want:  2000,
		}, {
			start: 0,
			end:   1000,
			want:  2000,
		}},
	}} {
		t.Run(test.name, func(t *testing.T) {
			var s cryptoStream
			var got []byte
			for _, f := range test.frames {
				t.Logf("receive [%v,%v)", f.start, f.end)
				s.handleCrypto(
					f.start,
					data[f.start:f.end],
					func(b []byte) error {
						t.Logf("got new bytes [%v,%v)", len(got), len(got)+len(b))
						got = append(got, b...)
						return nil
					},
				)
				if len(got) != f.want {
					t.Fatalf("have bytes [0,%v), want [0,%v)", len(got), f.want)
				}
				for i := range got {
					if got[i] != data[i] {
						t.Fatalf("byte %v of received data = %v, want %v", i, got[i], data[i])
					}
				}
			}
		})
	}
}

func TestCryptoStreamSends(t *testing.T) {
	data := make([]byte, 1<<20)
	rand.Read(data) // doesn't need to be crypto/rand, but non-deprecated and harmless
	type (
		sendOp i64range[int64]
		ackOp  i64range[int64]
		lossOp i64range[int64]
	)
	for _, test := range []struct {
		name        string
		size        int64
		ops         []any
		wantSend    []i64range[int64]
		wantPTOSend []i64range[int64]
	}{{
		name: "writes with data remaining",
		size: 4000,
		ops: []any{
			sendOp{0, 1000},
			sendOp{1000, 2000},
			sendOp{2000, 3000},
		},
		wantSend: []i64range[int64]{
			{3000, 4000},
		},
		wantPTOSend: []i64range[int64]{
			{0, 4000},
		},
	}, {
		name: "lost data is resent",
		size: 4000,
		ops: []any{
			sendOp{0, 1000},
			sendOp{1000, 2000},
			sendOp{2000, 3000},
			sendOp{3000, 4000},
			lossOp{1000, 2000},
			lossOp{3000, 4000},
		},
		wantSend: []i64range[int64]{
			{1000, 2000},
			{3000, 4000},
		},
		wantPTOSend: []i64range[int64]{
			{0, 4000},
		},
	}, {
		name: "acked data at start of range",
		size: 4000,
		ops: []any{
			sendOp{0, 4000},
			ackOp{0, 1000},
			ackOp{1000, 2000},
			ackOp{2000, 3000},
		},
		wantSend: nil,
		wantPTOSend: []i64range[int64]{
			{3000, 4000},
		},
	}, {
		name: "acked data is not resent on pto",
		size: 4000,
		ops: []any{
			sendOp{0, 4000},
			ackOp{1000, 2000},
		},
		wantSend: nil,
		wantPTOSend: []i64range[int64]{
			{0, 1000},
		},
	}, {
		// This is an unusual, but possible scenario:
		// Data is sent, resent, one of the two sends is acked, and the other is lost.
		name: "acked and then lost data is not resent",
		size: 4000,
		ops: []any{
			sendOp{0, 4000},
			sendOp{1000, 2000}, // resent, no-op
			ackOp{1000, 2000},
			lossOp{1000, 2000},
		},
		wantSend: nil,
		wantPTOSend: []i64range[int64]{
			{0, 1000},
		},
	}, {
		// The opposite of the above scenario: data is marked lost, and then acked
		// before being resent.
		name: "lost and then acked data is not resent",
		size: 4000,
		ops: []any{
			sendOp{0, 4000},
			sendOp{1000, 2000}, // resent, no-op
			lossOp{1000, 2000},
			ackOp{1000, 2000},
		},
		wantSend: nil,
		wantPTOSend: []i64range[int64]{
			{0, 1000},
		},
	}} {
		t.Run(test.name, func(t *testing.T) {
			var s cryptoStream
			s.write(data[:test.size])
			for _, op := range test.ops {
				switch op := op.(type) {
				case sendOp:
					t.Logf("send [%v,%v)", op.start, op.end)
					b := make([]byte, op.end-op.start)
					s.sendData(op.start, b)
				case ackOp:
					t.Logf("ack  [%v,%v)", op.start, op.end)
					s.ackOrLoss(op.start, op.end, packetAcked)
				case lossOp:
					t.Logf("loss [%v,%v)", op.start, op.end)
					s.ackOrLoss(op.start, op.end, packetLost)
				default:
					t.Fatalf("unhandled type %T", op)
				}
			}
			var gotSend []i64range[int64]
			s.dataToSend(true, func(off, size int64) (wrote int64) {
				gotSend = append(gotSend, i64range[int64]{off, off + size})
				return 0
			})
			if !reflect.DeepEqual(gotSend, test.wantPTOSend) {
				t.Fatalf("got data to send on PTO: %v, want %v", gotSend, test.wantPTOSend)
			}
			gotSend = nil
			s.dataToSend(false, func(off, size int64) (wrote int64) {
				gotSend = append(gotSend, i64range[int64]{off, off + size})
				b := make([]byte, size)
				s.sendData(off, b)
				return int64(len(b))
			})
			if !reflect.DeepEqual(gotSend, test.wantSend) {
				t.Fatalf("got data to send: %v, want %v", gotSend, test.wantSend)
			}
		})
	}
}
