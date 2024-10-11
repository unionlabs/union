// This file tests the popcnt funtions

package bitset

import (
	"testing"
)

func TestPopcntSlice(t *testing.T) {
	s := []uint64{2, 3, 5, 7, 11, 13, 17, 19, 23, 29}
	res := popcntSlice(s)
	const l uint64 = 27
	if res != l {
		t.Errorf("Wrong popcount %d != %d", res, l)
	}
}

func TestPopcntMaskSlice(t *testing.T) {
	s := []uint64{2, 3, 5, 7, 11, 13, 17, 19, 23, 29}
	m := []uint64{31, 37, 41, 43, 47, 53, 59, 61, 67, 71}
	res := popcntMaskSlice(s, m)
	const l uint64 = 9
	if res != l {
		t.Errorf("Wrong mask %d !=  %d", res, l)
	}
}

func TestPopcntAndSlice(t *testing.T) {
	s := []uint64{2, 3, 5, 7, 11, 13, 17, 19, 23, 29}
	m := []uint64{31, 37, 41, 43, 47, 53, 59, 61, 67, 71}
	res := popcntAndSlice(s, m)
	const l uint64 = 18
	if res != l {
		t.Errorf("Wrong And %d !=  %d", res, l)
	}
}

func TestPopcntOrSlice(t *testing.T) {
	s := []uint64{2, 3, 5, 7, 11, 13, 17, 19, 23, 29}
	m := []uint64{31, 37, 41, 43, 47, 53, 59, 61, 67, 71}
	res := popcntOrSlice(s, m)
	const l uint64 = 50
	if res != l {
		t.Errorf("Wrong OR %d !=  %d", res, l)
	}
}

func TestPopcntXorSlice(t *testing.T) {
	s := []uint64{2, 3, 5, 7, 11, 13, 17, 19, 23, 29}
	m := []uint64{31, 37, 41, 43, 47, 53, 59, 61, 67, 71}
	res := popcntXorSlice(s, m)
	const l uint64 = 32
	if res != l {
		t.Errorf("Wrong OR %d !=  %d", res, l)
	}
}
