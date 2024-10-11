package p2p

import (
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/libp2p/go-libp2p"
	"github.com/libp2p/go-libp2p/core/peer"
	ma "github.com/multiformats/go-multiaddr"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers/peerdata"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers/scorers"
	mockp2p "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	leakybucket "github.com/prysmaticlabs/prysm/v4/container/leaky-bucket"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/eth/v1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestPeer_AtMaxLimit(t *testing.T) {
	// create host and remote peer
	ipAddr, pkey := createAddrAndPrivKey(t)
	ipAddr2, pkey2 := createAddrAndPrivKey(t)

	listen, err := ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", ipAddr, 2000))
	require.NoError(t, err, "Failed to p2p listen")
	s := &Service{
		ipLimiter: leakybucket.NewCollector(ipLimit, ipBurst, 1*time.Second, false),
	}
	s.peers = peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 0,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: 3,
			},
		},
	})
	s.cfg = &Config{MaxPeers: 0}
	s.addrFilter, err = configureFilter(&Config{})
	require.NoError(t, err)
	s.started = true
	h1, err := libp2p.New([]libp2p.Option{privKeyOption(pkey), libp2p.ListenAddrs(listen), libp2p.ConnectionGater(s)}...)
	require.NoError(t, err)
	s.host = h1
	defer func() {
		err := h1.Close()
		require.NoError(t, err)
	}()

	for i := 0; i < highWatermarkBuffer; i++ {
		addPeer(t, s.peers, peers.PeerConnected)
	}

	// create alternate host
	listen, err = ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", ipAddr2, 3000))
	require.NoError(t, err, "Failed to p2p listen")
	h2, err := libp2p.New([]libp2p.Option{privKeyOption(pkey2), libp2p.ListenAddrs(listen)}...)
	require.NoError(t, err)
	defer func() {
		err := h2.Close()
		require.NoError(t, err)
	}()
	multiAddress, err := ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d/p2p/%s", ipAddr, 2000, h1.ID()))
	require.NoError(t, err)
	addrInfo, err := peer.AddrInfoFromP2pAddr(multiAddress)
	require.NoError(t, err)
	err = h2.Connect(context.Background(), *addrInfo)
	require.NotNil(t, err, "Wanted connection to fail with max peer")
}

func TestService_InterceptBannedIP(t *testing.T) {
	s := &Service{
		ipLimiter: leakybucket.NewCollector(ipLimit, ipBurst, 1*time.Second, false),
		peers: peers.NewStatus(context.Background(), &peers.StatusConfig{
			PeerLimit:    20,
			ScorerParams: &scorers.Config{},
		}),
	}
	var err error
	s.addrFilter, err = configureFilter(&Config{})
	require.NoError(t, err)
	ip := "212.67.10.122"
	multiAddress, err := ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", ip, 3000))
	require.NoError(t, err)
	s.started = true

	for i := 0; i < ipBurst; i++ {
		valid := s.validateDial(multiAddress)
		if !valid {
			t.Errorf("Expected multiaddress with ip %s to not be rejected", ip)
		}
	}
	valid := s.validateDial(multiAddress)
	if valid {
		t.Errorf("Expected multiaddress with ip %s to be rejected as it exceeds the burst limit", ip)
	}
}

func TestService_RejectInboundConnectionBeforeStarted(t *testing.T) {
	limit := 1
	s := &Service{
		ipLimiter: leakybucket.NewCollector(ipLimit, ipBurst, 1*time.Second, false),
		peers: peers.NewStatus(context.Background(), &peers.StatusConfig{
			PeerLimit:    limit,
			ScorerParams: &scorers.Config{},
		}),
		host: mockp2p.NewTestP2P(t).BHost,
		cfg:  &Config{MaxPeers: uint(limit)},
	}
	var err error
	s.addrFilter, err = configureFilter(&Config{})
	require.NoError(t, err)

	ip := "212.67.10.122"
	multiAddress, err := ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", ip, 3000))
	require.NoError(t, err)

	valid := s.InterceptAccept(&maEndpoints{raddr: multiAddress})
	if valid {
		t.Errorf("Expected multiaddress with ip %s to be rejected as p2p service is not ready", ip)
	}

	s.started = true
	valid = s.InterceptAccept(&maEndpoints{raddr: multiAddress})
	if !valid {
		t.Errorf("Expected multiaddress with ip %s to be accepted after service is started", ip)
	}
}

