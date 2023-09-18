package types_test

import (
	"bytes"
	"testing"

	"cosmossdk.io/math"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	sdk "github.com/cosmos/cosmos-sdk/types"
	bank "github.com/cosmos/cosmos-sdk/x/bank/types"
)

func TestBalanceValidate(t *testing.T) {
	testCases := []struct {
		name    string
		balance bank.Balance
		expErr  bool
	}{
		{
			"valid balance",
			bank.Balance{
				Address: "cosmos1yq8lgssgxlx9smjhes6ryjasmqmd3ts2559g0t",
				Coins:   sdk.Coins{sdk.NewInt64Coin("uatom", 1)},
			},
			false,
		},
		{"empty balance", bank.Balance{}, true},
		{
			"nil balance coins",
			bank.Balance{
				Address: "cosmos1yq8lgssgxlx9smjhes6ryjasmqmd3ts2559g0t",
			},
			false,
		},
		{
			"dup coins",
			bank.Balance{
				Address: "cosmos1yq8lgssgxlx9smjhes6ryjasmqmd3ts2559g0t",
				Coins: sdk.Coins{
					sdk.NewInt64Coin("uatom", 1),
					sdk.NewInt64Coin("uatom", 1),
				},
			},
			true,
		},
		{
			"invalid coin denom",
			bank.Balance{
				Address: "cosmos1yq8lgssgxlx9smjhes6ryjasmqmd3ts2559g0t",
				Coins: sdk.Coins{
					sdk.Coin{Denom: "", Amount: math.OneInt()},
				},
			},
			true,
		},
		{
			"negative coin",
			bank.Balance{
				Address: "cosmos1yq8lgssgxlx9smjhes6ryjasmqmd3ts2559g0t",
				Coins: sdk.Coins{
					sdk.Coin{Denom: "uatom", Amount: sdk.NewInt(-1)},
				},
			},
			true,
		},
		{
			"0 value coin",
			bank.Balance{
				Address: "cosmos1yq8lgssgxlx9smjhes6ryjasmqmd3ts2559g0t",
				Coins: sdk.Coins{
					sdk.NewInt64Coin("atom", 0),
					sdk.NewInt64Coin("zatom", 2),
				},
			},
			true,
		},
		{
			"unsorted coins",
			bank.Balance{
				Address: "cosmos1yq8lgssgxlx9smjhes6ryjasmqmd3ts2559g0t",
				Coins: sdk.Coins{
					sdk.NewInt64Coin("atom", 2),
					sdk.NewInt64Coin("zatom", 2),
					sdk.NewInt64Coin("batom", 12),
				},
			},
			true,
		},
		{
			"valid sorted coins",
			bank.Balance{
				Address: "cosmos1yq8lgssgxlx9smjhes6ryjasmqmd3ts2559g0t",
				Coins: sdk.Coins{
					sdk.NewInt64Coin("atom", 2),
					sdk.NewInt64Coin("batom", 12),
					sdk.NewInt64Coin("zatom", 2),
				},
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		t.Run(tc.name, func(t *testing.T) {
			err := tc.balance.Validate()

			if tc.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestBalance_GetAddress(t *testing.T) {
	tests := []struct {
		name      string
		Address   string
		wantPanic bool
	}{
		{"empty address", "", true},
		{"malformed address", "invalid", true},
		{"valid address", "cosmos1vy0ga0klndqy92ceqehfkvgmn4t94eteq4hmqv", false},
	}
	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			b := bank.Balance{Address: tt.Address}
			if tt.wantPanic {
				require.Panics(t, func() { b.GetAddress() })
			} else {
				require.False(t, b.GetAddress().Empty())
			}
		})
	}
}

func TestSanitizeBalances(t *testing.T) {
	// 1. Generate balances
	tokens := sdk.TokensFromConsensusPower(81, sdk.DefaultPowerReduction)
	coin := sdk.NewCoin("benchcoin", tokens)
	coins := sdk.Coins{coin}
	addrs, _ := makeRandomAddressesAndPublicKeys(20)

	var balances []bank.Balance
	for _, addr := range addrs {
		balances = append(balances, bank.Balance{
			Address: addr.String(),
			Coins:   coins,
		})
	}
	// 2. Sort the values.
	sorted := bank.SanitizeGenesisBalances(balances)

	// 3. Compare and ensure that all the values are sorted in ascending order.
	// Invariant after sorting:
	//    a[i] <= a[i+1...n]
	for i := 0; i < len(sorted); i++ {
		ai := sorted[i]
		// Ensure that every single value that comes after i is less than it.
		for j := i + 1; j < len(sorted); j++ {
			aj := sorted[j]

			if got := bytes.Compare(ai.GetAddress(), aj.GetAddress()); got > 0 {
				t.Errorf("Balance(%d) > Balance(%d)", i, j)
			}
		}
	}
}

func makeRandomAddressesAndPublicKeys(n int) (accL []sdk.AccAddress, pkL []*ed25519.PubKey) {
	for i := 0; i < n; i++ {
		pk := ed25519.GenPrivKey().PubKey().(*ed25519.PubKey)
		pkL = append(pkL, pk)
		accL = append(accL, sdk.AccAddress(pk.Address()))
	}
	return accL, pkL
}

var sink, revert interface{}

func BenchmarkSanitizeBalances500(b *testing.B) {
	benchmarkSanitizeBalances(b, 500)
}

func BenchmarkSanitizeBalances1000(b *testing.B) {
	benchmarkSanitizeBalances(b, 1000)
}

func benchmarkSanitizeBalances(b *testing.B, nAddresses int) {
	b.ReportAllocs()
	tokens := sdk.TokensFromConsensusPower(81, sdk.DefaultPowerReduction)
	coin := sdk.NewCoin("benchcoin", tokens)
	coins := sdk.Coins{coin}
	addrs, _ := makeRandomAddressesAndPublicKeys(nAddresses)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		var balances []bank.Balance
		for _, addr := range addrs {
			balances = append(balances, bank.Balance{
				Address: addr.String(),
				Coins:   coins,
			})
		}
		sink = bank.SanitizeGenesisBalances(balances)
	}
	if sink == nil {
		b.Fatal("Benchmark did not run")
	}
	sink = revert
}
