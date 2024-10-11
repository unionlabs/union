package keeper

import (
	"context"
	"fmt"

	"cosmossdk.io/collections"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/event"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/math"
	"cosmossdk.io/x/bank/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
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
	appmodule.Environment
	BaseViewKeeper

	cdc codec.BinaryCodec
	ak  types.AccountKeeper

	// list of addresses that are restricted from receiving transactions
	blockedAddrs map[string]bool

	// the address capable of executing a MsgUpdateParams message. Typically, this
	// should be the x/gov module account.
	authority string

	sendRestriction *sendRestriction
}

func NewBaseSendKeeper(
	env appmodule.Environment,
	cdc codec.BinaryCodec,
	ak types.AccountKeeper,
	blockedAddrs map[string]bool,
	authority string,
) BaseSendKeeper {
	if _, err := ak.AddressCodec().StringToBytes(authority); err != nil {
		panic(fmt.Errorf("invalid bank authority address: %w", err))
	}

	return BaseSendKeeper{
		Environment:     env,
		BaseViewKeeper:  NewBaseViewKeeper(env, cdc, ak),
		cdc:             cdc,
		ak:              ak,
		blockedAddrs:    blockedAddrs,
		authority:       authority,
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
	if len(params.SendEnabled) > 0 {
		k.SetAllSendEnabled(ctx, params.SendEnabled)

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

	// ensure all coins can be sent
	type toSend struct {
		AddressStr string
		Address    []byte
		Coins      sdk.Coins
	}
	sending := make([]toSend, 0)
	for _, out := range outputs {
		outAddress, err := k.ak.AddressCodec().StringToBytes(out.Address)
		if err != nil {
			return err
		}

		outAddress, err = k.sendRestriction.apply(ctx, inAddress, outAddress, out.Coins)
		if err != nil {
			return err
		}

		sending = append(sending, toSend{
			Address:    outAddress,
			AddressStr: out.Address,
			Coins:      out.Coins,
		})
	}

	if err := k.subUnlockedCoins(ctx, inAddress, input.Coins); err != nil {
		return err
	}

	for _, out := range sending {
		if err := k.addCoins(ctx, out.Address, out.Coins); err != nil {
			return err
		}

		if err := k.EventService.EventManager(ctx).EmitKV(
			types.EventTypeTransfer,
			event.NewAttribute(types.AttributeKeyRecipient, out.AddressStr),
			event.NewAttribute(types.AttributeKeySender, input.Address),
			event.NewAttribute(sdk.AttributeKeyAmount, out.Coins.String()),
		); err != nil {
			return err
		}
	}

	return nil
}

// SendCoins transfers amt coins from a sending account to a receiving account.
// An error is returned upon failure.
func (k BaseSendKeeper) SendCoins(ctx context.Context, fromAddr, toAddr sdk.AccAddress, amt sdk.Coins) error {
	if !amt.IsValid() {
		return errorsmod.Wrap(sdkerrors.ErrInvalidCoins, amt.String())
	}

	var err error
	toAddr, err = k.sendRestriction.apply(ctx, fromAddr, toAddr, amt)
	if err != nil {
		return err
	}

	err = k.subUnlockedCoins(ctx, fromAddr, amt)
	if err != nil {
		return err
	}

	err = k.addCoins(ctx, toAddr, amt)
	if err != nil {
		return err
	}

	fromAddrString, err := k.ak.AddressCodec().BytesToString(fromAddr)
	if err != nil {
		return err
	}
	toAddrString, err := k.ak.AddressCodec().BytesToString(toAddr)
	if err != nil {
		return err
	}

	return k.EventService.EventManager(ctx).EmitKV(
		types.EventTypeTransfer,
		event.NewAttribute(types.AttributeKeyRecipient, toAddrString),
		event.NewAttribute(types.AttributeKeySender, fromAddrString),
		event.NewAttribute(sdk.AttributeKeyAmount, amt.String()),
	)
}

// subUnlockedCoins removes the unlocked amt coins of the given account.
// An error is returned if the resulting balance is negative.
//
// CONTRACT: The provided amount (amt) must be valid, non-negative coins.
//
// A coin_spent event is emitted after the operation.
func (k BaseSendKeeper) subUnlockedCoins(ctx context.Context, addr sdk.AccAddress, amt sdk.Coins) error {
	lockedCoins := k.LockedCoins(ctx, addr)

	for _, coin := range amt {
		balance := k.GetBalance(ctx, addr, coin.Denom)
		ok, locked := lockedCoins.Find(coin.Denom)
		if !ok {
			locked = sdk.Coin{Denom: coin.Denom, Amount: math.ZeroInt()}
		}

		spendable, hasNeg := sdk.Coins{balance}.SafeSub(locked)
		if hasNeg {
			return errorsmod.Wrapf(sdkerrors.ErrInsufficientFunds,
				"locked amount exceeds account balance funds: %s > %s", locked, balance)
		}

		if _, hasNeg := spendable.SafeSub(coin); hasNeg {
			if len(spendable) == 0 {
				spendable = sdk.Coins{sdk.Coin{Denom: coin.Denom, Amount: math.ZeroInt()}}
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

	addrStr, err := k.ak.AddressCodec().BytesToString(addr)
	if err != nil {
		return err
	}

	return k.EventService.EventManager(ctx).EmitKV(
		types.EventTypeCoinSpent,
		event.NewAttribute(types.AttributeKeySpender, addrStr),
		event.NewAttribute(sdk.AttributeKeyAmount, amt.String()),
	)
}

// addCoins increases the balance of the given address by the specified amount.
//
// CONTRACT: The provided amount (amt) must be valid, non-negative coins.
//
// It emits a coin_received event after the operation.
func (k BaseSendKeeper) addCoins(ctx context.Context, addr sdk.AccAddress, amt sdk.Coins) error {
	for _, coin := range amt {
		balance := k.GetBalance(ctx, addr, coin.Denom)
		newBalance := balance.Add(coin)

		err := k.setBalance(ctx, addr, newBalance)
		if err != nil {
			return err
		}
	}

	addrStr, err := k.ak.AddressCodec().BytesToString(addr)
	if err != nil {
		return err
	}

	return k.EventService.EventManager(ctx).EmitKV(
		types.EventTypeCoinReceived,
		event.NewAttribute(types.AttributeKeyReceiver, addrStr),
		event.NewAttribute(sdk.AttributeKeyAmount, amt.String()),
	)
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
	addrStr, err := k.ak.AddressCodec().BytesToString(addr)
	if err != nil {
		panic(err)
	}
	return k.blockedAddrs[addrStr]
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
