package keeper

import (
	"context"
	"time"

	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/slashing/types"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var _ types.StakingHooks = Hooks{}

// Hooks wrapper struct for slashing keeper
type Hooks struct {
	k Keeper
}

// Return the slashing hooks
func (k Keeper) Hooks() Hooks {
	return Hooks{k}
}

// AfterValidatorBonded updates the signing info start height or create a new signing info
func (h Hooks) AfterValidatorBonded(ctx context.Context, consAddr sdk.ConsAddress, valAddr sdk.ValAddress) error {
	signingInfo, err := h.k.ValidatorSigningInfo.Get(ctx, consAddr)
	blockHeight := h.k.HeaderService.HeaderInfo(ctx).Height
	if err == nil {
		signingInfo.StartHeight = blockHeight
	} else {
		consStr, err := h.k.sk.ConsensusAddressCodec().BytesToString(consAddr)
		if err != nil {
			return err
		}
		signingInfo = types.NewValidatorSigningInfo(
			consStr,
			blockHeight,
			time.Unix(0, 0),
			false,
			0,
		)
	}

	return h.k.ValidatorSigningInfo.Set(ctx, consAddr, signingInfo)
}

// AfterValidatorRemoved deletes the address-pubkey relation when a validator is removed,
func (h Hooks) AfterValidatorRemoved(ctx context.Context, consAddr sdk.ConsAddress, _ sdk.ValAddress) error {
	return h.k.AddrPubkeyRelation.Remove(ctx, consAddr)
}

// AfterValidatorCreated adds the address-pubkey relation when a validator is created.
func (h Hooks) AfterValidatorCreated(ctx context.Context, valAddr sdk.ValAddress) error {
	validator, err := h.k.sk.Validator(ctx, valAddr)
	if err != nil {
		return err
	}

	consPk, err := validator.ConsPubKey()
	if err != nil {
		return err
	}

	return h.k.AddrPubkeyRelation.Set(ctx, consPk.Address(), consPk)
}

func (h Hooks) AfterValidatorBeginUnbonding(_ context.Context, _ sdk.ConsAddress, _ sdk.ValAddress) error {
	return nil
}

func (h Hooks) BeforeValidatorModified(_ context.Context, _ sdk.ValAddress) error {
	return nil
}

func (h Hooks) BeforeDelegationCreated(_ context.Context, _ sdk.AccAddress, _ sdk.ValAddress) error {
	return nil
}

func (h Hooks) BeforeDelegationSharesModified(_ context.Context, _ sdk.AccAddress, _ sdk.ValAddress) error {
	return nil
}

func (h Hooks) BeforeDelegationRemoved(_ context.Context, _ sdk.AccAddress, _ sdk.ValAddress) error {
	return nil
}

func (h Hooks) AfterDelegationModified(_ context.Context, _ sdk.AccAddress, _ sdk.ValAddress) error {
	return nil
}

func (h Hooks) BeforeValidatorSlashed(_ context.Context, _ sdk.ValAddress, _ sdkmath.LegacyDec) error {
	return nil
}

func (h Hooks) AfterUnbondingInitiated(_ context.Context, _ uint64) error {
	return nil
}

// AfterConsensusPubKeyUpdate handles the rotation of signing info and updates the address-pubkey relation after a consensus key update.
func (h Hooks) AfterConsensusPubKeyUpdate(ctx context.Context, oldPubKey, newPubKey cryptotypes.PubKey, _ sdk.Coin) error {
	if err := h.k.performConsensusPubKeyUpdate(ctx, oldPubKey, newPubKey); err != nil {
		return err
	}

	if err := h.k.AddrPubkeyRelation.Remove(ctx, oldPubKey.Address()); err != nil {
		return err
	}

	return nil
}
