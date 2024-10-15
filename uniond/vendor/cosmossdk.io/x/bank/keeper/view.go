package keeper

import (
	"context"
	"fmt"

	"cosmossdk.io/collections"
	"cosmossdk.io/collections/indexes"
	"cosmossdk.io/core/appmodule"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/math"
	"cosmossdk.io/x/bank/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

var _ ViewKeeper = (*BaseViewKeeper)(nil)

// ViewKeeper defines a module interface that facilitates read only access to
// account balances.
type ViewKeeper interface {
	ValidateBalance(ctx context.Context, addr sdk.AccAddress) error
	HasBalance(ctx context.Context, addr sdk.AccAddress, amt sdk.Coin) bool

	GetAllBalances(ctx context.Context, addr sdk.AccAddress) sdk.Coins
	GetAccountsBalances(ctx context.Context) []types.Balance
	GetBalance(ctx context.Context, addr sdk.AccAddress, denom string) sdk.Coin
	LockedCoins(ctx context.Context, addr sdk.AccAddress) sdk.Coins
	SpendableCoins(ctx context.Context, addr sdk.AccAddress) sdk.Coins
	SpendableCoin(ctx context.Context, addr sdk.AccAddress, denom string) sdk.Coin

	IterateAccountBalances(ctx context.Context, addr sdk.AccAddress, cb func(coin sdk.Coin) (stop bool))
	IterateAllBalances(ctx context.Context, cb func(address sdk.AccAddress, coin sdk.Coin) (stop bool))
}

func newBalancesIndexes(sb *collections.SchemaBuilder) BalancesIndexes {
	return BalancesIndexes{
		Denom: indexes.NewReversePair[math.Int](
			sb, types.DenomAddressPrefix, "address_by_denom_index",
			collections.PairKeyCodec(sdk.LengthPrefixedAddressKey(sdk.AccAddressKey), collections.StringKey), //nolint:staticcheck // Note: refer to the LengthPrefixedAddressKey docs to understand why we do this.
			indexes.WithReversePairUncheckedValue(),                                                          // denom to address indexes were stored as Key: Join(denom, address) Value: []byte{0}, this will migrate the value to []byte{} in a lazy way.
		),
	}
}

type BalancesIndexes struct {
	Denom *indexes.ReversePair[sdk.AccAddress, string, math.Int]
}

func (b BalancesIndexes) IndexesList() []collections.Index[collections.Pair[sdk.AccAddress, string], math.Int] {
	return []collections.Index[collections.Pair[sdk.AccAddress, string], math.Int]{b.Denom}
}

// BaseViewKeeper implements a read only keeper implementation of ViewKeeper.
type BaseViewKeeper struct {
	appmodule.Environment

	cdc codec.BinaryCodec
	ak  types.AccountKeeper

	Schema        collections.Schema
	Supply        collections.Map[string, math.Int]
	DenomMetadata collections.Map[string, types.Metadata]
	SendEnabled   collections.Map[string, bool]
	Balances      *collections.IndexedMap[collections.Pair[sdk.AccAddress, string], math.Int, BalancesIndexes]
	Params        collections.Item[types.Params]
}

// NewBaseViewKeeper returns a new BaseViewKeeper.
func NewBaseViewKeeper(env appmodule.Environment, cdc codec.BinaryCodec, ak types.AccountKeeper) BaseViewKeeper {
	sb := collections.NewSchemaBuilder(env.KVStoreService)
	k := BaseViewKeeper{
		Environment:   env,
		cdc:           cdc,
		ak:            ak,
		Supply:        collections.NewMap(sb, types.SupplyKey, "supply", collections.StringKey, sdk.IntValue),
		DenomMetadata: collections.NewMap(sb, types.DenomMetadataPrefix, "denom_metadata", collections.StringKey, codec.CollValue[types.Metadata](cdc)),
		SendEnabled:   collections.NewMap(sb, types.SendEnabledPrefix, "send_enabled", collections.StringKey, codec.BoolValue), // NOTE: we use a bool value which uses protobuf to retain state backwards compat
		Balances:      collections.NewIndexedMap(sb, types.BalancesPrefix, "balances", collections.PairKeyCodec(sdk.AccAddressKey, collections.StringKey), types.BalanceValueCodec, newBalancesIndexes(sb)),
		Params:        collections.NewItem(sb, types.ParamsKey, "params", codec.CollValue[types.Params](cdc)),
	}

	schema, err := sb.Build()
	if err != nil {
		panic(err)
	}
	k.Schema = schema
	return k
}

// HasBalance returns whether or not an account has at least amt balance.
func (k BaseViewKeeper) HasBalance(ctx context.Context, addr sdk.AccAddress, amt sdk.Coin) bool {
	return k.GetBalance(ctx, addr, amt.Denom).IsGTE(amt)
}

// GetAllBalances returns all the account balances for the given account address.
func (k BaseViewKeeper) GetAllBalances(ctx context.Context, addr sdk.AccAddress) sdk.Coins {
	balances := sdk.NewCoins()
	k.IterateAccountBalances(ctx, addr, func(balance sdk.Coin) bool {
		balances = balances.Add(balance)
		return false
	})

	return balances.Sort()
}

// GetAccountsBalances returns all the accounts balances from the store.
func (k BaseViewKeeper) GetAccountsBalances(ctx context.Context) []types.Balance {
	balances := make([]types.Balance, 0)
	mapAddressToBalancesIdx := make(map[string]int)

	k.IterateAllBalances(ctx, func(addr sdk.AccAddress, balance sdk.Coin) bool {
		addrStr, err := k.ak.AddressCodec().BytesToString(addr)
		if err != nil {
			panic(err)
		}
		idx, ok := mapAddressToBalancesIdx[addrStr]
		if ok {
			// address is already on the set of accounts balances
			balances[idx].Coins = balances[idx].Coins.Add(balance)
			balances[idx].Coins.Sort()
			return false
		}

		accountBalance := types.Balance{
			Address: addrStr,
			Coins:   sdk.NewCoins(balance),
		}
		balances = append(balances, accountBalance)
		mapAddressToBalancesIdx[addrStr] = len(balances) - 1
		return false
	})

	return balances
}

// GetBalance returns the balance of a specific denomination for a given account
// by address.
func (k BaseViewKeeper) GetBalance(ctx context.Context, addr sdk.AccAddress, denom string) sdk.Coin {
	amt, err := k.Balances.Get(ctx, collections.Join(addr, denom))
	if err != nil {
		return sdk.NewCoin(denom, math.ZeroInt())
	}
	return sdk.NewCoin(denom, amt)
}

// IterateAccountBalances iterates over the balances of a single account and
// provides the token balance to a callback. If true is returned from the
// callback, iteration is halted.
func (k BaseViewKeeper) IterateAccountBalances(ctx context.Context, addr sdk.AccAddress, cb func(sdk.Coin) bool) {
	err := k.Balances.Walk(ctx, collections.NewPrefixedPairRange[sdk.AccAddress, string](addr), func(key collections.Pair[sdk.AccAddress, string], value math.Int) (stop bool, err error) {
		return cb(sdk.NewCoin(key.K2(), value)), nil
	})
	if err != nil {
		panic(err)
	}
}

// IterateAllBalances iterates over all the balances of all accounts and
// denominations that are provided to a callback. If true is returned from the
// callback, iteration is halted.
func (k BaseViewKeeper) IterateAllBalances(ctx context.Context, cb func(sdk.AccAddress, sdk.Coin) bool) {
	err := k.Balances.Walk(ctx, nil, func(key collections.Pair[sdk.AccAddress, string], value math.Int) (stop bool, err error) {
		return cb(key.K1(), sdk.NewCoin(key.K2(), value)), nil
	})
	if err != nil {
		panic(err)
	}
}

// LockedCoins returns all the coins that are not spendable (i.e. locked) for an
// account by address. For standard accounts, the result will always be no coins.
// For vesting accounts, LockedCoins is delegated to the concrete vesting account
// type.
func (k BaseViewKeeper) LockedCoins(ctx context.Context, addr sdk.AccAddress) sdk.Coins {
	acc := k.ak.GetAccount(ctx, addr)
	if acc != nil {
		vacc, ok := acc.(types.VestingAccount)
		if ok {
			return vacc.LockedCoins(k.HeaderService.HeaderInfo(ctx).Time)
		}
	}

	return sdk.NewCoins()
}

// SpendableCoins returns the total balances of spendable coins for an account
// by address. If the account has no spendable coins, an empty Coins slice is
// returned.
func (k BaseViewKeeper) SpendableCoins(ctx context.Context, addr sdk.AccAddress) sdk.Coins {
	total := k.GetAllBalances(ctx, addr)
	allLocked := k.LockedCoins(ctx, addr)
	if allLocked.IsZero() {
		return total
	}

	unlocked, hasNeg := total.SafeSub(allLocked...)
	if !hasNeg {
		return unlocked
	}

	spendable := sdk.Coins{}
	for _, coin := range unlocked {
		if coin.IsPositive() {
			spendable = append(spendable, coin)
		}
	}
	return spendable
}

// SpendableCoin returns the balance of specific denomination of spendable coins
// for an account by address. If the account has no spendable coin, a zero Coin
// is returned.
func (k BaseViewKeeper) SpendableCoin(ctx context.Context, addr sdk.AccAddress, denom string) sdk.Coin {
	balance := k.GetBalance(ctx, addr, denom)
	lockedAmt := k.LockedCoins(ctx, addr).AmountOf(denom)
	if !lockedAmt.IsPositive() {
		return balance
	}
	if lockedAmt.LT(balance.Amount) {
		return balance.SubAmount(lockedAmt)
	}
	return sdk.NewCoin(denom, math.ZeroInt())
}

// ValidateBalance validates all balances for a given account address returning
// an error if any balance is invalid. It will check for vesting account types
// and validate the balances against the original vesting balances.
//
// CONTRACT: ValidateBalance should only be called upon genesis state. In the
// case of vesting accounts, balances may change in a valid manner that would
// otherwise yield an error from this call.
func (k BaseViewKeeper) ValidateBalance(ctx context.Context, addr sdk.AccAddress) error {
	acc := k.ak.GetAccount(ctx, addr)
	if acc == nil {
		return errorsmod.Wrapf(sdkerrors.ErrUnknownAddress, "account %s does not exist", addr)
	}

	balances := k.GetAllBalances(ctx, addr)
	if !balances.IsValid() {
		return fmt.Errorf("account balance of %s is invalid", balances)
	}

	vacc, ok := acc.(types.VestingAccount)
	if ok {
		ogv := vacc.GetOriginalVesting()
		if ogv.IsAnyGT(balances) {
			return fmt.Errorf("vesting amount %s cannot be greater than total amount %s", ogv, balances)
		}
	}

	return nil
}
