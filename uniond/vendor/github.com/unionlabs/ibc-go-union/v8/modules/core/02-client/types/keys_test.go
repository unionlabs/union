package types_test

import (
	"math"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
)

// tests ParseClientIdentifier and IsValidClientID
func TestParseClientIdentifier(t *testing.T) {
	testCases := []struct {
		name       string
		clientID   string
		clientType string
		expSeq     uint64
		expPass    bool
	}{
		{"valid 0", "tendermint-0", "tendermint", 0, true},
		{"valid 1", "tendermint-1", "tendermint", 1, true},
		{"valid solemachine", "solomachine-v1-1", "solomachine-v1", 1, true},
		{"valid large sequence", types.FormatClientIdentifier("tendermint", math.MaxUint64), "tendermint", math.MaxUint64, true},
		{"valid short client type", "t-0", "t", 0, true},
		// one above uint64 max
		{"invalid uint64", "tendermint-18446744073709551616", "tendermint", 0, false},
		// uint64 == 20 characters
		{"invalid large sequence", "tendermint-2345682193567182931243", "tendermint", 0, false},
		{"invalid newline in clientID", "tendermin\nt-1", "tendermin\nt", 0, false},
		{"invalid newline character before dash", "tendermint\n-1", "tendermint", 0, false},
		{"missing dash", "tendermint0", "tendermint", 0, false},
		{"blank id", "               ", "    ", 0, false},
		{"empty id", "", "", 0, false},
		{"negative sequence", "tendermint--1", "tendermint", 0, false},
		{"invalid format", "tendermint-tm", "tendermint", 0, false},
		{"empty clientype", " -100", "tendermint", 0, false},
		{"with in the middle tabs", "a\t\t\t-100", "tendermint", 0, false},
		{"leading tabs", "\t\t\ta-100", "tendermint", 0, false},
		{"with whitespace", "                  a-100", "tendermint", 0, false},
		{"leading hyphens", "-----a-100", "tendermint", 0, false},
		{"with slash", "tendermint/-100", "tendermint", 0, false},
		{"non-ASCII:: emoji", "🚨😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎😎-100", "tendermint", 0, false},
		{"non-ASCII:: others", "世界-100", "tendermint", 0, false},
	}

	for _, tc := range testCases {
		tc := tc
		t.Run(tc.name, func(t *testing.T) {
			clientType, seq, err := types.ParseClientIdentifier(tc.clientID)
			valid := types.IsValidClientID(tc.clientID)
			require.Equal(t, tc.expSeq, seq, tc.clientID)

			if tc.expPass {
				require.NoError(t, err, tc.name)
				require.True(t, valid)
				require.Equal(t, tc.clientType, clientType)
			} else {
				require.Error(t, err, tc.name, tc.clientID)
				require.False(t, valid)
				require.Equal(t, "", clientType)
			}
		})
	}
}
