// Copyright 2011 Huan Du. All rights reserved.
// Licensed under the MIT license that can be found in the LICENSE file.

package skiplist

import (
	"testing"

	"github.com/huandu/go-assert"
)

func TestKeyScore(t *testing.T) {
	a := assert.New(t)
	keys := []interface{}{
		byte(11), -12, int(13), int8(14), int16(-15), int32(16), int64(-17), rune(18),
		uint(20), uint8(21), uint16(22), uint32(23), uint64(24), uintptr(25),
		30.05, float32(30.1), float64(30.2),
		"abcde", "abcdefg", "abcdefghijk",
		[]byte("abcde"), []byte("abcdefg"), []byte("abcdefghijk"),
		[]int{1, 2, 3}, nil,
	}
	scores := []float64{
		11, -12, 13, 14, -15, 16, -17, 18,
		20, 21, 22, 23, 24, 25,
		30.05, float64(float32(30.1)), 30.2,
		0x6162636465000000, 0x6162636465666768, 0x6162636465666768,
		0x6162636465000000, 0x6162636465666768, 0x6162636465666768,
		0, 0,
	}

	for i, k := range keys {
		score := CalcScore(k)

		a.Use(&i, &k, &score)
		a.Equal(score, scores[i])
	}
}

type testScorable int

func (s testScorable) Score() float64 {
	return float64(s) * 2
}

func TestScarable(t *testing.T) {
	a := assert.New(t)

	s1 := testScorable(12)
	a.Equal(CalcScore(s1), 2*12.0)

	s2 := int(s1)
	a.Equal(CalcScore(s2), 12.0)
}
