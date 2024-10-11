/*
 *
 * Copyright 2021 Google LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package halfconn

import (
	"math"
	"testing"
)

func TestNewCounter(t *testing.T) {
	counter := newCounter(1)
	if got, want := counter.val, uint64(1); got != want {
		t.Errorf("counter.val = %v, want %v", got, want)
	}
	if got, want := counter.hasOverflowed, false; got != want {
		t.Errorf("counter.hasOverflowed = %v, want %v", got, want)
	}
}

func TestCounterInc(t *testing.T) {
	for _, tc := range []struct {
		desc                     string
		counter, expectedCounter uint64
		shouldOverflow           bool
	}{
		{
			desc:            "basic 1",
			counter:         0,
			expectedCounter: 1,
		},
		{
			desc:            "basic 2",
			counter:         123,
			expectedCounter: 124,
		},
		{
			desc:            "almost overflow",
			counter:         math.MaxUint64 - 1,
			expectedCounter: math.MaxUint64,
		},
		{
			desc:           "max overflow",
			counter:        math.MaxUint64,
			shouldOverflow: true,
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			c := counter{val: tc.counter}
			c.increment()
			val, err := c.value()
			if got, want := err == nil, !tc.shouldOverflow; got != want {
				t.Errorf("counter starting with %v, val()=(err=nil)=%v, want %v", tc.counter, got, want)
			}
			if got, want := val, tc.expectedCounter; err == nil && got != want {
				t.Errorf("counter starting with %v, val() = %v, want %v", tc.counter, got, want)
			}
			if got, want := tc.shouldOverflow, c.hasOverflowed; got != want {
				t.Errorf("counter starting with %v, c.hasOverflowed = %v, want %v", tc.counter, got, want)
			}
		})
	}
}

func TestCounterReset(t *testing.T) {
	for _, tc := range []struct {
		desc          string
		counter       uint64
		hasOverflowed bool
	}{
		{
			desc:          "non-zero no overflow",
			counter:       1,
			hasOverflowed: false,
		},
		{
			desc:          "zero no overflow",
			counter:       0,
			hasOverflowed: false,
		},
		{
			desc:          "non-zero has overflow",
			counter:       1,
			hasOverflowed: true,
		},
		{
			desc:          "zero has overflow",
			counter:       0,
			hasOverflowed: true,
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			c := counter{tc.counter, tc.hasOverflowed}
			c.reset()
			if got, want := c.val, uint64(0); got != want {
				t.Errorf("counter with value %v, c.value = %v, want %v", tc.counter, got, want)
			}
			if got, want := c.hasOverflowed, false; got != want {
				t.Errorf("counter with value %v, c.hasOverflowed = %v, want %v", tc.counter, got, want)
			}
		})
	}
}
