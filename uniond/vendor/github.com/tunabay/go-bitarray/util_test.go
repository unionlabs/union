// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"bytes"
	"fmt"
	"math/rand"
	"strings"
	"testing"
	"time"

	"github.com/tunabay/go-bitarray"
)

func TestCopyBitsB(t *testing.T) {
	test := func(
		dst, src []byte,
		dstOff, srcOff, nBits int,
		exp []byte,
	) {
		t.Helper()
		org := make([]byte, len(dst))
		copy(org, dst)
		bitarray.CopyBitsB(dst, src, dstOff, srcOff, nBits)
		if !bytes.Equal(dst, exp) {
			t.Error("unexpected result:")
			t.Logf("dstOff=%d, srcOff=%d, nBits=%d", dstOff, srcOff, nBits)
			t.Logf(" dst: %08b", org)
			t.Logf(" src: %08b", src)
			t.Logf(" got: %08b", dst)
			t.Logf("want: %08b", exp)
			t.FailNow()
		}
	}

	test(
		[]byte{0b_0000_0000},
		[]byte{0b_1110_0000},
		0, 0, 3,
		[]byte{0b_1110_0000},
	)
	test(
		[]byte{0b_1111_1011},
		[]byte{0b_0010_0000},
		0, 0, 4,
		[]byte{0b_0010_1011},
	)
	test(
		[]byte{0b_1111_1111},
		[]byte{0b_0001_0011},
		0, 0, 5,
		[]byte{0b_0001_0111},
	)
	test(
		[]byte{0b_1111_1111, 0b_0000_1111},
		[]byte{0b_0010_0000, 0b_1111_0000},
		3, 0, 4,
		[]byte{0b_1110_0101, 0b_0000_1111},
	)

	test(
		[]byte{0b_1111_1111, 0b_1111_1111, 0b_1111_1111, 0b_1111_1111, 0b_1111_1111},
		[]byte{0b_0000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000},
		10, 10, 12,
		[]byte{0b_1111_1111, 0b_1100_0000, 0b_0000_0011, 0b_1111_1111, 0b_1111_1111},
	)
	test(
		[]byte{0b_0000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000},
		[]byte{0b_1111_1111, 0b_1111_1111, 0b_1111_1111, 0b_1111_1111, 0b_1111_1111},
		8, 8, 12,
		[]byte{0b_0000_0000, 0b_1111_1111, 0b_1111_0000, 0b_0000_0000, 0b_0000_0000},
	)
	test(
		[]byte{0b_1111_1010, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_1111},
		[]byte{0b_0000_0111, 0b_1111_1111, 0b_1111_1111, 0b_1111_1111, 0b_1111_1111},
		11, 5, 22,
		[]byte{0b_1111_1010, 0b_0001_1111, 0b_1111_1111, 0b_1111_1111, 0b_1000_1111},
	)
}

func mkBitsStr(b []byte) string {
	s := ""
	for _, be := range b {
		s += fmt.Sprintf("%08b", be)
	}
	return s
}

func mkRandBits() []byte {
	b := make([]byte, 1+rand.Intn(8)) // len = 1..8
	rand.Read(b)
	return b
}

func dupBits(b []byte) []byte {
	d := make([]byte, len(b))
	copy(d, b)
	return d
}

func sfmt(s string) string {
	f := make([]string, (len(s)+7)>>3)
	for i := 0; i < len(f); i++ {
		f[i] = s[i<<3 : (i+1)<<3]
	}
	return fmt.Sprintf("[%s]", strings.Join(f, " "))
}

