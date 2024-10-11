package test

type embedded struct{}

type S struct{ embedded }

func (_ S) M() {}

type C interface {
	M()
	S
}

func G[T C]() {
	t := T{embedded{}}
	t.M()
}

func F() {
	G[S]()
}

// WANT:
// F: G[testdata.S]() -> G[testdata.S]
// G[testdata.S]: (S).M(t2) -> S.M
// S.M: (testdata.S).M(t1) -> S.M
