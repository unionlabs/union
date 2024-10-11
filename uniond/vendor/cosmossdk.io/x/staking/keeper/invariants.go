package keeper

import (
	"bytes"
	"fmt"

	"cosmossdk.io/collections"
	"cosmossdk.io/math"
	"cosmossdk.io/x/staking/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// RegisterInvariants registers all staking invariants
func RegisterInvariants(ir sdk.InvariantRegistry, k *Keeper) {
	ir.RegisterRoute(types.ModuleName, "module-accounts",
		ModuleAccountInvariants(k))
	ir.RegisterRoute(types.ModuleName, "nonnegative-power",
		NonNegativePowerInvariant(k))
	ir.RegisterRoute(types.ModuleName, "positive-delegation",
		PositiveDelegationInvariant(k))
	ir.RegisterRoute(types.ModuleName, "delegator-shares",
		DelegatorSharesInvariant(k))
}

// AllInvariants runs all invariants of the staking module.
func AllInvariants(k *Keeper) sdk.Invariant {
	return func(ctx sdk.Context) (string, bool) {
		res, stop := ModuleAccountInvariants(k)(ctx)
		if stop {
			return res, stop
		}

		res, stop = NonNegativePowerInvariant(k)(ctx)
		if stop {
			return res, stop
		}

		res, stop = PositiveDelegationInvariant(k)(ctx)
		if stop {
			return res, stop
		}

		return DelegatorSharesInvariant(k)(ctx)
	}
}

// ModuleAccountInvariants checks that the bonded and notBonded ModuleAccounts pools
// reflects the tokens actively bonded and not bonded
func ModuleAccountInvariants(k *Keeper) sdk.Invariant {
	return func(ctx sdk.Context) (string, bool) {
		bonded := math.ZeroInt()
		notBonded := math.ZeroInt()
		bondedPool := k.GetBondedPool(ctx)
		notBondedPool := k.GetNotBondedPool(ctx)
		bondDenom, err := k.BondDenom(ctx)
		if err != nil {
			panic(err)
		}

		err = k.IterateValidators(ctx, func(_ int64, validator sdk.ValidatorI) bool {
			switch validator.GetStatus() {
			case sdk.Bonded:
				bonded = bonded.Add(validator.GetTokens())
			case sdk.Unbonding, sdk.Unbonded:
				notBonded = notBonded.Add(validator.GetTokens())
			default:
				panic("invalid validator status")
			}
			return false
		})
		if err != nil {
			panic(err)
		}

		err = k.UnbondingDelegations.Walk(
			ctx,
			nil,
			func(key collections.Pair[[]byte, []byte], ubd types.UnbondingDelegation) (stop bool, err error) {
				for _, entry := range ubd.Entries {
					notBonded = notBonded.Add(entry.Balance)
				}
				return false, nil
			},
		)
		if err != nil {
			panic(err)
		}

		poolBonded := k.bankKeeper.GetBalance(ctx, bondedPool.GetAddress(), bondDenom)
		poolNotBonded := k.bankKeeper.GetBalance(ctx, notBondedPool.GetAddress(), bondDenom)
		broken := !poolBonded.Amount.Equal(bonded) || !poolNotBonded.Amount.Equal(notBonded)

		// Bonded tokens should equal sum of tokens with bonded validators
		// Not-bonded tokens should equal unbonding delegations	plus tokens on unbonded validators
		return sdk.FormatInvariant(types.ModuleName, "bonded and not bonded module account coins", fmt.Sprintf(
			"\tPool's bonded tokens: %v\n"+
				"\tsum of bonded tokens: %v\n"+
				"not bonded token invariance:\n"+
				"\tPool's not bonded tokens: %v\n"+
				"\tsum of not bonded tokens: %v\n"+
				"module accounts total (bonded + not bonded):\n"+
				"\tModule Accounts' tokens: %v\n"+
				"\tsum tokens:              %v\n",
			poolBonded, bonded, poolNotBonded, notBonded, poolBonded.Add(poolNotBonded), bonded.Add(notBonded))), broken
	}
}

// NonNegativePowerInvariant checks that all stored validators have >= 0 power.
func NonNegativePowerInvariant(k *Keeper) sdk.Invariant {
	return func(ctx sdk.Context) (string, bool) {
		var (
			msg    string
			broken bool
		)

		iterator, err := k.ValidatorsPowerStoreIterator(ctx)
		if err != nil {
			panic(err)
		}
		for ; iterator.Valid(); iterator.Next() {
			validator, err := k.GetValidator(ctx, iterator.Value())
			if err != nil {
				panic(fmt.Sprintf("validator record not found for address: %X\n", iterator.Value()))
			}

			powerKey := types.GetValidatorsByPowerIndexKey(validator, k.PowerReduction(ctx), k.ValidatorAddressCodec())

			if !bytes.Equal(iterator.Key(), powerKey) {
				broken = true
				msg += fmt.Sprintf("power store invariance:\n\tvalidator.Power: %v"+
					"\n\tkey should be: %v\n\tkey in store: %v\n",
					validator.GetConsensusPower(k.PowerReduction(ctx)), powerKey, iterator.Key())
			}

			if validator.Tokens.IsNegative() {
				broken = true
				msg += fmt.Sprintf("\tnegative tokens for validator: %v\n", validator)
			}
		}
		iterator.Close()

		return sdk.FormatInvariant(types.ModuleName, "nonnegative power", fmt.Sprintf("found invalid validator powers\n%s", msg)), broken
	}
}

// PositiveDelegationInvariant checks that all stored delegations have > 0 shares.
func PositiveDelegationInvariant(k *Keeper) sdk.Invariant {
	return func(ctx sdk.Context) (string, bool) {
		var (
			msg   string
			count int
		)

		delegations, err := k.GetAllDelegations(ctx)
		if err != nil {
			panic(err)
		}
		for _, delegation := range delegations {
			if delegation.Shares.IsNegative() {
				count++
				msg += fmt.Sprintf("\tdelegation with negative shares: %+v\n", delegation)
			}

			if delegation.Shares.IsZero() {
				count++
				msg += fmt.Sprintf("\tdelegation with zero shares: %+v\n", delegation)
			}
		}

		broken := count != 0

		return sdk.FormatInvariant(types.ModuleName, "positive delegations", fmt.Sprintf(
			"%d invalid delegations found\n%s", count, msg)), broken
	}
}

// DelegatorSharesInvariant checks whether all the delegator shares which persist
// in the delegator object add up to the correct total delegator shares
// amount stored in each validator.
func DelegatorSharesInvariant(k *Keeper) sdk.Invariant {
	return func(ctx sdk.Context) (string, bool) {
		var (
			msg    string
			broken bool
		)

		validators, err := k.GetAllValidators(ctx)
		if err != nil {
			panic(err)
		}

		validatorsDelegationShares := map[string]math.LegacyDec{}

		// initialize a map: validator -> its delegation shares
		for _, validator := range validators {
			validatorsDelegationShares[validator.GetOperator()] = math.LegacyZeroDec()
		}

		// iterate through all the delegations to calculate the total delegation shares for each validator
		delegations, err := k.GetAllDelegations(ctx)
		if err != nil {
			panic(err)
		}

		for _, delegation := range delegations {
			delegationValidatorAddr := delegation.GetValidatorAddr()
			validatorDelegationShares := validatorsDelegationShares[delegationValidatorAddr]
			validatorsDelegationShares[delegationValidatorAddr] = validatorDelegationShares.Add(delegation.Shares)
		}

		// for each validator, check if its total delegation shares calculated from the step above equals to its expected delegation shares
		for _, validator := range validators {
			expValTotalDelShares := validator.GetDelegatorShares()
			calculatedValTotalDelShares := validatorsDelegationShares[validator.GetOperator()]
			if !calculatedValTotalDelShares.Equal(expValTotalDelShares) {
				broken = true
				msg += fmt.Sprintf("broken delegator shares invariance:\n"+
					"\tvalidator.DelegatorShares: %v\n"+
					"\tsum of Delegator.Shares: %v\n", expValTotalDelShares, calculatedValTotalDelShares)
			}
		}

		return sdk.FormatInvariant(types.ModuleName, "delegator shares", msg), broken
	}
}