func TestService_RejectInboundPeersBeyondLimit(t *testing.T) {
	limit := 20
	s := &Service{
		ipLimiter: leakybucket.NewCollector(ipLimit, ipBurst, 1*time.Second, false),
		peers: peers.NewStatus(context.Background(), &peers.StatusConfig{
			PeerLimit:    limit,
			ScorerParams: &scorers.Config{},
		}),
		host: mockp2p.NewTestP2P(t).BHost,
		cfg:  &Config{MaxPeers: uint(limit)},
	}
	var err error
	s.addrFilter, err = configureFilter(&Config{})
	require.NoError(t, err)
	ip := "212.67.10.122"
	multiAddress, err := ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", ip, 3000))
	require.NoError(t, err)
	s.started = true

	valid := s.InterceptAccept(&maEndpoints{raddr: multiAddress})
	if !valid {
		t.Errorf("Expected multiaddress with ip %s to be accepted as it is below the inbound limit", ip)
	}

	inboundLimit := float64(limit) * peers.InboundRatio
	inboundLimit += highWatermarkBuffer
	// top off by 1 to trigger it above the limit.
	inboundLimit += 1
	// Add in up to inbound peer limit.
	for i := 0; i < int(inboundLimit); i++ {
		addPeer(t, s.peers, peerdata.PeerConnectionState(ethpb.ConnectionState_CONNECTED))
	}
	valid = s.InterceptAccept(&maEndpoints{raddr: multiAddress})
	if valid {
		t.Errorf("Expected multiaddress with ip %s to be rejected as it exceeds the inbound limit", ip)
	}
}

func TestPeer_BelowMaxLimit(t *testing.T) {
	// create host and remote peer
	ipAddr, pkey := createAddrAndPrivKey(t)
	ipAddr2, pkey2 := createAddrAndPrivKey(t)

	listen, err := ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", ipAddr, 2000))
	require.NoError(t, err, "Failed to p2p listen")
	s := &Service{
		ipLimiter: leakybucket.NewCollector(ipLimit, ipBurst, 1*time.Second, false),
	}
	s.peers = peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 1,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: 3,
			},
		},
	})
	s.cfg = &Config{MaxPeers: 1}
	s.addrFilter, err = configureFilter(&Config{})
	require.NoError(t, err)
	h1, err := libp2p.New([]libp2p.Option{privKeyOption(pkey), libp2p.ListenAddrs(listen), libp2p.ConnectionGater(s)}...)
	require.NoError(t, err)
	s.host = h1
	s.started = true
	defer func() {
		err := h1.Close()
		require.NoError(t, err)
	}()

	// create alternate host
	listen, err = ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", ipAddr2, 3000))
	require.NoError(t, err, "Failed to p2p listen")
	h2, err := libp2p.New([]libp2p.Option{privKeyOption(pkey2), libp2p.ListenAddrs(listen)}...)
	require.NoError(t, err)
	defer func() {
		err := h2.Close()
		require.NoError(t, err)
	}()
	multiAddress, err := ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d/p2p/%s", ipAddr, 2000, h1.ID()))
	require.NoError(t, err)
	addrInfo, err := peer.AddrInfoFromP2pAddr(multiAddress)
	require.NoError(t, err)
	err = h2.Connect(context.Background(), *addrInfo)
	assert.NoError(t, err, "Wanted connection to succeed")
}

