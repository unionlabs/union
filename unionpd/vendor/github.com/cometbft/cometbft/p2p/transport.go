package p2p

import (
	"context"
	"fmt"
	"net"
	"time"

	"golang.org/x/net/netutil"

	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/libs/protoio"
	"github.com/cometbft/cometbft/p2p/conn"
	cmtp2p "github.com/cometbft/cometbft/proto/tendermint/p2p"
	"github.com/cosmos/gogoproto/proto"
)

const (
	defaultDialTimeout      = time.Second
	defaultFilterTimeout    = 5 * time.Second
	defaultHandshakeTimeout = 3 * time.Second
)

// IPResolver is a behavior subset of net.Resolver.
type IPResolver interface {
	LookupIPAddr(context.Context, string) ([]net.IPAddr, error)
}

// accept is the container to carry the upgraded connection and NodeInfo from an
// asynchronously running routine to the Accept method.
type accept struct {
	netAddr  *NetAddress
	conn     net.Conn
	nodeInfo NodeInfo
	err      error
}

// peerConfig is used to bundle data we need to fully setup a Peer with an
// MConn, provided by the caller of Accept and Dial (currently the Switch). This
// a temporary measure until reactor setup is less dynamic and we introduce the
// concept of PeerBehaviour to communicate about significant Peer lifecycle
// events.
// TODO(xla): Refactor out with more static Reactor setup and PeerBehaviour.
type peerConfig struct {
	chDescs     []*conn.ChannelDescriptor
	onPeerError func(Peer, interface{})
	outbound    bool
	// isPersistent allows you to set a function, which, given socket address
	// (for outbound peers) OR self-reported address (for inbound peers), tells
	// if the peer is persistent or not.
	isPersistent  func(*NetAddress) bool
	reactorsByCh  map[byte]Reactor
	msgTypeByChID map[byte]proto.Message
	metrics       *Metrics
	mlc           *metricsLabelCache
}

// Transport emits and connects to Peers. The implementation of Peer is left to
// the transport. Each transport is also responsible to filter establishing
// peers specific to its domain.
type Transport interface {
	// Listening address.
	NetAddress() NetAddress

	// Accept returns a newly connected Peer.
	Accept(peerConfig) (Peer, error)

	// Dial connects to the Peer for the address.
	Dial(NetAddress, peerConfig) (Peer, error)

	// Cleanup any resources associated with Peer.
	Cleanup(Peer)
}

// transportLifecycle bundles the methods for callers to control start and stop
// behavior.
type transportLifecycle interface {
	Close() error
	Listen(NetAddress) error
}

// ConnFilterFunc to be implemented by filter hooks after a new connection has
// been established. The set of exisiting connections is passed along together
// with all resolved IPs for the new connection.
type ConnFilterFunc func(ConnSet, net.Conn, []net.IP) error

// ConnDuplicateIPFilter resolves and keeps all ips for an incoming connection
// and refuses new ones if they come from a known ip.
func ConnDuplicateIPFilter() ConnFilterFunc {
	return func(cs ConnSet, c net.Conn, ips []net.IP) error {
		for _, ip := range ips {
			if cs.HasIP(ip) {
				return ErrRejected{
					conn:        c,
					err:         fmt.Errorf("ip<%v> already connected", ip),
					isDuplicate: true,
				}
			}
		}

		return nil
	}
}

// MultiplexTransportOption sets an optional parameter on the
// MultiplexTransport.
type MultiplexTransportOption func(*MultiplexTransport)

// MultiplexTransportConnFilters sets the filters for rejection new connections.
func MultiplexTransportConnFilters(
	filters ...ConnFilterFunc,
) MultiplexTransportOption {
	return func(mt *MultiplexTransport) { mt.connFilters = filters }
}

// MultiplexTransportFilterTimeout sets the timeout waited for filter calls to
// return.
func MultiplexTransportFilterTimeout(
	timeout time.Duration,
) MultiplexTransportOption {
	return func(mt *MultiplexTransport) { mt.filterTimeout = timeout }
}

// MultiplexTransportResolver sets the Resolver used for ip lokkups, defaults to
// net.DefaultResolver.
func MultiplexTransportResolver(resolver IPResolver) MultiplexTransportOption {
	return func(mt *MultiplexTransport) { mt.resolver = resolver }
}

