package simulation

import (
	"math/rand"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/simulation"

	controllertypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/types"
	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host/types"
)

// Simulation operation weights constants
const (
	DefaultWeightMsgUpdateParams int = 100

	OpWeightMsgUpdateParams = "op_weight_msg_update_params" // #nosec
)

// ProposalMsgs defines the module weighted proposals' contents
func ProposalMsgs() []simtypes.WeightedProposalMsg {
	return []simtypes.WeightedProposalMsg{
		simulation.NewWeightedProposalMsg(
			OpWeightMsgUpdateParams,
			DefaultWeightMsgUpdateParams,
			SimulateHostMsgUpdateParams,
		),
		simulation.NewWeightedProposalMsg(
			OpWeightMsgUpdateParams,
			DefaultWeightMsgUpdateParams,
			SimulateControllerMsgUpdateParams,
		),
	}
}

// SimulateHostMsgUpdateParams returns a MsgUpdateParams for the host module
func SimulateHostMsgUpdateParams(_ *rand.Rand, _ sdk.Context, _ []simtypes.Account) sdk.Msg {
	var signer sdk.AccAddress = address.Module("gov")
	params := types.DefaultParams()
	params.HostEnabled = false

	return &types.MsgUpdateParams{
		Signer: signer.String(),
		Params: params,
	}
}

// SimulateControllerMsgUpdateParams returns a MsgUpdateParams for the controller module
func SimulateControllerMsgUpdateParams(_ *rand.Rand, _ sdk.Context, _ []simtypes.Account) sdk.Msg {
	var signer sdk.AccAddress = address.Module("gov")
	params := controllertypes.DefaultParams()
	params.ControllerEnabled = false

	return &controllertypes.MsgUpdateParams{
		Signer: signer.String(),
		Params: params,
	}
}
