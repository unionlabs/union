// Copyright 2022 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package rate_test

import (
	"fmt"
	"math"
	"testing"
	"time"

	"golang.org/x/time/rate"
)

func ExampleSometimes_once() {
	// The zero value of Sometimes behaves like sync.Once, though less efficiently.
	var s rate.Sometimes
	s.Do(func() { fmt.Println("1") })
	s.Do(func() { fmt.Println("2") })
	s.Do(func() { fmt.Println("3") })
	// Output:
	// 1
}

func ExampleSometimes_first() {
	s := rate.Sometimes{First: 2}
	s.Do(func() { fmt.Println("1") })
	s.Do(func() { fmt.Println("2") })
	s.Do(func() { fmt.Println("3") })
	// Output:
	// 1
	// 2
}

func ExampleSometimes_every() {
	s := rate.Sometimes{Every: 2}
	s.Do(func() { fmt.Println("1") })
	s.Do(func() { fmt.Println("2") })
	s.Do(func() { fmt.Println("3") })
	// Output:
	// 1
	// 3
}

func ExampleSometimes_interval() {
	s := rate.Sometimes{Interval: 1 * time.Second}
	s.Do(func() { fmt.Println("1") })
	s.Do(func() { fmt.Println("2") })
	time.Sleep(1 * time.Second)
	s.Do(func() { fmt.Println("3") })
	// Output:
	// 1
	// 3
}

func ExampleSometimes_mix() {
	s := rate.Sometimes{
		First:    2,
		Every:    2,
		Interval: 2 * time.Second,
	}
	s.Do(func() { fmt.Println("1 (First:2)") })
	s.Do(func() { fmt.Println("2 (First:2)") })
	s.Do(func() { fmt.Println("3 (Every:2)") })
	time.Sleep(2 * time.Second)
	s.Do(func() { fmt.Println("4 (Interval)") })
	s.Do(func() { fmt.Println("5 (Every:2)") })
	s.Do(func() { fmt.Println("6") })
	// Output:
	// 1 (First:2)
	// 2 (First:2)
	// 3 (Every:2)
	// 4 (Interval)
	// 5 (Every:2)
}

func TestSometimesZero(t *testing.T) {
	s := rate.Sometimes{Interval: 0}
	s.Do(func() {})
	s.Do(func() {})
}

func TestSometimesMax(t *testing.T) {
	s := rate.Sometimes{Interval: math.MaxInt64}
	s.Do(func() {})
	s.Do(func() {})
}

func TestSometimesNegative(t *testing.T) {
	s := rate.Sometimes{Interval: -1}
	s.Do(func() {})
	s.Do(func() {})
}
