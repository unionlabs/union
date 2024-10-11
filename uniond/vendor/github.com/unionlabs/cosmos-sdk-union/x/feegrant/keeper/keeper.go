package keeper

import (
	"context"
	"time"

	"cosmossdk.io/collections"
	"cosmossdk.io/core/appmodule"
	corecontext "cosmossdk.io/core/context"
	"cosmossdk.io/core/event"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/x/feegrant"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/auth/ante"
)

// Keeper manages state of all fee grants, as well as calculating approval.
// It must have a codec with all available allowances registered.
type Keeper struct {
	appmodule.Environment

	cdc        codec.BinaryCodec
	authKeeper feegrant.AccountKeeper
	Schema     collections.Schema
	// FeeAllowance key: grantee+granter | value: Grant
	FeeAllowance collections.Map[collections.Pair[sdk.AccAddress, sdk.AccAddress], feegrant.Grant]
	// FeeAllowanceQueue key: expiration time+grantee+granter | value: bool
	FeeAllowanceQueue collections.Map[collections.Triple[time.Time, sdk.AccAddress, sdk.AccAddress], bool]
}

var _ ante.FeegrantKeeper = &Keeper{}

// NewKeeper creates a feegrant Keeper
func NewKeeper(env appmodule.Environment, cdc codec.BinaryCodec, ak feegrant.AccountKeeper) Keeper {
	sb := collections.NewSchemaBuilder(env.KVStoreService)

	return Keeper{
		Environment: env,
		cdc:         cdc,
		authKeeper:  ak,
		FeeAllowance: collections.NewMap(
			sb,
			feegrant.FeeAllowanceKeyPrefix,
			"allowances",
			collections.PairKeyCodec(sdk.LengthPrefixedAddressKey(sdk.AccAddressKey), sdk.LengthPrefixedAddressKey(sdk.AccAddressKey)), //nolint: staticcheck // sdk.LengthPrefixedAddressKey is needed to retain state compatibility
			codec.CollValue[feegrant.Grant](cdc),
		),
		FeeAllowanceQueue: collections.NewMap(
			sb,
			feegrant.FeeAllowanceQueueKeyPrefix,
			"allowances_queue",
			collections.TripleKeyCodec(sdk.TimeKey, sdk.LengthPrefixedAddressKey(sdk.AccAddressKey), sdk.LengthPrefixedAddressKey(sdk.AccAddressKey)), //nolint: staticcheck // sdk.LengthPrefixedAddressKey is needed to retain state compatibility
			collections.BoolValue,
		),
	}
}

// GrantAllowance creates a new grant
func (k Keeper) GrantAllowance(ctx context.Context, granter, grantee sdk.AccAddress, feeAllowance feegrant.FeeAllowanceI) error {
	// Checking for duplicate entry
	if f, _ := k.GetAllowance(ctx, granter, grantee); f != nil {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "fee allowance already exists")
	}

	exp, err := feeAllowance.ExpiresAt()
	if err != nil {
		return err
	}

	// expiration shouldn't be in the past.

	now := k.HeaderService.HeaderInfo(ctx).Time
	if exp != nil && exp.Before(now) {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "expiration is before current block time")
	}

	// if expiry is not nil, add the new key to pruning queue.
	if exp != nil {
		err = k.FeeAllowanceQueue.Set(ctx, collections.Join3(*exp, grantee, granter), true)
		if err != nil {
			return err
		}
	}

	granterStr, err := k.authKeeper.AddressCodec().BytesToString(granter)
	if err != nil {
		return err
	}
	granteeStr, err := k.authKeeper.AddressCodec().BytesToString(grantee)
	if err != nil {
		return err
	}

	// if block time is not zero, update the period reset
	// if it is zero, it could be genesis initialization, so we don't need to update the period reset
	if !now.IsZero() {
		err = feeAllowance.UpdatePeriodReset(now)
		if err != nil {
			return err
		}
	}

	grant, err := feegrant.NewGrant(granterStr, granteeStr, feeAllowance)
	if err != nil {
		return err
	}

	if err := k.FeeAllowance.Set(ctx, collections.Join(grantee, granter), grant); err != nil {
		return err
	}

	return k.EventService.EventManager(ctx).EmitKV(
		feegrant.EventTypeSetFeeGrant,
		event.NewAttribute(feegrant.AttributeKeyGranter, grant.Granter),
		event.NewAttribute(feegrant.AttributeKeyGrantee, grant.Grantee),
	)
}

// UpdateAllowance updates the existing grant.
func (k Keeper) UpdateAllowance(ctx context.Context, granter, grantee sdk.AccAddress, feeAllowance feegrant.FeeAllowanceI) error {
	_, err := k.GetAllowance(ctx, granter, grantee)
	if err != nil {
		return err
	}

	granterStr, err := k.authKeeper.AddressCodec().BytesToString(granter)
	if err != nil {
		return err
	}
	granteeStr, err := k.authKeeper.AddressCodec().BytesToString(grantee)
	if err != nil {
		return err
	}

	grant, err := feegrant.NewGrant(granterStr, granteeStr, feeAllowance)
	if err != nil {
		return err
	}

	if err := k.FeeAllowance.Set(ctx, collections.Join(grantee, granter), grant); err != nil {
		return err
	}

	return k.EventService.EventManager(ctx).EmitKV(
		feegrant.EventTypeUpdateFeeGrant,
		event.NewAttribute(feegrant.AttributeKeyGranter, grant.Granter),
		event.NewAttribute(feegrant.AttributeKeyGrantee, grant.Grantee),
	)
}

