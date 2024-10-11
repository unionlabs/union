package p2p

import (
	"context"
	"fmt"
	"net"
	"reflect"
	"sync"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/p2p/discover"
	pubsub "github.com/libp2p/go-libp2p-pubsub"
	"github.com/libp2p/go-libp2p/core/host"
	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers/scorers"
	p2ptest "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/wrapper"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	testpb "github.com/prysmaticlabs/prysm/v4/proto/testing"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"google.golang.org/protobuf/proto"
)

func TestService_Broadcast(t *testing.T) {
	p1 := p2ptest.NewTestP2P(t)
	p2 := p2ptest.NewTestP2P(t)
	p1.Connect(p2)
	if len(p1.BHost.Network().Peers()) == 0 {
		t.Fatal("No peers")
	}

	p := &Service{
		host:                  p1.BHost,
		pubsub:                p1.PubSub(),
		joinedTopics:          map[string]*pubsub.Topic{},
		cfg:                   &Config{},
		genesisTime:           time.Now(),
		genesisValidatorsRoot: bytesutil.PadTo([]byte{'A'}, 32),
	}

	msg := &ethpb.Fork{
		Epoch:           55,
		CurrentVersion:  []byte("fooo"),
		PreviousVersion: []byte("barr"),
	}

	topic := "/eth2/%x/testing"
	// Set a test gossip mapping for testpb.TestSimpleMessage.
	GossipTypeMapping[reflect.TypeOf(msg)] = topic
	digest, err := p.currentForkDigest()
	require.NoError(t, err)
	topic = fmt.Sprintf(topic, digest)

	// External peer subscribes to the topic.
	topic += p.Encoding().ProtocolSuffix()
	sub, err := p2.SubscribeToTopic(topic)
	require.NoError(t, err)

	time.Sleep(50 * time.Millisecond) // libp2p fails without this delay...

	// Async listen for the pubsub, must be before the broadcast.
	var wg sync.WaitGroup
	wg.Add(1)
	go func(tt *testing.T) {
		defer wg.Done()
		ctx, cancel := context.WithTimeout(context.Background(), 1*time.Second)
		defer cancel()

		incomingMessage, err := sub.Next(ctx)
		require.NoError(t, err)

		result := &ethpb.Fork{}
		require.NoError(t, p.Encoding().DecodeGossip(incomingMessage.Data, result))
		if !proto.Equal(result, msg) {
			tt.Errorf("Did not receive expected message, got %+v, wanted %+v", result, msg)
		}
	}(t)

	// Broadcast to peers and wait.
	require.NoError(t, p.Broadcast(context.Background(), msg))
	if util.WaitTimeout(&wg, 1*time.Second) {
		t.Error("Failed to receive pubsub within 1s")
	}
}

func TestService_Broadcast_ReturnsErr_TopicNotMapped(t *testing.T) {
	p := Service{
		genesisTime:           time.Now(),
		genesisValidatorsRoot: bytesutil.PadTo([]byte{'A'}, 32),
	}
	assert.ErrorContains(t, ErrMessageNotMapped.Error(), p.Broadcast(context.Background(), &testpb.AddressBook{}))
}

func TestService_Attestation_Subnet(t *testing.T) {
	if gtm := GossipTypeMapping[reflect.TypeOf(&ethpb.Attestation{})]; gtm != AttestationSubnetTopicFormat {
		t.Errorf("Constant is out of date. Wanted %s, got %s", AttestationSubnetTopicFormat, gtm)
	}

	tests := []struct {
		att   *ethpb.Attestation
		topic string
	}{
		{
			att: &ethpb.Attestation{
				Data: &ethpb.AttestationData{
					CommitteeIndex: 0,
					Slot:           2,
				},
			},
			topic: "/eth2/00000000/beacon_attestation_2",
		},
		{
			att: &ethpb.Attestation{
				Data: &ethpb.AttestationData{
					CommitteeIndex: 11,
					Slot:           10,
				},
			},
			topic: "/eth2/00000000/beacon_attestation_21",
		},
		{
			att: &ethpb.Attestation{
				Data: &ethpb.AttestationData{
					CommitteeIndex: 55,
					Slot:           529,
				},
			},
			topic: "/eth2/00000000/beacon_attestation_8",
		},
	}
	for _, tt := range tests {
		subnet := helpers.ComputeSubnetFromCommitteeAndSlot(100, tt.att.Data.CommitteeIndex, tt.att.Data.Slot)
		assert.Equal(t, tt.topic, attestationToTopic(subnet, [4]byte{} /* fork digest */), "Wrong topic")
	}
}

