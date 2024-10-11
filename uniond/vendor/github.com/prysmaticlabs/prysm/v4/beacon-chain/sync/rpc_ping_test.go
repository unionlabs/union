package sync

import (
	"context"
	"sync"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/p2p/enr"
	"github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/protocol"
	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	db "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	p2ptest "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	p2ptypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/wrapper"
	leakybucket "github.com/prysmaticlabs/prysm/v4/container/leaky-bucket"
	pb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestPingRPCHandler_ReceivesPing(t *testing.T) {
	p1 := p2ptest.NewTestP2P(t)
	p2 := p2ptest.NewTestP2P(t)
	p1.Connect(p2)
	assert.Equal(t, 1, len(p1.BHost.Network().Peers()), "Expected peers to be connected")
	p1.LocalMetadata = wrapper.WrappedMetadataV0(&pb.MetaDataV0{
		SeqNumber: 2,
		Attnets:   []byte{'A', 'B'},
	})

	p2.LocalMetadata = wrapper.WrappedMetadataV0(&pb.MetaDataV0{
		SeqNumber: 2,
		Attnets:   []byte{'C', 'D'},
	})

	// Set up a head state in the database with data we expect.
	d := db.SetupDB(t)
	r := &Service{
		cfg: &config{
			beaconDB: d,
			p2p:      p1,
		},
		rateLimiter: newRateLimiter(p1),
	}

	p1.Peers().Add(new(enr.Record), p2.BHost.ID(), p2.BHost.Addrs()[0], network.DirUnknown)
	p1.Peers().SetMetadata(p2.BHost.ID(), p2.LocalMetadata)

	// Setup streams
	pcl := protocol.ID(p2p.RPCPingTopicV1)
	topic := string(pcl)
	r.rateLimiter.limiterMap[topic] = leakybucket.NewCollector(1, 1, time.Second, false)
	var wg sync.WaitGroup
	wg.Add(1)
	p2.BHost.SetStreamHandler(pcl, func(stream network.Stream) {
		defer wg.Done()
		expectSuccess(t, stream)
		out := new(primitives.SSZUint64)
		assert.NoError(t, r.cfg.p2p.Encoding().DecodeWithMaxLength(stream, out))
		assert.Equal(t, uint64(2), uint64(*out))
	})
	stream1, err := p1.BHost.NewStream(context.Background(), p2.BHost.ID(), pcl)
	require.NoError(t, err)
	seqNumber := primitives.SSZUint64(2)

	assert.NoError(t, r.pingHandler(context.Background(), &seqNumber, stream1))

	if util.WaitTimeout(&wg, 1*time.Second) {
		t.Fatal("Did not receive stream within 1 sec")
	}

	conns := p1.BHost.Network().ConnsToPeer(p2.BHost.ID())
	if len(conns) == 0 {
		t.Error("Peer is disconnected despite receiving a valid ping")
	}
}

func TestPingRPCHandler_SendsPing(t *testing.T) {
	p1 := p2ptest.NewTestP2P(t)
	p2 := p2ptest.NewTestP2P(t)
	p1.Connect(p2)
	assert.Equal(t, 1, len(p1.BHost.Network().Peers()), "Expected peers to be connected")
	p1.LocalMetadata = wrapper.WrappedMetadataV0(&pb.MetaDataV0{
		SeqNumber: 2,
		Attnets:   []byte{'A', 'B'},
	})

	p2.LocalMetadata = wrapper.WrappedMetadataV0(&pb.MetaDataV0{
		SeqNumber: 2,
		Attnets:   []byte{'C', 'D'},
	})

	// Set up a head state in the database with data we expect.
	d := db.SetupDB(t)
	chain := &mock.ChainService{ValidatorsRoot: [32]byte{}, Genesis: time.Now()}
	r := &Service{
		cfg: &config{
			beaconDB: d,
			p2p:      p1,
			chain:    chain,
			clock:    startup.NewClock(chain.Genesis, chain.ValidatorsRoot),
		},
		rateLimiter: newRateLimiter(p1),
	}

	p1.Peers().Add(new(enr.Record), p2.BHost.ID(), p2.BHost.Addrs()[0], network.DirUnknown)
	p1.Peers().SetMetadata(p2.BHost.ID(), p2.LocalMetadata)

	p2.Peers().Add(new(enr.Record), p1.BHost.ID(), p1.BHost.Addrs()[0], network.DirUnknown)
	p2.Peers().SetMetadata(p1.BHost.ID(), p1.LocalMetadata)

	chain2 := &mock.ChainService{ValidatorsRoot: [32]byte{}, Genesis: time.Now()}
	r2 := &Service{
		cfg: &config{
			beaconDB: d,
			p2p:      p2,
			chain:    chain2,
			clock:    startup.NewClock(chain2.Genesis, chain.ValidatorsRoot),
		},
		rateLimiter: newRateLimiter(p2),
	}
	// Setup streams
	pcl := protocol.ID("/eth2/beacon_chain/req/ping/1/ssz_snappy")
	topic := string(pcl)
	r.rateLimiter.limiterMap[topic] = leakybucket.NewCollector(1, 1, time.Second, false)

	var wg sync.WaitGroup
	wg.Add(1)
	p2.BHost.SetStreamHandler(pcl, func(stream network.Stream) {
		defer wg.Done()
		out := new(primitives.SSZUint64)
		assert.NoError(t, r2.cfg.p2p.Encoding().DecodeWithMaxLength(stream, out))
		assert.Equal(t, uint64(2), uint64(*out))
		assert.NoError(t, r2.pingHandler(context.Background(), out, stream))
	})

	assert.NoError(t, r.sendPingRequest(context.Background(), p2.BHost.ID()))

	if util.WaitTimeout(&wg, 1*time.Second) {
		t.Fatal("Did not receive stream within 1 sec")
	}

	conns := p1.BHost.Network().ConnsToPeer(p2.BHost.ID())
	if len(conns) == 0 {
		t.Error("Peer is disconnected despite receiving a valid ping")
	}
}

func TestPingRPCHandler_BadSequenceNumber(t *testing.T) {
	p1 := p2ptest.NewTestP2P(t)
	p2 := p2ptest.NewTestP2P(t)
	p1.Connect(p2)
	assert.Equal(t, 1, len(p1.BHost.Network().Peers()), "Expected peers to be connected")
	p1.LocalMetadata = wrapper.WrappedMetadataV0(&pb.MetaDataV0{
		SeqNumber: 2,
		Attnets:   []byte{'A', 'B'},
	})

	p2.LocalMetadata = wrapper.WrappedMetadataV0(&pb.MetaDataV0{
		SeqNumber: 2,
		Attnets:   []byte{'C', 'D'},
	})

	// Set up a head state in the database with data we expect.
	d := db.SetupDB(t)
	r := &Service{
		cfg: &config{
			beaconDB: d,
			p2p:      p1,
		},
		rateLimiter: newRateLimiter(p1),
	}

	badMetadata := &pb.MetaDataV0{
		SeqNumber: 3,
		Attnets:   []byte{'E', 'F'},
	}

	p1.Peers().Add(new(enr.Record), p2.BHost.ID(), p2.BHost.Addrs()[0], network.DirUnknown)
	p1.Peers().SetMetadata(p2.BHost.ID(), wrapper.WrappedMetadataV0(badMetadata))

	// Setup streams
	pcl := protocol.ID("/testing")
	topic := string(pcl)
	r.rateLimiter.limiterMap[topic] = leakybucket.NewCollector(1, 1, time.Second, false)
	var wg sync.WaitGroup
	wg.Add(1)
	p2.BHost.SetStreamHandler(pcl, func(stream network.Stream) {
		defer wg.Done()
		expectFailure(t, responseCodeInvalidRequest, p2ptypes.ErrInvalidSequenceNum.Error(), stream)

	})
	stream1, err := p1.BHost.NewStream(context.Background(), p2.BHost.ID(), pcl)
	require.NoError(t, err)

	wantedSeq := primitives.SSZUint64(p2.LocalMetadata.SequenceNumber())
	err = r.pingHandler(context.Background(), &wantedSeq, stream1)
	assert.ErrorContains(t, p2ptypes.ErrInvalidSequenceNum.Error(), err)

	if util.WaitTimeout(&wg, 1*time.Second) {
		t.Fatal("Did not receive stream within 1 sec")
	}

	res, err := p1.Peers().Scorers().BadResponsesScorer().Count(p2.BHost.ID())
	assert.NoError(t, err)
	assert.Equal(t, 1, res, "Peer wasn't penalised for providing a bad sequence number")
}
