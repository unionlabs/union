package pass

import (
	"fmt"
	"math/big"

	"github.com/mmcloughlin/addchain/acc/ir"
	"github.com/mmcloughlin/addchain/internal/bigint"
	"github.com/mmcloughlin/addchain/internal/errutil"
)

// References:
//
//	[curvechains]  Brian Smith. The Most Efficient Known Addition Chains for Field Element and
//	               Scalar Inversion for the Most Popular and Most Unpopular Elliptic Curves. 2017.
//	               https://briansmith.org/ecc-inversion-addition-chains-01 (accessed June 30, 2019)

// Naming conventions described in [curvechains].
var (
	NameByteValues = NameBinaryValues(8, "_%b")
	NameXRuns      = NameBinaryRuns("x%d")
)

// ClearNames deletes all operand names.
func ClearNames(p *ir.Program) error {
	if err := CanonicalizeOperands(p); err != nil {
		return err
	}

	for _, operand := range p.Operands {
		operand.Identifier = ""
	}

	return nil
}

// NameBinaryValues assigns variable names to operands with values less than 2ᵏ.
// The identifier is determined from the format string, which should expect to
// take one *big.Int argument.
func NameBinaryValues(k int, format string) Interface {
	return NameOperands(func(_ int, x *big.Int) string {
		if x.BitLen() > k {
			return ""
		}
		return fmt.Sprintf(format, x)
	})
}

// NameBinaryRuns assigns variable names to operands with values of the form 2ⁿ
// - 1. The identifier is determined from the format string, which takes the
// length of the run as a parameter.
func NameBinaryRuns(format string) Interface {
	return NameOperands(func(_ int, x *big.Int) string {
		n := uint(x.BitLen())
		if !bigint.Equal(x, bigint.Ones(n)) {
			return ""
		}
		return fmt.Sprintf(format, n)
	})
}

// NameOperands builds a pass that names operands according to the given scheme.
func NameOperands(name func(int, *big.Int) string) Interface {
	return Func(func(p *ir.Program) error {
		// We need canonical operands, and we need to know the chain values.
		if err := Exec(p, Func(CanonicalizeOperands), Func(Eval)); err != nil {
			return err
		}

		for _, operand := range p.Operands {
			// Skip if it already has a name.
			if operand.Identifier != "" {
				continue
			}

			// Fetch referenced value.
			idx := operand.Index
			if idx >= len(p.Chain) {
				return errutil.AssertionFailure("operand index %d out of bounds", idx)
			}
			x := p.Chain[idx]

			// Set name.
			operand.Identifier = name(idx, x)
		}

		return nil
	})
}
