// Copyright 2019 Gregory Petrosyan <gregory.petrosyan@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package rapid

import "testing"

// https://github.com/leanovate/gopter/blob/master/commands/example_circularqueue_test.go
var gopterBug = false

// https://pkg.go.dev/github.com/leanovate/gopter/commands?tab=doc#example-package-BuggyCounter
type buggyCounter struct {
	n int
}

func (c *buggyCounter) Get() int {
	return c.n
}

func (c *buggyCounter) Inc() {
	c.n++
}

func (c *buggyCounter) Dec() {
	if c.n > 3 {
		c.n -= 2
	} else {
		c.n--
	}
}

func (c *buggyCounter) Reset() {
	c.n = 0
}

func TestStateMachine_Counter(t *testing.T) {
	t.Parallel()

	checkShrink(t, func(t *T) {
		var c buggyCounter
		var incs, decs int
		t.Repeat(map[string]func(*T){
			"Inc": func(_ *T) {
				c.Inc()
				incs++
			},
			"Dec": func(_ *T) {
				c.Dec()
				decs++
			},
			"Reset": func(_ *T) {
				c.Reset()
				incs = 0
				decs = 0
			},
			"": func(t *T) {
				if c.Get() != incs-decs {
					t.Fatalf("counter value is %v with %v incs and %v decs", c.Get(), incs, decs)
				}
			},
		})
	},
		"Inc", "Inc", "Inc", "Inc",
		"Dec",
	)
}

func TestStateMachine_Halting(t *testing.T) {
	t.Parallel()

	a := []any{"A", 0, "A", 0, "A", 0}
	for i := 0; i < 100; i++ {
		a = append(a, "A") // TODO proper shrinking of "stuck" state machines
	}

	checkShrink(t, func(t *T) {
		var a, b, c []int
		t.Repeat(map[string]func(*T){
			"A": func(t *T) {
				if len(a) == 3 {
					t.SkipNow()
				}
				a = append(a, Int().Draw(t, "a"))
			},
			"B": func(t *T) {
				if len(b) == 3 {
					t.SkipNow()
				}
				b = append(b, Int().Draw(t, "b"))
			},
			"C": func(t *T) {
				if len(c) == 3 {
					t.SkipNow()
				}
				c = append(c, Int().Draw(t, "c"))
			},
			"": func(t *T) {
				if len(a) > 3 || len(b) > 3 || len(c) > 3 {
					t.Fatalf("too many elements: %v, %v, %v", len(a), len(b), len(c))
				}
			},
		})
	}, a...)
}

// https://www.cs.tufts.edu/~nr/cs257/archive/john-hughes/quviq-testing.pdf
type buggyQueue struct {
	buf []int
	in  int
	out int
}

func newBuggyQueue(size int) *buggyQueue {
	return &buggyQueue{
		buf: make([]int, size+1),
	}
}

func (q *buggyQueue) Get() int {
	n := q.buf[q.out]
	q.out = (q.out + 1) % len(q.buf)
	return n
}

func (q *buggyQueue) Put(i int) {
	if gopterBug && q.in == 4 && i > 0 {
		q.buf[len(q.buf)-1] *= i
	}

	q.buf[q.in] = i
	q.in = (q.in + 1) % len(q.buf)
}

func (q *buggyQueue) Size() int {
	if gopterBug {
		return (q.in - q.out + len(q.buf)) % len(q.buf)
	} else {
		return (q.in - q.out) % len(q.buf)
	}
}

func queueTest(t *T) {
	size := IntRange(1, 1000).Draw(t, "size")
	q := newBuggyQueue(size)
	var state []int
	t.Repeat(map[string]func(*T){
		"Get": func(t *T) {
			if q.Size() == 0 {
				t.Skip("queue empty")
			}

			n := q.Get()
			if n != state[0] {
				t.Fatalf("got invalid value: %v vs expected %v", n, state[0])
			}
			state = state[1:]
		},
		"Put": func(t *T) {
			if q.Size() == size {
				t.Skip("queue full")
			}

			n := Int().Draw(t, "n")
			q.Put(n)
			state = append(state, n)
		},
		"": func(t *T) {
			if q.Size() != len(state) {
				t.Fatalf("queue size mismatch: %v vs expected %v", q.Size(), len(state))
			}
		},
	})
}

func TestStateMachine_Queue(t *testing.T) {
	t.Parallel()

	checkShrink(t, queueTest,
		1,
		"Put", 0,
		"Get",
		"Put", 0,
	)
}

func TestStateMachine_DiscardGarbage(t *testing.T) {
	t.Parallel()

	checkShrink(t, func(t *T) {
		var a, b []int
		t.Repeat(map[string]func(*T){
			"AddA": func(t *T) {
				if len(b) < 3 {
					t.Skip("too early")
				}
				n := Int().Draw(t, "a")
				a = append(a, n)
			},
			"AddB": func(t *T) {
				n := Int().Draw(t, "b")
				b = append(b, n)
			},
			"Whatever1": func(t *T) {
				b := Bool().Draw(t, "whatever 1/1")
				if b {
					t.Skip("arbitrary decision")
				}
				Float64().Draw(t, "whatever 1/2")
			},
			"Whatever2": func(t *T) {
				SliceOfDistinct(Int(), ID[int]).Draw(t, "whatever 2")
			},
			"Whatever3": func(t *T) {
				OneOf(SliceOf(Byte()), SliceOf(ByteMax(239))).Draw(t, "whatever 3")
			},
			"": func(t *T) {
				if len(a) > len(b) {
					t.Fatalf("`a` has outgrown `b`: %v vs %v", len(a), len(b))
				}
			},
		})
	},
		"AddB", 0,
		"AddB", 0,
		"AddB", 0,
		"AddA", 0,
		"AddA", 0,
		"AddA", 0,
		"AddA", 0,
	)
}

func BenchmarkCheckQueue(b *testing.B) {
	for i := 0; i < b.N; i++ {
		_, _, _, _, _, _, _, _ = doCheck(b, checkDeadline(nil), 100, baseSeed(), "", false, queueTest)
	}
}