func TestPeerAllowList(t *testing.T) {
	// create host with allow list
	ipAddr, pkey := createAddrAndPrivKey(t)
	ipAddr2, pkey2 := createAddrAndPrivKey(t)

	// use unattainable subnet, which will lead to
	// peer rejecting all peers, except for those
	// from that subnet.
	cidr := "202.35.89.12/16"

	listen, err := ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", ipAddr, 2000))
	require.NoError(t, err, "Failed to p2p listen")
	s := &Service{
		ipLimiter: leakybucket.NewCollector(ipLimit, ipBurst, 1*time.Second, false),
		peers: peers.NewStatus(context.Background(), &peers.StatusConfig{
			ScorerParams: &scorers.Config{},
		}),
	}
	s.addrFilter, err = configureFilter(&Config{AllowListCIDR: cidr})
	require.NoError(t, err)
	h1, err := libp2p.New([]libp2p.Option{privKeyOption(pkey), libp2p.ListenAddrs(listen), libp2p.ConnectionGater(s)}...)
	require.NoError(t, err)
	s.host = h1
	s.started = true
	defer func() {
		err := h1.Close()
		require.NoError(t, err)
	}()

	// create alternate host
	listen, err = ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", ipAddr2, 3000))
	require.NoError(t, err, "Failed to p2p listen")
	h2, err := libp2p.New([]libp2p.Option{privKeyOption(pkey2), libp2p.ListenAddrs(listen)}...)
	require.NoError(t, err)
	defer func() {
		err := h2.Close()
		require.NoError(t, err)
	}()
	multiAddress, err := ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d/p2p/%s", ipAddr2, 3000, h2.ID()))
	require.NoError(t, err)
	addrInfo, err := peer.AddrInfoFromP2pAddr(multiAddress)
	require.NoError(t, err)
	err = h1.Connect(context.Background(), *addrInfo)
	assert.NotNil(t, err, "Wanted connection to fail with allow list")
	assert.ErrorContains(t, "no good addresses", err)
}

func TestPeerDenyList(t *testing.T) {
	// create host with deny list
	ipAddr, pkey := createAddrAndPrivKey(t)
	ipAddr2, pkey2 := createAddrAndPrivKey(t)

	mask := ipAddr2.DefaultMask()
	ones, _ := mask.Size()
	maskedIP := ipAddr2.Mask(mask)
	cidr := maskedIP.String() + fmt.Sprintf("/%d", ones)

	listen, err := ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", ipAddr, 2000))
	require.NoError(t, err, "Failed to p2p listen")
	s := &Service{
		ipLimiter: leakybucket.NewCollector(ipLimit, ipBurst, 1*time.Second, false),
		peers: peers.NewStatus(context.Background(), &peers.StatusConfig{
			ScorerParams: &scorers.Config{},
		}),
	}
	s.addrFilter, err = configureFilter(&Config{DenyListCIDR: []string{cidr}})
	require.NoError(t, err)
	h1, err := libp2p.New([]libp2p.Option{privKeyOption(pkey), libp2p.ListenAddrs(listen), libp2p.ConnectionGater(s)}...)
	require.NoError(t, err)
	s.host = h1
	s.started = true
	defer func() {
		err := h1.Close()
		require.NoError(t, err)
	}()

	// create alternate host
	listen, err = ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", ipAddr2, 3000))
	require.NoError(t, err, "Failed to p2p listen")
	h2, err := libp2p.New([]libp2p.Option{privKeyOption(pkey2), libp2p.ListenAddrs(listen)}...)
	require.NoError(t, err)
	defer func() {
		err := h2.Close()
		require.NoError(t, err)
	}()
	multiAddress, err := ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d/p2p/%s", ipAddr2, 3000, h2.ID()))
	require.NoError(t, err)
	addrInfo, err := peer.AddrInfoFromP2pAddr(multiAddress)
	require.NoError(t, err)
	err = h1.Connect(context.Background(), *addrInfo)
	assert.NotNil(t, err, "Wanted connection to fail with deny list")
	assert.ErrorContains(t, "no good addresses", err)
}

