package types_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
)

// tests ParseChannelSequence and IsValidChannelID
func TestParseChannelSequence(t *testing.T) {
	testCases := []struct {
		name      string
		channelID string
		expSeq    uint64
		expPass   bool
	}{
		{"valid 0", "channel-0", 0, true},
		{"valid 1", "channel-1", 1, true},
		{"valid large sequence", "channel-234568219356718293", 234568219356718293, true},
		// one above uint64 max
		{"invalid uint64", "channel-18446744073709551616", 0, false},
		// uint64 == 20 characters
		{"invalid large sequence", "channel-2345682193567182931243", 0, false},
		{"capital prefix", "Channel-0", 0, false},
		{"missing dash", "channel0", 0, false},
		{"blank id", "               ", 0, false},
		{"empty id", "", 0, false},
		{"negative sequence", "channel--1", 0, false},
	}

	for _, tc := range testCases {
		tc := tc

		seq, err := types.ParseChannelSequence(tc.channelID)
		valid := types.IsValidChannelID(tc.channelID)
		require.Equal(t, tc.expSeq, seq)

		if tc.expPass {
			require.NoError(t, err, tc.name)
			require.True(t, valid)
		} else {
			require.Error(t, err, tc.name)
			require.False(t, valid)
		}
	}
}
