package simulation

import (
	"context"
	"math/rand"

	coreaddress "cosmossdk.io/core/address"
	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/mint/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/simulation"
)

// Simulation operation weights constants
const (
	DefaultWeightMsgUpdateParams int = 100

	OpWeightMsgUpdateParams = "op_weight_msg_update_params"
)

// ProposalMsgs defines the module's weighted proposals contents
func ProposalMsgs() []simtypes.WeightedProposalMsg {
	return []simtypes.WeightedProposalMsg{
		simulation.NewWeightedProposalMsgX(
			OpWeightMsgUpdateParams,
			DefaultWeightMsgUpdateParams,
			SimulateMsgUpdateParams,
		),
	}
}

// SimulateMsgUpdateParams returns a random MsgUpdateParams
func SimulateMsgUpdateParams(_ context.Context, r *rand.Rand, _ []simtypes.Account, ac coreaddress.Codec) (sdk.Msg, error) {
	// use the default gov module account address as authority
	var authority sdk.AccAddress = address.Module("gov")

	authorityAddr, err := ac.BytesToString(authority)
	if err != nil {
		return nil, err
	}

	params := types.DefaultParams()
	params.BlocksPerYear = uint64(simtypes.RandIntBetween(r, 1, 1000000))
	params.GoalBonded = sdkmath.LegacyNewDecWithPrec(int64(simtypes.RandIntBetween(r, 1, 100)), 2)
	params.InflationMin = sdkmath.LegacyNewDecWithPrec(int64(simtypes.RandIntBetween(r, 1, 50)), 2)
	params.InflationMax = sdkmath.LegacyNewDecWithPrec(int64(simtypes.RandIntBetween(r, 50, 100)), 2)
	params.InflationRateChange = sdkmath.LegacyNewDecWithPrec(int64(simtypes.RandIntBetween(r, 1, 100)), 2)
	params.MintDenom = simtypes.RandStringOfLength(r, 10)

	return &types.MsgUpdateParams{
		Authority: authorityAddr,
		Params:    params,
	}, nil
}
