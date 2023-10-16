package bits

import (
	"math/big"

	"github.com/consensys/gnark/frontend"
)

// bitsComparatorConstant allows to use the built-in comparison against a
// constant bound. We use the direct implementation due to the added efficiency
// of directly creating constraints instead of using API.
type bitsComparatorConstant interface {
	MustBeLessOrEqCst(aBits []frontend.Variable, bound *big.Int, aForDebug frontend.Variable)
}
