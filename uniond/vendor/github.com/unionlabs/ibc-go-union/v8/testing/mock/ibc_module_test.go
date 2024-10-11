package mock_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	"github.com/cosmos/ibc-go/v8/testing/mock"
)

func TestCreateCapabilityName(t *testing.T) {
	packet := channeltypes.NewPacket(
		[]byte{},
		1,
		mock.PortID,
		"channel-0",
		mock.PortID,
		"channel-0",
		clienttypes.NewHeight(0, 100),
		0,
	)

	name := mock.GetMockRecvCanaryCapabilityName(packet)
	require.Equal(t, "mock receive canary capability namemockchannel-01", name)

	name = mock.GetMockAckCanaryCapabilityName(packet)
	require.Equal(t, "mock acknowledgement canary capability namemockchannel-01", name)

	name = mock.GetMockTimeoutCanaryCapabilityName(packet)
	require.Equal(t, "mock timeout canary capability namemockchannel-01", name)
}
