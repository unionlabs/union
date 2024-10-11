package testutil

import (
	"fmt"

	"cosmossdk.io/core/address"
	"cosmossdk.io/math"
	"cosmossdk.io/x/distribution/keeper"
	stakingtypes "cosmossdk.io/x/staking/types"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func CreateValidator(pk cryptotypes.PubKey, operator string, stake math.Int) (stakingtypes.Validator, error) {
	val, err := stakingtypes.NewValidator(operator, pk, stakingtypes.Description{Moniker: "TestValidator"})
	val.Tokens = stake
	val.DelegatorShares = math.LegacyNewDecFromInt(val.Tokens)
	return val, err
}

func CallCreateValidatorHooks(ctx sdk.Context, k keeper.Keeper, addr sdk.AccAddress, valAddr sdk.ValAddress) error {
	err := k.Hooks().AfterValidatorCreated(ctx, valAddr)
	if err != nil {
		return err
	}

	err = k.Hooks().BeforeDelegationCreated(ctx, addr, valAddr)
	if err != nil {
		return err
	}

	err = k.Hooks().AfterDelegationModified(ctx, addr, valAddr)
	if err != nil {
		return err
	}

	return nil
}

// SlashValidator copies what x/staking Slash does. It should be used for testing only.
// And it must be updated whenever the original function is updated.
// The passed validator will get its tokens updated.
func SlashValidator(
	ctx sdk.Context,
	consAddr sdk.ConsAddress,
	infractionHeight int64,
	power int64,
	slashFactor math.LegacyDec,
	validator *stakingtypes.Validator,
	distrKeeper *keeper.Keeper,
	sk *MockStakingKeeper,
) math.Int {
	if slashFactor.IsNegative() {
		panic(fmt.Errorf("attempted to slash with a negative slash factor: %v", slashFactor))
	}

	valBz, err := sk.ValidatorAddressCodec().StringToBytes(validator.GetOperator())
	if err != nil {
		panic(err)
	}

	// call the before-modification hook
	err = distrKeeper.Hooks().BeforeValidatorModified(ctx, valBz)
	if err != nil {
		panic(err)
	}

	// we simplify this part, as we won't be able to test redelegations or
	// unbonding delegations
	if infractionHeight != ctx.BlockHeight() {
		// if a new test lands here we might need to update this function to handle redelegations and unbonding
		// or just make it an integration test.
		panic("we can't test any other case here")
	}

	slashAmountDec := math.LegacyNewDecFromInt(validator.Tokens).Mul(math.LegacyNewDecWithPrec(5, 1))
	slashAmount := slashAmountDec.TruncateInt()

	// cannot decrease balance below zero
	tokensToBurn := math.MinInt(slashAmount, validator.Tokens)
	tokensToBurn = math.MaxInt(tokensToBurn, math.ZeroInt()) // defensive.

	// we need to calculate the *effective* slash fraction for distribution
	if validator.Tokens.IsPositive() {
		effectiveFraction := math.LegacyNewDecFromInt(tokensToBurn).QuoRoundUp(math.LegacyNewDecFromInt(validator.Tokens))
		// possible if power has changed
		if effectiveFraction.GT(math.LegacyOneDec()) {
			effectiveFraction = math.LegacyOneDec()
		}
		// call the before-slashed hook
		err := distrKeeper.Hooks().BeforeValidatorSlashed(ctx, valBz, effectiveFraction)
		if err != nil {
			panic(err)
		}
	}
	// Deduct from validator's bonded tokens and update the validator.
	// Burn the slashed tokens from the pool account and decrease the total supply.
	validator.Tokens = validator.Tokens.Sub(tokensToBurn)

	return tokensToBurn
}

// Delegate imitate what x/staking Delegate does. It should be used for testing only.
// If a delegation is passed we are simulating an update to a previous delegation,
// if it's nil then we simulate a new delegation.
func Delegate(
	ctx sdk.Context,
	distrKeeper keeper.Keeper,
	delegator sdk.AccAddress,
	validator *stakingtypes.Validator,
	amount math.Int,
	delegation *stakingtypes.Delegation,
	sk *MockStakingKeeper,
	addressCodec address.Codec,
) (
	newShares math.LegacyDec,
	updatedDel stakingtypes.Delegation,
	err error,
) {
	valBz, err := sk.ValidatorAddressCodec().StringToBytes(validator.GetOperator())
	if err != nil {
		return math.LegacyZeroDec(), stakingtypes.Delegation{}, err
	}
	if delegation != nil {
		err = distrKeeper.Hooks().BeforeDelegationSharesModified(ctx, delegator, valBz)
	} else {
		err = distrKeeper.Hooks().BeforeDelegationCreated(ctx, delegator, valBz)
		if err != nil {
			return math.LegacyZeroDec(), stakingtypes.Delegation{}, err
		}
		delAddr, err := addressCodec.BytesToString(delegator)
		if err != nil {
			return math.LegacyZeroDec(), stakingtypes.Delegation{}, err
		}
		del := stakingtypes.NewDelegation(delAddr, validator.GetOperator(), math.LegacyZeroDec())
		delegation = &del
	}

	if err != nil {
		return math.LegacyZeroDec(), stakingtypes.Delegation{}, err
	}

	// Add tokens from delegation to validator
	updateVal, newShares := validator.AddTokensFromDel(amount)
	*validator = updateVal

	delegation.Shares = delegation.Shares.Add(newShares)

	return newShares, *delegation, nil
}
