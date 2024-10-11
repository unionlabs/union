//go:build !go1.9 && amd64 && !appengine
// +build !go1.9,amd64,!appengine

// This file tests the popcnt funtions

package bitset

import (
	"testing"
)

func TestComparePopcntSlice(t *testing.T) {
	s := []uint64{2, 3, 5, 7, 11, 13, 17, 19, 23, 29}
	resGo := popcntSliceGo(s)
	resAsm := popcntSliceAsm(s)
	if resGo != resAsm {
		t.Errorf("The implementations are different: GO %d != ASM %d", resGo, resAsm)
	}
}

func TestComparePopcntMaskSlice(t *testing.T) {
	s := []uint64{2, 3, 5, 7, 11, 13, 17, 19, 23, 29}
	m := []uint64{31, 37, 41, 43, 47, 53, 59, 61, 67, 71}
	resGo := popcntMaskSliceGo(s, m)
	resAsm := popcntMaskSliceAsm(s, m)
	if resGo != resAsm {
		t.Errorf("The implementations are different: GO %d != ASM %d", resGo, resAsm)
	}
}

func TestComparePopcntAndSlice(t *testing.T) {
	s := []uint64{2, 3, 5, 7, 11, 13, 17, 19, 23, 29}
	m := []uint64{31, 37, 41, 43, 47, 53, 59, 61, 67, 71}
	resGo := popcntAndSliceGo(s, m)
	resAsm := popcntAndSliceAsm(s, m)
	if resGo != resAsm {
		t.Errorf("The implementations are different: GO %d != ASM %d", resGo, resAsm)
	}
}

func TestComparePopcntOrSlice(t *testing.T) {
	s := []uint64{2, 3, 5, 7, 11, 13, 17, 19, 23, 29}
	m := []uint64{31, 37, 41, 43, 47, 53, 59, 61, 67, 71}
	resGo := popcntOrSliceGo(s, m)
	resAsm := popcntOrSliceAsm(s, m)
	if resGo != resAsm {
		t.Errorf("The implementations are different: GO %d != ASM %d", resGo, resAsm)
	}
}

func TestComparePopcntXorSlice(t *testing.T) {
	s := []uint64{2, 3, 5, 7, 11, 13, 17, 19, 23, 29}
	m := []uint64{31, 37, 41, 43, 47, 53, 59, 61, 67, 71}
	resGo := popcntXorSliceGo(s, m)
	resAsm := popcntXorSliceAsm(s, m)
	if resGo != resAsm {
		t.Errorf("The implementations are different: GO %d != ASM %d", resGo, resAsm)
	}
}
