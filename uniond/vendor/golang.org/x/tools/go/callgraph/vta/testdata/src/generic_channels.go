// Copyright 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// go:build ignore

package testdata

type I1 interface{}
type I2 interface{}
type I3 interface{}

func Foo[C interface{ ~chan I1 | ~chan<- I1 }](c C, j int) {
	c <- j
}

func Bar[C interface{ ~chan I2 | ~<-chan I2 }](c C) {
	x := <-c
	print(x)
}

func Baz[C interface{ ~chan I3 | ~<-chan I3 }](c C) {
	select {
	case x := <-c:
		print(x)
	default:
	}
}

// WANT:
// Local(t0) -> Channel(chan testdata.I1)
// Channel(chan testdata.I2) -> Local(t0)
// Channel(chan testdata.I3) -> Local(t0[2])