func TestService_InterceptAddrDial_Allow(t *testing.T) {
	s := &Service{
		ipLimiter: leakybucket.NewCollector(ipLimit, ipBurst, 1*time.Second, false),
		peers: peers.NewStatus(context.Background(), &peers.StatusConfig{
			ScorerParams: &scorers.Config{},
		}),
	}
	var err error
	cidr := "212.67.89.112/16"
	s.addrFilter, err = configureFilter(&Config{AllowListCIDR: cidr})
	require.NoError(t, err)
	ip := "212.67.10.122"
	multiAddress, err := ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", ip, 3000))
	require.NoError(t, err)
	valid := s.InterceptAddrDial("", multiAddress)
	if !valid {
		t.Errorf("Expected multiaddress with ip %s to not be rejected with an allow cidr mask of %s", ip, cidr)
	}
}

func TestService_InterceptAddrDial_Public(t *testing.T) {
	s := &Service{
		ipLimiter: leakybucket.NewCollector(ipLimit, ipBurst, 1*time.Second, false),
		peers: peers.NewStatus(context.Background(), &peers.StatusConfig{
			ScorerParams: &scorers.Config{},
		}),
	}
	var err error
	//test with public filter
	cidr := "public"
	ip := "212.67.10.122"
	s.addrFilter, err = configureFilter(&Config{AllowListCIDR: cidr})
	require.NoError(t, err)
	multiAddress, err := ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", ip, 3000))
	require.NoError(t, err)
	valid := s.InterceptAddrDial("", multiAddress)
	if !valid {
		t.Errorf("Expected multiaddress with ip %s to not be rejected since we allow public addresses", ip)
	}

	ip = "192.168.1.0" //this is private and should fail
	multiAddress, err = ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", ip, 3000))
	require.NoError(t, err)
	valid = s.InterceptAddrDial("", multiAddress)
	if valid {
		t.Errorf("Expected multiaddress with ip %s to be rejected since we are only allowing public addresses", ip)
	}

	//test with public allow filter, with a public address added to the deny list
	invalidPublicIp := "212.67.10.122"
	validPublicIp := "91.65.69.69"
	s.addrFilter, err = configureFilter(&Config{AllowListCIDR: "public", DenyListCIDR: []string{"212.67.89.112/16"}})
	require.NoError(t, err)
	multiAddress, err = ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", validPublicIp, 3000))
	require.NoError(t, err)
	valid = s.InterceptAddrDial("", multiAddress)
	if !valid {
		t.Errorf("Expected multiaddress with ip %s to not be rejected since it is a public address that is not in the deny list", ip)
	}
	multiAddress, err = ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", invalidPublicIp, 3000))
	require.NoError(t, err)
	valid = s.InterceptAddrDial("", multiAddress)
	if valid {
		t.Errorf("Expected multiaddress with ip %s to be rejected since it is on the deny list", ip)
	}

}

func TestService_InterceptAddrDial_Private(t *testing.T) {
	s := &Service{
		ipLimiter: leakybucket.NewCollector(ipLimit, ipBurst, 1*time.Second, false),
		peers: peers.NewStatus(context.Background(), &peers.StatusConfig{
			ScorerParams: &scorers.Config{},
		}),
	}
	var err error
	//test with private filter
	cidr := "private"
	s.addrFilter, err = configureFilter(&Config{DenyListCIDR: []string{cidr}})
	require.NoError(t, err)
	ip := "212.67.10.122"
	multiAddress, err := ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", ip, 3000))
	require.NoError(t, err)
	valid := s.InterceptAddrDial("", multiAddress)
	if !valid {
		t.Errorf("Expected multiaddress with ip %s to be allowed since we are only denying private addresses", ip)
	}

	ip = "192.168.1.0"
	multiAddress, err = ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", ip, 3000))
	require.NoError(t, err)
	valid = s.InterceptAddrDial("", multiAddress)
	if valid {
		t.Errorf("Expected multiaddress with ip %s to be rejected since we are denying private addresses", ip)
	}
}

