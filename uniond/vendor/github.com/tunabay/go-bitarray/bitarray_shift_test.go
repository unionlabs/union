// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"math/rand"
	"strings"
	"testing"
	"time"

	"github.com/tunabay/go-bitarray"
)

func TestBitArray_Reverse_rand(t *testing.T) {
	const testIterations = 30000
	rand.Seed(time.Now().UnixNano())
	for i := 0; i < testIterations; i++ {
		var nBits int
		switch rand.Intn(5) {
		case 0:
			nBits = rand.Intn(25)
		case 1:
			nBits = 8*(1+rand.Intn(8)) - 1 + rand.Intn(3)
		default:
			nBits = rand.Intn(512)
		}
		sb0, sbR := make([]byte, nBits), make([]byte, nBits)
		for j := 0; j < nBits; j++ {
			b := '0' + byte(rand.Intn(2))
			sb0[j], sbR[nBits-1-j] = b, b
		}
		ba0 := bitarray.MustParse(string(sb0))
		exp := bitarray.MustParse(string(sbR))

		baR := ba0.Reverse()
		baR.V()
		baE, baO := ba0.ZExpand(), ba0.ZOptimize()
		baER, baOR := baE.Reverse(), baO.Reverse()
		baRE, baRO := baR.ZExpand(), baR.ZOptimize()

		switch {
		case !baR.Equal(exp):
			t.Error("unexpected Reverse result:")
		case !baER.Equal(baR), !baOR.Equal(baR):
			t.Error("unexpected Reverse result(2):")
		case !baRE.Equal(baR), !baRO.Equal(baR):
			t.Error("unexpected Reverse result(3):")
		}
		if t.Failed() {
			t.Logf(" src: %#b", ba0)
			t.Logf(" got: %#b", baR)
			t.Logf("want: %#b", exp)
			t.FailNow()
		}
		// if i < 32 {
		// 	t.Logf(" src: %#b", ba0)
		// 	t.Logf("pass: %#b", baR)
		// 	t.Logf("pass: %s", baR.D())
		// }
	}
}

func TestBitArray_ShiftLeft_rand(t *testing.T) {
	const testIterations = 16000
	rand.Seed(time.Now().UnixNano())
	for i := 0; i < testIterations; i++ {
		var nBits int
		switch rand.Intn(5) {
		case 0:
			nBits = rand.Intn(25)
		case 1:
			nBits = 8*(1+rand.Intn(8)) - 1 + rand.Intn(3)
		default:
			nBits = rand.Intn(512)
		}
		var sbb strings.Builder
		sbb.Grow(nBits)
		for j := 0; j < nBits; j++ {
			sbb.WriteByte('0' + byte(rand.Intn(2)))
		}
		sb0 := sbb.String()
		ba0 := bitarray.MustParse(sb0)

		nShift := 4
		if 16 < nBits {
			nShift = 8
		}
		for j := 0; j < nShift; j++ {
			var leftK int
			switch rand.Intn(40) {
			case 0:
				leftK = nBits + rand.Intn(nBits+10)
			case 1:
				leftK = -nBits - rand.Intn(nBits+10)
			default:
				leftK = -nBits + rand.Intn(nBits*2+1)
			}

			expS := sb0
			switch {
			case nBits < leftK, leftK < -nBits:
				expS = strings.Repeat("0", nBits)
			case 0 < leftK:
				expS = sb0[leftK:] + strings.Repeat("0", leftK)
			case leftK < 0:
				expS = strings.Repeat("0", -leftK) + sb0[:nBits+leftK]
			}
			expB := bitarray.MustParse(expS)
			expBE, expBO := expB.ZExpand(), expB.ZOptimize()

			gotB := ba0.ShiftLeft(leftK)
			gotBE := ba0.ZExpand().ShiftLeft(leftK)
			gotBO := ba0.ZOptimize().ShiftLeft(leftK)
			gotB.V()
			gotBE.V()
			gotBO.V()

			switch {
			case !gotB.Equal(expB), !gotB.Equal(expBE), !gotB.Equal(expBO):
				t.Errorf("unexpected result: leftK=%d", leftK)
				t.Logf(" src: %s", ba0.D())
			case !gotBE.Equal(expB):
				t.Errorf("unexpected result (e): leftK=%d", leftK)
				t.Logf(" src: %s", ba0.ZExpand().D())
			case !gotBO.Equal(expB):
				t.Errorf("unexpected result (o): leftK=%d", leftK)
				t.Logf(" src: %s", ba0.ZOptimize().D())
			}
			if t.Failed() {
				t.Logf(" src: %#b", ba0)
				t.Logf(" got: %#b", gotB)
				t.Logf(" got: %s", gotB.D())
				t.Logf("want: %#b", expB)
				t.FailNow()
			}
			// if i < 8 {
			// 	t.Logf("pass: leftK=%d", leftK)
			// 	t.Logf(" src: %#b", ba0)
			// 	t.Logf(" got: %#b", gotB)
			// 	t.Logf(" got: %s", gotB.D())
			// }
		}
	}
}

