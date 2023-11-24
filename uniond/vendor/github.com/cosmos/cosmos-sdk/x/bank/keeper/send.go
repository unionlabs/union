package keeper

import (
	"context"
	"fmt"

	"cosmossdk.io/collections"
	"cosmossdk.io/core/store"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/log"
	"cosmossdk.io/math"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/telemetry"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/bank/types"
)

// SendKeeper defines a module interface that facilitates the transfer of coins
// between accounts without the possibility of creating coins.
type SendKeeper interface {
	ViewKeeper

	AppendSendRestriction(restriction types.SendRestrictionFn)
	PrependSendRestriction(restriction types.SendRestrictionFn)
	ClearSendRestriction()

	InputOutputCoins(ctx context.Context, input types.Input, outputs []types.Output) error
	SendCoins(ctx context.Context, fromAddr, toAddr sdk.AccAddress, amt sdk.Coins) error

	GetParams(ctx context.Context) types.Params
	SetParams(ctx context.Context, params types.Params) error

	IsSendEnabledDenom(ctx context.Context, denom string) bool
	GetSendEnabledEntry(ctx context.Context, denom string) (types.SendEnabled, bool)
	SetSendEnabled(ctx context.Context, denom string, value bool)
	SetAllSendEnabled(ctx context.Context, sendEnableds []*types.SendEnabled)
	DeleteSendEnabled(ctx context.Context, denoms ...string)
	IterateSendEnabledEntries(ctx context.Context, cb func(denom string, sendEnabled bool) (stop bool))
	GetAllSendEnabledEntries(ctx context.Context) []types.SendEnabled

	IsSendEnabledCoin(ctx context.Context, coin sdk.Coin) bool
	IsSendEnabledCoins(ctx context.Context, coins ...sdk.Coin) error

	BlockedAddr(addr sdk.AccAddress) bool
	GetBlockedAddresses() map[string]bool

	GetAuthority() string
}

var _ SendKeeper = (*BaseSendKeeper)(nil)

// BaseSendKeeper only allows transfers between accounts without the possibility of
// creating coins. It implements the SendKeeper interface.
type BaseSendKeeper struct {
	BaseViewKeeper

	cdc          codec.BinaryCodec
	ak           types.AccountKeeper
	storeService store.KVStoreService
	logger       log.Logger

	// list of addresses that are restricted from receiving transactions
	blockedAddrs map[string]bool

	// the address capable of executing a MsgUpdateParams message. Typically, this
	// should be the x/gov module account.
	authority string

	sendRestriction *sendRestriction
}

func NewBaseSendKeeper(
	cdc codec.BinaryCodec,
	storeService store.KVStoreService,
	ak types.AccountKeeper,
	blockedAddrs map[string]bool,
	authority string,
	logger log.Logger,
) BaseSendKeeper {
	if _, err := ak.AddressCodec().StringToBytes(authority); err != nil {
		panic(fmt.Errorf("invalid bank authority address: %w", err))
	}

	return BaseSendKeeper{
		BaseViewKeeper:  NewBaseViewKeeper(cdc, storeService, ak, logger),
		cdc:             cdc,
		ak:              ak,
		storeService:    storeService,
		blockedAddrs:    blockedAddrs,
		authority:       authority,
		logger:          logger,
		sendRestriction: newSendRestriction(),
	}
}

// AppendSendRestriction adds the provided SendRestrictionFn to run after previously provided restrictions.
func (k BaseSendKeeper) AppendSendRestriction(restriction types.SendRestrictionFn) {
	k.sendRestriction.append(restriction)
}

// PrependSendRestriction adds the provided SendRestrictionFn to run before previously provided restrictions.
func (k BaseSendKeeper) PrependSendRestriction(restriction types.SendRestrictionFn) {
	k.sendRestriction.prepend(restriction)
}

// ClearSendRestriction removes the send restriction (if there is one).
func (k BaseSendKeeper) ClearSendRestriction() {
	k.sendRestriction.clear()
}

// GetAuthority returns the x/bank module's authority.
func (k BaseSendKeeper) GetAuthority() string {
	return k.authority
}

// GetParams returns the total set of bank parameters.
func (k BaseSendKeeper) GetParams(ctx context.Context) (params types.Params) {
	p, _ := k.Params.Get(ctx)
	return p
}

