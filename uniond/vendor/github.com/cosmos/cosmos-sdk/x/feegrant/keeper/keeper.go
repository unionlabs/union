package keeper

import (
	"fmt"
	"time"

	"github.com/cometbft/cometbft/libs/log"

	storetypes "github.com/cosmos/cosmos-sdk/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/auth/ante"
	"github.com/cosmos/cosmos-sdk/x/feegrant"
)

// Keeper manages state of all fee grants, as well as calculating approval.
// It must have a codec with all available allowances registered.
type Keeper struct {
	cdc        codec.BinaryCodec
	storeKey   storetypes.StoreKey
	authKeeper feegrant.AccountKeeper
}

var _ ante.FeegrantKeeper = &Keeper{}

// NewKeeper creates a fee grant Keeper
func NewKeeper(cdc codec.BinaryCodec, storeKey storetypes.StoreKey, ak feegrant.AccountKeeper) Keeper {
	return Keeper{
		cdc:        cdc,
		storeKey:   storeKey,
		authKeeper: ak,
	}
}

// Logger returns a module-specific logger.
func (k Keeper) Logger(ctx sdk.Context) log.Logger {
	return ctx.Logger().With("module", fmt.Sprintf("x/%s", feegrant.ModuleName))
}

// GrantAllowance creates a new grant
func (k Keeper) GrantAllowance(ctx sdk.Context, granter, grantee sdk.AccAddress, feeAllowance feegrant.FeeAllowanceI) error {
	// create the account if it is not in account state
	granteeAcc := k.authKeeper.GetAccount(ctx, grantee)
	if granteeAcc == nil {
		granteeAcc = k.authKeeper.NewAccountWithAddress(ctx, grantee)
		k.authKeeper.SetAccount(ctx, granteeAcc)
	}

	store := ctx.KVStore(k.storeKey)
	key := feegrant.FeeAllowanceKey(granter, grantee)

	var oldExp *time.Time
	existingGrant, err := k.getGrant(ctx, granter, grantee)

	// If we didn't find any grant, we don't return any error.
	// All other kinds of errors are returned.
	if err != nil && !sdkerrors.IsOf(err, sdkerrors.ErrNotFound) {
		return err
	}

	if existingGrant != nil && existingGrant.GetAllowance() != nil {
		grantInfo, err := existingGrant.GetGrant()
		if err != nil {
			return err
		}

		oldExp, err = grantInfo.ExpiresAt()
		if err != nil {
			return err
		}
	}

	newExp, err := feeAllowance.ExpiresAt()
	switch {
	case err != nil:
		return err

	case newExp != nil && newExp.Before(ctx.BlockTime()):
		return sdkerrors.Wrap(sdkerrors.ErrInvalidRequest, "expiration is before current block time")

	case oldExp == nil && newExp != nil:
		// when old oldExp is nil there won't be any key added before to queue.
		// add the new key to queue directly.
		k.addToFeeAllowanceQueue(ctx, key[1:], newExp)

	case oldExp != nil && newExp == nil:
		// when newExp is nil no need of adding the key to the pruning queue
		// remove the old key from queue.
		k.removeFromGrantQueue(ctx, oldExp, key[1:])

	case oldExp != nil && newExp != nil && !oldExp.Equal(*newExp):
		// `key` formed here with the prefix of `FeeAllowanceKeyPrefix` (which is `0x00`)
		// remove the 1st byte and reuse the remaining key as it is.

		// remove the old key from queue.
		k.removeFromGrantQueue(ctx, oldExp, key[1:])

		// add the new key to queue.
		k.addToFeeAllowanceQueue(ctx, key[1:], newExp)
	}

	grant, err := feegrant.NewGrant(granter, grantee, feeAllowance)
	if err != nil {
		return err
	}

	bz, err := k.cdc.Marshal(&grant)
	if err != nil {
		return err
	}

	store.Set(key, bz)
	ctx.EventManager().EmitEvent(
		sdk.NewEvent(
			feegrant.EventTypeSetFeeGrant,
			sdk.NewAttribute(feegrant.AttributeKeyGranter, grant.Granter),
			sdk.NewAttribute(feegrant.AttributeKeyGrantee, grant.Grantee),
		),
	)

	return nil
}

// UpdateAllowance updates the existing grant.
func (k Keeper) UpdateAllowance(ctx sdk.Context, granter, grantee sdk.AccAddress, feeAllowance feegrant.FeeAllowanceI) error {
	store := ctx.KVStore(k.storeKey)
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

	store.Set(key, bz)

	ctx.EventManager().EmitEvent(
		sdk.NewEvent(
			feegrant.EventTypeUpdateFeeGrant,
			sdk.NewAttribute(feegrant.AttributeKeyGranter, grant.Granter),
			sdk.NewAttribute(feegrant.AttributeKeyGrantee, grant.Grantee),
		),
	)

	return nil
}