func TestCopyBitsB_rand(t *testing.T) {
	const testIterations = 50000
	rand.Seed(time.Now().UnixNano())
	for i := 0; i < testIterations; i++ {
		dstB, srcB := mkRandBits(), mkRandBits()
		dstS, srcS := mkBitsStr(dstB), mkBitsStr(srcB)
		orgB := dupBits(dstB)
		orgS := mkBitsStr(orgB)

		wid := len(dstS)
		if len(srcS) < wid {
			wid = len(srcS)
		}
		nBits := rand.Intn(wid) + 1
		dst0 := rand.Intn(len(dstS) - nBits + 1)
		src0 := rand.Intn(len(srcS) - nBits + 1)

		subpS := srcS[src0 : src0+nBits]
		wantS := orgS[:dst0] + subpS + orgS[dst0+nBits:]
		gotZF := bitarray.CopyBitsB(dstB, srcB, dst0, src0, nBits)
		dstS = mkBitsStr(dstB)

		if gotZF && strings.Contains(subpS, "1") {
			t.Errorf("unexpected ZF: true for %q", subpS)
		}
		if dstS != wantS {
			t.Errorf("unexpected result: dstOff=%d, srcOff=%d, nBits=%d", dst0, src0, nBits)
		}
		if t.Failed() {
			t.Logf(" dst: %08b", orgB)
			t.Logf(" src: %08b", srcB)
			t.Logf(" got: %08b", dstB)
			t.Logf("want: %s", sfmt(wantS))
			t.FailNow()
		}
		// if i < 30 {
		// 	t.Logf("pass: dstOff=%d, srcOff=%d, nBits=%d", dst0, src0, nBits)
		// 	t.Logf(" dst: %08b", orgB)
		// 	t.Logf(" src: %08b", srcB)
		// 	t.Logf(" got: %08b", dstB)
		// }
	}
}

func TestClearBits(t *testing.T) {
	test := func(
		dst []byte,
		off, nBits int,
		exp []byte,
	) {
		t.Helper()
		org := make([]byte, len(dst))
		copy(org, dst)
		bitarray.ClearBits(dst, off, nBits)
		if !bytes.Equal(dst, exp) {
			t.Error("unexpected result:")
			t.Logf("off=%d, nBits=%d", off, nBits)
			t.Logf(" dst: %08b", org)
			t.Logf(" got: %08b", dst)
			t.Logf("want: %08b", exp)
			t.FailNow()
		}
	}

	test(
		[]byte{0b_1111_1111},
		0, 1,
		[]byte{0b_0111_1111},
	)
	test(
		[]byte{0b_1111_1111},
		0, 3,
		[]byte{0b_0001_1111},
	)
	test(
		[]byte{0b_1111_1111},
		5, 2,
		[]byte{0b_1111_1001},
	)
	test(
		[]byte{0b_1111_1111},
		7, 1,
		[]byte{0b_1111_1110},
	)
	test(
		[]byte{0b_1111_1111, 0b_1111_1111},
		4, 3,
		[]byte{0b_1111_0001, 0b_1111_1111},
	)
	test(
		[]byte{0b_1111_1111, 0b_1111_1111},
		5, 3,
		[]byte{0b_1111_1000, 0b_1111_1111},
	)
	test(
		[]byte{0b_1111_1111, 0b_1111_1111},
		5, 4,
		[]byte{0b_1111_1000, 0b_0111_1111},
	)
	test(
		[]byte{0b_1111_1111, 0b_1111_1111},
		7, 1,
		[]byte{0b_1111_1110, 0b_1111_1111},
	)

	test(
		[]byte{0b_1111_1111, 0b_1111_1111, 0b_1111_1111, 0b_1111_1111, 0b_1111_1111},
		8, 8,
		[]byte{0b_1111_1111, 0b_0000_0000, 0b_1111_1111, 0b_1111_1111, 0b_1111_1111},
	)
	test(
		[]byte{0b_1111_1111, 0b_1111_1111, 0b_1111_1111, 0b_1111_1111, 0b_1111_1111},
		6, 21,
		[]byte{0b_1111_1100, 0b_0000_0000, 0b_0000_0000, 0b_0001_1111, 0b_1111_1111},
	)
}

func TestClearBits_rand(t *testing.T) {
	const testIterations = 50000
	rand.Seed(time.Now().UnixNano())
	for i := 0; i < testIterations; i++ {
		dstB := mkRandBits()
		dstS := mkBitsStr(dstB)
		orgB := dupBits(dstB)
		orgS := mkBitsStr(orgB)

		nBits := rand.Intn(len(dstS)) + 1
		dst0 := rand.Intn(len(dstS) - nBits + 1)

		wantS := orgS[:dst0] + strings.Repeat("0", nBits) + orgS[dst0+nBits:]
		bitarray.ClearBits(dstB, dst0, nBits)
		dstS = mkBitsStr(dstB)

		if dstS != wantS {
			t.Errorf("unexpected result: off=%d, nBits=%d", dst0, nBits)
			t.Logf(" dst: %08b", orgB)
			t.Logf(" got: %08b", dstB)
			t.Logf("want: %s", sfmt(wantS))
			t.FailNow()
		}
		// if i < 30 {
		// 	t.Logf("pass: off=%d, nBits=%d", dst0, nBits)
		// 	t.Logf(" dst: %08b", orgB)
		// 	t.Logf(" got: %08b", dstB)
		// }
	}
}

