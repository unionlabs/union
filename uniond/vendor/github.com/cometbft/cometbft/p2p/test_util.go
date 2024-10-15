package p2p

import (
	"fmt"
	"net"
	"time"

	"github.com/cometbft/cometbft/config"
	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/ed25519"
	cmtnet "github.com/cometbft/cometbft/internal/net"
	cmtrand "github.com/cometbft/cometbft/internal/rand"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/p2p/conn"
)

const testCh = 0x01

// ------------------------------------------------

type mockNodeInfo struct {
	addr *NetAddress
}

func (ni mockNodeInfo) ID() ID                           { return ni.addr.ID }
func (ni mockNodeInfo) NetAddress() (*NetAddress, error) { return ni.addr, nil }
func (mockNodeInfo) Validate() error                     { return nil }
func (mockNodeInfo) CompatibleWith(NodeInfo) error       { return nil }

func AddPeerToSwitchPeerSet(sw *Switch, peer Peer) {
	sw.peers.Add(peer) //nolint:errcheck // ignore error
}

func CreateRandomPeer(outbound bool) Peer {
	addr, netAddr := CreateRoutableAddr()
	p := &peer{
		peerConn: peerConn{
			outbound:   outbound,
			socketAddr: netAddr,
		},
		nodeInfo: mockNodeInfo{netAddr},
		mconn:    &conn.MConnection{},
		metrics:  NopMetrics(),
	}
	p.SetLogger(log.TestingLogger().With("peer", addr))
	return p
}

func CreateRoutableAddr() (addr string, netAddr *NetAddress) {
	for {
		var err error
		addr = fmt.Sprintf("%X@%v.%v.%v.%v:26656",
			cmtrand.Bytes(20),
			cmtrand.Int()%256,
			cmtrand.Int()%256,
			cmtrand.Int()%256,
			cmtrand.Int()%256)
		netAddr, err = NewNetAddressString(addr)
		if err != nil {
			panic(err)
		}
		if netAddr.Routable() {
			break
		}
	}
	return addr, netAddr
}

// ------------------------------------------------------------------
// Connects switches via arbitrary net.Conn. Used for testing.

const TestHost = "localhost"

// MakeConnectedSwitches returns n switches, initialized according to the
// initSwitch function, and connected according to the connect function.
func MakeConnectedSwitches(cfg *config.P2PConfig,
	n int,
	initSwitch func(int, *Switch) *Switch,
	connect func([]*Switch, int, int),
) []*Switch {
	switches := MakeSwitches(cfg, n, initSwitch)
	return StartAndConnectSwitches(switches, connect)
}

// MakeSwitches returns n switches.
// initSwitch defines how the i'th switch should be initialized (ie. with what reactors).
func MakeSwitches(
	cfg *config.P2PConfig,
	n int,
	initSwitch func(int, *Switch) *Switch,
) []*Switch {
	switches := make([]*Switch, n)
	for i := 0; i < n; i++ {
		switches[i] = MakeSwitch(cfg, i, initSwitch)
	}
	return switches
}

// StartAndConnectSwitches connects the switches according to the connect function.
// If connect==Connect2Switches, the switches will be fully connected.
// NOTE: panics if any switch fails to start.
func StartAndConnectSwitches(
	switches []*Switch,
	connect func([]*Switch, int, int),
) []*Switch {
	if err := StartSwitches(switches); err != nil {
		panic(err)
	}

	for i := 0; i < len(switches); i++ {
		for j := i + 1; j < len(switches); j++ {
			connect(switches, i, j)
		}
	}

	return switches
}

// Connect2Switches will connect switches i and j via net.Pipe().
// Blocks until a connection is established.
// NOTE: caller ensures i and j are within bounds.
func Connect2Switches(switches []*Switch, i, j int) {
	switchI := switches[i]
	switchJ := switches[j]

	c1, c2 := conn.NetPipe()

	doneCh := make(chan struct{})
	go func() {
		err := switchI.addPeerWithConnection(c1)
		if err != nil {
			panic(err)
		}
		doneCh <- struct{}{}
	}()
	go func() {
		err := switchJ.addPeerWithConnection(c2)
		if err != nil {
			panic(err)
		}
		doneCh <- struct{}{}
	}()
	<-doneCh
	<-doneCh
}

// ConnectStartSwitches will connect switches c and j via net.Pipe().
func ConnectStarSwitches(c int) func([]*Switch, int, int) {
	// Blocks until a connection is established.
	// NOTE: caller ensures i and j is within bounds.
	return func(switches []*Switch, i, j int) {
		if i != c {
			return
		}

		switchI := switches[i]
		switchJ := switches[j]

		c1, c2 := conn.NetPipe()

		doneCh := make(chan struct{})
		go func() {
			err := switchI.addPeerWithConnection(c1)
			if err != nil {
				panic(err)
			}
			doneCh <- struct{}{}
		}()
		go func() {
			err := switchJ.addPeerWithConnection(c2)
			if err != nil {
				panic(err)
			}
			doneCh <- struct{}{}
		}()
		<-doneCh
		<-doneCh
	}
}

