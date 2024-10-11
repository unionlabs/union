package simulation

import (
	"context"
	"math/rand"

	"cosmossdk.io/core/address"

	sdk "github.com/cosmos/cosmos-sdk/types"
	addresstypes "github.com/cosmos/cosmos-sdk/types/address"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/simulation"

	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
)

// Simulation operation weights constants
const (
	DefaultWeightMsgStoreCode int = 100

	OpWeightMsgStoreCode = "op_weight_msg_store_code" // #nosec
)

// ProposalMsgs defines the module weighted proposals' contents
func ProposalMsgs() []simtypes.WeightedProposalMsg {
	return []simtypes.WeightedProposalMsg{
		simulation.NewWeightedProposalMsgX(
			OpWeightMsgStoreCode,
			DefaultWeightMsgStoreCode,
			SimulateMsgStoreCode,
		),
	}
}

// SimulateMsgStoreCode returns a random MsgStoreCode for the 08-wasm module
func SimulateMsgStoreCode(ctx context.Context, r *rand.Rand, _ []simtypes.Account, _ address.Codec) (sdk.Msg, error) {
	var signer sdk.AccAddress = addresstypes.Module("gov")

	return &types.MsgStoreCode{
		Signer:       signer.String(),
		WasmByteCode: []byte{0x01},
	}, nil
}