func TestSetBits_rand(t *testing.T) {
	const testIterations = 50000
	rand.Seed(time.Now().UnixNano())
	for i := 0; i < testIterations; i++ {
		dstB := mkRandBits()
		dstS := mkBitsStr(dstB)
		orgB := dupBits(dstB)
		orgS := mkBitsStr(orgB)

		nBits := rand.Intn(len(dstS)) + 1
		dst0 := rand.Intn(len(dstS) - nBits + 1)

		wantS := orgS[:dst0] + strings.Repeat("1", nBits) + orgS[dst0+nBits:]
		bitarray.SetBits(dstB, dst0, nBits)
		dstS = mkBitsStr(dstB)

		if dstS != wantS {
			t.Errorf("unexpected result: off=%d, nBits=%d", dst0, nBits)
			t.Logf(" dst: %08b", orgB)
			t.Logf(" got: %08b", dstB)
			t.Logf("want: %s", sfmt(wantS))
			t.FailNow()
		}
		// if i < 30 {
		// 	t.Logf("pass: off=%d, nBits=%d", dst0, nBits)
		// 	t.Logf(" dst: %08b", orgB)
		// 	t.Logf(" got: %08b", dstB)
		// }
	}
}

func TestToggleBits_rand(t *testing.T) {
	const testIterations = 50000
	rand.Seed(time.Now().UnixNano())
	for i := 0; i < testIterations; i++ {
		dstB := mkRandBits()
		dstS := mkBitsStr(dstB)
		orgB := dupBits(dstB)
		orgS := mkBitsStr(orgB)

		nBits := rand.Intn(len(dstS)) + 1
		dst0 := rand.Intn(len(dstS) - nBits + 1)

		mapfn := func(r rune) rune {
			switch r {
			case '0':
				return '1'
			case '1':
				return '0'
			}
			return r
		}
		subpS := strings.Map(mapfn, dstS[dst0:dst0+nBits])
		wantS := orgS[:dst0] + subpS + orgS[dst0+nBits:]
		bitarray.ToggleBits(dstB, dst0, nBits)
		dstS = mkBitsStr(dstB)

		if dstS != wantS {
			t.Errorf("unexpected result: off=%d, nBits=%d", dst0, nBits)
			t.Logf(" dst: %08b", orgB)
			t.Logf(" got: %08b", dstB)
			t.Logf("want: %s", sfmt(wantS))
			t.FailNow()
		}
		// if i < 30 {
		// 	t.Logf("pass: off=%d, nBits=%d", dst0, nBits)
		// 	t.Logf(" dst: %08b", orgB)
		// 	t.Logf(" got: %08b", dstB)
		// }
	}
}

func TestAndBits_rand(t *testing.T) {
	const testIterations = 50000
	rand.Seed(time.Now().UnixNano())
	for i := 0; i < testIterations; i++ {
		dstB, srcB := mkRandBits(), mkRandBits()
		dstS, srcS := mkBitsStr(dstB), mkBitsStr(srcB)
		orgB := dupBits(dstB)
		orgS := mkBitsStr(orgB)

		wid := len(dstS)
		if len(srcS) < wid {
			wid = len(srcS)
		}
		nBits := rand.Intn(wid) + 1
		dst0 := rand.Intn(len(dstS) - nBits + 1)
		src0 := rand.Intn(len(srcS) - nBits + 1)

		subpS := ""
		for i := 0; i < nBits; i++ {
			switch dstS[dst0+i:dst0+i+1] + srcS[src0+i:src0+i+1] {
			case "11":
				subpS += "1"
			default:
				subpS += "0"
			}
		}
		wantS := orgS[:dst0] + subpS + orgS[dst0+nBits:]
		bitarray.AndBits(dstB, srcB, dst0, src0, nBits)
		dstS = mkBitsStr(dstB)

		if dstS != wantS {
			t.Errorf("unexpected result: dstOff=%d, srcOff=%d, nBits=%d", dst0, src0, nBits)
			t.Logf(" dst: %08b", orgB)
			t.Logf(" src: %08b", srcB)
			t.Logf(" got: %08b", dstB)
			t.Logf("want: %s", sfmt(wantS))
			t.FailNow()
		}
		// if i < 30 {
		// 	t.Logf("pass: dstOff=%d, srcOff=%d, nBits=%d", dst0, src0, nBits)
		// 	t.Logf(" dst: %08b", orgB)
		// 	t.Logf(" src: %08b", srcB)
		// 	t.Logf(" got: %08b", dstB)
		// }
	}
}

