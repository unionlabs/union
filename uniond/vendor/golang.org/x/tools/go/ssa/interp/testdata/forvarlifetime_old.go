// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.19

// goversion can be pinned to anything strictly before 1.22.

package main

import (
	"reflect"
)

func main() {
	test_init()
	bound()
	manyvars()
	nocond()
	nopost()
	address_sequences()
	post_escapes()

	// Clones from cmd/compile/internal/loopvar/testdata .
	for_complicated_esc_address()
	for_esc_address()
	for_esc_closure()
	for_esc_method()
}

// pre-go1.22 all of i will have the same address and the value of 6.
var same = func(m, n int) []*int {
	var r []*int
	for i := m; i <= n; i++ {
		r = append(r, &i)
	}
	return r
}(3, 5)

func test_init() {
	if len(same) != 3 {
		panic(same)
	}
	for i := range same {
		for j := range same {
			if !(same[i] == same[j]) {
				panic(same)
			}
		}
	}
	for i := range same {
		if *(same[i]) != 6 {
			panic(same)
		}
	}
}

func bound() {
	b := func(k int) func() int {
		var f func() int
		for i := 0; i < k; i++ {
			f = func() int { return i } // shared address will equal k.
		}
		return f
	}

	if got := b(0); got != nil {
		panic(got)
	}
	if got := b(5); got() != 5 {
		panic(got())
	}
}

func manyvars() {
	// Tests declaring many variables and having one in the middle escape.
	var f func() int
	for i, j, k, l, m, n, o, p := 7, 6, 5, 4, 3, 2, 1, 0; p < 6; l, p = l+1, p+1 {
		_, _, _, _, _, _, _, _ = i, j, k, l, m, n, o, p
		f = func() int { return l } // address *after* post updates l
	}
	if f() != 10 { // l == p+4
		panic(f())
	}
}

func nocond() {
	var c, b, e *int
	for p := 0; ; p++ {
		if p%7 == 0 {
			c = &p
			continue
		} else if p == 20 {
			b = &p
			break
		}
		e = &p
	}

	if *c != 20 {
		panic(c)
	}
	if *b != 20 {
		panic(b)
	}
	if *e != 20 {
		panic(e)
	}
}

func nopost() {
	var first, last *int
	for p := 0; p < 20; {
		if first == nil {
			first = &p
		}
		last = &p

		p++
	}

	if *first != 20 {
		panic(first)
	}
	if *last != 20 {
		panic(last)
	}
}

func address_sequences() {
	var c, b, p []*int

	cond := func(x *int) bool {
		c = append(c, x)
		return *x < 5
	}
	body := func(x *int) {
		b = append(b, x)
	}
	post := func(x *int) {
		p = append(p, x)
		(*x)++
	}
	for i := 0; cond(&i); post(&i) {
		body(&i)
	}

	if c[0] != c[1] {
		panic(c)
	}

	if !reflect.DeepEqual(c[:5], b) {
		panic(c)
	}

	if !reflect.DeepEqual(c[1:], p) {
		panic(c)
	}

	if !reflect.DeepEqual(b[1:], p[:4]) {
		panic(b)
	}
}

func post_escapes() {
	var p []*int
	post := func(x *int) {
		p = append(p, x)
		(*x)++
	}

	for i := 0; i < 5; post(&i) {
	}

	var got []int
	for _, x := range p {
		got = append(got, *x)
	}
	if want := []int{5, 5, 5, 5, 5}; !reflect.DeepEqual(got, want) {
		panic(got)
	}
}

func for_complicated_esc_address() {
	// Clone of for_complicated_esc_adress.go
	ss, sa := shared(23)
	ps, pa := private(23)
	es, ea := experiment(23)

	if ss != ps || ss != es || sa != ea || pa != 188 {
		println("shared s, a", ss, sa, "; private, s, a", ps, pa, "; experiment s, a", es, ea)
		panic("for_complicated_esc_address")
	}
}

