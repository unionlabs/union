//go:build !go1.9 && amd64 && !appengine
// +build !go1.9,amd64,!appengine

// This file tests the popcnt funtions

package bitset

import (
	"testing"
)

func TestPopcntSliceCond(t *testing.T) {
	s := []uint64{2, 3, 5, 7, 11, 13, 17, 19, 23, 29}
	oldUseAsm := useAsm
	defer func() { useAsm = oldUseAsm }()
	useAsm = false
	resGo := popcntSlice(s)
	useAsm = (true && oldUseAsm)
	resAsm := popcntSlice(s)
	if resGo != resAsm {
		t.Errorf("The implementations are different: GO %d != ASM %d", resGo, resAsm)
	}
}

func TestPopcntMaskSliceCond(t *testing.T) {
	s := []uint64{2, 3, 5, 7, 11, 13, 17, 19, 23, 29}
	m := []uint64{31, 37, 41, 43, 47, 53, 59, 61, 67, 71}
	oldUseAsm := useAsm
	defer func() { useAsm = oldUseAsm }()
	useAsm = false
	resGo := popcntMaskSlice(s, m)
	useAsm = (true && oldUseAsm)
	resAsm := popcntMaskSlice(s, m)
	if resGo != resAsm {
		t.Errorf("The implementations are different: GO %d != ASM %d", resGo, resAsm)
	}
}

func TestPopcntAndSliceCond(t *testing.T) {
	s := []uint64{2, 3, 5, 7, 11, 13, 17, 19, 23, 29}
	m := []uint64{31, 37, 41, 43, 47, 53, 59, 61, 67, 71}
	oldUseAsm := useAsm
	defer func() { useAsm = oldUseAsm }()
	useAsm = false
	resGo := popcntAndSlice(s, m)
	useAsm = (true && oldUseAsm)
	resAsm := popcntAndSlice(s, m)
	if resGo != resAsm {
		t.Errorf("The implementations are different: GO %d != ASM %d", resGo, resAsm)
	}
}

func TestPopcntOrSliceCond(t *testing.T) {
	s := []uint64{2, 3, 5, 7, 11, 13, 17, 19, 23, 29}
	m := []uint64{31, 37, 41, 43, 47, 53, 59, 61, 67, 71}
	oldUseAsm := useAsm
	defer func() { useAsm = oldUseAsm }()
	useAsm = false
	resGo := popcntOrSlice(s, m)
	useAsm = (true && oldUseAsm)
	resAsm := popcntOrSlice(s, m)
	if resGo != resAsm {
		t.Errorf("The implementations are different: GO %d != ASM %d", resGo, resAsm)
	}
}

func TestPopcntXorSliceCond(t *testing.T) {
	s := []uint64{2, 3, 5, 7, 11, 13, 17, 19, 23, 29}
	m := []uint64{31, 37, 41, 43, 47, 53, 59, 61, 67, 71}
	oldUseAsm := useAsm
	defer func() { useAsm = oldUseAsm }()
	useAsm = false
	resGo := popcntXorSlice(s, m)
	useAsm = (true && oldUseAsm)
	resAsm := popcntXorSlice(s, m)
	if resGo != resAsm {
		t.Errorf("The implementations are different: GO %d != ASM %d", resGo, resAsm)
	}
}
