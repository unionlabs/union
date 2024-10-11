package types_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

var (
	chainID             = "gaiamainnet"
	connectionID        = "connection-0"
	clientID            = "clientidone"
	connectionID2       = "connectionidtwo"
	clientID2           = "clientidtwo"
	invalidConnectionID = "(invalidConnectionID)"
	clientHeight        = clienttypes.NewHeight(0, 6)
)

func TestConnectionValidateBasic(t *testing.T) {
	testCases := []struct {
		name       string
		connection types.ConnectionEnd
		expPass    bool
	}{
		{
			"valid connection",
			types.ConnectionEnd{clientID, []*types.Version{ibctesting.ConnectionVersion}, types.INIT, types.Counterparty{clientID2, connectionID2, commitmenttypes.NewMerklePrefix([]byte("prefix"))}, 500},
			true,
		},
		{
			"invalid client id",
			types.ConnectionEnd{"(clientID1)", []*types.Version{ibctesting.ConnectionVersion}, types.INIT, types.Counterparty{clientID2, connectionID2, commitmenttypes.NewMerklePrefix([]byte("prefix"))}, 500},
			false,
		},
		{
			"empty versions",
			types.ConnectionEnd{clientID, nil, types.INIT, types.Counterparty{clientID2, connectionID2, commitmenttypes.NewMerklePrefix([]byte("prefix"))}, 500},
			false,
		},
		{
			"invalid version",
			types.ConnectionEnd{clientID, []*types.Version{{}}, types.INIT, types.Counterparty{clientID2, connectionID2, commitmenttypes.NewMerklePrefix([]byte("prefix"))}, 500},
			false,
		},
		{
			"invalid counterparty",
			types.ConnectionEnd{clientID, []*types.Version{ibctesting.ConnectionVersion}, types.INIT, types.Counterparty{clientID2, connectionID2, emptyPrefix}, 500},
			false,
		},
	}

	for i, tc := range testCases {
		tc := tc

		err := tc.connection.ValidateBasic()
		if tc.expPass {
			require.NoError(t, err, "valid test case %d failed: %s", i, tc.name)
		} else {
			require.Error(t, err, "invalid test case %d passed: %s", i, tc.name)
		}
	}
}

func TestCounterpartyValidateBasic(t *testing.T) {
	testCases := []struct {
		name         string
		counterparty types.Counterparty
		expPass      bool
	}{
		{"valid counterparty", types.Counterparty{clientID, connectionID2, commitmenttypes.NewMerklePrefix([]byte("prefix"))}, true},
		{"invalid client id", types.Counterparty{"(InvalidClient)", connectionID2, commitmenttypes.NewMerklePrefix([]byte("prefix"))}, false},
		{"invalid connection id", types.Counterparty{clientID, "(InvalidConnection)", commitmenttypes.NewMerklePrefix([]byte("prefix"))}, false},
		{"invalid prefix", types.Counterparty{clientID, connectionID2, emptyPrefix}, false},
	}

	for i, tc := range testCases {
		tc := tc

		err := tc.counterparty.ValidateBasic()
		if tc.expPass {
			require.NoError(t, err, "valid test case %d failed: %s", i, tc.name)
		} else {
			require.Error(t, err, "invalid test case %d passed: %s", i, tc.name)
		}
	}
}

func TestIdentifiedConnectionValidateBasic(t *testing.T) {
	testCases := []struct {
		name       string
		connection types.IdentifiedConnection
		expPass    bool
	}{
		{
			"valid connection",
			types.NewIdentifiedConnection(clientID, types.ConnectionEnd{clientID, []*types.Version{ibctesting.ConnectionVersion}, types.INIT, types.Counterparty{clientID2, connectionID2, commitmenttypes.NewMerklePrefix([]byte("prefix"))}, 500}),
			true,
		},
		{
			"invalid connection id",
			types.NewIdentifiedConnection("(connectionIDONE)", types.ConnectionEnd{clientID, []*types.Version{ibctesting.ConnectionVersion}, types.INIT, types.Counterparty{clientID2, connectionID2, commitmenttypes.NewMerklePrefix([]byte("prefix"))}, 500}),
			false,
		},
	}

	for i, tc := range testCases {
		tc := tc

		err := tc.connection.ValidateBasic()
		if tc.expPass {
			require.NoError(t, err, "valid test case %d failed: %s", i, tc.name)
		} else {
			require.Error(t, err, "invalid test case %d passed: %s", i, tc.name)
		}
	}
}
