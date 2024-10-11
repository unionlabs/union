package p2p

import (
	"context"
	"crypto/ecdsa"
	"fmt"
	"net"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/p2p/discover"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/libp2p/go-libp2p"
	"github.com/libp2p/go-libp2p/core/host"
	"github.com/libp2p/go-libp2p/core/peer"
	noise "github.com/libp2p/go-libp2p/p2p/security/noise"
	"github.com/multiformats/go-multiaddr"
	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/encoder"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers/scorers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/network/forks"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	prysmTime "github.com/prysmaticlabs/prysm/v4/time"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

type mockListener struct {
	localNode *enode.LocalNode
}

func (m mockListener) Self() *enode.Node {
	return m.localNode.Node()
}

func (mockListener) Close() {
	// no-op
}

func (mockListener) Lookup(enode.ID) []*enode.Node {
	panic("implement me")
}

func (mockListener) ReadRandomNodes(_ []*enode.Node) int {
	panic("implement me")
}

func (mockListener) Resolve(*enode.Node) *enode.Node {
	panic("implement me")
}

func (mockListener) Ping(*enode.Node) error {
	panic("implement me")
}

func (mockListener) RequestENR(*enode.Node) (*enode.Node, error) {
	panic("implement me")
}

func (mockListener) LocalNode() *enode.LocalNode {
	panic("implement me")
}

func (mockListener) RandomNodes() enode.Iterator {
	panic("implement me")
}

func createHost(t *testing.T, port int) (host.Host, *ecdsa.PrivateKey, net.IP) {
	_, pkey := createAddrAndPrivKey(t)
	ipAddr := net.ParseIP("127.0.0.1")
	listen, err := multiaddr.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", ipAddr, port))
	require.NoError(t, err, "Failed to p2p listen")
	h, err := libp2p.New([]libp2p.Option{privKeyOption(pkey), libp2p.ListenAddrs(listen), libp2p.Security(noise.ID, noise.New)}...)
	require.NoError(t, err)
	return h, pkey, ipAddr
}

func TestService_Stop_SetsStartedToFalse(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	s, err := NewService(context.Background(), &Config{StateNotifier: &mock.MockStateNotifier{}})
	require.NoError(t, err)
	s.started = true
	s.dv5Listener = &mockListener{}
	assert.NoError(t, s.Stop())
	assert.Equal(t, false, s.started)
}

func TestService_Stop_DontPanicIfDv5ListenerIsNotInited(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	s, err := NewService(context.Background(), &Config{StateNotifier: &mock.MockStateNotifier{}})
	require.NoError(t, err)
	assert.NoError(t, s.Stop())
}

func TestService_Start_OnlyStartsOnce(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	hook := logTest.NewGlobal()

	cs := startup.NewClockSynchronizer()
	cfg := &Config{
		TCPPort:     2000,
		UDPPort:     2000,
		ClockWaiter: cs,
	}
	s, err := NewService(context.Background(), cfg)
	require.NoError(t, err)
	s.dv5Listener = &mockListener{}
	exitRoutine := make(chan bool)
	go func() {
		s.Start()
		<-exitRoutine
	}()
	var vr [32]byte
	require.NoError(t, cs.SetClock(startup.NewClock(time.Now(), vr)))
	time.Sleep(time.Second * 2)
	assert.Equal(t, true, s.started, "Expected service to be started")
	s.Start()
	require.LogsContain(t, hook, "Attempted to start p2p service when it was already started")
	require.NoError(t, s.Stop())
	exitRoutine <- true
}

func TestService_Status_NotRunning(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	s := &Service{started: false}
	s.dv5Listener = &mockListener{}
	assert.ErrorContains(t, "not running", s.Status(), "Status returned wrong error")
}

func TestService_Status_NoGenesisTimeSet(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	s := &Service{started: true}
	s.dv5Listener = &mockListener{}
	assert.ErrorContains(t, "no genesis time set", s.Status(), "Status returned wrong error")

	s.genesisTime = time.Now()

	assert.NoError(t, s.Status(), "Status returned error")
}

func TestService_Start_NoDiscoverFlag(t *testing.T) {
	params.SetupTestConfigCleanup(t)

	cs := startup.NewClockSynchronizer()
	cfg := &Config{
		TCPPort:       2000,
		UDPPort:       2000,
		StateNotifier: &mock.MockStateNotifier{},
		NoDiscovery:   true, // <-- no s.dv5Listener is created
		ClockWaiter:   cs,
	}
	s, err := NewService(context.Background(), cfg)
	require.NoError(t, err)

	// required params to addForkEntry in s.forkWatcher
	s.genesisTime = time.Now()
	beaconCfg := params.BeaconConfig().Copy()
	beaconCfg.AltairForkEpoch = 0
	beaconCfg.BellatrixForkEpoch = 0
	beaconCfg.CapellaForkEpoch = 0
	beaconCfg.SecondsPerSlot = 1
	params.OverrideBeaconConfig(beaconCfg)

	exitRoutine := make(chan bool)
	go func() {
		s.Start()
		<-exitRoutine
	}()

	var vr [32]byte
	require.NoError(t, cs.SetClock(startup.NewClock(time.Now(), vr)))

	time.Sleep(time.Second * 2)

	exitRoutine <- true
}

func TestListenForNewNodes(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	// Setup bootnode.
	notifier := &mock.MockStateNotifier{}
	cfg := &Config{StateNotifier: notifier}
	port := 2000
	cfg.UDPPort = uint(port)
	_, pkey := createAddrAndPrivKey(t)
	ipAddr := net.ParseIP("127.0.0.1")
	genesisTime := prysmTime.Now()
	var gvr [32]byte
	s := &Service{
		cfg:                   cfg,
		genesisTime:           genesisTime,
		genesisValidatorsRoot: gvr[:],
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

	var listeners []*discover.UDPv5
	var hosts []host.Host
	// setup other nodes.
	cs := startup.NewClockSynchronizer()
	cfg = &Config{
		BootstrapNodeAddr:   []string{bootNode.String()},
		Discv5BootStrapAddr: []string{bootNode.String()},
		MaxPeers:            30,
		ClockWaiter:         cs,
	}
	for i := 1; i <= 5; i++ {
		h, pkey, ipAddr := createHost(t, port+i)
		cfg.UDPPort = uint(port + i)
		cfg.TCPPort = uint(port + i)
		s := &Service{
			cfg:                   cfg,
			genesisTime:           genesisTime,
			genesisValidatorsRoot: gvr[:],
		}
		listener, err := s.startDiscoveryV5(ipAddr, pkey)
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

	cfg.UDPPort = 14000
	cfg.TCPPort = 14001

	s, err = NewService(context.Background(), cfg)
	require.NoError(t, err)
	exitRoutine := make(chan bool)
	go func() {
		s.Start()
		<-exitRoutine
	}()
	time.Sleep(1 * time.Second)

	require.NoError(t, cs.SetClock(startup.NewClock(genesisTime, gvr)))

	time.Sleep(4 * time.Second)
	assert.Equal(t, 5, len(s.host.Network().Peers()), "Not all peers added to peerstore")
	require.NoError(t, s.Stop())
	exitRoutine <- true
}

func TestPeer_Disconnect(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	h1, _, _ := createHost(t, 5000)
	defer func() {
		if err := h1.Close(); err != nil {
			t.Log(err)
		}
	}()

	s := &Service{
		host: h1,
	}

	h2, _, ipaddr := createHost(t, 5001)
	defer func() {
		if err := h2.Close(); err != nil {
			t.Log(err)
		}
	}()

	h2Addr, err := multiaddr.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d/p2p/%s", ipaddr, 5001, h2.ID()))
	require.NoError(t, err)
	addrInfo, err := peer.AddrInfoFromP2pAddr(h2Addr)
	require.NoError(t, err)
	require.NoError(t, s.host.Connect(context.Background(), *addrInfo))
	assert.Equal(t, 1, len(s.host.Network().Peers()), "Invalid number of peers")
	assert.Equal(t, 1, len(s.host.Network().Conns()), "Invalid number of connections")
	require.NoError(t, s.Disconnect(h2.ID()))
	assert.Equal(t, 0, len(s.host.Network().Conns()), "Invalid number of connections")
}

func TestService_JoinLeaveTopic(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()
	gs := startup.NewClockSynchronizer()
	s, err := NewService(ctx, &Config{StateNotifier: &mock.MockStateNotifier{}, ClockWaiter: gs})
	require.NoError(t, err)

	go s.awaitStateInitialized()
	fd := initializeStateWithForkDigest(ctx, t, gs)

	assert.Equal(t, 0, len(s.joinedTopics))

	topic := fmt.Sprintf(AttestationSubnetTopicFormat, fd, 42) + "/" + encoder.ProtocolSuffixSSZSnappy
	topicHandle, err := s.JoinTopic(topic)
	assert.NoError(t, err)
	assert.Equal(t, 1, len(s.joinedTopics))

	if topicHandle == nil {
		t.Fatal("topic is nil")
	}

	sub, err := topicHandle.Subscribe()
	assert.NoError(t, err)

	// Try leaving topic that has subscriptions.
	want := "cannot close topic: outstanding event handlers or subscriptions"
	assert.ErrorContains(t, want, s.LeaveTopic(topic))

	// After subscription is cancelled, leaving topic should not result in error.
	sub.Cancel()
	assert.NoError(t, s.LeaveTopic(topic))
}

// initializeStateWithForkDigest sets up the state feed initialized event and returns the fork
// digest associated with that genesis event.
func initializeStateWithForkDigest(_ context.Context, t *testing.T, gs startup.ClockSetter) [4]byte {
	gt := prysmTime.Now()
	gvr := bytesutil.ToBytes32(bytesutil.PadTo([]byte("genesis validators root"), 32))
	require.NoError(t, gs.SetClock(startup.NewClock(gt, gvr)))

	fd, err := forks.CreateForkDigest(gt, gvr[:])
	require.NoError(t, err)

	time.Sleep(50 * time.Millisecond) // wait for pubsub filter to initialize.

	return fd
}

func TestService_connectWithPeer(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	tests := []struct {
		name    string
		peers   *peers.Status
		info    peer.AddrInfo
		wantErr string
	}{
		{
			name: "bad peer",
			peers: func() *peers.Status {
				ps := peers.NewStatus(context.Background(), &peers.StatusConfig{
					ScorerParams: &scorers.Config{},
				})
				for i := 0; i < 10; i++ {
					ps.Scorers().BadResponsesScorer().Increment("bad")
				}
				return ps
			}(),
			info:    peer.AddrInfo{ID: "bad"},
			wantErr: "refused to connect to bad peer",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			h, _, _ := createHost(t, 34567)
			defer func() {
				if err := h.Close(); err != nil {
					t.Fatal(err)
				}
			}()
			ctx := context.Background()
			s := &Service{
				host:  h,
				peers: tt.peers,
			}
			err := s.connectWithPeer(ctx, tt.info)
			if len(tt.wantErr) > 0 {
				require.ErrorContains(t, tt.wantErr, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}
