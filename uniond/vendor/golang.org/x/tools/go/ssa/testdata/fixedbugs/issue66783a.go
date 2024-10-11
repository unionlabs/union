//go:build ignore
// +build ignore

package issue66783a

type S[T any] struct {
	a T
}

func (s S[T]) M() {
	type A S[T]
	type B[U any] A
	_ = B[rune](s)
}

// M[int]

// panic: in (issue66783a.S[int]).M[int]:
// cannot convert term *t0 (issue66783a.S[int] [within struct{a int}])
// to type issue66783a.B[rune] [within struct{a T}] [recovered]

func M() {
	S[int]{}.M()
}
