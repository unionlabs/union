package host_test

import (
	"fmt"
	"math"
	"testing"

	"github.com/stretchr/testify/require"

	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

func TestParseIdentifier(t *testing.T) {
	testCases := []struct {
		name       string
		identifier string
		prefix     string
		expSeq     uint64
		expPass    bool
	}{
		{"valid 0", "connection-0", "connection-", 0, true},
		{"valid 1", "connection-1", "connection-", 1, true},
		{"valid large sequence", connectiontypes.FormatConnectionIdentifier(math.MaxUint64), "connection-", math.MaxUint64, true},
		// one above uint64 max
		{"invalid uint64", "connection-18446744073709551616", "connection-", 0, false},
		// uint64 == 20 characters
		{"invalid large sequence", "connection-2345682193567182931243", "connection-", 0, false},
		{"capital prefix", "Connection-0", "connection-", 0, false},
		{"double prefix", "connection-connection-0", "connection-", 0, false},
		{"doesn't have prefix", "connection-0", "prefix", 0, false},
		{"missing dash", "connection0", "connection-", 0, false},
		{"blank id", "               ", "connection-", 0, false},
		{"empty id", "", "connection-", 0, false},
		{"negative sequence", "connection--1", "connection-", 0, false},
	}

	for _, tc := range testCases {
		tc := tc

		seq, err := host.ParseIdentifier(tc.identifier, tc.prefix)
		require.Equal(t, tc.expSeq, seq)

		if tc.expPass {
			require.NoError(t, err, tc.name)
		} else {
			require.Error(t, err, tc.name)
		}
	}
}

func TestMustParseClientStatePath(t *testing.T) {
	testCases := []struct {
		name    string
		path    string
		expPass bool
	}{
		{"valid", host.FullClientStatePath(ibctesting.FirstClientID), true},
		{"path too large", fmt.Sprintf("clients/clients/%s/clientState", ibctesting.FirstClientID), false},
		{"path too small", fmt.Sprintf("clients/%s", ibctesting.FirstClientID), false},
		{"path does not begin with client store", fmt.Sprintf("cli/%s/%s", ibctesting.FirstClientID, host.KeyClientState), false},
		{"path does not end with client state key", fmt.Sprintf("%s/%s/consensus", string(host.KeyClientStorePrefix), ibctesting.FirstClientID), false},
		{"client ID is empty", host.FullClientStatePath(""), false},
		{"client ID is only spaces", host.FullClientStatePath("   "), false},
	}

	for _, tc := range testCases {
		tc := tc

		if tc.expPass {
			require.NotPanics(t, func() {
				clientID := host.MustParseClientStatePath(tc.path)
				require.Equal(t, ibctesting.FirstClientID, clientID)
			})
		} else {
			require.Panics(t, func() {
				host.MustParseClientStatePath(tc.path)
			})
		}
	}
}

func TestMustParseConnectionPath(t *testing.T) {
	testCases := []struct {
		name     string
		path     string
		expected string
		expPass  bool
	}{
		{"valid", "a/connection", "connection", true},
		{"valid localhost", "/connection-localhost", "connection-localhost", true},
		{"invalid empty path", "", "", false},
	}

	for _, tc := range testCases {
		if tc.expPass {
			require.NotPanics(t, func() {
				connID := host.MustParseConnectionPath(tc.path)
				require.Equal(t, connID, tc.expected)
			})
		} else {
			require.Panics(t, func() {
				host.MustParseConnectionPath(tc.path)
			})
		}
	}
}
