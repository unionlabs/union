package main

// Range over integers.

// Currently requires 1.22 and GOEXPERIMENT=range.

import "fmt"

func f() {
	s := "AB"
	for range 5 {
		s += s
	}
	if s != "ABABABABABABABABABABABABABABABABABABABABABABABABABABABABABABABAB" {
		panic(s)
	}

	var t []int
	for i := range 10 {
		t = append(t, i)
	}
	if got, want := fmt.Sprint(t), "[0 1 2 3 4 5 6 7 8 9]"; got != want {
		panic(got)
	}

	var u []uint
	for i := range uint(3) {
		u = append(u, i)
	}
	if got, want := fmt.Sprint(u), "[0 1 2]"; got != want {
		panic(got)
	}

	for i := range 0 {
		panic(i)
	}

	for i := range int(-1) {
		panic(i)
	}

	for _, test := range []struct {
		x    int
		b, c bool
		want string
	}{
		{-1, false, false, "[-123 -123]"},
		{0, false, false, "[-123 -123]"},
		{1, false, false, "[-123 0 333 333]"},
		{2, false, false, "[-123 0 333 1 333 333]"},
		{2, false, true, "[-123 0 222 1 222 222]"},
		{2, true, false, "[-123 0 111 111]"},
		{3, false, false, "[-123 0 333 1 333 2 333 333]"},
	} {
		got := fmt.Sprint(valueSequence(test.x, test.b, test.c))
		if got != test.want {
			panic(fmt.Sprint(test, got))
		}
	}
}

// valueSequence returns a sequence of the values of i.
// b causes an early break and c causes a continue.
func valueSequence(x int, b, c bool) []int {
	var vals []int
	var i int = -123
	vals = append(vals, i)
	for i = range x {
		vals = append(vals, i)
		if b {
			i = 111
			vals = append(vals, i)
			break
		} else if c {
			i = 222
			vals = append(vals, i)
			continue
		}
		i = 333
		vals = append(vals, i)
	}
	vals = append(vals, i)
	return vals
}

func main() { f() }
