package main

import "fmt"

func Fn[N any]() (any, any, any) {
	// Very recursive type to exercise substitution.
	type t[x any, ignored *N] struct {
		f  x
		g  N
		nx *t[x, *N]
		nn *t[N, *N]
	}
	n := t[N, *N]{}
	s := t[string, *N]{}
	i := t[int, *N]{}
	return n, s, i
}

func main() {

	sn, ss, si := Fn[string]()
	in, is, ii := Fn[int]()

	for i, t := range []struct {
		x, y any
		want bool
	}{
		{sn, ss, true},  // main.t[string;string,*string] == main.t[string;string,*string]
		{sn, si, false}, // main.t[string;string,*string] != main.t[string;int,*string]
		{sn, in, false}, // main.t[string;string,*string] != main.t[int;int,*int]
		{sn, is, false}, // main.t[string;string,*string] != main.t[int;string,*int]
		{sn, ii, false}, // main.t[string;string,*string] != main.t[int;int,*int]

		{ss, si, false}, // main.t[string;string,*string] != main.t[string;int,*string]
		{ss, in, false}, // main.t[string;string,*string] != main.t[int;int,*int]
		{ss, is, false}, // main.t[string;string,*string] != main.t[int;string,*int]
		{ss, ii, false}, // main.t[string;string,*string] != main.t[int;int,*int]

		{si, in, false}, // main.t[string;int,*string] != main.t[int;int,*int]
		{si, is, false}, // main.t[string;int,*string] != main.t[int;string,*int]
		{si, ii, false}, // main.t[string;int,*string] != main.t[int;int,*int]

		{in, is, false}, // main.t[int;int,*int] != main.t[int;string,*int]
		{in, ii, true},  // main.t[int;int,*int] == main.t[int;int,*int]

		{is, ii, false}, // main.t[int;string,*int] != main.t[int;int,*int]
	} {
		x, y, want := t.x, t.y, t.want
		if got := x == y; got != want {
			msg := fmt.Sprintf("(case %d) %T == %T. got %v. wanted %v", i, x, y, got, want)
			panic(msg)
		}
	}
}