func TestService_BroadcastAttestation(t *testing.T) {
	p1 := p2ptest.NewTestP2P(t)
	p2 := p2ptest.NewTestP2P(t)
	p1.Connect(p2)
	if len(p1.BHost.Network().Peers()) == 0 {
		t.Fatal("No peers")
	}

	p := &Service{
		host:                  p1.BHost,
		pubsub:                p1.PubSub(),
		joinedTopics:          map[string]*pubsub.Topic{},
		cfg:                   &Config{},
		genesisTime:           time.Now(),
		genesisValidatorsRoot: bytesutil.PadTo([]byte{'A'}, 32),
		subnetsLock:           make(map[uint64]*sync.RWMutex),
		subnetsLockLock:       sync.Mutex{},
		peers: peers.NewStatus(context.Background(), &peers.StatusConfig{
			ScorerParams: &scorers.Config{},
		}),
	}

	msg := util.HydrateAttestation(&ethpb.Attestation{AggregationBits: bitfield.NewBitlist(7)})
	subnet := uint64(5)

	topic := AttestationSubnetTopicFormat
	GossipTypeMapping[reflect.TypeOf(msg)] = topic
	digest, err := p.currentForkDigest()
	require.NoError(t, err)
	topic = fmt.Sprintf(topic, digest, subnet)

	// External peer subscribes to the topic.
	topic += p.Encoding().ProtocolSuffix()
	sub, err := p2.SubscribeToTopic(topic)
	require.NoError(t, err)

	time.Sleep(50 * time.Millisecond) // libp2p fails without this delay...

	// Async listen for the pubsub, must be before the broadcast.
	var wg sync.WaitGroup
	wg.Add(1)
	go func(tt *testing.T) {
		defer wg.Done()
		ctx, cancel := context.WithTimeout(context.Background(), 1*time.Second)
		defer cancel()

		incomingMessage, err := sub.Next(ctx)
		require.NoError(t, err)

		result := &ethpb.Attestation{}
		require.NoError(t, p.Encoding().DecodeGossip(incomingMessage.Data, result))
		if !proto.Equal(result, msg) {
			tt.Errorf("Did not receive expected message, got %+v, wanted %+v", result, msg)
		}
	}(t)

	// Attempt to broadcast nil object should fail.
	ctx := context.Background()
	require.ErrorContains(t, "attempted to broadcast nil", p.BroadcastAttestation(ctx, subnet, nil))

	// Broadcast to peers and wait.
	require.NoError(t, p.BroadcastAttestation(ctx, subnet, msg))
	if util.WaitTimeout(&wg, 1*time.Second) {
		t.Error("Failed to receive pubsub within 1s")
	}
}

