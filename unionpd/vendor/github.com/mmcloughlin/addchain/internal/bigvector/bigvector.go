// Package bigvector implements operations on vectors of immutable
// multi-precision integers.
package bigvector

import (
	"math/big"

	"github.com/mmcloughlin/addchain/internal/bigint"
)

// Vector of immutable multi-precision integers.
type Vector interface {
	// Len returns vector length.
	Len() int

	// Idx returns integer at index i. Returned integer must not be written to.
	Idx(i int) *big.Int
}

// New constructs an n-dimensional zero vector.
func New(n int) Vector {
	return make(vector, n)
}

type vector []big.Int

func (v vector) Len() int           { return len(v) }
func (v vector) Idx(i int) *big.Int { return &v[i] }

// NewBasis constructs an n-dimensional basis vector with a 1 in position i.
func NewBasis(n, i int) Vector {
	return basis{n: n, i: i}
}

// Basis implementation saves allocations by returning pre-allocated zero and
// one integers based on the index requested.
var (
	zero = bigint.Zero()
	one  = bigint.One()
)

type basis struct {
	n int
	i int
}

func (b basis) Len() int { return b.n }

func (b basis) Idx(i int) *big.Int {
	switch {
	case i >= b.n:
		panic("bigvector: index out of range")
	case i == b.i:
		return one
	default:
		return zero
	}
}

// Add vectors.
func Add(u, v Vector) Vector {
	assertsamelen(u, v)
	n := u.Len()
	w := make(vector, n)
	for i := 0; i < n; i++ {
		w[i].Add(u.Idx(i), v.Idx(i))
	}
	return w
}

// Lsh left shifts every element of the vector v.
func Lsh(v Vector, s uint) Vector {
	n := v.Len()
	w := make(vector, n)
	for i := 0; i < n; i++ {
		w[i].Lsh(v.Idx(i), s)
	}
	return w
}

// assertsamelen panics if u and v are different lengths.
func assertsamelen(u, v Vector) {
	if u.Len() != v.Len() {
		panic("bigvector: length mismatch")
	}
}
