package keeper

import (
	"context"

	"cosmossdk.io/x/slashing/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// InitGenesis initializes default parameters and the keeper's address to
// pubkey map.
func (keeper Keeper) InitGenesis(ctx context.Context, stakingKeeper types.StakingKeeper, data *types.GenesisState) error {
	var fnErr error
	err := stakingKeeper.IterateValidators(ctx,
		func(index int64, validator sdk.ValidatorI) bool {
			consPk, err := validator.ConsPubKey()
			if err != nil {
				fnErr = err
				return true
			}

			err = keeper.AddrPubkeyRelation.Set(ctx, consPk.Address(), consPk)
			if err != nil {
				fnErr = err
				return true
			}
			return false
		},
	)
	if err != nil {
		return err
	}
	if fnErr != nil {
		return fnErr
	}

	for _, info := range data.SigningInfos {
		address, err := keeper.sk.ConsensusAddressCodec().StringToBytes(info.Address)
		if err != nil {
			return err
		}
		err = keeper.ValidatorSigningInfo.Set(ctx, address, info.ValidatorSigningInfo)
		if err != nil {
			return err
		}
	}

	for _, array := range data.MissedBlocks {
		address, err := keeper.sk.ConsensusAddressCodec().StringToBytes(array.Address)
		if err != nil {
			return err
		}

		for _, missed := range array.MissedBlocks {
			if err := keeper.SetMissedBlockBitmapValue(ctx, address, missed.Index, missed.Missed); err != nil {
				return err
			}
		}
	}

	if err := keeper.Params.Set(ctx, data.Params); err != nil {
		return err
	}
	return nil
}

// ExportGenesis writes the current store values
// to a genesis file, which can be imported again
// with InitGenesis
func (keeper Keeper) ExportGenesis(ctx context.Context) (*types.GenesisState, error) {
	params, err := keeper.Params.Get(ctx)
	if err != nil {
		return nil, err
	}
	signingInfos := make([]types.SigningInfo, 0)
	missedBlocks := make([]types.ValidatorMissedBlocks, 0)
	err = keeper.ValidatorSigningInfo.Walk(ctx, nil, func(address sdk.ConsAddress, info types.ValidatorSigningInfo) (stop bool, err error) {
		bechAddr, err := keeper.sk.ConsensusAddressCodec().BytesToString(address)
		if err != nil {
			return true, err
		}
		signingInfos = append(signingInfos, types.SigningInfo{
			Address:              bechAddr,
			ValidatorSigningInfo: info,
		})

		localMissedBlocks, err := keeper.GetValidatorMissedBlocks(ctx, address)
		if err != nil {
			return true, err
		}

		missedBlocks = append(missedBlocks, types.ValidatorMissedBlocks{
			Address:      bechAddr,
			MissedBlocks: localMissedBlocks,
		})

		return false, nil
	})
	if err != nil {
		return nil, err
	}
	return types.NewGenesisState(params, signingInfos, missedBlocks), nil
}
