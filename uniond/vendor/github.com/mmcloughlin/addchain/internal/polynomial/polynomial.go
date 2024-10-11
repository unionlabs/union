// Package polynomial provides a polynomial type.
package polynomial

import (
	"fmt"
	"math/big"
	"strings"

	"github.com/mmcloughlin/addchain/internal/bigint"
)

// Term is the term A*xᴺ in a polynomial.
type Term struct {
	A int64
	N uint
}

func (t Term) String() string {
	return Polynomial{t}.String()
}

// Evaluate term at x.
func (t Term) Evaluate(x *big.Int) *big.Int {
	n := big.NewInt(int64(t.N))
	y := new(big.Int).Exp(x, n, nil)
	return y.Mul(big.NewInt(t.A), y)
}

// Polynomial is a single-variable polynomial. Terms are expected (but not
// required) to be in increasing order of exponent.
type Polynomial []Term

func (p Polynomial) String() string {
	return p.Format("x")
}

// Format polynomial as a string, using v to represent the variable.
func (p Polynomial) Format(v string) string {
	s := ""
	for i := len(p) - 1; i >= 0; i-- {
		t := p[i]

		if t.N == 0 {
			s += fmt.Sprintf("%+d", t.A)
			continue
		}

		switch t.A {
		case 1:
			s += "+"
		case -1:
			s += "-"
		default:
			s += fmt.Sprintf("%+d", t.A)
		}

		s += v

		if t.N > 1 {
			s += fmt.Sprintf("^%d", t.N)
		}
	}
	return strings.TrimPrefix(s, "+")
}

// Degree returns the degree of p, namely the highest exponent.
func (p Polynomial) Degree() uint {
	n := uint(0)
	for _, t := range p {
		if t.N > n {
			n = t.N
		}
	}
	return n
}

// Evaluate p at x.
func (p Polynomial) Evaluate(x *big.Int) *big.Int {
	y := bigint.Zero()
	for _, t := range p {
		y.Add(y, t.Evaluate(x))
	}
	return y
}
