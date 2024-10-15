package keeper

import (
	"context"
	"fmt"

	st "cosmossdk.io/api/cosmos/staking/v1beta1"
	"cosmossdk.io/x/evidence/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// HandleEquivocationEvidence implements an equivocation evidence handler. Assuming the
// evidence is valid, the validator committing the misbehavior will be slashed,
// jailed and tombstoned. Once tombstoned, the validator will not be able to
// recover. Note, the evidence contains the block time and height at the time of
// the equivocation.
//
// The evidence is considered invalid if:
// - the evidence is too old
// - the validator is unbonded or does not exist
// - the signing info does not exist (will panic)
// - is already tombstoned
//
// TODO: Some of the invalid constraints listed above may need to be reconsidered
// in the case of a lunatic attack.
func (k Keeper) handleEquivocationEvidence(ctx context.Context, evidence *types.Equivocation) error {
	consAddr := evidence.GetConsensusAddress(k.stakingKeeper.ConsensusAddressCodec())

	validator, err := k.stakingKeeper.ValidatorByConsAddr(ctx, consAddr)
	if err != nil {
		return err
	}
	if validator == nil || validator.IsUnbonded() {
		// Defensive: Simulation doesn't take unbonding periods into account, and
		// CometBFT might break this assumption at some point.
		return nil
	}

	if len(validator.GetOperator()) != 0 {
		// Get the consAddr from the validator read from the store and not from the evidence,
		// because if the validator has rotated its key, the key in evidence could be outdated.
		// (ValidatorByConsAddr can get a validator even if the key has been rotated)
		valConsAddr, err := validator.GetConsAddr()
		if err != nil {
			return err
		}
		consAddr = valConsAddr

		if _, err := k.slashingKeeper.GetPubkey(ctx, consAddr.Bytes()); err != nil {
			// Ignore evidence that cannot be handled.
			//
			// NOTE: We used to panic with:
			// `panic(fmt.Sprintf("Validator consensus-address %v not found", consAddr))`,
			// but this couples the expectations of the app to both CometBFT and
			// the simulator.  Both are expected to provide the full range of
			// allowable but none of the disallowed evidence types.  Instead of
			// getting this coordination right, it is easier to relax the
			// constraints and ignore evidence that cannot be handled.
			k.Logger.Error(fmt.Sprintf("ignore evidence; expected public key for validator %s not found", consAddr))
			return nil
		}
	}

	headerInfo := k.HeaderService.HeaderInfo(ctx)
	// calculate the age of the evidence
	infractionHeight := evidence.GetHeight()
	infractionTime := evidence.GetTime()
	ageDuration := headerInfo.Time.Sub(infractionTime)
	ageBlocks := headerInfo.Height - infractionHeight

	// Reject evidence if the double-sign is too old. Evidence is considered stale
	// if the difference in time and number of blocks is greater than the allowed
	// parameters defined.

	eviAgeBlocks, eviAgeDuration, _, err := k.consensusKeeper.EvidenceParams(ctx)
	if err == nil && ageDuration > eviAgeDuration && ageBlocks > eviAgeBlocks {
		k.Logger.Info(
			"ignored equivocation; evidence too old",
			"validator", consAddr,
			"infraction_height", infractionHeight,
			"max_age_num_blocks", eviAgeBlocks,
			"infraction_time", infractionTime,
			"max_age_duration", eviAgeDuration,
		)
		return nil
	}

	if ok := k.slashingKeeper.HasValidatorSigningInfo(ctx, consAddr); !ok {
		panic(fmt.Sprintf("expected signing info for validator %s but not found", consAddr))
	}

	// ignore if the validator is already tombstoned
	if k.slashingKeeper.IsTombstoned(ctx, consAddr) {
		k.Logger.Info(
			"ignored equivocation; validator already tombstoned",
			"validator", consAddr,
			"infraction_height", infractionHeight,
			"infraction_time", infractionTime,
		)
		return nil
	}

	k.Logger.Info(
		"confirmed equivocation",
		"validator", consAddr,
		"infraction_height", infractionHeight,
		"infraction_time", infractionTime,
	)

	// We need to retrieve the stake distribution which signed the block, so we
	// subtract ValidatorUpdateDelay from the evidence height.
	// Note, that this *can* result in a negative "distributionHeight", up to
	// -ValidatorUpdateDelay, i.e. at the end of the
	// pre-genesis block (none) = at the beginning of the genesis block.
	// That's fine since this is just used to filter unbonding delegations & redelegations.
	distributionHeight := infractionHeight - sdk.ValidatorUpdateDelay

	// Slash validator. The `power` is the int64 power of the validator as provided
	// to/by CometBFT. This value is validator.Tokens as sent to CometBFT via
	// ABCI, and now received as evidence. The fraction is passed in to separately
	// to slash unbonding and rebonding delegations.
	slashFractionDoubleSign, err := k.slashingKeeper.SlashFractionDoubleSign(ctx)
	if err != nil {
		return err
	}

	err = k.slashingKeeper.SlashWithInfractionReason(
		ctx,
		consAddr,
		slashFractionDoubleSign,
		evidence.GetValidatorPower(), distributionHeight,
		st.Infraction_INFRACTION_DOUBLE_SIGN,
	)
	if err != nil {
		return err
	}

	// Jail the validator if not already jailed. This will begin unbonding the
	// validator if not already unbonding (tombstoned).
	if !validator.IsJailed() {
		err = k.slashingKeeper.Jail(ctx, consAddr)
		if err != nil {
			return err
		}
	}

	err = k.slashingKeeper.JailUntil(ctx, consAddr, types.DoubleSignJailEndTime)
	if err != nil {
		return err
	}

	err = k.slashingKeeper.Tombstone(ctx, consAddr)
	if err != nil {
		return err
	}
	return k.Evidences.Set(ctx, evidence.Hash(), evidence)
}