// MultiplexTransportMaxIncomingConnections sets the maximum number of
// simultaneous connections (incoming). Default: 0 (unlimited)
func MultiplexTransportMaxIncomingConnections(n int) MultiplexTransportOption {
	return func(mt *MultiplexTransport) { mt.maxIncomingConnections = n }
}

// MultiplexTransport accepts and dials tcp connections and upgrades them to
// multiplexed peers.
type MultiplexTransport struct {
	netAddr                NetAddress
	listener               net.Listener
	maxIncomingConnections int // see MaxIncomingConnections

	acceptc chan accept
	closec  chan struct{}

	// Lookup table for duplicate ip and id checks.
	conns       ConnSet
	connFilters []ConnFilterFunc

	dialTimeout      time.Duration
	filterTimeout    time.Duration
	handshakeTimeout time.Duration
	nodeInfo         NodeInfo
	nodeKey          NodeKey
	resolver         IPResolver

	// TODO(xla): This config is still needed as we parameterise peerConn and
	// peer currently. All relevant configuration should be refactored into options
	// with sane defaults.
	mConfig conn.MConnConfig
}

// Test multiplexTransport for interface completeness.
var _ Transport = (*MultiplexTransport)(nil)
var _ transportLifecycle = (*MultiplexTransport)(nil)

// NewMultiplexTransport returns a tcp connected multiplexed peer.
func NewMultiplexTransport(
	nodeInfo NodeInfo,
	nodeKey NodeKey,
	mConfig conn.MConnConfig,
) *MultiplexTransport {
	return &MultiplexTransport{
		acceptc:          make(chan accept),
		closec:           make(chan struct{}),
		dialTimeout:      defaultDialTimeout,
		filterTimeout:    defaultFilterTimeout,
		handshakeTimeout: defaultHandshakeTimeout,
		mConfig:          mConfig,
		nodeInfo:         nodeInfo,
		nodeKey:          nodeKey,
		conns:            NewConnSet(),
		resolver:         net.DefaultResolver,
	}
}

// NetAddress implements Transport.
func (mt *MultiplexTransport) NetAddress() NetAddress {
	return mt.netAddr
}

// Accept implements Transport.
func (mt *MultiplexTransport) Accept(cfg peerConfig) (Peer, error) {
	select {
	// This case should never have any side-effectful/blocking operations to
	// ensure that quality peers are ready to be used.
	case a := <-mt.acceptc:
		if a.err != nil {
			return nil, a.err
		}

		cfg.outbound = false

		return mt.wrapPeer(a.conn, a.nodeInfo, cfg, a.netAddr), nil
	case <-mt.closec:
		return nil, ErrTransportClosed{}
	}
}

// Dial implements Transport.
func (mt *MultiplexTransport) Dial(
	addr NetAddress,
	cfg peerConfig,
) (Peer, error) {
	c, err := addr.DialTimeout(mt.dialTimeout)
	if err != nil {
		return nil, err
	}

	// TODO(xla): Evaluate if we should apply filters if we explicitly dial.
	if err := mt.filterConn(c); err != nil {
		return nil, err
	}

	secretConn, nodeInfo, err := mt.upgrade(c, &addr)
	if err != nil {
		return nil, err
	}

	cfg.outbound = true

	p := mt.wrapPeer(secretConn, nodeInfo, cfg, &addr)

	return p, nil
}

// Close implements transportLifecycle.
func (mt *MultiplexTransport) Close() error {
	close(mt.closec)

	if mt.listener != nil {
		return mt.listener.Close()
	}

	return nil
}

// Listen implements transportLifecycle.
func (mt *MultiplexTransport) Listen(addr NetAddress) error {
	ln, err := net.Listen("tcp", addr.DialString())
	if err != nil {
		return err
	}

	if mt.maxIncomingConnections > 0 {
		ln = netutil.LimitListener(ln, mt.maxIncomingConnections)
	}

	mt.netAddr = addr
	mt.listener = ln

	go mt.acceptPeers()

	return nil
}

// AddChannel registers a channel to nodeInfo.
// NOTE: NodeInfo must be of type DefaultNodeInfo else channels won't be updated
// This is a bit messy at the moment but is cleaned up in the following version
// when NodeInfo changes from an interface to a concrete type
func (mt *MultiplexTransport) AddChannel(chID byte) {
	if ni, ok := mt.nodeInfo.(DefaultNodeInfo); ok {
		if !ni.HasChannel(chID) {
			ni.Channels = append(ni.Channels, chID)
		}
		mt.nodeInfo = ni
	}
}

