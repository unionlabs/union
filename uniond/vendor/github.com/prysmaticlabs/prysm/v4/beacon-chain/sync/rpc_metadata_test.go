package sync

import (
	"context"
	"reflect"
	"sync"
	"testing"
	"time"

	"github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/protocol"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	db "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	p2ptest "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/wrapper"
	leakybucket "github.com/prysmaticlabs/prysm/v4/container/leaky-bucket"
	"github.com/prysmaticlabs/prysm/v4/encoding/ssz/equality"
	pb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/metadata"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestMetaDataRPCHandler_ReceivesMetadata(t *testing.T) {
	p1 := p2ptest.NewTestP2P(t)
	p2 := p2ptest.NewTestP2P(t)
	p1.Connect(p2)
	assert.Equal(t, 1, len(p1.BHost.Network().Peers()), "Expected peers to be connected")
	bitfield := [8]byte{'A', 'B'}
	p1.LocalMetadata = wrapper.WrappedMetadataV0(&pb.MetaDataV0{
		SeqNumber: 2,
		Attnets:   bitfield[:],
	})

	// Set up a head state in the database with data we expect.
	d := db.SetupDB(t)
	r := &Service{
		cfg: &config{
			beaconDB: d,
			p2p:      p1,
			chain: &mock.ChainService{
				ValidatorsRoot: [32]byte{},
			},
		},
		rateLimiter: newRateLimiter(p1),
	}

	// Setup streams
	pcl := protocol.ID(p2p.RPCMetaDataTopicV1)
	topic := string(pcl)
	r.rateLimiter.limiterMap[topic] = leakybucket.NewCollector(1, 1, time.Second, false)
	var wg sync.WaitGroup
	wg.Add(1)
	p2.BHost.SetStreamHandler(pcl, func(stream network.Stream) {
		defer wg.Done()
		expectSuccess(t, stream)
		out := new(pb.MetaDataV0)
		assert.NoError(t, r.cfg.p2p.Encoding().DecodeWithMaxLength(stream, out))
		assert.DeepEqual(t, p1.LocalMetadata.InnerObject(), out, "MetadataV0 unequal")
	})
	stream1, err := p1.BHost.NewStream(context.Background(), p2.BHost.ID(), pcl)
	require.NoError(t, err)

	assert.NoError(t, r.metaDataHandler(context.Background(), new(interface{}), stream1))

	if util.WaitTimeout(&wg, 1*time.Second) {
		t.Fatal("Did not receive stream within 1 sec")
	}

	conns := p1.BHost.Network().ConnsToPeer(p2.BHost.ID())
	if len(conns) == 0 {
		t.Error("Peer is disconnected despite receiving a valid ping")
	}
}

func TestMetadataRPCHandler_SendsMetadata(t *testing.T) {
	p1 := p2ptest.NewTestP2P(t)
	p2 := p2ptest.NewTestP2P(t)
	p1.Connect(p2)
	assert.Equal(t, 1, len(p1.BHost.Network().Peers()), "Expected peers to be connected")
	bitfield := [8]byte{'A', 'B'}
	p2.LocalMetadata = wrapper.WrappedMetadataV0(&pb.MetaDataV0{
		SeqNumber: 2,
		Attnets:   bitfield[:],
	})

	// Set up a head state in the database with data we expect.
	chain := &mock.ChainService{Genesis: time.Now(), ValidatorsRoot: [32]byte{}}
	d := db.SetupDB(t)
	r := &Service{
		cfg: &config{
			beaconDB: d,
			p2p:      p1,
			chain:    chain,
			clock:    startup.NewClock(chain.Genesis, chain.ValidatorsRoot),
		},
		rateLimiter: newRateLimiter(p1),
	}

	r2 := &Service{
		cfg: &config{
			beaconDB: d,
			p2p:      p2,
			chain:    &mock.ChainService{Genesis: time.Now(), ValidatorsRoot: [32]byte{}},
		},
		rateLimiter: newRateLimiter(p2),
	}

	// Setup streams
	pcl := protocol.ID(p2p.RPCMetaDataTopicV1 + r.cfg.p2p.Encoding().ProtocolSuffix())
	topic := string(pcl)
	r.rateLimiter.limiterMap[topic] = leakybucket.NewCollector(1, 1, time.Second, false)
	r2.rateLimiter.limiterMap[topic] = leakybucket.NewCollector(1, 1, time.Second, false)

	var wg sync.WaitGroup
	wg.Add(1)
	p2.BHost.SetStreamHandler(pcl, func(stream network.Stream) {
		defer wg.Done()
		assert.NoError(t, r2.metaDataHandler(context.Background(), new(interface{}), stream))
	})

	md, err := r.sendMetaDataRequest(context.Background(), p2.BHost.ID())
	assert.NoError(t, err)

	if !equality.DeepEqual(md.InnerObject(), p2.LocalMetadata.InnerObject()) {
		t.Fatalf("MetadataV0 unequal, received %v but wanted %v", md, p2.LocalMetadata)
	}

	if util.WaitTimeout(&wg, 1*time.Second) {
		t.Fatal("Did not receive stream within 1 sec")
	}

	conns := p1.BHost.Network().ConnsToPeer(p2.BHost.ID())
	if len(conns) == 0 {
		t.Error("Peer is disconnected despite receiving a valid ping")
	}
}

