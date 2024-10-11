package simulation_test

import (
	"math/rand"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/simulation"
)

func TestRandSubsetCoins(t *testing.T) {
	t.Parallel()
	tests := []struct {
		name  string
		r     *rand.Rand
		coins sdk.Coins
	}{
		{"seed=1", rand.New(rand.NewSource(1)), mustParseCoins("100stake,2testtoken")},
		{"seed=50", rand.New(rand.NewSource(50)), mustParseCoins("100stake,2testtoken")},
		{"seed=99", rand.New(rand.NewSource(99)), mustParseCoins("100stake,2testtoken")},
		{"zero coins", rand.New(rand.NewSource(99)), sdk.Coins{}},
		{"too small amount", rand.New(rand.NewSource(99)), sdk.Coins{sdk.Coin{Denom: "aaa", Amount: math.NewInt(0)}}},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := simulation.RandSubsetCoins(tt.r, tt.coins)
			gotStringRep := got.String()
			sortedStringRep := got.Sort().String()
			require.Equal(t, gotStringRep, sortedStringRep)
		})
	}
}

func TestRandStringOfLength(t *testing.T) {
	t.Parallel()
	r := rand.New(rand.NewSource(time.Now().Unix()))
	tests := []struct {
		name string
		n    int
		want int
	}{
		{"0-size", 0, 0},
		{"10-size", 10, 10},
		{"1_000_000-size", 1_000_000, 1_000_000},
	}
	for _, tt := range tests {
		tc := tt
		t.Run(tc.name, func(t *testing.T) {
			got := simulation.RandStringOfLength(r, tc.n)
			require.Equal(t, tc.want, len(got))
		})
	}
}

func TestRandPositiveInt(t *testing.T) {
	t.Parallel()
	tests := []struct {
		name    string
		r       *rand.Rand
		max     math.Int
		wantErr bool
	}{
		{"max number is 1", rand.New(rand.NewSource(1)), math.NewInt(1), false},
		{"max number is 5", rand.New(rand.NewSource(50)), math.NewInt(5), false},
		{"negative max number", rand.New(rand.NewSource(99)), math.NewInt(-10), true},
		{"too small max number", rand.New(rand.NewSource(50)), math.NewInt(0), true},
	}
	for _, tt := range tests {
		tc := tt
		t.Run(tc.name, func(t *testing.T) {
			got, err := simulation.RandPositiveInt(tc.r, tc.max)
			if tc.wantErr {
				require.NotNil(t, err)
				return
			}
			require.Greater(t, got.Int64(), int64(0))
			require.LessOrEqual(t, got.Int64(), tc.max.Int64())
		})
	}
}

func TestRandomAmount(t *testing.T) {
	t.Parallel()
	tests := []struct {
		name string
		r    *rand.Rand
		max  math.Int
	}{
		{"max number is 1", rand.New(rand.NewSource(1)), math.NewInt(1)},
		{"max number is 5", rand.New(rand.NewSource(50)), math.NewInt(5)},
		{"negative max number", rand.New(rand.NewSource(99)), math.NewInt(-10)},
		{"too small max number", rand.New(rand.NewSource(50)), math.NewInt(0)},
	}
	for _, tt := range tests {
		tc := tt
		t.Run(tc.name, func(t *testing.T) {
			got := simulation.RandomAmount(tc.r, tc.max)
			if tc.max.Int64() < 0 {
				require.Equal(t, got.Int64(), int64(0))
				return
			}
			require.GreaterOrEqual(t, got.Int64(), int64(0))
			require.LessOrEqual(t, got.Int64(), tc.max.Int64())
		})
	}
}

func TestRandomDecAmount(t *testing.T) {
	t.Parallel()
	tests := []struct {
		name string
		r    *rand.Rand
		max  math.LegacyDec
		exp  string
	}{
		{"max number is 1000", rand.New(rand.NewSource(1)), math.LegacyNewDec(1000), "1000.000000000000000000"},
		{"max number is 531234567", rand.New(rand.NewSource(50)), math.LegacyNewDec(531234567), "531234567.000000000000000000"},
		{"negative max number", rand.New(rand.NewSource(99)), math.LegacyNewDec(-10), "0.000000000000000000"},
		{"max number is zero", rand.New(rand.NewSource(50)), math.LegacyNewDec(0), "0.000000000000000000"},
	}
	for _, tt := range tests {
		tc := tt
		t.Run(tc.name, func(t *testing.T) {
			got := simulation.RandomDecAmount(tc.r, tc.max)
			require.Equal(t, got.String(), tc.exp)
		})
	}
}

func TestRandomTimestamp(t *testing.T) {
	t.Parallel()
	tests := []struct {
		name string
		r    *rand.Rand
		exp  string
	}{
		{"rand is 0", rand.New(rand.NewSource(0)), "2083-05-23 03:26:06 +0000 UTC"},
		{"rand is greater than 1", rand.New(rand.NewSource(1)), "2126-08-31 20:37:51 +0000 UTC"},
		{"rand is negative", rand.New(rand.NewSource(-1)), "2179-08-09 15:02:17 +0000 UTC"},
	}
	for _, tt := range tests {
		tc := tt
		t.Run(tc.name, func(t *testing.T) {
			got := simulation.RandTimestamp(tc.r)
			require.Equal(t, got.UTC().String(), tc.exp)
		})
	}
}

func TestRandomIntBetween(t *testing.T) {
	t.Parallel()
	tests := []struct {
		name string
		r    *rand.Rand
		min  int
		max  int
	}{
		{"random int between 0 and 10", rand.New(rand.NewSource(0)), 0, 10},
		{"random int between 11 and 1000", rand.New(rand.NewSource(1)), 11, 1000},
		{"random int between 10001 and 100000", rand.New(rand.NewSource(-1)), 1001, 10000},
	}
	for _, tt := range tests {
		tc := tt
		t.Run(tc.name, func(t *testing.T) {
			got := simulation.RandIntBetween(tc.r, tc.min, tc.max)
			require.GreaterOrEqual(t, got, tc.min)
			require.LessOrEqual(t, got, tc.max)
		})
	}
}

func TestDeriveRand(t *testing.T) {
	src := rand.New(rand.NewSource(0))
	derived := simulation.DeriveRand(src)
	got := derived.Int()
	assert.NotEqual(t, got, src)
	assert.NotEqual(t, got, rand.New(rand.NewSource(0)).Int())
}

func mustParseCoins(s string) sdk.Coins {
	coins, err := sdk.ParseCoinsNormalized(s)
	if err != nil {
		panic(err)
	}
	return coins
}
