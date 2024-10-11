package simulation

import (
	"context"
	"math/rand"
	"time"

	coreaddress "cosmossdk.io/core/address"
	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/slashing/types"

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

// ProposalMsgs defines the module weighted proposals' contents
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
	var authority sdk.AccAddress = address.Module(types.GovModuleName)

	authorityAddr, err := ac.BytesToString(authority)
	if err != nil {
		return nil, err
	}

	params := types.DefaultParams()
	params.DowntimeJailDuration = time.Duration(simtypes.RandTimestamp(r).UnixNano())
	params.SignedBlocksWindow = int64(simtypes.RandIntBetween(r, 1, 1000))
	params.MinSignedPerWindow = sdkmath.LegacyNewDecWithPrec(int64(simtypes.RandIntBetween(r, 1, 100)), 2)
	params.SlashFractionDoubleSign = sdkmath.LegacyNewDecWithPrec(int64(simtypes.RandIntBetween(r, 1, 100)), 2)
	params.SlashFractionDowntime = sdkmath.LegacyNewDecWithPrec(int64(simtypes.RandIntBetween(r, 1, 100)), 2)

	return &types.MsgUpdateParams{
		Authority: authorityAddr,
		Params:    params,
	}, nil
}
