package tendermint

import (
	cmtmath "github.com/cometbft/cometbft/libs/math"
	"github.com/cometbft/cometbft/light"
)

// DefaultTrustLevel is the tendermint light client default trust level
var DefaultTrustLevel = NewFractionFromTm(light.DefaultTrustLevel)

// NewFractionFromTm returns a new Fraction instance from a tmmath.Fraction
func NewFractionFromTm(f cmtmath.Fraction) Fraction {
	return Fraction{
		Numerator:   f.Numerator,
		Denominator: f.Denominator,
	}
}

// ToTendermint converts Fraction to tmmath.Fraction
func (f Fraction) ToTendermint() cmtmath.Fraction {
	return cmtmath.Fraction{
		Numerator:   f.Numerator,
		Denominator: f.Denominator,
	}
}
