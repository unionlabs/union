package simulation

import (
	"context"
	"math/rand"

	coreaddress "cosmossdk.io/core/address"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/simulation"

	controllerkeeper "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/keeper"
	controllertypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/types"
	hostkeeper "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host/keeper"
	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host/types"
)

// Simulation operation weights constants
const (
	DefaultWeightMsgUpdateParams int = 100

	OpWeightMsgUpdateParams = "op_weight_msg_update_params" // #nosec
)

// ProposalMsgs defines the module weighted proposals' contents
func ProposalMsgs(controllerKeeper *controllerkeeper.Keeper, hostKeeper *hostkeeper.Keeper) []simtypes.WeightedProposalMsg {
	msgs := make([]simtypes.WeightedProposalMsg, 0, 2)
	if hostKeeper != nil {
		msgs = append(msgs, simulation.NewWeightedProposalMsgX(
			OpWeightMsgUpdateParams,
			DefaultWeightMsgUpdateParams,
			SimulateHostMsgUpdateParams,
		))
	}
	if controllerKeeper != nil {
		msgs = append(msgs, simulation.NewWeightedProposalMsgX(
			OpWeightMsgUpdateParams,
			DefaultWeightMsgUpdateParams,
			SimulateControllerMsgUpdateParams,
		))
	}
	return msgs
}

// SimulateHostMsgUpdateParams returns a MsgUpdateParams for the host module
func SimulateHostMsgUpdateParams(ctx context.Context, _ *rand.Rand, _ []simtypes.Account, _ coreaddress.Codec) (sdk.Msg, error) {
	var signer sdk.AccAddress = address.Module("gov")
	params := types.DefaultParams()
	params.HostEnabled = false

	return &types.MsgUpdateParams{
		Signer: signer.String(),
		Params: params,
	}, nil
}

// SimulateControllerMsgUpdateParams returns a MsgUpdateParams for the controller module
func SimulateControllerMsgUpdateParams(ctx context.Context, _ *rand.Rand, _ []simtypes.Account, _ coreaddress.Codec) (sdk.Msg, error) {
	var signer sdk.AccAddress = address.Module("gov")
	params := controllertypes.DefaultParams()
	params.ControllerEnabled = false

	return &controllertypes.MsgUpdateParams{
		Signer: signer.String(),
		Params: params,
	}, nil
}
