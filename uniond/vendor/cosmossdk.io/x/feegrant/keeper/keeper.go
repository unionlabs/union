package keeper

import (
	"context"
	"fmt"
	"time"

	"cosmossdk.io/core/store"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/log"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/feegrant"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/runtime"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/auth/ante"
)

// Keeper manages state of all fee grants, as well as calculating approval.
// It must have a codec with all available allowances registered.
type Keeper struct {
	cdc          codec.BinaryCodec
	storeService store.KVStoreService
	authKeeper   feegrant.AccountKeeper
	bankKeeper   feegrant.BankKeeper
}

var _ ante.FeegrantKeeper = &Keeper{}

// NewKeeper creates a feegrant Keeper
func NewKeeper(cdc codec.BinaryCodec, storeService store.KVStoreService, ak feegrant.AccountKeeper) Keeper {
	return Keeper{
		cdc:          cdc,
		storeService: storeService,
		authKeeper:   ak,
	}
}

// Super ugly hack to not be breaking in v0.50 and v0.47
// DO NOT USE.
func (k Keeper) SetBankKeeper(bk feegrant.BankKeeper) Keeper {
	k.bankKeeper = bk
	return k
}

// Logger returns a module-specific logger.
func (k Keeper) Logger(ctx sdk.Context) log.Logger {
	return ctx.Logger().With("module", fmt.Sprintf("x/%s", feegrant.ModuleName))
}

// GrantAllowance creates a new grant
func (k Keeper) GrantAllowance(ctx context.Context, granter, grantee sdk.AccAddress, feeAllowance feegrant.FeeAllowanceI) error {
	// Checking for duplicate entry
	if f, _ := k.GetAllowance(ctx, granter, grantee); f != nil {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "fee allowance already exists")
	}

	// create the account if it is not in account state
	granteeAcc := k.authKeeper.GetAccount(ctx, grantee)
	if granteeAcc == nil {
		if k.bankKeeper.BlockedAddr(grantee) {
			return errorsmod.Wrapf(sdkerrors.ErrUnauthorized, "%s is not allowed to receive funds", grantee)
		}

		granteeAcc = k.authKeeper.NewAccountWithAddress(ctx, grantee)
		k.authKeeper.SetAccount(ctx, granteeAcc)
	}

	store := k.storeService.OpenKVStore(ctx)
	key := feegrant.FeeAllowanceKey(granter, grantee)

	exp, err := feeAllowance.ExpiresAt()
	if err != nil {
		return err
	}

	// expiration shouldn't be in the past.
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	if exp != nil && exp.Before(sdkCtx.BlockTime()) {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "expiration is before current block time")
	}

	// if expiry is not nil, add the new key to pruning queue.
	if exp != nil {
		// `key` formed here with the prefix of `FeeAllowanceKeyPrefix` (which is `0x00`)
		// remove the 1st byte and reuse the remaining key as it is
		err = k.addToFeeAllowanceQueue(ctx, key[1:], exp)
		if err != nil {
			return err
		}
	}

	grant, err := feegrant.NewGrant(granter, grantee, feeAllowance)
	if err != nil {
		return err
	}

	bz, err := k.cdc.Marshal(&grant)
	if err != nil {
		return err
	}

	err = store.Set(key, bz)
	if err != nil {
		return err
	}

	sdkCtx.EventManager().EmitEvent(
		sdk.NewEvent(
			feegrant.EventTypeSetFeeGrant,
			sdk.NewAttribute(feegrant.AttributeKeyGranter, grant.Granter),
			sdk.NewAttribute(feegrant.AttributeKeyGrantee, grant.Grantee),
		),
	)

	return nil
}

