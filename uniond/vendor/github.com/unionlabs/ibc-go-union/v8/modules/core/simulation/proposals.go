package simulation

import (
	"context"
	"math/rand"
	"time"

	coreaddress "cosmossdk.io/core/address"
	upgradetypes "cosmossdk.io/x/upgrade/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/simulation"

	"github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	ibctm "github.com/cosmos/ibc-go/v8/modules/light-clients/07-tendermint"
)

// Simulation operation weights constants
const (
	DefaultWeight int = 100

	OpWeightMsgUpdateParams       = "op_weight_msg_update_params"                 // #nosec
	OpWeightMsgRecoverClient      = "op_weight_msg_recover_client"                // #nosec
	OpWeightMsgIBCSoftwareUpgrade = "op_weight_msg_schedule_ibc_software_upgrade" // #nosec
)

// ProposalMsgs defines the module weighted proposals' contents
func ProposalMsgs() []simtypes.WeightedProposalMsg {
	return []simtypes.WeightedProposalMsg{
		simulation.NewWeightedProposalMsgX(
			OpWeightMsgUpdateParams,
			DefaultWeight,
			SimulateClientMsgUpdateParams,
		),
		simulation.NewWeightedProposalMsgX(
			OpWeightMsgUpdateParams,
			DefaultWeight,
			SimulateConnectionMsgUpdateParams,
		),
		simulation.NewWeightedProposalMsgX(
			OpWeightMsgRecoverClient,
			DefaultWeight,
			SimulateClientMsgRecoverClient,
		),
		simulation.NewWeightedProposalMsgX(
			OpWeightMsgIBCSoftwareUpgrade,
			DefaultWeight,
			SimulateClientMsgScheduleIBCSoftwareUpgrade,
		),
	}
}

// SimulateClientMsgUpdateParams returns a MsgUpdateParams for 02-client
func SimulateClientMsgUpdateParams(ctx context.Context, r *rand.Rand, _ []simtypes.Account, _ coreaddress.Codec) (sdk.Msg, error) {
	var signer sdk.AccAddress = address.Module("gov")
	params := types.DefaultParams()
	params.AllowedClients = []string{"06-solomachine", "07-tendermint"}

	return &types.MsgUpdateParams{
		Signer: signer.String(),
		Params: params,
	}, nil
}

// SimulateClientMsgRecoverClient returns a MsgRecoverClient for 02-client
func SimulateClientMsgRecoverClient(ctx context.Context, r *rand.Rand, _ []simtypes.Account, _ coreaddress.Codec) (sdk.Msg, error) {
	var signer sdk.AccAddress = address.Module("gov")

	return &types.MsgRecoverClient{
		Signer:             signer.String(),
		SubjectClientId:    "07-tendermint-0",
		SubstituteClientId: "07-tendermint-1",
	}, nil
}

// SimulateClientMsgScheduleIBCSoftwareUpgrade returns a MsgScheduleIBCSoftwareUpgrade for 02-client
func SimulateClientMsgScheduleIBCSoftwareUpgrade(ctx context.Context, r *rand.Rand, _ []simtypes.Account, _ coreaddress.Codec) (sdk.Msg, error) {
	var signer sdk.AccAddress = address.Module("gov")

	chainID := "chain-a-0"
	ubdPeriod := time.Hour * 24 * 7 * 2
	upgradePath := []string{"upgrade", "upgradedIBCState"}

	upgradedClientState := &ibctm.ClientState{
		ChainId:         chainID,
		UnbondingPeriod: ubdPeriod,
		ProofSpecs:      commitmenttypes.GetSDKSpecs(),
		UpgradePath:     upgradePath,
	}
	anyClient, err := types.PackClientState(upgradedClientState)
	if err != nil {
		panic(err)
	}

	return &types.MsgIBCSoftwareUpgrade{
		Signer: signer.String(),
		Plan: upgradetypes.Plan{
			Name:   "upgrade",
			Height: 100,
		},
		UpgradedClientState: anyClient,
	}, nil
}

// SimulateConnectionMsgUpdateParams returns a MsgUpdateParams 03-connection
func SimulateConnectionMsgUpdateParams(ctx context.Context, r *rand.Rand, _ []simtypes.Account, _ coreaddress.Codec) (sdk.Msg, error) {
	var signer sdk.AccAddress = address.Module("gov")
	params := connectiontypes.DefaultParams()
	params.MaxExpectedTimePerBlock = uint64(100)

	return &connectiontypes.MsgUpdateParams{
		Signer: signer.String(),
		Params: params,
	}, nil
}
