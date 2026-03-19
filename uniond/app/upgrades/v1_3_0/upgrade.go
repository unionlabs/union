package v1_3_0

import (
	"bytes"
	"context"
	sdkmath "cosmossdk.io/math"
	"math"

	storetypes "cosmossdk.io/store/types"
	upgradetypes "cosmossdk.io/x/upgrade/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	query "github.com/cosmos/cosmos-sdk/types/query"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"

	"github.com/unionlabs/union/uniond/app/upgrades"
)

const UNION_TESTNET = "union-testnet-10"

func CreateUpgradeHandler(mm *module.Manager, configurator module.Configurator, keepers *upgrades.AppKeepers, getKey upgrades.GetKeyFunc) upgradetypes.UpgradeHandler {
	return func(ctx context.Context, plan upgradetypes.Plan, vm module.VersionMap) (module.VersionMap, error) {
		migrations, err := mm.RunMigrations(ctx, configurator, vm)
		if err != nil {
			return nil, err
		}

		sdkCtx := sdk.UnwrapSDKContext(ctx)

		validators, err := keepers.StakingKeeper.GetAllValidators(ctx)
		if err != nil {
			return nil, err
		}

		if sdkCtx.ChainID() == UNION_TESTNET {
			// adjust power reduction of all validators
			// Adapted from https://github.com/DoraFactory/doravota/blob/b735125f0dfa2a0f50afac0685aa73698cf2227f/app/app.go#L1079-L1114
			sdkCtx.Logger().Info("resetting validator state")
			for _, validator := range validators {

				// Direct access to the staking store
				// Staking interfaces were not sufficient for removing duplicates in the set
				store := sdkCtx.KVStore(getKey(stakingtypes.StoreKey))

				deleted := false

				iterator := storetypes.KVStorePrefixIterator(store, stakingtypes.ValidatorsByPowerIndexKey)
				defer iterator.Close()

				for ; iterator.Valid(); iterator.Next() {
					valAddr := stakingtypes.ParseValidatorPowerRankKey(iterator.Key())

					bz, err := keepers.StakingKeeper.ValidatorAddressCodec().StringToBytes(validator.GetOperator())
					if err != nil {
						panic(err)
					}

					// Delete all entries of a validator including duplicates
					// Duplicates seem to exists due to PoA interacting with the new power reduction
					if bytes.Equal(valAddr, bz) {
						if deleted {
							sdkCtx.Logger().Info("deleting duplicate validator")
						} else {
							deleted = true
							sdkCtx.Logger().Info("deleting validator first record")
						}

						store.Delete(iterator.Key())
						sdkCtx.Logger().Info("deleted the key")
					}
				}

				// Set validator again after deleting, recreating the validator with the correct power reduction
				keepers.StakingKeeper.SetValidatorByPowerIndex(ctx, validator)
				sdkCtx.Logger().Info("reset validator")
				_, err := keepers.StakingKeeper.ApplyAndReturnValidatorSetUpdates(ctx)
				sdkCtx.Logger().Info("update valset")
				if err != nil {
					panic(err)
				}

				sdkCtx.Logger().Info("done with validator")
			}
		}

		if sdkCtx.ChainID() == UNION_TESTNET {
			// Burn old tokens
			burnToken(ctx, *keepers, "muno") // union-minimal-devnet-1 (local devnet) and union-testnet-10 gas token
			burnToken(ctx, *keepers, "upoa") // union-1 and union-testnet-10 PoA token
			burnToken(ctx, *keepers, "ugas") // union-1 gas token
		}

		return migrations, nil
	}
}

func burnToken(ctx context.Context, keepers upgrades.AppKeepers, denom string) error {
	var tokenOwners *banktypes.QueryDenomOwnersResponse
	var err error

	tokenOwners, err = keepers.BankKeeper.DenomOwners(ctx, &banktypes.QueryDenomOwnersRequest{
		Denom:      denom,
		Pagination: nil,
	})
	tokenOwners, err = keepers.BankKeeper.DenomOwners(ctx, &banktypes.QueryDenomOwnersRequest{
		Denom: denom,
		Pagination: &query.PageRequest{
			Key:        []byte{},
			Offset:     0,
			Limit:      math.MaxUint64,
			CountTotal: false,
			Reverse:    false,
		},
	})
	if err != nil {
		return err
	}

	tokenSum := sdkmath.ZeroInt()
	for _, tokenOwner := range tokenOwners.DenomOwners {
		accAddr, err := sdk.AccAddressFromBech32(tokenOwner.Address)
		if err != nil {
			return err
		}
		err = keepers.BankKeeper.SendCoinsFromAccountToModule(ctx, accAddr, govtypes.ModuleName, sdk.NewCoins(tokenOwner.Balance))

		if err != nil {
			return err
		}
		tokenSum = tokenSum.Add(tokenOwner.Balance.Amount)
	}
	if tokenSum.GT(sdkmath.ZeroInt()) {
		err = keepers.BankKeeper.BurnCoins(ctx, govtypes.ModuleName, sdk.NewCoins(sdk.Coin{
			Denom:  denom,
			Amount: tokenSum,
		}))
		if err != nil {
			return err
		}
	}
	return nil
}