// UpdateAllowance updates the existing grant.
func (k Keeper) UpdateAllowance(ctx context.Context, granter, grantee sdk.AccAddress, feeAllowance feegrant.FeeAllowanceI) error {
	store := k.storeService.OpenKVStore(ctx)
	key := feegrant.FeeAllowanceKey(granter, grantee)

	_, err := k.getGrant(ctx, granter, grantee)
	if err != nil {
		return err
	}

	grant, err := feegrant.NewGrant(granter, grantee, feeAllowance)
	if err != nil {
		return err
	}

	bz, err := k.cdc.Marshal(&grant)
	if err != nil {
		return err
	}

	err = store.Set(key, bz)
	if err != nil {
		return err
	}

	sdk.UnwrapSDKContext(ctx).EventManager().EmitEvent(
		sdk.NewEvent(
			feegrant.EventTypeUpdateFeeGrant,
			sdk.NewAttribute(feegrant.AttributeKeyGranter, grant.Granter),
			sdk.NewAttribute(feegrant.AttributeKeyGrantee, grant.Grantee),
		),
	)

	return nil
}

// revokeAllowance removes an existing grant
func (k Keeper) revokeAllowance(ctx context.Context, granter, grantee sdk.AccAddress) error {
	grant, err := k.GetAllowance(ctx, granter, grantee)
	if err != nil {
		return err
	}

	store := k.storeService.OpenKVStore(ctx)
	key := feegrant.FeeAllowanceKey(granter, grantee)
	err = store.Delete(key)
	if err != nil {
		return err
	}

	exp, err := grant.ExpiresAt()
	if err != nil {
		return err
	}

	if exp != nil {
		if err := store.Delete(feegrant.FeeAllowancePrefixQueue(exp, feegrant.FeeAllowanceKey(grantee, granter)[1:])); err != nil {
			return err
		}
	}

	sdk.UnwrapSDKContext(ctx).EventManager().EmitEvent(
		sdk.NewEvent(
			feegrant.EventTypeRevokeFeeGrant,
			sdk.NewAttribute(feegrant.AttributeKeyGranter, granter.String()),
			sdk.NewAttribute(feegrant.AttributeKeyGrantee, grantee.String()),
		),
	)
	return nil
}

// GetAllowance returns the allowance between the granter and grantee.
// If there is none, it returns nil, nil.
// Returns an error on parsing issues
func (k Keeper) GetAllowance(ctx context.Context, granter, grantee sdk.AccAddress) (feegrant.FeeAllowanceI, error) {
	grant, err := k.getGrant(ctx, granter, grantee)
	if err != nil {
		return nil, err
	}

	return grant.GetGrant()
}

// getGrant returns entire grant between both accounts
func (k Keeper) getGrant(ctx context.Context, granter, grantee sdk.AccAddress) (*feegrant.Grant, error) {
	store := k.storeService.OpenKVStore(ctx)
	key := feegrant.FeeAllowanceKey(granter, grantee)
	bz, err := store.Get(key)
	if err != nil {
		return nil, err
	}

	if len(bz) == 0 {
		return nil, sdkerrors.ErrNotFound.Wrap("fee-grant not found")
	}

	var feegrant feegrant.Grant
	if err := k.cdc.Unmarshal(bz, &feegrant); err != nil {
		return nil, err
	}

	return &feegrant, nil
}

// IterateAllFeeAllowances iterates over all the grants in the store.
// Callback to get all data, returns true to stop, false to keep reading
// Calling this without pagination is very expensive and only designed for export genesis
func (k Keeper) IterateAllFeeAllowances(ctx context.Context, cb func(grant feegrant.Grant) bool) error {
	store := k.storeService.OpenKVStore(ctx)
	iter := storetypes.KVStorePrefixIterator(runtime.KVStoreAdapter(store), feegrant.FeeAllowanceKeyPrefix)
	defer iter.Close()

	stop := false
	for ; iter.Valid() && !stop; iter.Next() {
		bz := iter.Value()
		var feeGrant feegrant.Grant
		if err := k.cdc.Unmarshal(bz, &feeGrant); err != nil {
			return err
		}
		stop = cb(feeGrant)
	}

	return nil
}

