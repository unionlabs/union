package sync

import (
	"bytes"
	"context"
	"sync"
	"testing"
	"time"

	libp2pcore "github.com/libp2p/go-libp2p/core"
	"github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/protocol"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	prysmP2P "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/encoder"
	p2ptest "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func init() {
	transition.SkipSlotCache.Disable()
}

// expectSuccess status code from a stream in regular sync.
func expectSuccess(t *testing.T, stream network.Stream) {
	code, errMsg, err := ReadStatusCode(stream, &encoder.SszNetworkEncoder{})
	require.NoError(t, err)
	require.Equal(t, uint8(0), code, "Received non-zero response code")
	require.Equal(t, "", errMsg, "Received error message from stream")
}

// expectSuccess status code from a stream in regular sync.
func expectFailure(t *testing.T, expectedCode uint8, expectedErrorMsg string, stream network.Stream) {
	code, errMsg, err := ReadStatusCode(stream, &encoder.SszNetworkEncoder{})
	require.NoError(t, err)
	require.NotEqual(t, uint8(0), code, "Expected request to fail but got a 0 response code")
	require.Equal(t, expectedCode, code, "Received incorrect response code")
	require.Equal(t, expectedErrorMsg, errMsg)
}

// expectResetStream status code from a stream in regular sync.
func expectResetStream(t *testing.T, stream network.Stream) {
	expectedErr := "stream reset"
	_, _, err := ReadStatusCode(stream, &encoder.SszNetworkEncoder{})
	require.ErrorContains(t, expectedErr, err)
}

func TestRegisterRPC_ReceivesValidMessage(t *testing.T) {
	p2p := p2ptest.NewTestP2P(t)
	r := &Service{
		ctx:         context.Background(),
		cfg:         &config{p2p: p2p},
		rateLimiter: newRateLimiter(p2p),
	}

	var wg sync.WaitGroup
	wg.Add(1)
	topic := "/testing/foobar/1"
	handler := func(ctx context.Context, msg interface{}, stream libp2pcore.Stream) error {
		m, ok := msg.(*ethpb.Fork)
		if !ok {
			t.Error("Object is not of type *pb.TestSimpleMessage")
		}
		assert.DeepEqual(t, []byte("fooo"), m.CurrentVersion, "Unexpected incoming message")
		wg.Done()

		return nil
	}
	prysmP2P.RPCTopicMappings[topic] = new(ethpb.Fork)
	// Cleanup Topic mappings
	defer func() {
		delete(prysmP2P.RPCTopicMappings, topic)
	}()
	r.registerRPC(topic, handler)

	p2p.ReceiveRPC(topic, &ethpb.Fork{CurrentVersion: []byte("fooo"), PreviousVersion: []byte("barr")})

	if util.WaitTimeout(&wg, time.Second) {
		t.Fatal("Did not receive RPC in 1 second")
	}
}

func TestRPC_ReceivesInvalidMessage(t *testing.T) {
	p2p := p2ptest.NewTestP2P(t)
	remotePeer := p2ptest.NewTestP2P(t)
	remotePeer.Connect(p2p)

	r := &Service{
		ctx:         context.Background(),
		cfg:         &config{p2p: p2p},
		rateLimiter: newRateLimiter(p2p),
	}

	topic := "/testing/foobar/1"
	handler := func(ctx context.Context, msg interface{}, stream libp2pcore.Stream) error {
		m, ok := msg.(*ethpb.Fork)
		if !ok {
			t.Error("Object is not of type *pb.Fork")
		}
		if !bytes.Equal(m.CurrentVersion, []byte("fooo")) {
			t.Errorf("Unexpected incoming message: %+v", m)
		}
		return nil
	}
	prysmP2P.RPCTopicMappings[topic] = new(ethpb.Fork)
	// Cleanup Topic mappings
	defer func() {
		delete(prysmP2P.RPCTopicMappings, topic)
	}()
	r.registerRPC(topic, handler)

	stream, err := remotePeer.Host().NewStream(context.Background(), p2p.BHost.ID(), protocol.ID(topic+p2p.Encoding().ProtocolSuffix()))
	require.NoError(t, err)
	// Write invalid SSZ object to peer.
	_, err = stream.Write([]byte("JUNK MESSAGE"))
	require.NoError(t, err)

	time.Sleep(1 * time.Second)
	faultCount, err := p2p.Peers().Scorers().BadResponsesScorer().Count(remotePeer.BHost.ID())
	require.NoError(t, err)

	assert.Equal(t, 1, faultCount, "peer was not penalised for sending bad message")

}
