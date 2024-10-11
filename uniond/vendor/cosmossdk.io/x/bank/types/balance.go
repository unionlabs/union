package types

import (
	"bytes"
	"encoding/json"
	"fmt"
	"sort"

	"cosmossdk.io/core/address"
	"cosmossdk.io/x/bank/exported"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var _ exported.GenesisBalance = (*Balance)(nil)

// GetAddress returns the account address of the Balance object.
func (b Balance) GetAddress() string {
	return b.Address
}

// GetCoins returns the account coins of the Balance object.
func (b Balance) GetCoins() sdk.Coins {
	return b.Coins
}

// Validate checks for address and coins correctness.
func (b Balance) Validate() error {
	if _, err := sdk.AccAddressFromBech32(b.Address); err != nil {
		return err
	}

	if err := b.Coins.Validate(); err != nil {
		return err
	}

	return nil
}

type balanceByAddress struct {
	addresses []sdk.AccAddress
	balances  []Balance
}

func (b balanceByAddress) Len() int { return len(b.addresses) }
func (b balanceByAddress) Less(i, j int) bool {
	return bytes.Compare(b.addresses[i], b.addresses[j]) < 0
}

func (b balanceByAddress) Swap(i, j int) {
	b.addresses[i], b.addresses[j] = b.addresses[j], b.addresses[i]
	b.balances[i], b.balances[j] = b.balances[j], b.balances[i]
}

// SanitizeGenesisBalances checks for duplicates and sorts addresses and coin sets.
func SanitizeGenesisBalances(balances []Balance, addressCodec address.Codec) ([]Balance, error) {
	// Given that this function sorts balances, using the standard library's
	// Quicksort based algorithms, we have algorithmic complexities of:
	// * Best case: O(nlogn)
	// * Worst case: O(n^2)
	// The comparator used MUST be cheap to use lest we incur expenses like we had
	// before whereby sdk.AccAddressFromBech32, which is a very expensive operation
	// compared n * n elements yet discarded computations each time, as per:
	//  https://github.com/cosmos/cosmos-sdk/issues/7766#issuecomment-786671734

	// 1. Retrieve the address equivalents for each Balance's address.
	addresses := make([]sdk.AccAddress, len(balances))
	// 2. Track any duplicate addresses to avoid false positives on invariant checks.
	seen := make(map[string]struct{})
	for i := range balances {
		addr, err := addressCodec.StringToBytes(balances[i].Address)
		if err != nil {
			return nil, err
		}
		addresses[i] = addr
		if _, exists := seen[string(addr)]; exists {
			panic(fmt.Sprintf("genesis state has a duplicate account: %q aka %x", balances[i].Address, addr))
		}
		seen[string(addr)] = struct{}{}
	}

	// 3. Sort balances.
	sort.Sort(balanceByAddress{addresses: addresses, balances: balances})

	return balances, nil
}

// GenesisBalancesIterator implements genesis account iteration.
type GenesisBalancesIterator struct{}

// IterateGenesisBalances iterates over all the genesis balances found in
// appGenesis and invokes a callback on each genesis account. If any call
// returns true, iteration stops.
func (GenesisBalancesIterator) IterateGenesisBalances(
	cdc codec.JSONCodec, appState map[string]json.RawMessage, cb func(exported.GenesisBalance) (stop bool),
) {
	for _, balance := range GetGenesisStateFromAppState(cdc, appState).Balances {
		if cb(balance) {
			break
		}
	}
}