func TestService_BroadcastAttestationWithDiscoveryAttempts(t *testing.T) {
	// Setup bootnode.
	cfg := &Config{}
	port := 2000
	cfg.UDPPort = uint(port)
	_, pkey := createAddrAndPrivKey(t)
	ipAddr := net.ParseIP("127.0.0.1")
	genesisTime := time.Now()
	genesisValidatorsRoot := make([]byte, 32)
	s := &Service{
		cfg:                   cfg,
		genesisTime:           genesisTime,
		genesisValidatorsRoot: genesisValidatorsRoot,
	}
	bootListener, err := s.createListener(ipAddr, pkey)
	require.NoError(t, err)
	defer bootListener.Close()

	// Use shorter period for testing.
	currentPeriod := pollingPeriod
	pollingPeriod = 1 * time.Second
	defer func() {
		pollingPeriod = currentPeriod
	}()

	bootNode := bootListener.Self()
	subnet := uint64(5)

	var listeners []*discover.UDPv5
	var hosts []host.Host
	// setup other nodes.
	cfg = &Config{
		BootstrapNodeAddr:   []string{bootNode.String()},
		Discv5BootStrapAddr: []string{bootNode.String()},
		MaxPeers:            30,
	}
	// Setup 2 different hosts
	for i := 1; i <= 2; i++ {
		h, pkey, ipAddr := createHost(t, port+i)
		cfg.UDPPort = uint(port + i)
		cfg.TCPPort = uint(port + i)
		s := &Service{
			cfg:                   cfg,
			genesisTime:           genesisTime,
			genesisValidatorsRoot: genesisValidatorsRoot,
		}
		listener, err := s.startDiscoveryV5(ipAddr, pkey)
		// Set for 2nd peer
		if i == 2 {
			s.dv5Listener = listener
			s.metaData = wrapper.WrappedMetadataV0(new(ethpb.MetaDataV0))
			bitV := bitfield.NewBitvector64()
			bitV.SetBitAt(subnet, true)
			s.updateSubnetRecordWithMetadata(bitV)
		}
		assert.NoError(t, err, "Could not start discovery for node")
		listeners = append(listeners, listener)
		hosts = append(hosts, h)
	}
	defer func() {
		// Close down all peers.
		for _, listener := range listeners {
			listener.Close()
		}
	}()

	// close peers upon exit of test
	defer func() {
		for _, h := range hosts {
			if err := h.Close(); err != nil {
				t.Log(err)
			}
		}
	}()

	ps1, err := pubsub.NewGossipSub(context.Background(), hosts[0],
		pubsub.WithMessageSigning(false),
		pubsub.WithStrictSignatureVerification(false),
	)
	require.NoError(t, err)

	ps2, err := pubsub.NewGossipSub(context.Background(), hosts[1],
		pubsub.WithMessageSigning(false),
		pubsub.WithStrictSignatureVerification(false),
	)
	require.NoError(t, err)
	p := &Service{
		host:                  hosts[0],
		ctx:                   context.Background(),
		pubsub:                ps1,
		dv5Listener:           listeners[0],
		joinedTopics:          map[string]*pubsub.Topic{},
		cfg:                   cfg,
		genesisTime:           time.Now(),
		genesisValidatorsRoot: bytesutil.PadTo([]byte{'A'}, 32),
		subnetsLock:           make(map[uint64]*sync.RWMutex),
		subnetsLockLock:       sync.Mutex{},
		peers: peers.NewStatus(context.Background(), &peers.StatusConfig{
			ScorerParams: &scorers.Config{},
		}),
	}

	p2 := &Service{
		host:                  hosts[1],
		ctx:                   context.Background(),
		pubsub:                ps2,
		dv5Listener:           listeners[1],
		joinedTopics:          map[string]*pubsub.Topic{},
		cfg:                   cfg,
		genesisTime:           time.Now(),
		genesisValidatorsRoot: bytesutil.PadTo([]byte{'A'}, 32),
		subnetsLock:           make(map[uint64]*sync.RWMutex),
		subnetsLockLock:       sync.Mutex{},
		peers: peers.NewStatus(context.Background(), &peers.StatusConfig{
			ScorerParams: &scorers.Config{},
		}),
	}
	go p.listenForNewNodes()
	go p2.listenForNewNodes()

	msg := util.HydrateAttestation(&ethpb.Attestation{AggregationBits: bitfield.NewBitlist(7)})
	topic := AttestationSubnetTopicFormat
	GossipTypeMapping[reflect.TypeOf(msg)] = topic
	digest, err := p.currentForkDigest()
	require.NoError(t, err)
	topic = fmt.Sprintf(topic, digest, subnet)

	// External peer subscribes to the topic.
	topic += p.Encoding().ProtocolSuffix()
	// We don't use our internal subscribe method
	// due to using floodsub over here.
	tpHandle, err := p2.JoinTopic(topic)
	require.NoError(t, err)
	sub, err := tpHandle.Subscribe()
	require.NoError(t, err)

	tpHandle, err = p.JoinTopic(topic)
	require.NoError(t, err)
	_, err = tpHandle.Subscribe()
	require.NoError(t, err)

	time.Sleep(500 * time.Millisecond) // libp2p fails without this delay...

	nodePeers := p.pubsub.ListPeers(topic)
	nodePeers2 := p2.pubsub.ListPeers(topic)

	assert.Equal(t, 1, len(nodePeers))
	assert.Equal(t, 1, len(nodePeers2))

	// Async listen for the pubsub, must be before the broadcast.
	var wg sync.WaitGroup
	wg.Add(1)
	go func(tt *testing.T) {
		defer wg.Done()
		ctx, cancel := context.WithTimeout(context.Background(), 4*time.Second)
		defer cancel()

		incomingMessage, err := sub.Next(ctx)
		require.NoError(t, err)

		result := &ethpb.Attestation{}
		require.NoError(t, p.Encoding().DecodeGossip(incomingMessage.Data, result))
		if !proto.Equal(result, msg) {
			tt.Errorf("Did not receive expected message, got %+v, wanted %+v", result, msg)
		}
	}(t)

	// Broadcast to peers and wait.
	require.NoError(t, p.BroadcastAttestation(context.Background(), subnet, msg))
	if util.WaitTimeout(&wg, 4*time.Second) {
		t.Error("Failed to receive pubsub within 4s")
	}
}