// revokeAllowance removes an existing grant
func (k Keeper) revokeAllowance(ctx sdk.Context, granter, grantee sdk.AccAddress) error {
	_, err := k.getGrant(ctx, granter, grantee)
	if err != nil {
		return err
	}

	store := ctx.KVStore(k.storeKey)
	key := feegrant.FeeAllowanceKey(granter, grantee)
	store.Delete(key)

	ctx.EventManager().EmitEvent(
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
func (k Keeper) GetAllowance(ctx sdk.Context, granter, grantee sdk.AccAddress) (feegrant.FeeAllowanceI, error) {
	grant, err := k.getGrant(ctx, granter, grantee)
	if err != nil {
		return nil, err
	}

	return grant.GetGrant()
}

// getGrant returns entire grant between both accounts
func (k Keeper) getGrant(ctx sdk.Context, granter sdk.AccAddress, grantee sdk.AccAddress) (*feegrant.Grant, error) {
	store := ctx.KVStore(k.storeKey)
	key := feegrant.FeeAllowanceKey(granter, grantee)
	bz := store.Get(key)
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
func (k Keeper) IterateAllFeeAllowances(ctx sdk.Context, cb func(grant feegrant.Grant) bool) error {
	store := ctx.KVStore(k.storeKey)
	iter := sdk.KVStorePrefixIterator(store, feegrant.FeeAllowanceKeyPrefix)
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
func (k Keeper) UseGrantedFees(ctx sdk.Context, granter, grantee sdk.AccAddress, fee sdk.Coins, msgs []sdk.Msg) error {
	f, err := k.getGrant(ctx, granter, grantee)
	if err != nil {
		return err
	}

	grant, err := f.GetGrant()
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

func emitUseGrantEvent(ctx sdk.Context, granter, grantee string) {
	ctx.EventManager().EmitEvent(
		sdk.NewEvent(
			feegrant.EventTypeUseFeeGrant,
			sdk.NewAttribute(feegrant.AttributeKeyGranter, granter),
			sdk.NewAttribute(feegrant.AttributeKeyGrantee, grantee),
		),
	)
}

// InitGenesis will initialize the keeper from a *previously validated* GenesisState
func (k Keeper) InitGenesis(ctx sdk.Context, data *feegrant.GenesisState) error {
	for _, f := range data.Allowances {
		granter, err := sdk.AccAddressFromBech32(f.Granter)
		if err != nil {
			return err
		}
		grantee, err := sdk.AccAddressFromBech32(f.Grantee)
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
func (k Keeper) ExportGenesis(ctx sdk.Context) (*feegrant.GenesisState, error) {
	var grants []feegrant.Grant

	err := k.IterateAllFeeAllowances(ctx, func(grant feegrant.Grant) bool {
		grants = append(grants, grant)
		return false
	})

	return &feegrant.GenesisState{
		Allowances: grants,
	}, err
}

func (k Keeper) removeFromGrantQueue(ctx sdk.Context, exp *time.Time, allowanceKey []byte) {
	key := feegrant.FeeAllowancePrefixQueue(exp, allowanceKey)
	store := ctx.KVStore(k.storeKey)
	store.Delete(key)
}

func (k Keeper) addToFeeAllowanceQueue(ctx sdk.Context, grantKey []byte, exp *time.Time) {
	store := ctx.KVStore(k.storeKey)
	store.Set(feegrant.FeeAllowancePrefixQueue(exp, grantKey), []byte{})
}

// RemoveExpiredAllowances iterates grantsByExpiryQueue and deletes the expired grants.
func (k Keeper) RemoveExpiredAllowances(ctx sdk.Context) {
	exp := ctx.BlockTime()
	store := ctx.KVStore(k.storeKey)
	iterator := store.Iterator(feegrant.FeeAllowanceQueueKeyPrefix, sdk.InclusiveEndBytes(feegrant.AllowanceByExpTimeKey(&exp)))
	defer iterator.Close()

	for ; iterator.Valid(); iterator.Next() {
		store.Delete(iterator.Key())

		granter, grantee := feegrant.ParseAddressesFromFeeAllowanceQueueKey(iterator.Key())
		store.Delete(feegrant.FeeAllowanceKey(granter, grantee))
	}
}
