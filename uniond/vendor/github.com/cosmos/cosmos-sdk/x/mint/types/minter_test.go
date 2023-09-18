package types

import (
	"math/rand"
	"testing"

	"cosmossdk.io/math"
	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

func TestNextInflation(t *testing.T) {
	minter := DefaultInitialMinter()
	params := DefaultParams()
	blocksPerYr := math.LegacyNewDec(int64(params.BlocksPerYear))

	// Governing Mechanism:
	//    inflationRateChangePerYear = (1- BondedRatio/ GoalBonded) * MaxInflationRateChange

	tests := []struct {
		bondedRatio, setInflation, expChange sdk.Dec
	}{
		// with 0% bonded atom supply the inflation should increase by InflationRateChange
		{math.LegacyZeroDec(), sdk.NewDecWithPrec(7, 2), params.InflationRateChange.Quo(blocksPerYr)},

		// 100% bonded, starting at 20% inflation and being reduced
		// (1 - (1/0.67))*(0.13/8667)
		{
			math.LegacyOneDec(), sdk.NewDecWithPrec(20, 2),
			math.LegacyOneDec().Sub(math.LegacyOneDec().Quo(params.GoalBonded)).Mul(params.InflationRateChange).Quo(blocksPerYr),
		},

		// 50% bonded, starting at 10% inflation and being increased
		{
			sdk.NewDecWithPrec(5, 1), sdk.NewDecWithPrec(10, 2),
			math.LegacyOneDec().Sub(sdk.NewDecWithPrec(5, 1).Quo(params.GoalBonded)).Mul(params.InflationRateChange).Quo(blocksPerYr),
		},

		// test 7% minimum stop (testing with 100% bonded)
		{math.LegacyOneDec(), sdk.NewDecWithPrec(7, 2), math.LegacyZeroDec()},
		{math.LegacyOneDec(), sdk.NewDecWithPrec(700000001, 10), sdk.NewDecWithPrec(-1, 10)},

		// test 20% maximum stop (testing with 0% bonded)
		{math.LegacyZeroDec(), sdk.NewDecWithPrec(20, 2), math.LegacyZeroDec()},
		{math.LegacyZeroDec(), sdk.NewDecWithPrec(1999999999, 10), sdk.NewDecWithPrec(1, 10)},

		// perfect balance shouldn't change inflation
		{sdk.NewDecWithPrec(67, 2), sdk.NewDecWithPrec(15, 2), math.LegacyZeroDec()},
	}
	for i, tc := range tests {
		minter.Inflation = tc.setInflation

		inflation := minter.NextInflationRate(params, tc.bondedRatio)
		diffInflation := inflation.Sub(tc.setInflation)

		require.True(t, diffInflation.Equal(tc.expChange),
			"Test Index: %v\nDiff:  %v\nExpected: %v\n", i, diffInflation, tc.expChange)
	}
}

func TestBlockProvision(t *testing.T) {
	minter := InitialMinter(sdk.NewDecWithPrec(1, 1))
	params := DefaultParams()

	secondsPerYear := int64(60 * 60 * 8766)

	tests := []struct {
		annualProvisions int64
		expProvisions    int64
	}{
		{secondsPerYear / 5, 1},
		{secondsPerYear/5 + 1, 1},
		{(secondsPerYear / 5) * 2, 2},
		{(secondsPerYear / 5) / 2, 0},
	}
	for i, tc := range tests {
		minter.AnnualProvisions = math.LegacyNewDec(tc.annualProvisions)
		provisions := minter.BlockProvision(params)

		expProvisions := sdk.NewCoin(params.MintDenom,
			sdk.NewInt(tc.expProvisions))

		require.True(t, expProvisions.IsEqual(provisions),
			"test: %v\n\tExp: %v\n\tGot: %v\n",
			i, tc.expProvisions, provisions)
	}
}

// Benchmarking :)
// previously using math.Int operations:
// BenchmarkBlockProvision-4 5000000 220 ns/op
//
// using math.LegacyDec operations: (current implementation)
// BenchmarkBlockProvision-4 3000000 429 ns/op
func BenchmarkBlockProvision(b *testing.B) {
	b.ReportAllocs()
	minter := InitialMinter(sdk.NewDecWithPrec(1, 1))
	params := DefaultParams()

	s1 := rand.NewSource(100)
	r1 := rand.New(s1)
	minter.AnnualProvisions = math.LegacyNewDec(r1.Int63n(1000000))

	// run the BlockProvision function b.N times
	for n := 0; n < b.N; n++ {
		minter.BlockProvision(params)
	}
}

// Next inflation benchmarking
// BenchmarkNextInflation-4 1000000 1828 ns/op
func BenchmarkNextInflation(b *testing.B) {
	b.ReportAllocs()
	minter := InitialMinter(sdk.NewDecWithPrec(1, 1))
	params := DefaultParams()
	bondedRatio := sdk.NewDecWithPrec(1, 1)

	// run the NextInflationRate function b.N times
	for n := 0; n < b.N; n++ {
		minter.NextInflationRate(params, bondedRatio)
	}
}

// Next annual provisions benchmarking
// BenchmarkNextAnnualProvisions-4 5000000 251 ns/op
func BenchmarkNextAnnualProvisions(b *testing.B) {
	b.ReportAllocs()
	minter := InitialMinter(sdk.NewDecWithPrec(1, 1))
	params := DefaultParams()
	totalSupply := sdk.NewInt(100000000000000)

	// run the NextAnnualProvisions function b.N times
	for n := 0; n < b.N; n++ {
		minter.NextAnnualProvisions(params, totalSupply)
	}
}
