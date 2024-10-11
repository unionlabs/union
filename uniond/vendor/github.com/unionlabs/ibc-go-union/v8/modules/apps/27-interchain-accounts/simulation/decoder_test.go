package simulation_test

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/types/kv"

	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/simulation"
	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

func TestDecodeStore(t *testing.T) {
	var (
		owner     = "owner"
		channelID = ibctesting.FirstChannelID
	)

	dec := simulation.NewDecodeStore()

	kvPairs := kv.Pairs{
		Pairs: []kv.Pair{
			{
				Key:   []byte(types.PortKeyPrefix),
				Value: []byte(types.HostPortID),
			},
			{
				Key:   []byte(types.OwnerKeyPrefix),
				Value: []byte("owner"),
			},
			{
				Key:   []byte(types.ActiveChannelKeyPrefix),
				Value: []byte("channel-0"),
			},
			{
				Key:   []byte(types.IsMiddlewareEnabledPrefix),
				Value: []byte("false"),
			},
		},
	}
	tests := []struct {
		name        string
		expectedLog string
	}{
		{"PortID", fmt.Sprintf("Port A: %s\nPort B: %s", types.HostPortID, types.HostPortID)},
		{"Owner", fmt.Sprintf("Owner A: %s\nOwner B: %s", owner, owner)},
		{"ActiveChannel", fmt.Sprintf("ActiveChannel A: %s\nActiveChannel B: %s", channelID, channelID)},
		{"IsMiddlewareEnabled", fmt.Sprintf("IsMiddlewareEnabled A: %s\nIsMiddlewareEnabled B: %s", "false", "false")},
		{"other", ""},
	}

	for i, tt := range tests {
		i, tt := i, tt
		t.Run(tt.name, func(t *testing.T) {
			if i == len(tests)-1 {
				require.Panics(t, func() { dec(kvPairs.Pairs[i], kvPairs.Pairs[i]) }, tt.name)
			} else {
				require.Equal(t, tt.expectedLog, dec(kvPairs.Pairs[i], kvPairs.Pairs[i]), tt.name)
			}
		})
	}
}
