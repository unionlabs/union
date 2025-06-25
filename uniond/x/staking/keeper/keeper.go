package keeper

import (
	stakingkeeper "github.com/cosmos/cosmos-sdk/x/staking/keeper"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
	types "github.com/unionlabs/union/uniond/x/staking/types"
)

type Keeper struct {
	stakingMsgServer stakingtypes.MsgServer
	StakingHooks     *Hooks
}

func NewKeeper(stakingKeeper types.StakingKeeper) Keeper {
	return Keeper{
		stakingMsgServer: stakingkeeper.NewMsgServerImpl(stakingKeeper.(*stakingkeeper.Keeper)),
		StakingHooks: &Hooks{
			ProofOfPossessionPassed: false,
		},
	}
}
