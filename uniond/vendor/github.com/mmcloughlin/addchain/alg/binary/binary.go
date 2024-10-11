// Package binary implements generic binary addition chain algorithms.
package binary

import (
	"math/big"

	"github.com/mmcloughlin/addchain"
	"github.com/mmcloughlin/addchain/internal/bigint"
)

// References:
//
//	[hehcc:exp]  Christophe Doche. Exponentiation. Handbook of Elliptic and Hyperelliptic Curve
//	             Cryptography, chapter 9. 2006.
//	             http://koclab.cs.ucsb.edu/teaching/ecc/eccPapers/Doche-ch09.pdf

// RightToLeft builds a chain algorithm for the right-to-left binary method,
// akin to [hehcc:exp] Algorithm 9.2.
type RightToLeft struct{}

func (RightToLeft) String() string { return "binary_right_to_left" }

// FindChain applies the right-to-left binary method to n.
func (RightToLeft) FindChain(n *big.Int) (addchain.Chain, error) {
	c := addchain.Chain{}
	b := new(big.Int).Set(n)
	d := bigint.One()
	var x *big.Int
	for bigint.IsNonZero(b) {
		c.AppendClone(d)
		if b.Bit(0) == 1 {
			if x == nil {
				x = bigint.Clone(d)
			} else {
				x.Add(x, d)
				c.AppendClone(x)
			}
		}
		b.Rsh(b, 1)
		d.Lsh(d, 1)
	}
	return c, nil
}
