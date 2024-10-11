package types_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

func TestValidateGenesis(t *testing.T) {
	testCases := []struct {
		name     string
		genState types.GenesisState
		expPass  bool
	}{
		{
			name:     "default",
			genState: types.DefaultGenesisState(),
			expPass:  true,
		},
		{
			name: "valid genesis",
			genState: types.NewGenesisState(
				[]types.IdentifiedConnection{
					types.NewIdentifiedConnection(connectionID, types.NewConnectionEnd(types.INIT, clientID, types.Counterparty{clientID2, connectionID2, commitmenttypes.NewMerklePrefix([]byte("prefix"))}, []*types.Version{ibctesting.ConnectionVersion}, 500)),
				},
				[]types.ConnectionPaths{
					{clientID, []string{connectionID}},
				},
				0,
				types.DefaultParams(),
			),
			expPass: true,
		},
		{
			name: "invalid connection",
			genState: types.NewGenesisState(
				[]types.IdentifiedConnection{
					types.NewIdentifiedConnection(connectionID, types.NewConnectionEnd(types.INIT, "(CLIENTIDONE)", types.Counterparty{clientID, connectionID, commitmenttypes.NewMerklePrefix([]byte("prefix"))}, []*types.Version{ibctesting.ConnectionVersion}, 500)),
				},
				[]types.ConnectionPaths{
					{clientID, []string{connectionID}},
				},
				0,
				types.DefaultParams(),
			),
			expPass: false,
		},
		{
			name: "invalid client id",
			genState: types.NewGenesisState(
				[]types.IdentifiedConnection{
					types.NewIdentifiedConnection(connectionID, types.NewConnectionEnd(types.INIT, clientID, types.Counterparty{clientID2, connectionID2, commitmenttypes.NewMerklePrefix([]byte("prefix"))}, []*types.Version{ibctesting.ConnectionVersion}, 500)),
				},
				[]types.ConnectionPaths{
					{"(CLIENTIDONE)", []string{connectionID}},
				},
				0,
				types.DefaultParams(),
			),
			expPass: false,
		},
		{
			name: "invalid path",
			genState: types.NewGenesisState(
				[]types.IdentifiedConnection{
					types.NewIdentifiedConnection(connectionID, types.NewConnectionEnd(types.INIT, clientID, types.Counterparty{clientID2, connectionID2, commitmenttypes.NewMerklePrefix([]byte("prefix"))}, []*types.Version{ibctesting.ConnectionVersion}, 500)),
				},
				[]types.ConnectionPaths{
					{clientID, []string{invalidConnectionID}},
				},
				0,
				types.DefaultParams(),
			),
			expPass: false,
		},
		{
			name: "invalid connection identifier",
			genState: types.NewGenesisState(
				[]types.IdentifiedConnection{
					types.NewIdentifiedConnection("conn-0", types.NewConnectionEnd(types.INIT, clientID, types.Counterparty{clientID2, connectionID2, commitmenttypes.NewMerklePrefix([]byte("prefix"))}, []*types.Version{ibctesting.ConnectionVersion}, 500)),
				},
				[]types.ConnectionPaths{
					{clientID, []string{connectionID}},
				},
				0,
				types.DefaultParams(),
			),
			expPass: false,
		},
		{
			name: "next connection sequence is not greater than maximum connection identifier sequence provided",
			genState: types.NewGenesisState(
				[]types.IdentifiedConnection{
					types.NewIdentifiedConnection(types.FormatConnectionIdentifier(10), types.NewConnectionEnd(types.INIT, clientID, types.Counterparty{clientID2, connectionID2, commitmenttypes.NewMerklePrefix([]byte("prefix"))}, []*types.Version{ibctesting.ConnectionVersion}, 500)),
				},
				[]types.ConnectionPaths{
					{clientID, []string{connectionID}},
				},
				0,
				types.DefaultParams(),
			),
			expPass: false,
		},
		{
			name: "invalid params",
			genState: types.NewGenesisState(
				[]types.IdentifiedConnection{
					types.NewIdentifiedConnection(connectionID, types.NewConnectionEnd(types.INIT, clientID, types.Counterparty{clientID2, connectionID2, commitmenttypes.NewMerklePrefix([]byte("prefix"))}, []*types.Version{ibctesting.ConnectionVersion}, 500)),
				},
				[]types.ConnectionPaths{
					{clientID, []string{connectionID}},
				},
				0,
				types.Params{},
			),
			expPass: false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		err := tc.genState.Validate()
		if tc.expPass {
			require.NoError(t, err, tc.name)
		} else {
			require.Error(t, err, tc.name)
		}
	}
}