func (sw *Switch) addPeerWithConnection(conn net.Conn) error {
	pc, err := testInboundPeerConn(conn, sw.config, sw.nodeKey.PrivKey)
	if err != nil {
		if err := conn.Close(); err != nil {
			sw.Logger.Error("Error closing connection", "err", err)
		}
		return err
	}

	ni, err := handshake(conn, time.Second, sw.nodeInfo)
	if err != nil {
		if err := conn.Close(); err != nil {
			sw.Logger.Error("Error closing connection", "err", err)
		}
		return err
	}

	p := newPeer(
		pc,
		MConnConfig(sw.config),
		ni,
		sw.reactorsByCh,
		sw.msgTypeByChID,
		sw.chDescs,
		sw.StopPeerForError,
	)

	if err = sw.addPeer(p); err != nil {
		pc.CloseConn()
		return err
	}

	return nil
}

// StartSwitches calls sw.Start() for each given switch.
// It returns the first encountered error.
func StartSwitches(switches []*Switch) error {
	for _, s := range switches {
		err := s.Start() // start switch and reactors
		if err != nil {
			return err
		}
	}
	return nil
}

func MakeSwitch(
	cfg *config.P2PConfig,
	i int,
	initSwitch func(int, *Switch) *Switch,
	opts ...SwitchOption,
) *Switch {
	nodeKey := NodeKey{
		PrivKey: ed25519.GenPrivKey(),
	}
	nodeInfo := testNodeInfo(nodeKey.ID(), fmt.Sprintf("node%d", i))
	addr, err := NewNetAddressString(
		IDAddressString(nodeKey.ID(), nodeInfo.(DefaultNodeInfo).ListenAddr),
	)
	if err != nil {
		panic(err)
	}

	t := NewMultiplexTransport(nodeInfo, nodeKey, MConnConfig(cfg))

	if err := t.Listen(*addr); err != nil {
		panic(err)
	}

	// TODO: let the config be passed in?
	sw := initSwitch(i, NewSwitch(cfg, t, opts...))
	sw.SetLogger(log.TestingLogger().With("switch", i))
	sw.SetNodeKey(&nodeKey)

	ni := nodeInfo.(DefaultNodeInfo)
	for ch := range sw.reactorsByCh {
		ni.Channels = append(ni.Channels, ch)
	}
	nodeInfo = ni

	// TODO: We need to setup reactors ahead of time so the NodeInfo is properly
	// populated and we don't have to do those awkward overrides and setters.
	t.nodeInfo = nodeInfo
	sw.SetNodeInfo(nodeInfo)

	return sw
}

func testInboundPeerConn(
	conn net.Conn,
	config *config.P2PConfig,
	ourNodePrivKey crypto.PrivKey,
) (peerConn, error) {
	return testPeerConn(conn, config, false, false, ourNodePrivKey, nil)
}

func testPeerConn(
	rawConn net.Conn,
	cfg *config.P2PConfig,
	outbound, persistent bool,
	ourNodePrivKey crypto.PrivKey,
	socketAddr *NetAddress,
) (pc peerConn, err error) {
	conn := rawConn

	// Fuzz connection
	if cfg.TestFuzz {
		// so we have time to do peer handshakes and get set up
		conn = FuzzConnAfterFromConfig(conn, 10*time.Second, cfg.TestFuzzConfig)
	}

	// Encrypt connection
	conn, err = upgradeSecretConn(conn, cfg.HandshakeTimeout, ourNodePrivKey)
	if err != nil {
		return pc, fmt.Errorf("error creating peer: %w", err)
	}

	// Only the information we already have
	return newPeerConn(outbound, persistent, conn, socketAddr), nil
}

// ----------------------------------------------------------------
// rand node info

func testNodeInfo(id ID, name string) NodeInfo {
	return testNodeInfoWithNetwork(id, name, "testing")
}

func testNodeInfoWithNetwork(id ID, name, network string) NodeInfo {
	return DefaultNodeInfo{
		ProtocolVersion: defaultProtocolVersion,
		DefaultNodeID:   id,
		ListenAddr:      fmt.Sprintf("127.0.0.1:%d", getFreePort()),
		Network:         network,
		Version:         "1.2.3-rc0-deadbeef",
		Channels:        []byte{testCh},
		Moniker:         name,
		Other: DefaultNodeInfoOther{
			TxIndex:    "on",
			RPCAddress: fmt.Sprintf("127.0.0.1:%d", getFreePort()),
		},
	}
}

func getFreePort() int {
	port, err := cmtnet.GetFreePort()
	if err != nil {
		panic(err)
	}
	return port
}

type AddrBookMock struct {
	Addrs        map[string]struct{}
	OurAddrs     map[string]struct{}
	PrivateAddrs map[string]struct{}
}

var _ AddrBook = (*AddrBookMock)(nil)

func (book *AddrBookMock) AddAddress(addr *NetAddress, _ *NetAddress) error {
	book.Addrs[addr.String()] = struct{}{}
	return nil
}
func (book *AddrBookMock) AddOurAddress(addr *NetAddress) { book.OurAddrs[addr.String()] = struct{}{} }
func (book *AddrBookMock) OurAddress(addr *NetAddress) bool {
	_, ok := book.OurAddrs[addr.String()]
	return ok
}
func (*AddrBookMock) MarkGood(ID) {}
func (book *AddrBookMock) HasAddress(addr *NetAddress) bool {
	_, ok := book.Addrs[addr.String()]
	return ok
}

func (book *AddrBookMock) RemoveAddress(addr *NetAddress) {
	delete(book.Addrs, addr.String())
}
func (*AddrBookMock) Save() {}
func (book *AddrBookMock) AddPrivateIDs(addrs []string) {
	for _, addr := range addrs {
		book.PrivateAddrs[addr] = struct{}{}
	}
}
