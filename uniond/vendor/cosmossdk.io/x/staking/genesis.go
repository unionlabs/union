package staking

import (
	"context"
	"fmt"

	gogotypes "github.com/cosmos/gogoproto/types"

	"cosmossdk.io/x/staking/keeper"
	"cosmossdk.io/x/staking/types"

	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// WriteValidators returns a slice of bonded genesis validators.
func WriteValidators(ctx context.Context, keeper *keeper.Keeper) (vals []sdk.GenesisValidator, returnErr error) {
	err := keeper.LastValidatorPower.Walk(ctx, nil, func(key []byte, _ gogotypes.Int64Value) (bool, error) {
		validator, err := keeper.GetValidator(ctx, key)
		if err != nil {
			return true, err
		}

		pk, err := validator.ConsPubKey()
		if err != nil {
			returnErr = err
			return true, err
		}
		jsonPk, err := cryptocodec.PubKeyFromProto(pk)
		if err != nil {
			return true, err
		}

		vals = append(vals, sdk.GenesisValidator{
			Address: pk.Address().Bytes(),
			PubKey:  jsonPk,
			Power:   validator.GetConsensusPower(keeper.PowerReduction(ctx)),
			Name:    validator.GetMoniker(),
		})

		return false, nil
	})
	if err != nil {
		return nil, err
	}

	return vals, returnErr
}

// ValidateGenesis validates the provided staking genesis state to ensure the
// expected invariants holds. (i.e. params in correct bounds, no duplicate validators)
func ValidateGenesis(data *types.GenesisState) error {
	if err := validateGenesisStateValidators(data.Validators); err != nil {
		return err
	}

	return data.Params.Validate()
}

func validateGenesisStateValidators(validators []types.Validator) error {
	addrMap := make(map[string]bool, len(validators))

	for i := 0; i < len(validators); i++ {
		val := validators[i]
		consPk, err := val.ConsPubKey()
		if err != nil {
			return err
		}

		strKey := string(consPk.Bytes())

		if _, ok := addrMap[strKey]; ok {
			consAddr, err := val.GetConsAddr()
			if err != nil {
				return err
			}
			return fmt.Errorf("duplicate validator in genesis state: moniker %v, address %v", val.Description.Moniker, consAddr)
		}

		if val.Jailed && val.IsBonded() {
			consAddr, err := val.GetConsAddr()
			if err != nil {
				return err
			}
			return fmt.Errorf("validator is bonded and jailed in genesis state: moniker %v, address %v", val.Description.Moniker, consAddr)
		}

		if val.DelegatorShares.IsZero() && !val.IsUnbonding() {
			return fmt.Errorf("bonded/unbonded genesis validator cannot have zero delegator shares, validator: %v", val)
		}

		addrMap[strKey] = true
	}

	return nil
}