func experiment(x int) (int, int) {
	sum := 0
	var is []*int
	for i := x; i != 1; i = i / 2 {
		for j := 0; j < 10; j++ {
			if i == j { // 10 skips
				continue
			}
			sum++
		}
		i = i*3 + 1
		if i&1 == 0 {
			is = append(is, &i)
			for i&2 == 0 {
				i = i >> 1
			}
		} else {
			i = i + i
		}
	}

	asum := 0
	for _, pi := range is {
		asum += *pi
	}

	return sum, asum
}

func private(x int) (int, int) {
	sum := 0
	var is []*int
	I := x
	for ; I != 1; I = I / 2 {
		i := I
		for j := 0; j < 10; j++ {
			if i == j { // 10 skips
				I = i
				continue
			}
			sum++
		}
		i = i*3 + 1
		if i&1 == 0 {
			is = append(is, &i)
			for i&2 == 0 {
				i = i >> 1
			}
		} else {
			i = i + i
		}
		I = i
	}

	asum := 0
	for _, pi := range is {
		asum += *pi
	}

	return sum, asum
}

func shared(x int) (int, int) {
	sum := 0
	var is []*int
	i := x
	for ; i != 1; i = i / 2 {
		for j := 0; j < 10; j++ {
			if i == j { // 10 skips
				continue
			}
			sum++
		}
		i = i*3 + 1
		if i&1 == 0 {
			is = append(is, &i)
			for i&2 == 0 {
				i = i >> 1
			}
		} else {
			i = i + i
		}
	}

	asum := 0
	for _, pi := range is {
		asum += *pi
	}
	return sum, asum
}

func for_esc_address() {
	// Clone of for_esc_address.go
	sum := 0
	var is []*int
	for i := 0; i < 10; i++ {
		for j := 0; j < 10; j++ {
			if i == j { // 10 skips
				continue
			}
			sum++
		}
		if i&1 == 0 {
			is = append(is, &i)
		}
	}

	bug := false
	if sum != 100-10 {
		println("wrong sum, expected", 90, ", saw", sum)
		bug = true
	}
	if len(is) != 5 {
		println("wrong iterations, expected ", 5, ", saw", len(is))
		bug = true
	}
	sum = 0
	for _, pi := range is {
		sum += *pi
	}
	if sum != 10+10+10+10+10 {
		println("wrong sum, expected ", 50, ", saw ", sum)
		bug = true
	}
	if bug {
		panic("for_esc_address")
	}
}

func for_esc_closure() {
	// Clone of for_esc_closure.go
	var is []func() int
	sum := 0
	for i := 0; i < 10; i++ {
		for j := 0; j < 10; j++ {
			if i == j { // 10 skips
				continue
			}
			sum++
		}
		if i&1 == 0 {
			is = append(is, func() int {
				if i%17 == 15 {
					i++
				}
				return i
			})
		}
	}

	bug := false
	if sum != 100-10 {
		println("wrong sum, expected ", 90, ", saw", sum)
		bug = true
	}
	if len(is) != 5 {
		println("wrong iterations, expected ", 5, ", saw", len(is))
		bug = true
	}
	sum = 0
	for _, f := range is {
		sum += f()
	}
	if sum != 10+10+10+10+10 {
		println("wrong sum, expected ", 50, ", saw ", sum)
		bug = true
	}
	if bug {
		panic("for_esc_closure")
	}
}

type I int

func (x *I) method() int {
	return int(*x)
}

func for_esc_method() {
	// Clone of for_esc_method.go
	sum := 0
	var is []func() int
	for i := I(0); int(i) < 10; i++ {
		for j := 0; j < 10; j++ {
			if int(i) == j { // 10 skips
				continue
			}
			sum++
		}
		if i&1 == 0 {
			is = append(is, i.method)
		}
	}

	bug := false
	if sum != 100-10 {
		println("wrong sum, expected ", 90, ", saw ", sum)
		bug = true
	}
	if len(is) != 5 {
		println("wrong iterations, expected ", 5, ", saw", len(is))
		bug = true
	}
	sum = 0
	for _, m := range is {
		sum += m()
	}
	if sum != 10+10+10+10+10 {
		println("wrong sum, expected ", 50, ", saw ", sum)
		bug = true
	}
	if bug {
		panic("for_esc_method")
	}
}
