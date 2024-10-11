package const_length

import "math"

//go:generate mockgen -package const_length -destination mock.go -source input.go

const C = 2

type I interface {
	Foo() [C]int
	Bar() [2]int
	Baz() [math.MaxInt8]int
}
