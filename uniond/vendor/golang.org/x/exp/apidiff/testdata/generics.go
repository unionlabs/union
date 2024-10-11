package p

//// Generics

// old
type G[T any] []T

// new
// OK: param name change
type G[A any] []A

// old
type GM[A, B comparable] map[A]B

// new
// i GM: changed from map[A]B to map[B]A
type GM[A, B comparable] map[B]A

// old
type GT[V any] struct {
}

func (GT[V]) M(*GT[V]) {}

// new
// OK
type GT[V any] struct {
}

func (GT[V]) M(*GT[V]) {}

// old
type GT2[V any] struct {
}

func (GT2[V]) M(*GT2[V]) {}

// new
// i GT2: changed from GT2[V any] to GT2[V comparable]
type GT2[V comparable] struct {
}

func (GT2[V]) M(*GT2[V]) {}

// both
type custom interface {
	int
}

type GT3[E custom] map[E]int

// Instantiated types:
// Two instantiations of generic types
// with different type parameters are different.

// both
type H[T any] []T

// old
var V1 H[int]

type T int

var V2 H[T]

// new
// i V1: changed from H[int] to H[bool]
var V1 H[bool]

// i T: changed from int to bool
type T bool

// OK: we reported on T, so we don't need to here.
var V2 H[T]

// old
type t int

// new
// i t: changed from int to byte
type t byte

// both
var V3 H[t]
