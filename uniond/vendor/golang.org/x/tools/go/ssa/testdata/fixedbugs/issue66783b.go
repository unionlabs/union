//go:build ignore
// +build ignore

package issue66783b

type I1[T any] interface {
	M(T)
}

type I2[T any] I1[T]

func foo[T any](i I2[T]) {
	_ = i.M
}

type S[T any] struct{}

func (s S[T]) M(t T) {}

func M2() {
	foo[int](I2[int](S[int]{}))
}