func TestService_InterceptAddrDial_AllowPrivate(t *testing.T) {
	s := &Service{
		ipLimiter: leakybucket.NewCollector(ipLimit, ipBurst, 1*time.Second, false),
		peers: peers.NewStatus(context.Background(), &peers.StatusConfig{
			ScorerParams: &scorers.Config{},
		}),
	}
	var err error
	//test with private filter
	cidr := "private"
	s.addrFilter, err = configureFilter(&Config{AllowListCIDR: cidr})
	require.NoError(t, err)
	ip := "212.67.10.122"
	multiAddress, err := ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", ip, 3000))
	require.NoError(t, err)
	valid := s.InterceptAddrDial("", multiAddress)
	if valid {
		t.Errorf("Expected multiaddress with ip %s to be denied since we are only allowing private addresses", ip)
	}

	ip = "192.168.1.0"
	multiAddress, err = ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", ip, 3000))
	require.NoError(t, err)
	valid = s.InterceptAddrDial("", multiAddress)
	if !valid {
		t.Errorf("Expected multiaddress with ip %s to be allowed since we are allowing private addresses", ip)
	}
}

func TestService_InterceptAddrDial_DenyPublic(t *testing.T) {
	s := &Service{
		ipLimiter: leakybucket.NewCollector(ipLimit, ipBurst, 1*time.Second, false),
		peers: peers.NewStatus(context.Background(), &peers.StatusConfig{
			ScorerParams: &scorers.Config{},
		}),
	}
	var err error
	//test with private filter
	cidr := "public"
	s.addrFilter, err = configureFilter(&Config{DenyListCIDR: []string{cidr}})
	require.NoError(t, err)
	ip := "212.67.10.122"
	multiAddress, err := ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", ip, 3000))
	require.NoError(t, err)
	valid := s.InterceptAddrDial("", multiAddress)
	if valid {
		t.Errorf("Expected multiaddress with ip %s to be denied since we are denying public addresses", ip)
	}

	ip = "192.168.1.0"
	multiAddress, err = ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", ip, 3000))
	require.NoError(t, err)
	valid = s.InterceptAddrDial("", multiAddress)
	if !valid {
		t.Errorf("Expected multiaddress with ip %s to be allowed since we are only denying public addresses", ip)
	}
}

func TestService_InterceptAddrDial_AllowConflict(t *testing.T) {
	s := &Service{
		ipLimiter: leakybucket.NewCollector(ipLimit, ipBurst, 1*time.Second, false),
		peers: peers.NewStatus(context.Background(), &peers.StatusConfig{
			ScorerParams: &scorers.Config{},
		}),
	}
	var err error
	//test with private filter
	cidr := "public"
	s.addrFilter, err = configureFilter(&Config{DenyListCIDR: []string{cidr, "192.168.0.0/16"}})
	require.NoError(t, err)
	ip := "212.67.10.122"
	multiAddress, err := ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", ip, 3000))
	require.NoError(t, err)
	valid := s.InterceptAddrDial("", multiAddress)
	if valid {
		t.Errorf("Expected multiaddress with ip %s to be denied since we are denying public addresses", ip)
	}

	ip = "192.168.1.0"
	multiAddress, err = ma.NewMultiaddr(fmt.Sprintf("/ip4/%s/tcp/%d", ip, 3000))
	require.NoError(t, err)
	valid = s.InterceptAddrDial("", multiAddress)
	if valid {
		t.Errorf("Expected multiaddress with ip %s will be denied since after denying public addresses, we then also deny this private address", ip)
	}
}

// Mock type for testing.
type maEndpoints struct {
	laddr ma.Multiaddr
	raddr ma.Multiaddr
}

// LocalMultiaddr returns the local address associated with
// this connection
func (c *maEndpoints) LocalMultiaddr() ma.Multiaddr {
	return c.laddr
}

// RemoteMultiaddr returns the remote address associated with
// this connection
func (c *maEndpoints) RemoteMultiaddr() ma.Multiaddr {
	return c.raddr
}
