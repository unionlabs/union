package sync

import (
	"context"
	"sync"
	"testing"
	"time"

	core "github.com/libp2p/go-libp2p/core"
	"github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/protocol"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	p2ptest "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestContextWrite_NoWrites(t *testing.T) {
	p1 := p2ptest.NewTestP2P(t)
	nPeer := p2ptest.NewTestP2P(t)
	p1.Connect(nPeer)

	wg := new(sync.WaitGroup)
	prID := p2p.RPCPingTopicV1
	wg.Add(1)
	nPeer.BHost.SetStreamHandler(core.ProtocolID(prID), func(stream network.Stream) {
		wg.Done()
		// no-op
	})
	strm, err := p1.BHost.NewStream(context.Background(), nPeer.PeerID(), p2p.RPCPingTopicV1)
	assert.NoError(t, err)

	// Nothing will be written to the stream
	assert.NoError(t, writeContextToStream([]byte{}, strm))
	if util.WaitTimeout(wg, 1*time.Second) {
		t.Fatal("Did not receive stream within 1 sec")
	}
}

func TestContextRead_NoReads(t *testing.T) {
	p1 := p2ptest.NewTestP2P(t)
	nPeer := p2ptest.NewTestP2P(t)
	p1.Connect(nPeer)

	wg := new(sync.WaitGroup)
	prID := p2p.RPCPingTopicV1
	wg.Add(1)
	wantedData := []byte{'A', 'B', 'C', 'D'}
	nPeer.BHost.SetStreamHandler(core.ProtocolID(prID), func(stream network.Stream) {
		// No Context will be read from it
		dt, err := readContextFromStream(stream)
		assert.NoError(t, err)
		assert.Equal(t, 0, len(dt))

		// Ensure sent over data hasn't been modified.
		buf := make([]byte, len(wantedData))
		n, err := stream.Read(buf)
		assert.NoError(t, err)
		assert.Equal(t, len(wantedData), n)
		assert.DeepEqual(t, wantedData, buf)

		wg.Done()
	})
	strm, err := p1.BHost.NewStream(context.Background(), nPeer.PeerID(), p2p.RPCPingTopicV1)
	assert.NoError(t, err)

	n, err := strm.Write(wantedData)
	assert.NoError(t, err)
	assert.Equal(t, len(wantedData), n)
	if util.WaitTimeout(wg, 1*time.Second) {
		t.Fatal("Did not receive stream within 1 sec")
	}
}

var _ = withProtocol(&fakeStream{})

type fakeStream struct {
	protocol protocol.ID
}

func (fs *fakeStream) Protocol() protocol.ID {
	return fs.protocol
}

func TestValidateVersion(t *testing.T) {
	tests := []struct {
		name     string
		version  string
		protocol string
		error    string
		wantErr  bool
	}{
		{
			name:     "bad topic",
			version:  p2p.SchemaVersionV1,
			protocol: "random",
			error:    "unable to find a valid protocol prefix",
			wantErr:  true,
		},
		{
			name:     "valid topic with incorrect version",
			version:  p2p.SchemaVersionV1,
			protocol: p2p.RPCBlocksByRootTopicV2,
			error:    "doesn't match provided version",
			wantErr:  true,
		},
		{
			name:     "valid topic with correct version",
			version:  p2p.SchemaVersionV2,
			protocol: p2p.RPCBlocksByRootTopicV2,
			error:    "",
			wantErr:  false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			stream := &fakeStream{protocol: protocol.ID(tt.protocol)}
			err := validateVersion(tt.version, stream)
			if (err != nil) != tt.wantErr {
				t.Errorf("validateVersion() error = %v, wantErr %v", err, tt.wantErr)
			}
			if err != nil {
				assert.ErrorContains(t, tt.error, err)
			}
		})
	}
}
