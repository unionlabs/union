// Package addchain provides addition chain types and operations on them.
package addchain

import (
	"errors"
	"fmt"
	"math/big"

	"github.com/mmcloughlin/addchain/internal/bigint"
	"github.com/mmcloughlin/addchain/internal/bigints"
)

// References:
//
//	[efficientcompaddchain]  Bergeron, F., Berstel, J. and Brlek, S. Efficient computation of addition
//	                         chains. Journal de theorie des nombres de Bordeaux. 1994.
//	                         http://www.numdam.org/item/JTNB_1994__6_1_21_0
//	[knuth]                  Knuth, Donald E. Evaluation of Powers. The Art of Computer Programming, Volume 2
//	                         (Third Edition): Seminumerical Algorithms, chapter 4.6.3. 1997.
//	                         https://www-cs-faculty.stanford.edu/~knuth/taocp.html

// Chain is an addition chain.
type Chain []*big.Int

// New constructs the minimal chain {1}.
func New() Chain {
	return Chain{big.NewInt(1)}
}

// Int64s builds a chain from the given int64 values.
func Int64s(xs ...int64) Chain {
	return Chain(bigints.Int64s(xs...))
}

// Clone the chain.
func (c Chain) Clone() Chain {
	return bigints.Clone(c)
}

// AppendClone appends a copy of x to c.
func (c *Chain) AppendClone(x *big.Int) {
	*c = append(*c, bigint.Clone(x))
}

// End returns the last element of the chain.
func (c Chain) End() *big.Int {
	return c[len(c)-1]
}

// Ops returns all operations that produce the kth position. This could be empty
// for an invalid chain.
func (c Chain) Ops(k int) []Op {
	ops := []Op{}
	s := new(big.Int)

	// If the prefix is ascending this can be done in linear time.
	if c[:k].IsAscending() {
		for l, r := 0, k-1; l <= r; {
			s.Add(c[l], c[r])
			cmp := s.Cmp(c[k])
			if cmp == 0 {
				ops = append(ops, Op{l, r})
			}
			if cmp <= 0 {
				l++
			} else {
				r--
			}
		}
		return ops
	}

	// Fallback to quadratic.
	for i := 0; i < k; i++ {
		for j := i; j < k; j++ {
			s.Add(c[i], c[j])
			if s.Cmp(c[k]) == 0 {
				ops = append(ops, Op{i, j})
			}
		}
	}

	return ops
}

// Op returns an Op that produces the kth position.
func (c Chain) Op(k int) (Op, error) {
	ops := c.Ops(k)
	if len(ops) == 0 {
		return Op{}, fmt.Errorf("position %d is not the sum of previous entries", k)
	}
	return ops[0], nil
}

// Program produces a program that generates the chain.
func (c Chain) Program() (Program, error) {
	// Sanity checks.
	if len(c) == 0 {
		return nil, errors.New("chain empty")
	}

	if c[0].Cmp(big.NewInt(1)) != 0 {
		return nil, errors.New("chain must start with 1")
	}

	if bigints.Contains(bigint.Zero(), c) {
		return nil, errors.New("chain contains zero")
	}

	for i := 0; i < len(c); i++ {
		for j := i + 1; j < len(c); j++ {
			if bigint.Equal(c[i], c[j]) {
				return nil, fmt.Errorf("chain contains duplicate: %v at positions %d and %d", c[i], i, j)
			}
		}
	}

	// Produce the program.
	p := Program{}
	for k := 1; k < len(c); k++ {
		op, err := c.Op(k)
		if err != nil {
			return nil, err
		}
		p = append(p, op)
	}

	return p, nil
}

// Validate checks that c is in fact an addition chain.
func (c Chain) Validate() error {
	_, err := c.Program()
	return err
}

// Produces checks that c is a valid chain ending with target.
func (c Chain) Produces(target *big.Int) error {
	if err := c.Validate(); err != nil {
		return err
	}
	if c.End().Cmp(target) != 0 {
		return errors.New("chain does not end with target")
	}
	return nil
}

// Superset checks that c is a valid chain containing all the targets.
func (c Chain) Superset(targets []*big.Int) error {
	if err := c.Validate(); err != nil {
		return err
	}
	for _, target := range targets {
		if !bigints.Contains(target, c) {
			return fmt.Errorf("chain does not contain %v", target)
		}
	}
	return nil
}

// IsAscending reports whether the chain is ascending, that is if it's in sorted
// order without repeats, as defined in [knuth] Section 4.6.3 formula (11).
// Does not fully validate the chain, only that it is ascending.
func (c Chain) IsAscending() bool {
	if len(c) == 0 || !bigint.EqualInt64(c[0], 1) {
		return false
	}
	for i := 1; i < len(c); i++ {
		if c[i-1].Cmp(c[i]) >= 0 {
			return false
		}
	}
	return true
}

// Product computes the product of two addition chains. The is the "o times"
// operator defined in [efficientcompaddchain] Section 2.
func Product(a, b Chain) Chain {
	c := a.Clone()
	last := c.End()
	for _, x := range b[1:] {
		y := new(big.Int).Mul(last, x)
		c = append(c, y)
	}
	return c
}

// Plus adds x to the addition chain. This is the "o plus" operator defined in
// [efficientcompaddchain] Section 2.
func Plus(a Chain, x *big.Int) Chain {
	c := a.Clone()
	y := new(big.Int).Add(c.End(), x)
	return append(c, y)
}
