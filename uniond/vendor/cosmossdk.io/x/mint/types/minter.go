package types

import (
	"bytes"
	"fmt"

	"cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// NewMinter returns a new Minter object with the given inflation and annual
// provisions values.
func NewMinter(inflation, annualProvisions math.LegacyDec) Minter {
	return Minter{
		Inflation:        inflation,
		AnnualProvisions: annualProvisions,
	}
}

// InitialMinter returns an initial Minter object with a given inflation value.
func InitialMinter(inflation math.LegacyDec) Minter {
	return NewMinter(
		inflation,
		math.LegacyNewDec(0),
	)
}

// DefaultInitialMinter returns a default initial Minter object for a new chain
// which uses an inflation rate of 13%.
func DefaultInitialMinter() Minter {
	return InitialMinter(
		math.LegacyNewDecWithPrec(13, 2),
	)
}

// ValidateMinter does a basic validation on minter.
func ValidateMinter(minter Minter) error {
	if minter.Inflation.IsNegative() {
		return fmt.Errorf("mint parameter Inflation should be positive, is %s",
			minter.Inflation.String())
	}
	return nil
}

// NextInflationRate returns the new inflation rate for the next block.
func (m Minter) NextInflationRate(params Params, bondedRatio math.LegacyDec) math.LegacyDec {
	// The target annual inflation rate is recalculated for each block. The inflation
	// is also subject to a rate change (positive or negative) depending on the
	// distance from the desired ratio (67%). The maximum rate change possible is
	// defined to be 13% per year, however the annual inflation is capped as between
	// 7% and 20%.

	// (1 - bondedRatio/GoalBonded) * InflationRateChange
	inflationRateChangePerYear := math.LegacyOneDec().
		Sub(bondedRatio.Quo(params.GoalBonded)).
		Mul(params.InflationRateChange)
	inflationRateChange := inflationRateChangePerYear.Quo(math.LegacyNewDec(int64(params.BlocksPerYear)))

	// adjust the new annual inflation for this next block
	inflation := m.Inflation.Add(inflationRateChange) // note inflationRateChange may be negative
	if inflation.GT(params.InflationMax) {
		inflation = params.InflationMax
	}
	if inflation.LT(params.InflationMin) {
		inflation = params.InflationMin
	}

	return inflation
}

// NextAnnualProvisions returns the annual provisions based on current total
// supply and inflation rate.
func (m Minter) NextAnnualProvisions(_ Params, totalSupply math.Int) math.LegacyDec {
	return m.Inflation.MulInt(totalSupply)
}

// BlockProvision returns the provisions for a block based on the annual
// provisions rate.
func (m Minter) BlockProvision(params Params) sdk.Coin {
	provisionAmt := m.AnnualProvisions.QuoInt(math.NewInt(int64(params.BlocksPerYear)))
	return sdk.NewCoin(params.MintDenom, provisionAmt.TruncateInt())
}

// IsEqual returns true if two minters are equal, it checks all the fields
func (m Minter) IsEqual(minter Minter) bool {
	if !m.Inflation.Equal(minter.Inflation) {
		return false
	}

	if !m.AnnualProvisions.Equal(minter.AnnualProvisions) {
		return false
	}

	if !bytes.Equal(m.Data, minter.Data) {
		return false
	}

	return true
}