func TestOrBits_rand(t *testing.T) {
	const testIterations = 50000
	rand.Seed(time.Now().UnixNano())
	for i := 0; i < testIterations; i++ {
		dstB, srcB := mkRandBits(), mkRandBits()
		dstS, srcS := mkBitsStr(dstB), mkBitsStr(srcB)
		orgB := dupBits(dstB)
		orgS := mkBitsStr(orgB)

		wid := len(dstS)
		if len(srcS) < wid {
			wid = len(srcS)
		}
		nBits := rand.Intn(wid) + 1
		dst0 := rand.Intn(len(dstS) - nBits + 1)
		src0 := rand.Intn(len(srcS) - nBits + 1)

		subpS := ""
		for i := 0; i < nBits; i++ {
			switch dstS[dst0+i:dst0+i+1] + srcS[src0+i:src0+i+1] {
			case "00":
				subpS += "0"
			default:
				subpS += "1"
			}
		}
		wantS := orgS[:dst0] + subpS + orgS[dst0+nBits:]
		bitarray.OrBits(dstB, srcB, dst0, src0, nBits)
		dstS = mkBitsStr(dstB)

		if dstS != wantS {
			t.Errorf("unexpected result: dstOff=%d, srcOff=%d, nBits=%d", dst0, src0, nBits)
			t.Logf(" dst: %08b", orgB)
			t.Logf(" src: %08b", srcB)
			t.Logf(" got: %08b", dstB)
			t.Logf("want: %s", sfmt(wantS))
			t.FailNow()
		}
		// if i < 30 {
		// 	t.Logf("pass: dstOff=%d, srcOff=%d, nBits=%d", dst0, src0, nBits)
		// 	t.Logf(" dst: %08b", orgB)
		// 	t.Logf(" src: %08b", srcB)
		// 	t.Logf(" got: %08b", dstB)
		// }
	}
}

func TestXorBits_rand(t *testing.T) {
	const testIterations = 50000
	rand.Seed(time.Now().UnixNano())
	for i := 0; i < testIterations; i++ {
		dstB, srcB := mkRandBits(), mkRandBits()
		dstS, srcS := mkBitsStr(dstB), mkBitsStr(srcB)
		orgB := dupBits(dstB)
		orgS := mkBitsStr(orgB)

		wid := len(dstS)
		if len(srcS) < wid {
			wid = len(srcS)
		}
		nBits := rand.Intn(wid) + 1
		dst0 := rand.Intn(len(dstS) - nBits + 1)
		src0 := rand.Intn(len(srcS) - nBits + 1)

		subpS := ""
		for i := 0; i < nBits; i++ {
			switch dstS[dst0+i:dst0+i+1] + srcS[src0+i:src0+i+1] {
			case "01", "10":
				subpS += "1"
			default:
				subpS += "0"
			}
		}
		wantS := orgS[:dst0] + subpS + orgS[dst0+nBits:]
		bitarray.XorBits(dstB, srcB, dst0, src0, nBits)
		dstS = mkBitsStr(dstB)

		if dstS != wantS {
			t.Errorf("unexpected result: dstOff=%d, srcOff=%d, nBits=%d", dst0, src0, nBits)
			t.Logf(" dst: %08b", orgB)
			t.Logf(" src: %08b", srcB)
			t.Logf(" got: %08b", dstB)
			t.Logf("want: %s", sfmt(wantS))
			t.FailNow()
		}
		// if i < 30 {
		// 	t.Logf("pass: dstOff=%d, srcOff=%d, nBits=%d", dst0, src0, nBits)
		// 	t.Logf(" dst: %08b", orgB)
		// 	t.Logf(" src: %08b", srcB)
		// 	t.Logf(" got: %08b", dstB)
		// }
	}
}