// revokeAllowance removes an existing grant
func (k Keeper) revokeAllowance(ctx context.Context, granter, grantee sdk.AccAddress) error {
	grant, err := k.GetAllowance(ctx, granter, grantee)
	if err != nil {
		return err
	}

	if err := k.FeeAllowance.Remove(ctx, collections.Join(grantee, granter)); err != nil {
		return err
	}

	exp, err := grant.ExpiresAt()
	if err != nil {
		return err
	}

	if exp != nil {
		if err := k.FeeAllowanceQueue.Remove(ctx, collections.Join3(*exp, grantee, granter)); err != nil {
			return err
		}
	}

	granterStr, err := k.authKeeper.AddressCodec().BytesToString(granter)
	if err != nil {
		return err
	}
	granteeStr, err := k.authKeeper.AddressCodec().BytesToString(grantee)
	if err != nil {
		return err
	}

	return k.EventService.EventManager(ctx).EmitKV(
		feegrant.EventTypeRevokeFeeGrant,
		event.NewAttribute(feegrant.AttributeKeyGranter, granterStr),
		event.NewAttribute(feegrant.AttributeKeyGrantee, granteeStr),
	)
}

// GetAllowance returns the allowance between the granter and grantee.
// If there is none, it returns nil, collections.ErrNotFound.
// Returns an error on parsing issues
func (k Keeper) GetAllowance(ctx context.Context, granter, grantee sdk.AccAddress) (feegrant.FeeAllowanceI, error) {
	grant, err := k.FeeAllowance.Get(ctx, collections.Join(grantee, granter))
	if err != nil {
		return nil, err
	}

	return grant.GetGrant()
}

// IterateAllFeeAllowances iterates over all the grants in the store.
// Callback to get all data, returns true to stop, false to keep reading
// Calling this without pagination is very expensive and only designed for export genesis
func (k Keeper) IterateAllFeeAllowances(ctx context.Context, cb func(grant feegrant.Grant) bool) error {
	return k.FeeAllowance.Walk(ctx, nil, func(key collections.Pair[sdk.AccAddress, sdk.AccAddress], grant feegrant.Grant) (stop bool, err error) {
		return cb(grant), nil
	})
}

// UseGrantedFees will try to pay the given fee from the granter's account as requested by the grantee
func (k Keeper) UseGrantedFees(ctx context.Context, granter, grantee sdk.AccAddress, fee sdk.Coins, msgs []sdk.Msg) error {
	grant, err := k.GetAllowance(ctx, granter, grantee)
	if err != nil {
		return err
	}

	granterStr, err := k.authKeeper.AddressCodec().BytesToString(granter)
	if err != nil {
		return err
	}
	granteeStr, err := k.authKeeper.AddressCodec().BytesToString(grantee)
	if err != nil {
		return err
	}

	remove, err := grant.Accept(context.WithValue(ctx, corecontext.EnvironmentContextKey, k.Environment), fee, msgs)
	if remove && err == nil {
		// Ignoring the `revokeFeeAllowance` error, because the user has enough grants to perform this transaction.
		_ = k.revokeAllowance(ctx, granter, grantee)

		return k.emitUseGrantEvent(ctx, granterStr, granteeStr)
	}
	if err != nil {
		return err
	}
	if err := k.emitUseGrantEvent(ctx, granterStr, granteeStr); err != nil {
		return err
	}

	// if fee allowance is accepted, store the updated state of the allowance
	return k.UpdateAllowance(ctx, granter, grantee, grant)
}

func (k *Keeper) emitUseGrantEvent(ctx context.Context, granter, grantee string) error {
	return k.EventService.EventManager(ctx).EmitKV(
		feegrant.EventTypeUseFeeGrant,
		event.NewAttribute(feegrant.AttributeKeyGranter, granter),
		event.NewAttribute(feegrant.AttributeKeyGrantee, grantee),
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

// RemoveExpiredAllowances iterates grantsByExpiryQueue and deletes the expired grants.
func (k Keeper) RemoveExpiredAllowances(ctx context.Context, limit int) error {
	exp := k.HeaderService.HeaderInfo(ctx).Time
	rng := collections.NewPrefixUntilTripleRange[time.Time, sdk.AccAddress, sdk.AccAddress](exp)
	count := 0

	keysToRemove := []collections.Triple[time.Time, sdk.AccAddress, sdk.AccAddress]{}
	err := k.FeeAllowanceQueue.Walk(ctx, rng, func(key collections.Triple[time.Time, sdk.AccAddress, sdk.AccAddress], value bool) (stop bool, err error) {
		grantee, granter := key.K2(), key.K3()

		if err := k.FeeAllowance.Remove(ctx, collections.Join(grantee, granter)); err != nil {
			return true, err
		}

		keysToRemove = append(keysToRemove, key)

		// limit the amount of iterations to avoid taking too much time
		count++
		if count == limit {
			return true, nil
		}

		return false, nil
	})
	if err != nil {
		return err
	}

	for _, key := range keysToRemove {
		if err := k.FeeAllowanceQueue.Remove(ctx, key); err != nil {
			return err
		}
	}

	return nil
}