// SetParams sets the total set of bank parameters.
//
// Note: params.SendEnabled is deprecated but it should be here regardless.
func (k BaseSendKeeper) SetParams(ctx context.Context, params types.Params) error {
	// Normally SendEnabled is deprecated but we still support it for backwards
	// compatibility. Using params.Validate() would fail due to the SendEnabled
	// deprecation.
	if len(params.SendEnabled) > 0 { //nolint:staticcheck // SA1019: params.SendEnabled is deprecated
		k.SetAllSendEnabled(ctx, params.SendEnabled) //nolint:staticcheck // SA1019: params.SendEnabled is deprecated

		// override params without SendEnabled
		params = types.NewParams(params.DefaultSendEnabled)
	}
	return k.Params.Set(ctx, params)
}

// InputOutputCoins performs multi-send functionality. It accepts an
// input that corresponds to a series of outputs. It returns an error if the
// input and outputs don't line up or if any single transfer of tokens fails.
func (k BaseSendKeeper) InputOutputCoins(ctx context.Context, input types.Input, outputs []types.Output) error {
	// Safety check ensuring that when sending coins the keeper must maintain the
	// Check supply invariant and validity of Coins.
	if err := types.ValidateInputOutputs(input, outputs); err != nil {
		return err
	}

	inAddress, err := k.ak.AddressCodec().StringToBytes(input.Address)
	if err != nil {
		return err
	}

	err = k.subUnlockedCoins(ctx, inAddress, input.Coins)
	if err != nil {
		return err
	}

	sdkCtx := sdk.UnwrapSDKContext(ctx)
	sdkCtx.EventManager().EmitEvent(
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(types.AttributeKeySender, input.Address),
		),
	)

	var outAddress sdk.AccAddress
	for _, out := range outputs {
		outAddress, err = k.ak.AddressCodec().StringToBytes(out.Address)
		if err != nil {
			return err
		}

		outAddress, err = k.sendRestriction.apply(ctx, inAddress, outAddress, out.Coins)
		if err != nil {
			return err
		}

		if err := k.addCoins(ctx, outAddress, out.Coins); err != nil {
			return err
		}

		sdkCtx.EventManager().EmitEvent(
			sdk.NewEvent(
				types.EventTypeTransfer,
				sdk.NewAttribute(types.AttributeKeyRecipient, outAddress.String()),
				sdk.NewAttribute(sdk.AttributeKeyAmount, out.Coins.String()),
			),
		)

		// Create account if recipient does not exist.
		//
		// NOTE: This should ultimately be removed in favor a more flexible approach
		// such as delegated fee messages.
		accExists := k.ak.HasAccount(ctx, outAddress)
		if !accExists {
			defer telemetry.IncrCounter(1, "new", "account")
			k.ak.SetAccount(ctx, k.ak.NewAccountWithAddress(ctx, outAddress))
		}
	}

	return nil
}

// SendCoins transfers amt coins from a sending account to a receiving account.
// An error is returned upon failure.
func (k BaseSendKeeper) SendCoins(ctx context.Context, fromAddr, toAddr sdk.AccAddress, amt sdk.Coins) error {
	var err error
	err = k.subUnlockedCoins(ctx, fromAddr, amt)
	if err != nil {
		return err
	}

	toAddr, err = k.sendRestriction.apply(ctx, fromAddr, toAddr, amt)
	if err != nil {
		return err
	}

	err = k.addCoins(ctx, toAddr, amt)
	if err != nil {
		return err
	}

	// Create account if recipient does not exist.
	//
	// NOTE: This should ultimately be removed in favor a more flexible approach
	// such as delegated fee messages.
	accExists := k.ak.HasAccount(ctx, toAddr)
	if !accExists {
		defer telemetry.IncrCounter(1, "new", "account")
		k.ak.SetAccount(ctx, k.ak.NewAccountWithAddress(ctx, toAddr))
	}

	// bech32 encoding is expensive! Only do it once for fromAddr
	fromAddrString := fromAddr.String()
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	sdkCtx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeTransfer,
			sdk.NewAttribute(types.AttributeKeyRecipient, toAddr.String()),
			sdk.NewAttribute(types.AttributeKeySender, fromAddrString),
			sdk.NewAttribute(sdk.AttributeKeyAmount, amt.String()),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(types.AttributeKeySender, fromAddr.String()),
		),
	})

	return nil
}

