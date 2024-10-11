package types

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

func TestIsAllowedClient(t *testing.T) {
	testCases := []struct {
		name       string
		clientType string
		params     Params
		expPass    bool
	}{
		{"success: valid client", exported.Tendermint, DefaultParams(), true},
		{"success: valid client with custom params", exported.Tendermint, NewParams(exported.Tendermint), true},
		{"success: invalid blank client", " ", DefaultParams(), false},
		{"success: invalid client with custom params", exported.Localhost, NewParams(exported.Tendermint), false},
		{"success: wildcard allow all clients", "test-client-type", NewParams(AllowAllClients), true},
		{"success: wildcard allow all clients with blank client", " ", NewParams(AllowAllClients), false},
	}

	for _, tc := range testCases {
		tc := tc
		require.Equal(t, tc.expPass, tc.params.IsAllowedClient(tc.clientType), tc.name)
	}
}

func TestValidateParams(t *testing.T) {
	testCases := []struct {
		name    string
		params  Params
		expPass bool
	}{
		{"default params", DefaultParams(), true},
		{"custom params", NewParams(exported.Tendermint), true},
		{"blank client", NewParams(" "), false},
		{"duplicate clients", NewParams(exported.Tendermint, exported.Tendermint), false},
		{"allow all clients plus valid client", NewParams(AllowAllClients, exported.Tendermint), false},
	}

	for _, tc := range testCases {
		tc := tc

		err := tc.params.Validate()
		if tc.expPass {
			require.NoError(t, err, tc.name)
		} else {
			require.Error(t, err, tc.name)
		}
	}
}