func (mt *MultiplexTransport) acceptPeers() {
	for {
		c, err := mt.listener.Accept()
		if err != nil {
			// If Close() has been called, silently exit.
			select {
			case _, ok := <-mt.closec:
				if !ok {
					return
				}
			default:
				// Transport is not closed
			}

			mt.acceptc <- accept{err: err}
			return
		}

		// Connection upgrade and filtering should be asynchronous to avoid
		// Head-of-line blocking[0].
		// Reference:  https://github.com/tendermint/tendermint/issues/2047
		//
		// [0] https://en.wikipedia.org/wiki/Head-of-line_blocking
		go func(c net.Conn) {
			defer func() {
				if r := recover(); r != nil {
					err := ErrRejected{
						conn:          c,
						err:           fmt.Errorf("recovered from panic: %v", r),
						isAuthFailure: true,
					}
					select {
					case mt.acceptc <- accept{err: err}:
					case <-mt.closec:
						// Give up if the transport was closed.
						_ = c.Close()
						return
					}
				}
			}()

			var (
				nodeInfo   NodeInfo
				secretConn *conn.SecretConnection
				netAddr    *NetAddress
			)

			err := mt.filterConn(c)
			if err == nil {
				secretConn, nodeInfo, err = mt.upgrade(c, nil)
				if err == nil {
					addr := c.RemoteAddr()
					id := PubKeyToID(secretConn.RemotePubKey())
					netAddr = NewNetAddress(id, addr)
				}
			}

			select {
			case mt.acceptc <- accept{netAddr, secretConn, nodeInfo, err}:
				// Make the upgraded peer available.
			case <-mt.closec:
				// Give up if the transport was closed.
				_ = c.Close()
				return
			}
		}(c)
	}
}

// Cleanup removes the given address from the connections set and
// closes the connection.
func (mt *MultiplexTransport) Cleanup(p Peer) {
	mt.conns.RemoveAddr(p.RemoteAddr())
	_ = p.CloseConn()
}

func (mt *MultiplexTransport) cleanup(c net.Conn) error {
	mt.conns.Remove(c)

	return c.Close()
}

func (mt *MultiplexTransport) filterConn(c net.Conn) (err error) {
	defer func() {
		if err != nil {
			_ = c.Close()
		}
	}()

	// Reject if connection is already present.
	if mt.conns.Has(c) {
		return ErrRejected{conn: c, isDuplicate: true}
	}

	// Resolve ips for incoming conn.
	ips, err := resolveIPs(mt.resolver, c)
	if err != nil {
		return err
	}

	errc := make(chan error, len(mt.connFilters))

	for _, f := range mt.connFilters {
		go func(f ConnFilterFunc, c net.Conn, ips []net.IP, errc chan<- error) {
			errc <- f(mt.conns, c, ips)
		}(f, c, ips, errc)
	}

	for i := 0; i < cap(errc); i++ {
		select {
		case err := <-errc:
			if err != nil {
				return ErrRejected{conn: c, err: err, isFiltered: true}
			}
		case <-time.After(mt.filterTimeout):
			return ErrFilterTimeout{}
		}

	}

	mt.conns.Set(c, ips)

	return nil
}