// subUnlockedCoins removes the unlocked amt coins of the given account. An error is
// returned if the resulting balance is negative or the initial amount is invalid.
// A coin_spent event is emitted after.
func (k BaseSendKeeper) subUnlockedCoins(ctx context.Context, addr sdk.AccAddress, amt sdk.Coins) error {
	if !amt.IsValid() {
		return errorsmod.Wrap(sdkerrors.ErrInvalidCoins, amt.String())
	}

	lockedCoins := k.LockedCoins(ctx, addr)

	for _, coin := range amt {
		balance := k.GetBalance(ctx, addr, coin.Denom)
		locked := sdk.NewCoin(coin.Denom, lockedCoins.AmountOf(coin.Denom))

		spendable, hasNeg := sdk.Coins{balance}.SafeSub(locked)
		if hasNeg {
			return errorsmod.Wrapf(sdkerrors.ErrInsufficientFunds,
				"locked amount exceeds account balance funds: %s > %s", locked, balance)
		}

		if _, hasNeg := spendable.SafeSub(coin); hasNeg {
			if len(spendable) == 0 {
				spendable = sdk.Coins{sdk.NewCoin(coin.Denom, math.ZeroInt())}
			}
			return errorsmod.Wrapf(
				sdkerrors.ErrInsufficientFunds,
				"spendable balance %s is smaller than %s",
				spendable, coin,
			)
		}

		newBalance := balance.Sub(coin)

		if err := k.setBalance(ctx, addr, newBalance); err != nil {
			return err
		}
	}

	sdkCtx := sdk.UnwrapSDKContext(ctx)
	sdkCtx.EventManager().EmitEvent(
		types.NewCoinSpentEvent(addr, amt),
	)

	return nil
}

// addCoins increase the addr balance by the given amt. Fails if the provided
// amt is invalid. It emits a coin received event.
func (k BaseSendKeeper) addCoins(ctx context.Context, addr sdk.AccAddress, amt sdk.Coins) error {
	if !amt.IsValid() {
		return errorsmod.Wrap(sdkerrors.ErrInvalidCoins, amt.String())
	}

	for _, coin := range amt {
		balance := k.GetBalance(ctx, addr, coin.Denom)
		newBalance := balance.Add(coin)

		err := k.setBalance(ctx, addr, newBalance)
		if err != nil {
			return err
		}
	}

	// emit coin received event
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	sdkCtx.EventManager().EmitEvent(
		types.NewCoinReceivedEvent(addr, amt),
	)

	return nil
}

// setBalance sets the coin balance for an account by address.
func (k BaseSendKeeper) setBalance(ctx context.Context, addr sdk.AccAddress, balance sdk.Coin) error {
	if !balance.IsValid() {
		return errorsmod.Wrap(sdkerrors.ErrInvalidCoins, balance.String())
	}

	// x/bank invariants prohibit persistence of zero balances
	if balance.IsZero() {
		err := k.Balances.Remove(ctx, collections.Join(addr, balance.Denom))
		if err != nil {
			return err
		}
		return nil
	}
	return k.Balances.Set(ctx, collections.Join(addr, balance.Denom), balance.Amount)
}

// IsSendEnabledCoins checks the coins provided and returns an ErrSendDisabled
// if any of the coins are not configured for sending. Returns nil if sending is
// enabled for all provided coins.
func (k BaseSendKeeper) IsSendEnabledCoins(ctx context.Context, coins ...sdk.Coin) error {
	if len(coins) == 0 {
		return nil
	}

	defaultVal := k.GetParams(ctx).DefaultSendEnabled

	for _, coin := range coins {
		if !k.getSendEnabledOrDefault(ctx, coin.Denom, defaultVal) {
			return types.ErrSendDisabled.Wrapf("%s transfers are currently disabled", coin.Denom)
		}
	}

	return nil
}

// IsSendEnabledCoin returns the current SendEnabled status of the provided coin's denom
func (k BaseSendKeeper) IsSendEnabledCoin(ctx context.Context, coin sdk.Coin) bool {
	return k.IsSendEnabledDenom(ctx, coin.Denom)
}

// BlockedAddr checks if a given address is restricted from
// receiving funds.
func (k BaseSendKeeper) BlockedAddr(addr sdk.AccAddress) bool {
	return k.blockedAddrs[addr.String()]
}

// GetBlockedAddresses returns the full list of addresses restricted from receiving funds.
func (k BaseSendKeeper) GetBlockedAddresses() map[string]bool {
	return k.blockedAddrs
}

// IsSendEnabledDenom returns the current SendEnabled status of the provided denom.
func (k BaseSendKeeper) IsSendEnabledDenom(ctx context.Context, denom string) bool {
	return k.getSendEnabledOrDefault(ctx, denom, k.GetParams(ctx).DefaultSendEnabled)
}

