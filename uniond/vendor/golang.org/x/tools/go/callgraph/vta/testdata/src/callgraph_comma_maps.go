// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// go:build ignore

package testdata

type I interface {
	Name() string
	Foo()
}

var is = make(map[string]I)

func init() {
	register(A{})
	register(B{})
}

func register(i I) {
	is[i.Name()] = i
}

type A struct{}

func (a A) Foo()         {}
func (a A) Name() string { return "a" }

type B struct{}

func (b B) Foo()         {}
func (b B) Name() string { return "b" }

func Do(n string) {
	i, ok := is[n]
	if !ok {
		return
	}
	i.Foo()
}

func Go(n string) {
	if i, ok := is[n]; !ok {
		return
	} else {
		i.Foo()
	}
}

func To(n string) {
	var i I
	var ok bool

	if i, ok = is[n]; !ok {
		return
	}
	i.Foo()
}

func Ro(n string) {
	i := is[n]
	i.Foo()
}

// Relevant SSA:
// func Do(n string):
//        t0 = *is
//        t1 = t0[n],ok
//        t2 = extract t1 #0
//        t3 = extract t1 #1
//        if t3 goto 2 else 1
// 1:
//        return
// 2:
//        t4 = invoke t2.Foo()
//        return

// WANT:
// register: invoke i.Name() -> A.Name, B.Name
// Do: invoke t2.Foo() -> A.Foo, B.Foo
// Go: invoke t2.Foo() -> A.Foo, B.Foo
// To: invoke t2.Foo() -> A.Foo, B.Foo
// Ro: invoke t1.Foo() -> A.Foo, B.Foo
