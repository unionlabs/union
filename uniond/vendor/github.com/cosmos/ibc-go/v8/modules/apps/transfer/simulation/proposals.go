package simulation

import (
	"context"
	"math/rand"

	coreaddress "cosmossdk.io/core/address"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/simulation"

	"github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
)

// Simulation operation weights constants
const (
	DefaultWeightMsgUpdateParams int = 100

	OpWeightMsgUpdateParams = "op_weight_msg_update_params" // #nosec
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

// SimulateMsgUpdateParams returns a MsgUpdateParams
func SimulateMsgUpdateParams(_ context.Context, _ *rand.Rand, _ []simtypes.Account, accCdc coreaddress.Codec) (sdk.Msg, error) {
	gov := address.Module("gov")
	govString, err := accCdc.BytesToString(gov)
	if err != nil {
		return nil, err
	}
	params := types.DefaultParams()
	params.SendEnabled = false

	return &types.MsgUpdateParams{
		Signer: govString,
		Params: params,
	}, nil
}
