package simulation

import (
	"context"
	"slices"
	"time"

	"cosmossdk.io/math"
	"cosmossdk.io/x/staking/keeper"
	"cosmossdk.io/x/staking/types"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/simsx"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func MsgCreateValidatorFactory(k *keeper.Keeper) simsx.SimMsgFactoryFn[*types.MsgCreateValidator] {
	return func(ctx context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *types.MsgCreateValidator) {
		r := testData.Rand()
		withoutValidators := simsx.SimAccountFilterFn(func(a simsx.SimAccount) bool {
			_, err := k.GetValidator(ctx, sdk.ValAddress(a.Address))
			return err != nil
		})
		withoutConsAddrUsed := simsx.SimAccountFilterFn(func(a simsx.SimAccount) bool {
			consPubKey := sdk.GetConsAddress(a.ConsKey.PubKey())
			_, err := k.GetValidatorByConsAddr(ctx, consPubKey)
			return err != nil
		})
		bondDenom := must(k.BondDenom(ctx))
		valOper := testData.AnyAccount(reporter, withoutValidators, withoutConsAddrUsed, simsx.WithDenomBalance(bondDenom))
		if reporter.IsSkipped() {
			return nil, nil
		}

		newPubKey := valOper.ConsKey.PubKey()
		assertKeyUnused(ctx, reporter, k, newPubKey)
		if reporter.IsSkipped() {
			return nil, nil
		}

		selfDelegation := valOper.LiquidBalance().RandSubsetCoin(reporter, bondDenom)
		description := types.NewDescription(
			r.StringN(10),
			r.StringN(10),
			r.StringN(10),
			r.StringN(10),
			r.StringN(10),
		)

		maxCommission := math.LegacyNewDecWithPrec(int64(r.IntInRange(0, 100)), 2)
		commission := types.NewCommissionRates(
			r.DecN(maxCommission),
			maxCommission,
			r.DecN(maxCommission),
		)

		addr := must(k.ValidatorAddressCodec().BytesToString(valOper.Address))
		msg, err := types.NewMsgCreateValidator(addr, newPubKey, selfDelegation, description, commission, math.OneInt())
		if err != nil {
			reporter.Skip(err.Error())
			return nil, nil
		}

		return []simsx.SimAccount{valOper}, msg
	}
}

func MsgDelegateFactory(k *keeper.Keeper) simsx.SimMsgFactoryFn[*types.MsgDelegate] {
	return func(ctx context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *types.MsgDelegate) {
		r := testData.Rand()
		bondDenom := must(k.BondDenom(ctx))
		val := randomValidator(ctx, reporter, k, r)
		if reporter.IsSkipped() {
			return nil, nil
		}

		if val.InvalidExRate() {
			reporter.Skip("validator's invalid exchange rate")
			return nil, nil
		}
		sender := testData.AnyAccount(reporter)
		delegation := sender.LiquidBalance().RandSubsetCoin(reporter, bondDenom)
		return []simsx.SimAccount{sender}, types.NewMsgDelegate(sender.AddressBech32, val.GetOperator(), delegation)
	}
}

func MsgUndelegateFactory(k *keeper.Keeper) simsx.SimMsgFactoryFn[*types.MsgUndelegate] {
	return func(ctx context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *types.MsgUndelegate) {
		r := testData.Rand()
		bondDenom := must(k.BondDenom(ctx))
		val := randomValidator(ctx, reporter, k, r)
		if reporter.IsSkipped() {
			return nil, nil
		}

		// select delegator and amount for undelegate
		valAddr := must(k.ValidatorAddressCodec().StringToBytes(val.GetOperator()))
		delegations := must(k.GetValidatorDelegations(ctx, valAddr))
		if delegations == nil {
			reporter.Skip("no delegation entries")
			return nil, nil
		}
		// get random delegator from validator
		delegation := delegations[r.Intn(len(delegations))]
		delAddr := delegation.GetDelegatorAddr()
		delegator := testData.GetAccount(reporter, delAddr)

		if hasMaxUD := must(k.HasMaxUnbondingDelegationEntries(ctx, delegator.Address, valAddr)); hasMaxUD {
			reporter.Skipf("max unbodings")
			return nil, nil
		}

		totalBond := val.TokensFromShares(delegation.GetShares()).TruncateInt()
		if !totalBond.IsPositive() {
			reporter.Skip("total bond is negative")
			return nil, nil
		}

		unbondAmt := must(r.PositiveSDKIntn(totalBond))
		msg := types.NewMsgUndelegate(delAddr, val.GetOperator(), sdk.NewCoin(bondDenom, unbondAmt))
		return []simsx.SimAccount{delegator}, msg
	}
}

