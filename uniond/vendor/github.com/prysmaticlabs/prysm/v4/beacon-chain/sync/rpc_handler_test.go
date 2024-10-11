package sync

import (
	"context"
	"testing"
	"time"

	"github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/protocol"
	p2ptest "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

type rpcHandlerTest struct {
	t       *testing.T
	topic   protocol.ID
	timeout time.Duration
	err     error
	s       *Service
}

func (rt *rpcHandlerTest) testHandler(nh network.StreamHandler, rh rpcHandler, rhi interface{}) {
	ctx, cancel := context.WithTimeout(context.Background(), rt.timeout)
	defer func() {
		cancel()
	}()

	w := util.NewWaiter()
	server := p2ptest.NewTestP2P(rt.t)

	client, ok := rt.s.cfg.p2p.(*p2ptest.TestP2P)
	require.Equal(rt.t, true, ok)

	client.Connect(server)
	defer func() {
		require.NoError(rt.t, client.Disconnect(server.PeerID()))
	}()
	require.Equal(rt.t, 1, len(client.BHost.Network().Peers()), "Expected peers to be connected")
	h := func(stream network.Stream) {
		defer w.Done()
		nh(stream)
	}
	server.BHost.SetStreamHandler(rt.topic, h)
	stream, err := client.BHost.NewStream(ctx, server.BHost.ID(), rt.topic)
	require.NoError(rt.t, err)

	err = rh(ctx, rhi, stream)
	if rt.err == nil {
		require.NoError(rt.t, err)
	} else {
		require.ErrorIs(rt.t, err, rt.err)
	}

	w.RequireDoneBeforeCancel(ctx, rt.t)
}
