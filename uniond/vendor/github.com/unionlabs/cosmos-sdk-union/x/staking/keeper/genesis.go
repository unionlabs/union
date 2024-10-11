package keeper

import (
	"context"
	"fmt"
	"time"

	"cosmossdk.io/collections"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/math"
	"cosmossdk.io/x/staking/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// InitGenesis sets the pool and parameters for the provided keeper.  For each
// validator in data, it sets that validator in the keeper along with manually
// setting the indexes. In addition, it also sets any delegations found in
// data. Finally, it updates the bonded validators.
// Returns final validator set after applying all declaration and delegations
func (k Keeper) InitGenesis(ctx context.Context, data *types.GenesisState) ([]appmodule.ValidatorUpdate, error) {
	bondedTokens := math.ZeroInt()
	notBondedTokens := math.ZeroInt()

	// We need to pretend to be "n blocks before genesis", where "n" is the
	// validator update delay, so that e.g. slashing periods are correctly
	// initialized for the validator set e.g. with a one-block offset - the
	// first TM block is at height 1, so state updates applied from
	// genesis.json are in block 0.
	if sdkCtx, ok := sdk.TryUnwrapSDKContext(ctx); ok {
		// this munging of the context is not necessary for server/v2 code paths, `ok` will be false
		sdkCtx = sdkCtx.WithBlockHeight(1 - sdk.ValidatorUpdateDelay) // TODO: remove this need for WithBlockHeight
		ctx = sdkCtx
	}

	if err := k.Params.Set(ctx, data.Params); err != nil {
		return nil, err
	}

	if err := k.LastTotalPower.Set(ctx, data.LastTotalPower); err != nil {
		return nil, err
	}

	for _, validator := range data.Validators {
		if err := k.SetValidator(ctx, validator); err != nil {
			return nil, err
		}

		// Manually set indices for the first time
		if err := k.SetValidatorByConsAddr(ctx, validator); err != nil {
			return nil, err
		}

		if err := k.SetValidatorByPowerIndex(ctx, validator); err != nil {
			return nil, err
		}

		// Call the creation hook if not exported
		if !data.Exported {
			valbz, err := k.ValidatorAddressCodec().StringToBytes(validator.GetOperator())
			if err != nil {
				return nil, err
			}
			if err := k.Hooks().AfterValidatorCreated(ctx, valbz); err != nil {
				return nil, err
			}
		}

		// update timeslice if necessary
		if validator.IsUnbonding() {
			if err := k.InsertUnbondingValidatorQueue(ctx, validator); err != nil {
				return nil, err
			}
		}

		switch validator.GetStatus() {
		case sdk.Bonded:
			bondedTokens = bondedTokens.Add(validator.GetTokens())

		case sdk.Unbonding, sdk.Unbonded:
			notBondedTokens = notBondedTokens.Add(validator.GetTokens())

		default:
			return nil, fmt.Errorf("invalid validator status: %v", validator.GetStatus())
		}
	}

	for _, delegation := range data.Delegations {
		delegatorAddress, err := k.authKeeper.AddressCodec().StringToBytes(delegation.DelegatorAddress)
		if err != nil {
			return nil, fmt.Errorf("invalid delegator address: %w", err)
		}

		valAddr, err := k.validatorAddressCodec.StringToBytes(delegation.GetValidatorAddr())
		if err != nil {
			return nil, err
		}

		// Call the before-creation hook if not exported
		if !data.Exported {
			if err := k.Hooks().BeforeDelegationCreated(ctx, delegatorAddress, valAddr); err != nil {
				return nil, err
			}
		}

		if err := k.SetDelegation(ctx, delegation); err != nil {
			return nil, err
		}

		// Call the after-modification hook if not exported
		if !data.Exported {
			if err := k.Hooks().AfterDelegationModified(ctx, delegatorAddress, valAddr); err != nil {
				return nil, err
			}
		}
	}

	for _, ubd := range data.UnbondingDelegations {
		if err := k.SetUnbondingDelegation(ctx, ubd); err != nil {
			return nil, err
		}

		for _, entry := range ubd.Entries {
			if err := k.InsertUBDQueue(ctx, ubd, entry.CompletionTime); err != nil {
				return nil, err
			}
			notBondedTokens = notBondedTokens.Add(entry.Balance)
		}
	}

	for _, red := range data.Redelegations {
		if err := k.SetRedelegation(ctx, red); err != nil {
			return nil, err
		}

		for _, entry := range red.Entries {
			if err := k.InsertRedelegationQueue(ctx, red, entry.CompletionTime); err != nil {
				return nil, err
			}
		}
	}

	bondedCoins := sdk.NewCoins(sdk.NewCoin(data.Params.BondDenom, bondedTokens))
	notBondedCoins := sdk.NewCoins(sdk.NewCoin(data.Params.BondDenom, notBondedTokens))

	// check if the unbonded and bonded pools accounts exists
	bondedPool := k.GetBondedPool(ctx)
	if bondedPool == nil {
		return nil, fmt.Errorf("%s module account has not been set", types.BondedPoolName)
	}

	// TODO: remove with genesis 2-phases refactor https://github.com/cosmos/cosmos-sdk/issues/2862

	bondedBalance := k.bankKeeper.GetAllBalances(ctx, bondedPool.GetAddress())
	if bondedBalance.IsZero() {
		k.authKeeper.SetModuleAccount(ctx, bondedPool)
	}

	// if balance is different from bonded coins error because genesis is most likely malformed
	if !bondedBalance.Equal(bondedCoins) {
		return nil, fmt.Errorf("bonded pool balance is different from bonded coins: %s <-> %s", bondedBalance, bondedCoins)
	}

	notBondedPool := k.GetNotBondedPool(ctx)
	if notBondedPool == nil {
		return nil, fmt.Errorf("%s module account has not been set", types.NotBondedPoolName)
	}

	notBondedBalance := k.bankKeeper.GetAllBalances(ctx, notBondedPool.GetAddress())
	if notBondedBalance.IsZero() {
		k.authKeeper.SetModuleAccount(ctx, notBondedPool)
	}

	// If balance is different from non bonded coins error because genesis is most
	// likely malformed.
	if !notBondedBalance.Equal(notBondedCoins) {
		return nil, fmt.Errorf("not bonded pool balance is different from not bonded coins: %s <-> %s", notBondedBalance, notBondedCoins)
	}

	for _, record := range data.RotationIndexRecords {
		if err := k.ValidatorConsensusKeyRotationRecordIndexKey.Set(ctx, collections.Join(record.Address, *record.Time)); err != nil {
			return nil, err
		}
	}

	for _, history := range data.RotationHistory {
		if err := k.RotationHistory.Set(ctx, collections.Join(history.OperatorAddress, history.Height), history); err != nil {
			return nil, err
		}
	}

	for _, record := range data.RotationQueue {
		if err := k.ValidatorConsensusKeyRotationRecordQueue.Set(ctx, *record.Time, *record.ValAddrs); err != nil {
			return nil, err
		}
	}

	// don't need to run CometBFT updates if we exported
	var moduleValidatorUpdates []appmodule.ValidatorUpdate
	if data.Exported {
		for _, lv := range data.LastValidatorPowers {
			valAddr, err := k.validatorAddressCodec.StringToBytes(lv.Address)
			if err != nil {
				return nil, err
			}

			err = k.SetLastValidatorPower(ctx, valAddr, lv.Power)
			if err != nil {
				return nil, err
			}

			validator, err := k.GetValidator(ctx, valAddr)
			if err != nil {
				return nil, fmt.Errorf("validator %s not found", lv.Address)
			}

			update := validator.ModuleValidatorUpdate(k.PowerReduction(ctx))
			update.Power = lv.Power // keep the next-val-set offset, use the last power for the first block
			moduleValidatorUpdates = append(moduleValidatorUpdates, update)
		}
	} else {
		var err error

		moduleValidatorUpdates, err = k.ApplyAndReturnValidatorSetUpdates(ctx)
		if err != nil {
			return nil, err
		}
	}

	return moduleValidatorUpdates, nil
}

// ExportGenesis returns a GenesisState for a given context and keeper. The
// GenesisState will contain the pool, params, validators, and bonds found in
// the keeper.
func (k Keeper) ExportGenesis(ctx context.Context) (*types.GenesisState, error) {
	var unbondingDelegations []types.UnbondingDelegation
	var fnErr error
	err := k.UnbondingDelegations.Walk(
		ctx,
		nil,
		func(key collections.Pair[[]byte, []byte], value types.UnbondingDelegation) (stop bool, err error) {
			unbondingDelegations = append(unbondingDelegations, value)
			return false, nil
		},
	)
	if err != nil {
		return nil, err
	}

	var redelegations []types.Redelegation

	err = k.IterateRedelegations(ctx, func(_ int64, red types.Redelegation) (stop bool) {
		redelegations = append(redelegations, red)
		return false
	})
	if err != nil {
		return nil, err
	}

	var lastValidatorPowers []types.LastValidatorPower

	err = k.IterateLastValidatorPowers(ctx, func(addr sdk.ValAddress, power int64) (stop bool) {
		addrStr, err := k.validatorAddressCodec.BytesToString(addr)
		if err != nil {
			fnErr = err
			return true
		}
		lastValidatorPowers = append(lastValidatorPowers, types.LastValidatorPower{Address: addrStr, Power: power})
		return false
	})
	if err != nil {
		return nil, err
	}
	if fnErr != nil {
		return nil, fnErr
	}

	params, err := k.Params.Get(ctx)
	if err != nil {
		return nil, err
	}

	totalPower, err := k.LastTotalPower.Get(ctx)
	if err != nil {
		return nil, err
	}

	allDelegations, err := k.GetAllDelegations(ctx)
	if err != nil {
		return nil, err
	}

	allValidators, err := k.GetAllValidators(ctx)
	if err != nil {
		return nil, err
	}

	rotationIndex := []types.RotationIndexRecord{}
	err = k.ValidatorConsensusKeyRotationRecordIndexKey.Walk(ctx, nil, func(key collections.Pair[[]byte, time.Time]) (stop bool, err error) {
		t := key.K2()
		rotationIndex = append(rotationIndex, types.RotationIndexRecord{
			Address: key.K1(),
			Time:    &t,
		})
		return false, nil
	})
	if err != nil {
		return nil, err
	}

	conspubKeyRotationHistory := []types.ConsPubKeyRotationHistory{}
	err = k.RotationHistory.Walk(ctx, nil, func(key collections.Pair[[]byte, uint64], value types.ConsPubKeyRotationHistory) (stop bool, err error) {
		conspubKeyRotationHistory = append(conspubKeyRotationHistory, value)
		return false, nil
	})
	if err != nil {
		return nil, err
	}

	rotationQueue := []types.RotationQueueRecord{}
	err = k.ValidatorConsensusKeyRotationRecordQueue.Walk(ctx, nil, func(key time.Time, value types.ValAddrsOfRotatedConsKeys) (stop bool, err error) {
		record := types.RotationQueueRecord{
			Time:     &key,
			ValAddrs: &value,
		}
		rotationQueue = append(rotationQueue, record)
		return false, nil
	})
	if err != nil {
		return nil, err
	}

	return &types.GenesisState{
		Params:               params,
		LastTotalPower:       totalPower,
		LastValidatorPowers:  lastValidatorPowers,
		Validators:           allValidators,
		Delegations:          allDelegations,
		UnbondingDelegations: unbondingDelegations,
		Redelegations:        redelegations,
		Exported:             true,
		RotationIndexRecords: rotationIndex,
		RotationHistory:      conspubKeyRotationHistory,
		RotationQueue:        rotationQueue,
	}, nil
}