func MsgEditValidatorFactory(k *keeper.Keeper) simsx.SimMsgFactoryFn[*types.MsgEditValidator] {
	return func(ctx context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *types.MsgEditValidator) {
		r := testData.Rand()
		val := randomValidator(ctx, reporter, k, r)
		if reporter.IsSkipped() {
			return nil, nil
		}

		newCommissionRate := r.DecN(val.Commission.MaxRate)
		if err := val.Commission.ValidateNewRate(newCommissionRate, simsx.BlockTime(ctx)); err != nil {
			// skip as the commission is invalid
			reporter.Skip("invalid commission rate")
			return nil, nil
		}
		valOpAddrBz := must(k.ValidatorAddressCodec().StringToBytes(val.GetOperator()))
		valOper := testData.GetAccountbyAccAddr(reporter, valOpAddrBz)
		d := types.NewDescription(r.StringN(10), r.StringN(10), r.StringN(10), r.StringN(10), r.StringN(10))

		msg := types.NewMsgEditValidator(val.GetOperator(), d, &newCommissionRate, nil)
		return []simsx.SimAccount{valOper}, msg
	}
}

func MsgBeginRedelegateFactory(k *keeper.Keeper) simsx.SimMsgFactoryFn[*types.MsgBeginRedelegate] {
	return func(ctx context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *types.MsgBeginRedelegate) {
		bondDenom := must(k.BondDenom(ctx))
		if !testData.IsSendEnabledDenom(bondDenom) {
			reporter.Skip("bond denom send not enabled")
			return nil, nil
		}

		r := testData.Rand()
		// select random validator as src
		vals := must(k.GetAllValidators(ctx))
		if len(vals) < 2 {
			reporter.Skip("insufficient number of validators")
			return nil, nil
		}
		srcVal := simsx.OneOf(r, vals)
		srcValOpAddrBz := must(k.ValidatorAddressCodec().StringToBytes(srcVal.GetOperator()))
		delegations := must(k.GetValidatorDelegations(ctx, srcValOpAddrBz))
		if delegations == nil {
			reporter.Skip("no delegations")
			return nil, nil
		}
		// get random delegator from src validator
		delegation := simsx.OneOf(r, delegations)
		totalBond := srcVal.TokensFromShares(delegation.GetShares()).TruncateInt()
		if !totalBond.IsPositive() {
			reporter.Skip("total bond is negative")
			return nil, nil
		}
		redAmount, err := r.PositiveSDKIntn(totalBond)
		if err != nil || redAmount.IsZero() {
			reporter.Skip("unable to generate positive amount")
			return nil, nil
		}

		// check if the shares truncate to zero
		shares := must(srcVal.SharesFromTokens(redAmount))
		if srcVal.TokensFromShares(shares).TruncateInt().IsZero() {
			reporter.Skip("shares truncate to zero")
			return nil, nil
		}

		// pick a random delegator
		delAddr := delegation.GetDelegatorAddr()
		delAddrBz := must(testData.AddressCodec().StringToBytes(delAddr))
		if hasRecRedel := must(k.HasReceivingRedelegation(ctx, delAddrBz, srcValOpAddrBz)); hasRecRedel {
			reporter.Skip("receiving redelegation is not allowed")
			return nil, nil
		}
		delegator := testData.GetAccountbyAccAddr(reporter, delAddrBz)
		if reporter.IsSkipped() {
			return nil, nil
		}

		// get random destination validator
		destVal := simsx.OneOf(r, vals)
		if srcVal.Equal(&destVal) {
			destVal = simsx.OneOf(r, slices.DeleteFunc(vals, func(v types.Validator) bool { return srcVal.Equal(&v) }))
		}
		if destVal.InvalidExRate() {
			reporter.Skip("invalid delegation rate")
			return nil, nil
		}

		destAddrBz := must(k.ValidatorAddressCodec().StringToBytes(destVal.GetOperator()))
		if hasMaxRedel := must(k.HasMaxRedelegationEntries(ctx, delAddrBz, srcValOpAddrBz, destAddrBz)); hasMaxRedel {
			reporter.Skip("maximum redelegation entries reached")
			return nil, nil
		}

		msg := types.NewMsgBeginRedelegate(
			delAddr, srcVal.GetOperator(), destVal.GetOperator(),
			sdk.NewCoin(bondDenom, redAmount),
		)
		return []simsx.SimAccount{delegator}, msg
	}
}

