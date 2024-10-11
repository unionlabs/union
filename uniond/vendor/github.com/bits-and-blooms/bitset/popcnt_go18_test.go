// This file tests the popcnt funtions

package bitset

import (
	"testing"
)

func TestPopcntSliceGo(t *testing.T) {
	s := []uint64{2, 3, 5, 7, 11, 13, 17, 19, 23, 29}
	res := popcntSliceGo(s)
	const l uint64 = 27
	if res != l {
		t.Errorf("Wrong popcount %d != %d", res, l)
	}
}

func TestPopcntMaskSliceGo(t *testing.T) {
	s := []uint64{2, 3, 5, 7, 11, 13, 17, 19, 23, 29}
	m := []uint64{31, 37, 41, 43, 47, 53, 59, 61, 67, 71}
	res := popcntMaskSliceGo(s, m)
	const l uint64 = 9
	if res != l {
		t.Errorf("Wrong mask %d !=  %d", res, l)
	}
}

func TestPopcntAndSliceGo(t *testing.T) {
	s := []uint64{2, 3, 5, 7, 11, 13, 17, 19, 23, 29}
	m := []uint64{31, 37, 41, 43, 47, 53, 59, 61, 67, 71}
	res := popcntAndSliceGo(s, m)
	const l uint64 = 18
	if res != l {
		t.Errorf("Wrong And %d !=  %d", res, l)
	}
}

func TestPopcntOrSliceGo(t *testing.T) {
	s := []uint64{2, 3, 5, 7, 11, 13, 17, 19, 23, 29}
	m := []uint64{31, 37, 41, 43, 47, 53, 59, 61, 67, 71}
	res := popcntOrSliceGo(s, m)
	const l uint64 = 50
	if res != l {
		t.Errorf("Wrong OR %d !=  %d", res, l)
	}
}

func TestPopcntXorSliceGo(t *testing.T) {
	s := []uint64{2, 3, 5, 7, 11, 13, 17, 19, 23, 29}
	m := []uint64{31, 37, 41, 43, 47, 53, 59, 61, 67, 71}
	res := popcntXorSliceGo(s, m)
	const l uint64 = 32
	if res != l {
		t.Errorf("Wrong OR %d !=  %d", res, l)
	}
}
