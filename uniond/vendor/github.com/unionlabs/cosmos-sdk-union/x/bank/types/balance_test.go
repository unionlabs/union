package types_test

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/math"
	bank "cosmossdk.io/x/bank/types"

	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	sdk "github.com/cosmos/cosmos-sdk/types"
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
					sdk.Coin{Denom: "uatom", Amount: math.NewInt(-1)},
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
		name    string
		Address string
		err     bool
	}{
		{"empty address", "", true},
		{"malformed address", "invalid", true},
		{"valid address", "cosmos1vy0ga0klndqy92ceqehfkvgmn4t94eteq4hmqv", false},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			b := bank.Balance{Address: tt.Address}
			if !tt.err {
				require.Equal(t, b.GetAddress(), tt.Address)
			} else {
				require.False(t, len(b.GetAddress()) != 0 && b.GetAddress() != tt.Address)
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

	ac := codectestutil.CodecOptions{}.GetAddressCodec()
	var balances []bank.Balance
	for _, addr := range addrs {
		addrStr, err := ac.BytesToString(addr)
		require.NoError(t, err)
		balances = append(balances, bank.Balance{
			Address: addrStr,
			Coins:   coins,
		})
	}
	// 2. Sort the values.
	sorted, err := bank.SanitizeGenesisBalances(balances, ac)
	require.NoError(t, err)

	// 3. Compare and ensure that all the values are sorted in ascending order.
	// Invariant after sorting:
	//    a[i] <= a[i+1...n]
	for i := 0; i < len(sorted); i++ {
		ai := sorted[i]
		// Ensure that every single value that comes after i is less than it.
		for j := i + 1; j < len(sorted); j++ {
			aj := sorted[j]
			if ai.GetAddress() == aj.GetAddress() {
				t.Errorf("Balance(%d) > Balance(%d)", i, j)
			}
		}
	}
}

func TestSanitizeBalancesDuplicates(t *testing.T) {
	// 1. Generate balances
	tokens := sdk.TokensFromConsensusPower(81, sdk.DefaultPowerReduction)
	coin := sdk.NewCoin("benchcoin", tokens)
	coins := sdk.Coins{coin}
	addrs, _ := makeRandomAddressesAndPublicKeys(13)

	var balances []bank.Balance
	ac := codectestutil.CodecOptions{}.GetAddressCodec()
	for _, addr := range addrs {
		addrStr, err := ac.BytesToString(addr)
		require.NoError(t, err)
		balances = append(balances, bank.Balance{
			Address: addrStr,
			Coins:   coins,
		})
	}

	// 2. Add duplicate
	dupIdx := 3
	balances = append(balances, balances[dupIdx])
	addr, _ := ac.StringToBytes(balances[dupIdx].Address)
	expectedError := fmt.Sprintf("genesis state has a duplicate account: %q aka %x", balances[dupIdx].Address, addr)

	// 3. Add more balances
	coin2 := sdk.NewCoin("coinbench", tokens)
	coins2 := sdk.Coins{coin2, coin}
	addrs2, _ := makeRandomAddressesAndPublicKeys(31)
	for _, addr := range addrs2 {
		addrStr, err := ac.BytesToString(addr)
		require.NoError(t, err)
		balances = append(balances, bank.Balance{
			Address: addrStr,
			Coins:   coins2,
		})
	}

	// 4. Execute SanitizeGenesisBalances and expect an error
	require.PanicsWithValue(t, expectedError, func() {
		_, err := bank.SanitizeGenesisBalances(balances, ac)
		require.NoError(t, err)
	}, "SanitizeGenesisBalances should panic with duplicate accounts")
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
	b.Helper()
	b.ReportAllocs()
	tokens := sdk.TokensFromConsensusPower(81, sdk.DefaultPowerReduction)
	coin := sdk.NewCoin("benchcoin", tokens)
	coins := sdk.Coins{coin}
	addrs, _ := makeRandomAddressesAndPublicKeys(nAddresses)

	b.ResetTimer()
	var err error
	ac := codectestutil.CodecOptions{}.GetAddressCodec()
	for i := 0; i < b.N; i++ {
		var balances []bank.Balance
		for _, addr := range addrs {
			addrStr, err := ac.BytesToString(addr)
			require.NoError(b, err)
			balances = append(balances, bank.Balance{
				Address: addrStr,
				Coins:   coins,
			})
		}
		sink, err = bank.SanitizeGenesisBalances(balances, ac)
		require.NoError(b, err)
	}
	if sink == nil {
		b.Fatal("Benchmark did not run")
	}
	sink = revert
}