func MsgCancelUnbondingDelegationFactory(k *keeper.Keeper) simsx.SimMsgFactoryFn[*types.MsgCancelUnbondingDelegation] {
	return func(ctx context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *types.MsgCancelUnbondingDelegation) {
		r := testData.Rand()
		val := randomValidator(ctx, reporter, k, r)
		if reporter.IsSkipped() {
			return nil, nil
		}
		if val.IsJailed() || val.InvalidExRate() {
			reporter.Skip("validator is jailed")
			return nil, nil
		}
		valOpAddrBz := must(k.ValidatorAddressCodec().StringToBytes(val.GetOperator()))
		valOper := testData.GetAccountbyAccAddr(reporter, valOpAddrBz)
		unbondingDelegation, err := k.GetUnbondingDelegation(ctx, valOper.Address, valOpAddrBz)
		if err != nil {
			reporter.Skip("no unbonding delegation")
			return nil, nil
		}

		// This is a temporary fix to make staking simulation pass. We should fetch
		// the first unbondingDelegationEntry that matches the creationHeight, because
		// currently the staking msgServer chooses the first unbondingDelegationEntry
		// with the matching creationHeight.
		//
		// ref: https://github.com/cosmos/cosmos-sdk/issues/12932
		creationHeight := unbondingDelegation.Entries[r.Intn(len(unbondingDelegation.Entries))].CreationHeight

		var unbondingDelegationEntry types.UnbondingDelegationEntry
		for _, entry := range unbondingDelegation.Entries {
			if entry.CreationHeight == creationHeight {
				unbondingDelegationEntry = entry
				break
			}
		}
		if unbondingDelegationEntry.CompletionTime.Before(simsx.BlockTime(ctx)) {
			reporter.Skip("unbonding delegation is already processed")
			return nil, nil
		}

		if !unbondingDelegationEntry.Balance.IsPositive() {
			reporter.Skip("delegator receiving balance is negative")
			return nil, nil
		}
		cancelBondAmt := r.Amount(unbondingDelegationEntry.Balance)
		if cancelBondAmt.IsZero() {
			reporter.Skip("cancelBondAmt amount is zero")
			return nil, nil
		}

		msg := types.NewMsgCancelUnbondingDelegation(
			valOper.AddressBech32,
			val.GetOperator(),
			unbondingDelegationEntry.CreationHeight,
			sdk.NewCoin(must(k.BondDenom(ctx)), cancelBondAmt),
		)

		return []simsx.SimAccount{valOper}, msg
	}
}

