// Copyright 2019 Gregory Petrosyan <gregory.petrosyan@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package rapid

import (
	"fmt"
	"math"
	"math/bits"
	"sort"
	"strconv"
	"testing"
)

const shrinkTestRuns = 10

func TestShrink_IntCmp(t *testing.T) {
	t.Parallel()

	ref := []struct {
		gt bool
		a  int
		b  int
		eq bool
	}{
		{true, 1000000, 1000001, false},
		{true, -1000000, 0, false},
		{true, 0, 0, true},
		{false, 1000000, 0, false},
		{false, -1000000, -1000001, false},
		{false, 0, 0, true},
	}

	for _, r := range ref {
		t.Run(fmt.Sprintf("%v", r), func(t *testing.T) {
			checkShrink(t, func(t *T) {
				i := Int().Draw(t, "i")
				if ((r.gt && i > r.a) || (!r.gt && i < r.a)) || (r.eq && i == r.a) {
					t.Fail()
				}
			}, r.b)
		})
	}
}

func TestShrink_FloatCmp(t *testing.T) {
	t.Parallel()

	type cmp struct {
		gt bool
		a  float64
		b  float64
		eq bool
	}

	ref := []cmp{
		{true, 1000000, 1000000.5, false},
		{true, math.Pi, 3.5, false},
		{true, 1, 1, true},
		{true, -1000000, 1, false},
		{false, -1000000, -1000000.5, false},
		{false, -math.E, -2.75, false},
	}
	if *flaky {
		ref = append(ref, cmp{false, 0, -1, true}) // sometimes we end up at exactly 0
	}

	for _, r := range ref {
		t.Run(fmt.Sprintf("%v", r), func(t *testing.T) {
			checkShrink(t, func(t *T) {
				f := Float64().Draw(t, "f")
				if ((r.gt && f > r.a) || (!r.gt && f < r.a)) || (r.eq && f == r.a) {
					t.Fail()
				}
			}, r.b)
		})
	}
}

func TestShrink_IntSliceNElemsGt(t *testing.T) {
	t.Parallel()

	checkShrink(t, func(t *T) {
		s := SliceOf(Int()).Draw(t, "s")
		n := 0
		for _, i := range s {
			if i > 1000000 {
				n++
			}
		}
		if n > 1 {
			t.Fail()
		}
	}, []int{1000001, 1000001})
}

func TestShrink_IntSliceElemGe(t *testing.T) {
	t.Parallel()

	checkShrink(t, func(t *T) {
		s := SliceOfN(Int(), 1, -1).Draw(t, "s")
		ix := IntRange(0, len(s)-1).Draw(t, "ix")

		if s[ix] >= 100 {
			t.Fail()
		}
	}, []int{100}, 0)
}

func TestShrink_IntSliceElemSpanGe(t *testing.T) {
	t.Parallel()

	checkShrink(t, func(t *T) {
		s := SliceOfN(Int(), 4, -1).Draw(t, "s")
		if len(s)%3 == 1 && s[len(s)-1] >= 100 {
			t.Fail()
		}
	}, []int{0, 0, 0, 100})
}

func TestShrink_IntSliceNoDuplicates(t *testing.T) {
	t.Parallel()

	checkShrink(t, func(t *T) {
		s := SliceOfN(IntMin(1), 5, -1).Draw(t, "s")
		sort.Ints(s)
		last := 0
		for _, i := range s {
			if i == last {
				return
			}
			last = i
		}
		t.Fail()
	}, []int{1, 2, 3, 4, 5})
}

func TestShrink_String(t *testing.T) {
	t.Parallel()

	checkShrink(t, func(t *T) {
		s1 := String().Draw(t, "s1")
		s2 := String().Draw(t, "s2")
		if len(s1) > len(s2) {
			t.Fail()
		}
	}, "A", "")
}

func TestShrink_StringOf(t *testing.T) {
	t.Parallel()

	checkShrink(t, func(t *T) {
		s1 := StringOf(RuneFrom([]rune{'X', 'Y', 'Z'})).Draw(t, "s1")
		s2 := StringOf(RuneFrom([]rune{'U', 'V', 'W'})).Draw(t, "s2")
		if len(s1) > len(s2) {
			t.Fail()
		}
	}, "X", "")
}

func TestMinimize_UnsetBits(t *testing.T) {
	t.Parallel()

	Check(t, func(t *T) {
		mask := Uint64Range(0, math.MaxUint64).Draw(t, "mask")
		best := minimize(math.MaxUint64, func(x uint64, s string) bool { return x&mask == mask })
		if best != mask {
			t.Fatalf("unset to %v instead of %v", bin(best), bin(mask))
		}
	})
}

func TestMinimize_SortBits(t *testing.T) {
	t.Parallel()

	Check(t, func(t *T) {
		u := Uint64Range(0, math.MaxUint64).Draw(t, "u")
		n := bits.OnesCount64(u)
		v := uint64(1<<uint(n) - 1)

		best := minimize(u, func(x uint64, s string) bool { return bits.OnesCount64(x) == n })
		if best != v {
			t.Fatalf("minimized to %v instead of %v (%v bits set)", bin(best), bin(v), n)
		}
	})
}

func TestMinimize_LowerBound(t *testing.T) {
	t.Parallel()

	Check(t, func(t *T) {
		min := Uint64().Draw(t, "min")
		u := Uint64Min(min).Draw(t, "u")

		best := minimize(u, func(x uint64, s string) bool { return x >= min })
		if best != min {
			t.Fatalf("found %v instead of %v", best, min)
		}
	})
}

func checkShrink(t *testing.T, prop func(*T), draws ...any) {
	t.Helper()

	for i := 0; i < shrinkTestRuns; i++ {
		t.Run(strconv.Itoa(i), func(t *testing.T) {
			t.Helper()

			_, _, _, seed, _, buf, err1, err2 := doCheck(t, checkDeadline(nil), 100, baseSeed(), "", false, prop)
			if seed != 0 && err1 == nil && err2 == nil {
				t.Fatalf("shrink test did not fail (seed %v)", seed)
			}
			if traceback(err1) != traceback(err2) {
				t.Fatalf("flaky shrink test (seed %v)\nTraceback (%v):\n%vOriginal traceback (%v):\n%v", seed, err2, traceback(err2), err1, traceback(err1))
			}

			nt := newT(t, newBufBitStream(buf, false), false, nil, draws...)
			_ = checkOnce(nt, prop)
			if nt.draws != len(draws) {
				t.Fatalf("different number of draws: %v vs expected %v", nt.draws, len(draws))
			}
		})
	}
}

func bin(u uint64) string {
	return "0b" + strconv.FormatUint(u, 2)
}
