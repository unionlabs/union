package c

func instantiated[X any](x *X) int {
	if x == nil {
		print(*x) // want "nil dereference in load"
	}
	return 1
}

var g int

func init() {
	g = instantiated[int](&g)
}

// -- issue 66835 --

type Empty1 any
type Empty2 any

// T may be instantiated with an interface type, so any(x) may be nil.
func TypeParamInterface[T error](x T) {
	if any(x) == nil {
		print()
	}
}

// T may not be instantiated with an interface type, so any(x) is non-nil
func TypeParamTypeSetWithInt[T interface {
	error
	int
}](x T) {
	if any(x) == nil { // want "impossible condition: non-nil == nil"
		print()
	}
}

func TypeParamUnionEmptyEmpty[T Empty1 | Empty2](x T) {
	if any(x) == nil {
		print()
	}
}

func TypeParamUnionEmptyInt[T Empty1 | int](x T) {
	if any(x) == nil {
		print()
	}
}

func TypeParamUnionStringInt[T string | int](x T) {
	if any(x) == nil { // want "impossible condition: non-nil == nil"
		print()
	}
}