func MsgRotateConsPubKeyFactory(k *keeper.Keeper) simsx.SimMsgFactoryFn[*types.MsgRotateConsPubKey] {
	return func(ctx context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *types.MsgRotateConsPubKey) {
		r := testData.Rand()
		val := randomValidator(ctx, reporter, k, r)
		if reporter.IsSkipped() {
			return nil, nil
		}
		if val.Status != types.Bonded || val.ConsensusPower(sdk.DefaultPowerReduction) == 0 {
			reporter.Skip("validator not bonded.")
			return nil, nil
		}
		valOpAddrBz := must(k.ValidatorAddressCodec().StringToBytes(val.GetOperator()))
		valOper := testData.GetAccountbyAccAddr(reporter, valOpAddrBz)
		otherAccount := testData.AnyAccount(reporter, simsx.ExcludeAddresses(valOper.AddressBech32))

		consAddress := must(k.ConsensusAddressCodec().BytesToString(must(val.GetConsAddr())))
		accAddress := must(k.ConsensusAddressCodec().BytesToString(otherAccount.ConsKey.PubKey().Address()))
		if consAddress == accAddress {
			reporter.Skip("new pubkey and current pubkey should be different")
			return nil, nil
		}
		if !valOper.LiquidBalance().BlockAmount(must(k.Params.Get(ctx)).KeyRotationFee) {
			reporter.Skip("not enough balance to pay for key rotation fee")
			return nil, nil
		}
		if err := k.ExceedsMaxRotations(ctx, valOpAddrBz); err != nil {
			reporter.Skip("rotations limit reached within unbonding period")
			return nil, nil
		}
		// check whether the new cons key associated with another validator
		newConsAddr := sdk.ConsAddress(otherAccount.ConsKey.PubKey().Address())

		if _, err := k.GetValidatorByConsAddr(ctx, newConsAddr); err == nil {
			reporter.Skip("cons key already used")
			return nil, nil
		}
		msg := must(types.NewMsgRotateConsPubKey(val.GetOperator(), otherAccount.ConsKey.PubKey()))

		// check if there's another key rotation for this same key in the same block
		for _, r := range must(k.GetBlockConsPubKeyRotationHistory(ctx)) {
			if r.NewConsPubkey.Compare(msg.NewPubkey) == 0 {
				reporter.Skip("cons key already used in this block")
				return nil, nil
			}
		}
		return []simsx.SimAccount{valOper}, msg
	}
}

// MsgUpdateParamsFactory creates a gov proposal for param updates
func MsgUpdateParamsFactory() simsx.SimMsgFactoryFn[*types.MsgUpdateParams] {
	return func(_ context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *types.MsgUpdateParams) {
		r := testData.Rand()
		params := types.DefaultParams()
		// do not modify denom or staking will break
		params.HistoricalEntries = r.Uint32InRange(0, 1000)
		params.MaxEntries = r.Uint32InRange(1, 1000)
		params.MaxValidators = r.Uint32InRange(1, 1000)
		params.UnbondingTime = time.Duration(r.Timestamp().UnixNano())
		// modifying commission rate can cause issues for proposals within the same block
		// params.MinCommissionRate = r.DecN(sdkmath.LegacyNewDec(1))

		return nil, &types.MsgUpdateParams{
			Authority: testData.ModuleAccountAddress(reporter, "gov"),
			Params:    params,
		}
	}
}

func randomValidator(ctx context.Context, reporter simsx.SimulationReporter, k *keeper.Keeper, r *simsx.XRand) types.Validator {
	vals, err := k.GetAllValidators(ctx)
	if err != nil || len(vals) == 0 {
		reporter.Skipf("unable to get validators or empty list: %s", err)
		return types.Validator{}
	}
	return simsx.OneOf(r, vals)
}

// skips execution if there's another key rotation for the same key in the same block
func assertKeyUnused(ctx context.Context, reporter simsx.SimulationReporter, k *keeper.Keeper, newPubKey cryptotypes.PubKey) {
	allRotations, err := k.GetBlockConsPubKeyRotationHistory(ctx)
	if err != nil {
		reporter.Skipf("cannot get block cons key rotation history: %s", err.Error())
		return
	}
	for _, r := range allRotations {
		if r.NewConsPubkey.Compare(newPubKey) != 0 {
			reporter.Skip("cons key already used in this block")
			return
		}
	}
}

func must[T any](r T, err error) T {
	if err != nil {
		panic(err)
	}
	return r
}
