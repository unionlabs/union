package p

// This verifies that the code works even through
// multiple levels of unexported types.

// old
var Z w

type w []x
type x []z
type z int

// new
var Z w

type w []x
type x []z

// i z: changed from int to bool
type z bool

// old
type H struct{}

func (H) M() {}

// new
// i H: changed from struct{} to interface{M()}
type H interface {
	M()
}
