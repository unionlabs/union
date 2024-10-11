// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"errors"
	"fmt"
	"math/rand"
	"testing"
	"time"

	"github.com/tunabay/go-bitarray"
)

func TestBitArray_Iterate_rand(t *testing.T) {
	const testIterations = 10000
	rand.Seed(time.Now().UnixNano())

	for i := 0; i < testIterations; i++ {
		var ba *bitarray.BitArray
		switch rand.Intn(20) {
		case 0, 1, 2, 3:
			nBits := rand.Intn(64)
			ba = bitarray.PseudoRand(nBits, nil)
		case 4, 5, 6, 7:
			nBits := 8*(1+rand.Intn(64)) - 1 + rand.Intn(3)
			ba = bitarray.PseudoRand(nBits, nil)
		case 8:
			nBits := rand.Intn(1024)
			ba = bitarray.NewZeroFilled(nBits)
		case 9:
			nBits := rand.Intn(1024)
			ba = bitarray.NewOneFilled(nBits)
		default:
			nBits := rand.Intn(1024)
			ba = bitarray.PseudoRand(nBits, nil)
		}
		lim := ba.Len()
		if rand.Intn(20) == 0 {
			lim = ba.Len() - rand.Intn(ba.Len()>>1+1)
		}
		var myErr error
		if lim < ba.Len() && rand.Intn(5) == 0 {
			myErr = fmt.Errorf("custom error %d", i)
		}

		baS := ba.String()

		fn := func(i, b int) error {
			t.Helper()
			if baS[i] != '0'+byte(b) {
				return fmt.Errorf("%d: got %d, want %c", i, b, baS[i])
			}
			if lim < i {
				if myErr == nil {
					return bitarray.BreakIteration
				}
				return fmt.Errorf("test error: %d: %w", i, myErr)
			}
			return nil
		}
		if err := ba.Iterate(fn); err != nil {
			if myErr == nil || !errors.Is(err, myErr) {
				t.Errorf("unexpected error: %s", err)
				t.FailNow()
			}
		}
		if err := ba.ZExpand().Iterate(fn); err != nil {
			if myErr == nil || !errors.Is(err, myErr) {
				t.Errorf("unexpected bit (e): %s", err)
				t.FailNow()
			}
		}
		// if i < 32 {
		// 	t.Logf("pass: %#b", ba)
		// }
	}
}