// GetSendEnabledEntry gets a SendEnabled entry for the given denom.
// The second return argument is true iff a specific entry exists for the given denom.
func (k BaseSendKeeper) GetSendEnabledEntry(ctx context.Context, denom string) (types.SendEnabled, bool) {
	sendEnabled, found := k.getSendEnabled(ctx, denom)
	if !found {
		return types.SendEnabled{}, false
	}

	return types.SendEnabled{Denom: denom, Enabled: sendEnabled}, true
}

// SetSendEnabled sets the SendEnabled flag for a denom to the provided value.
func (k BaseSendKeeper) SetSendEnabled(ctx context.Context, denom string, value bool) {
	_ = k.SendEnabled.Set(ctx, denom, value)
}

// SetAllSendEnabled sets all the provided SendEnabled entries in the bank store.
func (k BaseSendKeeper) SetAllSendEnabled(ctx context.Context, entries []*types.SendEnabled) {
	for _, entry := range entries {
		_ = k.SendEnabled.Set(ctx, entry.Denom, entry.Enabled)
	}
}

// DeleteSendEnabled deletes the SendEnabled flags for one or more denoms.
// If a denom is provided that doesn't have a SendEnabled entry, it is ignored.
func (k BaseSendKeeper) DeleteSendEnabled(ctx context.Context, denoms ...string) {
	for _, denom := range denoms {
		_ = k.SendEnabled.Remove(ctx, denom)
	}
}

// IterateSendEnabledEntries iterates over all the SendEnabled entries.
func (k BaseSendKeeper) IterateSendEnabledEntries(ctx context.Context, cb func(denom string, sendEnabled bool) bool) {
	err := k.SendEnabled.Walk(ctx, nil, func(key string, value bool) (stop bool, err error) {
		return cb(key, value), nil
	})
	if err != nil {
		panic(err)
	}
}

// GetAllSendEnabledEntries gets all the SendEnabled entries that are stored.
// Any denominations not returned use the default value (set in Params).
func (k BaseSendKeeper) GetAllSendEnabledEntries(ctx context.Context) []types.SendEnabled {
	var rv []types.SendEnabled
	k.IterateSendEnabledEntries(ctx, func(denom string, sendEnabled bool) bool {
		rv = append(rv, types.SendEnabled{Denom: denom, Enabled: sendEnabled})
		return false
	})

	return rv
}

// getSendEnabled returns whether send is enabled and whether that flag was set
// for a denom.
//
// Example usage:
//
//	store := ctx.KVStore(k.storeKey)
//	sendEnabled, found := getSendEnabled(store, "atom")
//	if !found {
//	    sendEnabled = DefaultSendEnabled
//	}
func (k BaseSendKeeper) getSendEnabled(ctx context.Context, denom string) (bool, bool) {
	has, err := k.SendEnabled.Has(ctx, denom)
	if err != nil || !has {
		return false, false
	}

	v, err := k.SendEnabled.Get(ctx, denom)
	if err != nil {
		return false, false
	}

	return v, true
}

// getSendEnabledOrDefault gets the SendEnabled value for a denom. If it's not
// in the store, this will return defaultVal.
func (k BaseSendKeeper) getSendEnabledOrDefault(ctx context.Context, denom string, defaultVal bool) bool {
	sendEnabled, found := k.getSendEnabled(ctx, denom)
	if found {
		return sendEnabled
	}

	return defaultVal
}

// sendRestriction is a struct that houses a SendRestrictionFn.
// It exists so that the SendRestrictionFn can be updated in the SendKeeper without needing to have a pointer receiver.
type sendRestriction struct {
	fn types.SendRestrictionFn
}

// newSendRestriction creates a new sendRestriction with nil send restriction.
func newSendRestriction() *sendRestriction {
	return &sendRestriction{
		fn: nil,
	}
}

// append adds the provided restriction to this, to be run after the existing function.
func (r *sendRestriction) append(restriction types.SendRestrictionFn) {
	r.fn = r.fn.Then(restriction)
}

// prepend adds the provided restriction to this, to be run before the existing function.
func (r *sendRestriction) prepend(restriction types.SendRestrictionFn) {
	r.fn = restriction.Then(r.fn)
}

// clear removes the send restriction (sets it to nil).
func (r *sendRestriction) clear() {
	r.fn = nil
}

var _ types.SendRestrictionFn = (*sendRestriction)(nil).apply

// apply applies the send restriction if there is one. If not, it's a no-op.
func (r *sendRestriction) apply(ctx context.Context, fromAddr, toAddr sdk.AccAddress, amt sdk.Coins) (sdk.AccAddress, error) {
	if r == nil || r.fn == nil {
		return toAddr, nil
	}
	return r.fn(ctx, fromAddr, toAddr, amt)
}