func TestService_BroadcastSyncCommittee(t *testing.T) {
	p1 := p2ptest.NewTestP2P(t)
	p2 := p2ptest.NewTestP2P(t)
	p1.Connect(p2)
	if len(p1.BHost.Network().Peers()) == 0 {
		t.Fatal("No peers")
	}

	p := &Service{
		host:                  p1.BHost,
		pubsub:                p1.PubSub(),
		joinedTopics:          map[string]*pubsub.Topic{},
		cfg:                   &Config{},
		genesisTime:           time.Now(),
		genesisValidatorsRoot: bytesutil.PadTo([]byte{'A'}, 32),
		subnetsLock:           make(map[uint64]*sync.RWMutex),
		subnetsLockLock:       sync.Mutex{},
		peers: peers.NewStatus(context.Background(), &peers.StatusConfig{
			ScorerParams: &scorers.Config{},
		}),
	}

	msg := util.HydrateSyncCommittee(&ethpb.SyncCommitteeMessage{})
	subnet := uint64(5)

	topic := SyncCommitteeSubnetTopicFormat
	GossipTypeMapping[reflect.TypeOf(msg)] = topic
	digest, err := p.currentForkDigest()
	require.NoError(t, err)
	topic = fmt.Sprintf(topic, digest, subnet)

	// External peer subscribes to the topic.
	topic += p.Encoding().ProtocolSuffix()
	sub, err := p2.SubscribeToTopic(topic)
	require.NoError(t, err)

	time.Sleep(50 * time.Millisecond) // libp2p fails without this delay...

	// Async listen for the pubsub, must be before the broadcast.
	var wg sync.WaitGroup
	wg.Add(1)
	go func(tt *testing.T) {
		defer wg.Done()
		ctx, cancel := context.WithTimeout(context.Background(), 1*time.Second)
		defer cancel()

		incomingMessage, err := sub.Next(ctx)
		require.NoError(t, err)

		result := &ethpb.SyncCommitteeMessage{}
		require.NoError(t, p.Encoding().DecodeGossip(incomingMessage.Data, result))
		if !proto.Equal(result, msg) {
			tt.Errorf("Did not receive expected message, got %+v, wanted %+v", result, msg)
		}
	}(t)

	// Broadcasting nil should fail.
	ctx := context.Background()
	require.ErrorContains(t, "attempted to broadcast nil", p.BroadcastSyncCommitteeMessage(ctx, subnet, nil))

	// Broadcast to peers and wait.
	require.NoError(t, p.BroadcastSyncCommitteeMessage(ctx, subnet, msg))
	if util.WaitTimeout(&wg, 1*time.Second) {
		t.Error("Failed to receive pubsub within 1s")
	}
}