// UseGrantedFees will try to pay the given fee from the granter's account as requested by the grantee
func (k Keeper) UseGrantedFees(ctx context.Context, granter, grantee sdk.AccAddress, fee sdk.Coins, msgs []sdk.Msg) error {
	grant, err := k.GetAllowance(ctx, granter, grantee)
	if err != nil {
		return err
	}

	remove, err := grant.Accept(ctx, fee, msgs)

	if remove {
		// Ignoring the `revokeFeeAllowance` error, because the user has enough grants to perform this transaction.
		k.revokeAllowance(ctx, granter, grantee)
		if err != nil {
			return err
		}

		emitUseGrantEvent(ctx, granter.String(), grantee.String())

		return nil
	}

	if err != nil {
		return err
	}

	emitUseGrantEvent(ctx, granter.String(), grantee.String())

	// if fee allowance is accepted, store the updated state of the allowance
	return k.UpdateAllowance(ctx, granter, grantee, grant)
}

func emitUseGrantEvent(ctx context.Context, granter, grantee string) {
	sdk.UnwrapSDKContext(ctx).EventManager().EmitEvent(
		sdk.NewEvent(
			feegrant.EventTypeUseFeeGrant,
			sdk.NewAttribute(feegrant.AttributeKeyGranter, granter),
			sdk.NewAttribute(feegrant.AttributeKeyGrantee, grantee),
		),
	)
}

// InitGenesis will initialize the keeper from a *previously validated* GenesisState
func (k Keeper) InitGenesis(ctx context.Context, data *feegrant.GenesisState) error {
	for _, f := range data.Allowances {
		granter, err := k.authKeeper.AddressCodec().StringToBytes(f.Granter)
		if err != nil {
			return err
		}
		grantee, err := k.authKeeper.AddressCodec().StringToBytes(f.Grantee)
		if err != nil {
			return err
		}

		grant, err := f.GetGrant()
		if err != nil {
			return err
		}

		err = k.GrantAllowance(ctx, granter, grantee, grant)
		if err != nil {
			return err
		}
	}
	return nil
}

// ExportGenesis will dump the contents of the keeper into a serializable GenesisState.
func (k Keeper) ExportGenesis(ctx context.Context) (*feegrant.GenesisState, error) {
	var grants []feegrant.Grant

	err := k.IterateAllFeeAllowances(ctx, func(grant feegrant.Grant) bool {
		grants = append(grants, grant)
		return false
	})

	return &feegrant.GenesisState{
		Allowances: grants,
	}, err
}

func (k Keeper) addToFeeAllowanceQueue(ctx context.Context, grantKey []byte, exp *time.Time) error {
	store := k.storeService.OpenKVStore(ctx)
	return store.Set(feegrant.FeeAllowancePrefixQueue(exp, grantKey), []byte{})
}

// RemoveExpiredAllowances iterates grantsByExpiryQueue and deletes the expired grants.
func (k Keeper) RemoveExpiredAllowances(ctx context.Context, limit int32) error {
	exp := sdk.UnwrapSDKContext(ctx).BlockTime()
	store := k.storeService.OpenKVStore(ctx)
	iterator, err := store.Iterator(feegrant.FeeAllowanceQueueKeyPrefix, storetypes.InclusiveEndBytes(feegrant.AllowanceByExpTimeKey(&exp)))
	var count int32
	if err != nil {
		return err
	}
	defer iterator.Close()

	for ; iterator.Valid(); iterator.Next() {
		err = store.Delete(iterator.Key())
		if err != nil {
			return err
		}

		granter, grantee := feegrant.ParseAddressesFromFeeAllowanceQueueKey(iterator.Key())
		err = store.Delete(feegrant.FeeAllowanceKey(granter, grantee))
		if err != nil {
			return err
		}

		// limit the amount of iterations to avoid taking too much time
		count++
		if count == limit {
			return nil
		}
	}
	return nil
}
