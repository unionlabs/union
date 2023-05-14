package utils

import (
	"math/big"
	"math/bits"

	"github.com/consensys/gnark"
	"github.com/consensys/gnark-crypto/ecc"
)

var curves map[string]ecc.ID

func init() {
	curves = make(map[string]ecc.ID)
	for _, c := range gnark.Curves() {
		fHex := c.ScalarField().Text(16)
		curves[fHex] = c
	}
}

// ByteLen returns the number of bytes needed to encode 0 <= n < q
func ByteLen(q *big.Int) int {
	return len(q.Bits()) * (bits.UintSize / 8)
}

func FieldToCurve(q *big.Int) ecc.ID {
	fHex := q.Text(16)
	curve, ok := curves[fHex]
	if !ok {
		return ecc.UNKNOWN
	}
	return curve
}
