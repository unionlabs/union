// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// go:build ignore

package testdata

type I interface {
	Foo()
}

type A struct{}

func (a A) Foo() {}

type B struct{}

func (b B) Foo() {}

type C struct{}

func (c C) Foo() {} // Test that this is not called.

type iset []I

func (i iset) All() func(func(I) bool) {
	return func(yield func(I) bool) {
		for _, v := range i {
			if !yield(v) {
				return
			}
		}
	}
}

var x = iset([]I{A{}, B{}})

func X() {
	for i := range x.All() {
		i.Foo()
	}
}

func Y() I {
	for i := range x.All() {
		return i
	}
	return nil
}

func Bar() {
	X()
	y := Y()
	y.Foo()
}

// Relevant SSA:
//func X$1(I) bool:
//	t0 = *jump$1
//	t1 = t0 == 0:int
//	if t1 goto 1 else 2
//1:
//	*jump$1 = -1:int
//	t2 = invoke arg0.Foo()
//	*jump$1 = 0:int
//	return true:bool
//2:
//	t3 = make interface{} <- string ("yield function ca...":string) interface{}
//	panic t3
//
//func All$1(yield func(I) bool):
//	t0 = *i
//	t1 = len(t0)
//	jump 1
//1:
//	t2 = phi [0: -1:int, 2: t3] #rangeindex
//	t3 = t2 + 1:int
//	t4 = t3 < t1
//	if t4 goto 2 else 3
//2:
//	t5 = &t0[t3]
//	t6 = *t5
//	t7 = yield(t6)
//	if t7 goto 1 else 4
//
//func Bar():
//	t0 = X()
//	t1 = Y()
//	t2 = invoke t1.Foo()
//	return

// WANT:
// Bar: X() -> X; Y() -> Y; invoke t1.Foo() -> A.Foo, B.Foo
// X$1: invoke arg0.Foo() -> A.Foo, B.Foo
// All$1: yield(t6) -> X$1, Y$1
