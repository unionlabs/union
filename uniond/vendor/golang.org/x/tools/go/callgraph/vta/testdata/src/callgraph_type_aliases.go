// Copyright 2024 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// go:build ignore

// This file is the same as callgraph_interfaces.go except for
// types J, X, Y, and Z aliasing types I, A, B, and C, resp.
// This test requires GODEBUG=gotypesalias=1 (the default in go1.23).

package testdata

type I interface {
	Foo()
}

type A struct{}

func (a A) Foo() {}

type B struct{}

func (b B) Foo() {}

type C struct{}

func (c C) Foo() {}

type J = I
type X = A
type Y = B
type Z = C

func NewY() Y {
	return Y{}
}

func Do(b bool) J {
	if b {
		return X{}
	}

	z := Z{}
	z.Foo()

	return NewY()
}

func Baz(b bool) {
	Do(b).Foo()
}

// Relevant SSA:
// func Baz(b bool):
//   t0 = Do(b)
//   t1 = invoke t0.Foo()
//   return

// func Do(b bool) I:
//    ...
//   t1 = (C).Foo(struct{}{}:Z)
//   t2 = NewY()
//   t3 = make I <- B (t2)
//   return t3

// WANT:
// Baz: Do(b) -> Do; invoke t0.Foo() -> A.Foo, B.Foo
// Do: (C).Foo(struct{}{}:Z) -> C.Foo; NewY() -> NewY