func TestService_BroadcastBlob(t *testing.T) {
	p1 := p2ptest.NewTestP2P(t)
	p2 := p2ptest.NewTestP2P(t)
	p1.Connect(p2)
	require.NotEqual(t, 0, len(p1.BHost.Network().Peers()), "No peers")

	p := &Service{
		host:                  p1.BHost,
		pubsub:                p1.PubSub(),
		joinedTopics:          map[string]*pubsub.Topic{},
		cfg:                   &Config{},
		genesisTime:           time.Now(),
		genesisValidatorsRoot: bytesutil.PadTo([]byte{'A'}, 32),
		subnetsLock:           make(map[uint64]*sync.RWMutex),
		subnetsLockLock:       sync.Mutex{},
		peers: peers.NewStatus(context.Background(), &peers.StatusConfig{
			ScorerParams: &scorers.Config{},
		}),
	}

	header := util.HydrateSignedBeaconHeader(&ethpb.SignedBeaconBlockHeader{})
	commitmentInclusionProof := make([][]byte, 17)
	for i := range commitmentInclusionProof {
		commitmentInclusionProof[i] = bytesutil.PadTo([]byte{}, 32)
	}
	blobSidecar := &ethpb.BlobSidecar{
		Index:                    1,
		Blob:                     bytesutil.PadTo([]byte{'C'}, fieldparams.BlobLength),
		KzgCommitment:            bytesutil.PadTo([]byte{'D'}, fieldparams.BLSPubkeyLength),
		KzgProof:                 bytesutil.PadTo([]byte{'E'}, fieldparams.BLSPubkeyLength),
		SignedBlockHeader:        header,
		CommitmentInclusionProof: commitmentInclusionProof,
	}
	subnet := uint64(0)

	topic := BlobSubnetTopicFormat
	GossipTypeMapping[reflect.TypeOf(blobSidecar)] = topic
	digest, err := p.currentForkDigest()
	require.NoError(t, err)
	topic = fmt.Sprintf(topic, digest, subnet)

	// External peer subscribes to the topic.
	topic += p.Encoding().ProtocolSuffix()
	sub, err := p2.SubscribeToTopic(topic)
	require.NoError(t, err)

	time.Sleep(50 * time.Millisecond) // libp2p fails without this delay...

	// Async listen for the pubsub, must be before the broadcast.
	var wg sync.WaitGroup
	wg.Add(1)
	go func(tt *testing.T) {
		defer wg.Done()
		ctx, cancel := context.WithTimeout(context.Background(), 1*time.Second)
		defer cancel()

		incomingMessage, err := sub.Next(ctx)
		require.NoError(t, err)

		result := &ethpb.BlobSidecar{}
		require.NoError(t, p.Encoding().DecodeGossip(incomingMessage.Data, result))
		require.DeepEqual(t, result, blobSidecar)
	}(t)

	// Attempt to broadcast nil object should fail.
	ctx := context.Background()
	require.ErrorContains(t, "attempted to broadcast nil", p.BroadcastBlob(ctx, subnet, nil))

	// Broadcast to peers and wait.
	require.NoError(t, p.BroadcastBlob(ctx, subnet, blobSidecar))
	require.Equal(t, false, util.WaitTimeout(&wg, 1*time.Second), "Failed to receive pubsub within 1s")
}
