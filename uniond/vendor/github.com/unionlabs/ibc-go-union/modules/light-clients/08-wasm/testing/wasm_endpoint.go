package testing

import (
	"github.com/stretchr/testify/require"

	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

// WasmEndpoint is a wrapper around the ibctesting pkg Endpoint struct.
// It will override any functions which require special handling for the wasm client.
type WasmEndpoint struct {
	*ibctesting.Endpoint
}

// NewWasmEndpoint returns a wasm endpoint with the default ibctesting pkg
// Endpoint embedded.
func NewWasmEndpoint(chain *ibctesting.TestChain) *WasmEndpoint {
	return &WasmEndpoint{
		Endpoint: ibctesting.NewDefaultEndpoint(chain),
	}
}

// CreateClient creates an wasm client on a mock cometbft chain.
// The client and consensus states are represented by byte slices
// and the starting height is 1.
func (endpoint *WasmEndpoint) CreateClient() error {
	checksum, err := types.CreateChecksum(Code)
	require.NoError(endpoint.Chain.TB, err)

	wrappedClientStateBz := clienttypes.MustMarshalClientState(endpoint.Chain.App.AppCodec(), CreateMockTendermintClientState(clienttypes.NewHeight(1, 5)))
	wrappedClientConsensusStateBz := clienttypes.MustMarshalConsensusState(endpoint.Chain.App.AppCodec(), MockTendermintClientConsensusState)

	clientState := types.NewClientState(wrappedClientStateBz, checksum, clienttypes.NewHeight(0, 1))
	consensusState := types.NewConsensusState(wrappedClientConsensusStateBz)

	msg, err := clienttypes.NewMsgCreateClient(
		clientState, consensusState, endpoint.Chain.SenderAccount.GetAddress().String(),
	)
	require.NoError(endpoint.Chain.TB, err)

	res, err := endpoint.Chain.SendMsgs(msg)
	if err != nil {
		return err
	}

	endpoint.ClientID, err = ibctesting.ParseClientIDFromEvents(res.Events)
	require.NoError(endpoint.Chain.TB, err)

	return nil
}

// AllowWasmClients adds 08-wasm to the list of allowed clients
func AllowWasmClients(chain *ibctesting.TestChain) {
	ctx := chain.GetContext()
	clientKeeper := chain.App.GetIBCKeeper().ClientKeeper
	params := clientKeeper.GetParams(ctx)
	params.AllowedClients = append(params.AllowedClients, types.Wasm)
	clientKeeper.SetParams(ctx, params)
}
