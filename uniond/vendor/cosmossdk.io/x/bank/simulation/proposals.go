package simulation

import (
	"context"
	"math/rand"

	coreaddress "cosmossdk.io/core/address"
	"cosmossdk.io/x/bank/types"

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
	authority, err := ac.BytesToString(address.Module(types.GovModuleName))
	if err != nil {
		return nil, err
	}

	params := types.DefaultParams()
	params.DefaultSendEnabled = r.Intn(2) == 0

	return &types.MsgUpdateParams{
		Authority: authority,
		Params:    params,
	}, nil
}