func TestBitArray_RotateLeft_rand(t *testing.T) {
	const testIterations = 16000
	rand.Seed(time.Now().UnixNano())
	for i := 0; i < testIterations; i++ {
		var nBits int
		switch rand.Intn(5) {
		case 0:
			nBits = rand.Intn(25)
		case 1:
			nBits = 8*(1+rand.Intn(8)) - 1 + rand.Intn(3)
		default:
			nBits = rand.Intn(512)
		}
		var sbb strings.Builder
		sbb.Grow(nBits)
		for j := 0; j < nBits; j++ {
			sbb.WriteByte('0' + byte(rand.Intn(2)))
		}
		sb0 := sbb.String()
		ba0 := bitarray.MustParse(sb0)

		nShift := 4
		if 16 < nBits {
			nShift = 8
		}
		for j := 0; j < nShift; j++ {
			var leftK int
			switch rand.Intn(40) {
			case 0:
				leftK = nBits + rand.Intn(nBits+10)
			case 1:
				leftK = -nBits - rand.Intn(nBits+10)
			default:
				leftK = -nBits + rand.Intn(nBits*2+1)
			}

			expS := sb0
			switch {
			case nBits == 0:
			case 0 < leftK:
				effLK := leftK % nBits
				expS = sb0[effLK:] + sb0[0:effLK]
			case leftK < 0:
				effRK := (-leftK) % nBits
				expS = sb0[nBits-effRK:] + sb0[:nBits-effRK]
			}
			expB := bitarray.MustParse(expS)
			expBE, expBO := expB.ZExpand(), expB.ZOptimize()

			gotB := ba0.RotateLeft(leftK)
			gotBE := ba0.ZExpand().RotateLeft(leftK)
			gotBO := ba0.ZOptimize().RotateLeft(leftK)
			gotB.V()
			gotBE.V()
			gotBO.V()

			switch {
			case !gotB.Equal(expB), !gotB.Equal(expBE), !gotB.Equal(expBO):
				t.Errorf("unexpected result: leftK=%d", leftK)
				t.Logf(" src: %s", ba0.D())
			case !gotBE.Equal(expB):
				t.Errorf("unexpected result (e): leftK=%d", leftK)
				t.Logf(" src: %s", ba0.ZExpand().D())
			case !gotBO.Equal(expB):
				t.Errorf("unexpected result (o): leftK=%d", leftK)
				t.Logf(" src: %s", ba0.ZOptimize().D())
			}
			if t.Failed() {
				t.Logf(" src: %#b", ba0)
				t.Logf(" got: %#b", gotB)
				t.Logf(" got: %s", gotB.D())
				t.Logf("want: %#b", expB)
				t.FailNow()
			}
			// if i < 8 {
			// 	t.Logf("pass: leftK=%d", leftK)
			// 	t.Logf(" src: %#b", ba0)
			// 	t.Logf(" got: %#b", gotB)
			// 	t.Logf(" got: %s", gotB.D())
			// }
		}
	}
}
