package types

import (
	"math/rand"
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

func TestNextInflation(t *testing.T) {
	minter := DefaultInitialMinter()
	params := DefaultParams()
	blocksPerYr := math.LegacyNewDec(int64(params.BlocksPerYear))

	// Governing Mechanism:
	//    inflationRateChangePerYear = (1- BondedRatio/ GoalBonded) * MaxInflationRateChange

	tests := []struct {
		bondedRatio, setInflation, expChange math.LegacyDec
	}{
		// with 0% bonded atom supply the inflation should increase by InflationRateChange
		{math.LegacyZeroDec(), math.LegacyNewDecWithPrec(0, 2), params.InflationRateChange.Quo(blocksPerYr)},

		// 100% bonded, starting at 20% inflation and being reduced
		// (1 - (1/0.67))*(0.13/8667)
		{
			math.LegacyOneDec(), math.LegacyNewDecWithPrec(5, 2),
			math.LegacyOneDec().Sub(math.LegacyOneDec().Quo(params.GoalBonded)).Mul(params.InflationRateChange).Quo(blocksPerYr),
		},

		// 50% bonded, starting at 10% inflation and being increased
		{
			math.LegacyNewDecWithPrec(5, 1), math.LegacyNewDecWithPrec(2, 2),
			math.LegacyOneDec().Sub(math.LegacyNewDecWithPrec(5, 1).Quo(params.GoalBonded)).Mul(params.InflationRateChange).Quo(blocksPerYr),
		},

		// test 0% minimum stop (testing with 100% bonded)
		{math.LegacyOneDec(), math.LegacyNewDecWithPrec(0, 2), math.LegacyZeroDec()},
		{math.LegacyOneDec(), math.LegacyNewDecWithPrec(1, 10), math.LegacyNewDecWithPrec(-1, 10)},

		// test 5% maximum stop (testing with 0% bonded)
		{math.LegacyZeroDec(), math.LegacyNewDecWithPrec(5, 2), math.LegacyZeroDec()},
		{math.LegacyZeroDec(), math.LegacyNewDecWithPrec(499999999, 10), math.LegacyNewDecWithPrec(1, 10)},

		// perfect balance shouldn't change inflation
		{math.LegacyNewDecWithPrec(67, 2), math.LegacyNewDecWithPrec(5, 2), math.LegacyZeroDec()},
	}
	for i, tc := range tests {
		minter.Inflation = tc.setInflation

		inflation := minter.NextInflationRate(params, tc.bondedRatio)
		diffInflation := inflation.Sub(tc.setInflation)

		annualProvisions := minter.NextAnnualProvisions(params, math.NewInt(100000000000000))
		require.Equal(t, minter.Inflation.MulInt(math.NewInt(100000000000000)), annualProvisions)

		require.True(t, diffInflation.Equal(tc.expChange),
			"Test Index: %v\nDiff:  %v\nExpected: %v\n", i, diffInflation, tc.expChange)
	}
}

func TestBlockProvision(t *testing.T) {
	minter := InitialMinter(math.LegacyNewDecWithPrec(1, 1))
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
			math.NewInt(tc.expProvisions))

		require.True(t, expProvisions.IsEqual(provisions),
			"test: %v\n\tExp: %v\n\tGot: %v\n",
			i, tc.expProvisions, provisions)
	}
}

func TestValidateMinter(t *testing.T) {
	tests := []struct {
		minter Minter
		expErr bool
	}{
		{InitialMinter(math.LegacyNewDecWithPrec(1, 1)), false},
		{InitialMinter(math.LegacyNewDecWithPrec(-1, 1)), true},
		{InitialMinter(math.LegacyZeroDec()), false},
	}
	for i, tc := range tests {
		err := ValidateMinter(tc.minter)
		if tc.expErr {
			require.Error(t, err, "test: %v", i)
		} else {
			require.NoError(t, err, "test: %v", i)
		}
	}
}

func TestIsEqualMinter(t *testing.T) {
	tests := []struct {
		name  string
		a     Minter
		b     Minter
		equal bool
	}{
		{
			name: "equal minters",
			a: Minter{
				Inflation:        math.LegacyNewDec(10),
				AnnualProvisions: math.LegacyNewDec(10),
				Data:             []byte("data"),
			},
			b: Minter{
				Inflation:        math.LegacyNewDec(10),
				AnnualProvisions: math.LegacyNewDec(10),
				Data:             []byte("data"),
			},
			equal: true,
		},
		{
			name: "different inflation",
			a: Minter{
				Inflation:        math.LegacyNewDec(10),
				AnnualProvisions: math.LegacyNewDec(10),
				Data:             []byte("data"),
			},
			b: Minter{
				Inflation:        math.LegacyNewDec(100),
				AnnualProvisions: math.LegacyNewDec(10),
				Data:             []byte("data"),
			},
			equal: false,
		},
		{
			name: "different Annual Provisions",
			a: Minter{
				Inflation:        math.LegacyNewDec(10),
				AnnualProvisions: math.LegacyNewDec(10),
				Data:             []byte("data"),
			},
			b: Minter{
				Inflation:        math.LegacyNewDec(10),
				AnnualProvisions: math.LegacyNewDec(100),
				Data:             []byte("data"),
			},
			equal: false,
		},
		{
			name: "different data",
			a: Minter{
				Inflation:        math.LegacyNewDec(10),
				AnnualProvisions: math.LegacyNewDec(10),
				Data:             []byte("data"),
			},
			b: Minter{
				Inflation:        math.LegacyNewDec(10),
				AnnualProvisions: math.LegacyNewDec(10),
				Data:             []byte("no data"),
			},
			equal: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			equal := tt.a.IsEqual(tt.b)
			require.Equal(t, tt.equal, equal)
		})
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
	minter := InitialMinter(math.LegacyNewDecWithPrec(1, 1))
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
	minter := InitialMinter(math.LegacyNewDecWithPrec(1, 1))
	params := DefaultParams()
	bondedRatio := math.LegacyNewDecWithPrec(1, 1)

	// run the NextInflationRate function b.N times
	for n := 0; n < b.N; n++ {
		minter.NextInflationRate(params, bondedRatio)
	}
}

// Next annual provisions benchmarking
// BenchmarkNextAnnualProvisions-4 5000000 251 ns/op
func BenchmarkNextAnnualProvisions(b *testing.B) {
	b.ReportAllocs()
	minter := InitialMinter(math.LegacyNewDecWithPrec(1, 1))
	params := DefaultParams()
	totalSupply := math.NewInt(100000000000000)

	// run the NextAnnualProvisions function b.N times
	for n := 0; n < b.N; n++ {
		minter.NextAnnualProvisions(params, totalSupply)
	}
}