func (mt *MultiplexTransport) upgrade(
	c net.Conn,
	dialedAddr *NetAddress,
) (secretConn *conn.SecretConnection, nodeInfo NodeInfo, err error) {
	defer func() {
		if err != nil {
			_ = mt.cleanup(c)
		}
	}()

	secretConn, err = upgradeSecretConn(c, mt.handshakeTimeout, mt.nodeKey.PrivKey)
	if err != nil {
		return nil, nil, ErrRejected{
			conn:          c,
			err:           fmt.Errorf("secret conn failed: %v", err),
			isAuthFailure: true,
		}
	}

	// For outgoing conns, ensure connection key matches dialed key.
	connID := PubKeyToID(secretConn.RemotePubKey())
	if dialedAddr != nil {
		if dialedID := dialedAddr.ID; connID != dialedID {
			return nil, nil, ErrRejected{
				conn: c,
				id:   connID,
				err: fmt.Errorf(
					"conn.ID (%v) dialed ID (%v) mismatch",
					connID,
					dialedID,
				),
				isAuthFailure: true,
			}
		}
	}

	nodeInfo, err = handshake(secretConn, mt.handshakeTimeout, mt.nodeInfo)
	if err != nil {
		return nil, nil, ErrRejected{
			conn:          c,
			err:           fmt.Errorf("handshake failed: %v", err),
			isAuthFailure: true,
		}
	}

	if err := nodeInfo.Validate(); err != nil {
		return nil, nil, ErrRejected{
			conn:              c,
			err:               err,
			isNodeInfoInvalid: true,
		}
	}

	// Ensure connection key matches self reported key.
	if connID != nodeInfo.ID() {
		return nil, nil, ErrRejected{
			conn: c,
			id:   connID,
			err: fmt.Errorf(
				"conn.ID (%v) NodeInfo.ID (%v) mismatch",
				connID,
				nodeInfo.ID(),
			),
			isAuthFailure: true,
		}
	}

	// Reject self.
	if mt.nodeInfo.ID() == nodeInfo.ID() {
		return nil, nil, ErrRejected{
			addr:   *NewNetAddress(nodeInfo.ID(), c.RemoteAddr()),
			conn:   c,
			id:     nodeInfo.ID(),
			isSelf: true,
		}
	}

	if err := mt.nodeInfo.CompatibleWith(nodeInfo); err != nil {
		return nil, nil, ErrRejected{
			conn:           c,
			err:            err,
			id:             nodeInfo.ID(),
			isIncompatible: true,
		}
	}

	return secretConn, nodeInfo, nil
}

func (mt *MultiplexTransport) wrapPeer(
	c net.Conn,
	ni NodeInfo,
	cfg peerConfig,
	socketAddr *NetAddress,
) Peer {

	persistent := false
	if cfg.isPersistent != nil {
		if cfg.outbound {
			persistent = cfg.isPersistent(socketAddr)
		} else {
			selfReportedAddr, err := ni.NetAddress()
			if err == nil {
				persistent = cfg.isPersistent(selfReportedAddr)
			}
		}
	}

	peerConn := newPeerConn(
		cfg.outbound,
		persistent,
		c,
		socketAddr,
	)

	p := newPeer(
		peerConn,
		mt.mConfig,
		ni,
		cfg.reactorsByCh,
		cfg.msgTypeByChID,
		cfg.chDescs,
		cfg.onPeerError,
		cfg.mlc,
		PeerMetrics(cfg.metrics),
	)

	return p
}

func handshake(
	c net.Conn,
	timeout time.Duration,
	nodeInfo NodeInfo,
) (NodeInfo, error) {
	if err := c.SetDeadline(time.Now().Add(timeout)); err != nil {
		return nil, err
	}

	var (
		errc = make(chan error, 2)

		pbpeerNodeInfo cmtp2p.DefaultNodeInfo
		peerNodeInfo   DefaultNodeInfo
		ourNodeInfo    = nodeInfo.(DefaultNodeInfo)
	)

	go func(errc chan<- error, c net.Conn) {
		_, err := protoio.NewDelimitedWriter(c).WriteMsg(ourNodeInfo.ToProto())
		errc <- err
	}(errc, c)
	go func(errc chan<- error, c net.Conn) {
		protoReader := protoio.NewDelimitedReader(c, MaxNodeInfoSize())
		_, err := protoReader.ReadMsg(&pbpeerNodeInfo)
		errc <- err
	}(errc, c)

	for i := 0; i < cap(errc); i++ {
		err := <-errc
		if err != nil {
			return nil, err
		}
	}

	peerNodeInfo, err := DefaultNodeInfoFromToProto(&pbpeerNodeInfo)
	if err != nil {
		return nil, err
	}

	return peerNodeInfo, c.SetDeadline(time.Time{})
}

func upgradeSecretConn(
	c net.Conn,
	timeout time.Duration,
	privKey crypto.PrivKey,
) (*conn.SecretConnection, error) {
	if err := c.SetDeadline(time.Now().Add(timeout)); err != nil {
		return nil, err
	}

	sc, err := conn.MakeSecretConnection(c, privKey)
	if err != nil {
		return nil, err
	}

	return sc, sc.SetDeadline(time.Time{})
}

func resolveIPs(resolver IPResolver, c net.Conn) ([]net.IP, error) {
	host, _, err := net.SplitHostPort(c.RemoteAddr().String())
	if err != nil {
		return nil, err
	}

	addrs, err := resolver.LookupIPAddr(context.Background(), host)
	if err != nil {
		return nil, err
	}

	ips := []net.IP{}

	for _, addr := range addrs {
		ips = append(ips, addr.IP)
	}

	return ips, nil
}