func TestMetadataRPCHandler_SendsMetadataAltair(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	bCfg := params.BeaconConfig().Copy()
	bCfg.AltairForkEpoch = 5
	params.OverrideBeaconConfig(bCfg)
	params.BeaconConfig().InitializeForkSchedule()

	p1 := p2ptest.NewTestP2P(t)
	p2 := p2ptest.NewTestP2P(t)
	p1.Connect(p2)
	assert.Equal(t, 1, len(p1.BHost.Network().Peers()), "Expected peers to be connected")
	bitfield := [8]byte{'A', 'B'}
	p2.LocalMetadata = wrapper.WrappedMetadataV0(&pb.MetaDataV0{
		SeqNumber: 2,
		Attnets:   bitfield[:],
	})

	// Set up a head state in the database with data we expect.
	d := db.SetupDB(t)
	chain := &mock.ChainService{Genesis: time.Now().Add(-5 * oneEpoch()), ValidatorsRoot: [32]byte{}}
	r := &Service{
		cfg: &config{
			beaconDB: d,
			p2p:      p1,
			chain:    chain,
			clock:    startup.NewClock(chain.Genesis, chain.ValidatorsRoot),
		},
		rateLimiter: newRateLimiter(p1),
	}

	chain2 := &mock.ChainService{Genesis: time.Now().Add(-5 * oneEpoch()), ValidatorsRoot: [32]byte{}}
	r2 := &Service{
		cfg: &config{
			beaconDB: d,
			p2p:      p2,
			chain:    chain2,
			clock:    startup.NewClock(chain2.Genesis, chain2.ValidatorsRoot),
		},
		rateLimiter: newRateLimiter(p2),
	}

	// Setup streams
	pcl := protocol.ID(p2p.RPCMetaDataTopicV2 + r.cfg.p2p.Encoding().ProtocolSuffix())
	topic := string(pcl)
	r.rateLimiter.limiterMap[topic] = leakybucket.NewCollector(2, 2, time.Second, false)
	r2.rateLimiter.limiterMap[topic] = leakybucket.NewCollector(2, 2, time.Second, false)

	var wg sync.WaitGroup
	wg.Add(1)
	p2.BHost.SetStreamHandler(pcl, func(stream network.Stream) {
		defer wg.Done()
		err := r2.metaDataHandler(context.Background(), new(interface{}), stream)
		assert.NoError(t, err)
	})

	_, err := r.sendMetaDataRequest(context.Background(), p2.BHost.ID())
	assert.NoError(t, err)

	if util.WaitTimeout(&wg, 1*time.Second) {
		t.Fatal("Did not receive stream within 1 sec")
	}

	// Fix up peer with the correct metadata.
	p2.LocalMetadata = wrapper.WrappedMetadataV1(&pb.MetaDataV1{
		SeqNumber: 2,
		Attnets:   bitfield[:],
		Syncnets:  []byte{0x0},
	})

	wg.Add(1)
	p2.BHost.SetStreamHandler(pcl, func(stream network.Stream) {
		defer wg.Done()
		assert.NoError(t, r2.metaDataHandler(context.Background(), new(interface{}), stream))
	})

	md, err := r.sendMetaDataRequest(context.Background(), p2.BHost.ID())
	assert.NoError(t, err)

	if !equality.DeepEqual(md.InnerObject(), p2.LocalMetadata.InnerObject()) {
		t.Fatalf("MetadataV1 unequal, received %v but wanted %v", md, p2.LocalMetadata)
	}

	if util.WaitTimeout(&wg, 1*time.Second) {
		t.Fatal("Did not receive stream within 1 sec")
	}

	conns := p1.BHost.Network().ConnsToPeer(p2.BHost.ID())
	if len(conns) == 0 {
		t.Error("Peer is disconnected despite receiving a valid ping")
	}
}

func TestExtractMetaDataType(t *testing.T) {
	// Precompute digests
	genDigest, err := signing.ComputeForkDigest(params.BeaconConfig().GenesisForkVersion, params.BeaconConfig().ZeroHash[:])
	require.NoError(t, err)
	altairDigest, err := signing.ComputeForkDigest(params.BeaconConfig().AltairForkVersion, params.BeaconConfig().ZeroHash[:])
	require.NoError(t, err)

	type args struct {
		digest []byte
		clock  blockchain.TemporalOracle
	}
	tests := []struct {
		name    string
		args    args
		want    metadata.Metadata
		wantErr bool
	}{
		{
			name: "no digest",
			args: args{
				digest: []byte{},
				clock:  startup.NewClock(time.Now(), [32]byte{}),
			},
			want:    wrapper.WrappedMetadataV0(&pb.MetaDataV0{}),
			wantErr: false,
		},
		{
			name: "invalid digest",
			args: args{
				digest: []byte{0x00, 0x01},
				clock:  startup.NewClock(time.Now(), [32]byte{}),
			},
			want:    nil,
			wantErr: true,
		},
		{
			name: "non existent digest",
			args: args{
				digest: []byte{0x00, 0x01, 0x02, 0x03},
				clock:  startup.NewClock(time.Now(), [32]byte{}),
			},
			want:    nil,
			wantErr: true,
		},
		{
			name: "genesis fork version",
			args: args{
				digest: genDigest[:],
				clock:  startup.NewClock(time.Now(), [32]byte{}),
			},
			want:    wrapper.WrappedMetadataV0(&pb.MetaDataV0{}),
			wantErr: false,
		},
		{
			name: "altair fork version",
			args: args{
				digest: altairDigest[:],
				clock:  startup.NewClock(time.Now(), [32]byte{}),
			},
			want:    wrapper.WrappedMetadataV1(&pb.MetaDataV1{}),
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := extractMetaDataType(tt.args.digest, tt.args.clock)
			if (err != nil) != tt.wantErr {
				t.Errorf("extractMetaDataType() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if !reflect.DeepEqual(got, tt.want) {
				t.Errorf("extractMetaDataType() got = %v, want %v", got, tt.want)
			}
		})
	}
}
