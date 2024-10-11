package p2p

import (
	"bytes"
	"crypto/ecdsa"
	"net"
	"time"

	"github.com/ethereum/go-ethereum/p2p/discover"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/enr"
	"github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/peer"
	ma "github.com/multiformats/go-multiaddr"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	ecdsaprysm "github.com/prysmaticlabs/prysm/v4/crypto/ecdsa"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

// Listener defines the discovery V5 network interface that is used
// to communicate with other peers.
type Listener interface {
	Self() *enode.Node
	Close()
	Lookup(enode.ID) []*enode.Node
	Resolve(*enode.Node) *enode.Node
	RandomNodes() enode.Iterator
	Ping(*enode.Node) error
	RequestENR(*enode.Node) (*enode.Node, error)
	LocalNode() *enode.LocalNode
}

// RefreshENR uses an epoch to refresh the enr entry for our node
// with the tracked committee ids for the epoch, allowing our node
// to be dynamically discoverable by others given our tracked committee ids.
func (s *Service) RefreshENR() {
	// return early if discv5 isnt running
	if s.dv5Listener == nil || !s.isInitialized() {
		return
	}
	currEpoch := slots.ToEpoch(slots.CurrentSlot(uint64(s.genesisTime.Unix())))
	if err := initializePersistentSubnets(s.dv5Listener.LocalNode().ID(), currEpoch); err != nil {
		log.WithError(err).Error("Could not initialize persistent subnets")
		return
	}

	bitV := bitfield.NewBitvector64()
	committees := cache.SubnetIDs.GetAllSubnets()
	for _, idx := range committees {
		bitV.SetBitAt(idx, true)
	}
	currentBitV, err := attBitvector(s.dv5Listener.Self().Record())
	if err != nil {
		log.WithError(err).Error("Could not retrieve att bitfield")
		return
	}

	// Compare current epoch with our fork epochs
	altairForkEpoch := params.BeaconConfig().AltairForkEpoch
	switch {
	// Altair Behaviour
	case currEpoch >= altairForkEpoch:
		// Retrieve sync subnets from application level
		// cache.
		bitS := bitfield.Bitvector4{byte(0x00)}
		committees = cache.SyncSubnetIDs.GetAllSubnets(currEpoch)
		for _, idx := range committees {
			bitS.SetBitAt(idx, true)
		}
		currentBitS, err := syncBitvector(s.dv5Listener.Self().Record())
		if err != nil {
			log.WithError(err).Error("Could not retrieve sync bitfield")
			return
		}
		if bytes.Equal(bitV, currentBitV) && bytes.Equal(bitS, currentBitS) &&
			s.Metadata().Version() == version.Altair {
			// return early if bitfields haven't changed
			return
		}
		s.updateSubnetRecordWithMetadataV2(bitV, bitS)
	default:
		// Phase 0 behaviour.
		if bytes.Equal(bitV, currentBitV) {
			// return early if bitfield hasn't changed
			return
		}
		s.updateSubnetRecordWithMetadata(bitV)
	}
	// ping all peers to inform them of new metadata
	s.pingPeers()
}

// listen for new nodes watches for new nodes in the network and adds them to the peerstore.
func (s *Service) listenForNewNodes() {
	iterator := s.dv5Listener.RandomNodes()
	iterator = enode.Filter(iterator, s.filterPeer)
	defer iterator.Close()
	for {
		// Exit if service's context is canceled
		if s.ctx.Err() != nil {
			break
		}
		if s.isPeerAtLimit(false /* inbound */) {
			// Pause the main loop for a period to stop looking
			// for new peers.
			log.Trace("Not looking for peers, at peer limit")
			time.Sleep(pollingPeriod)
			continue
		}
		exists := iterator.Next()
		if !exists {
			break
		}
		node := iterator.Node()
		peerInfo, _, err := convertToAddrInfo(node)
		if err != nil {
			log.WithError(err).Error("Could not convert to peer info")
			continue
		}
		// Make sure that peer is not dialed too often, for each connection attempt there's a backoff period.
		s.Peers().RandomizeBackOff(peerInfo.ID)
		go func(info *peer.AddrInfo) {
			if err := s.connectWithPeer(s.ctx, *info); err != nil {
				log.WithError(err).Tracef("Could not connect with peer %s", info.String())
			}
		}(peerInfo)
	}
}

func (s *Service) createListener(
	ipAddr net.IP,
	privKey *ecdsa.PrivateKey,
) (*discover.UDPv5, error) {
	// BindIP is used to specify the ip
	// on which we will bind our listener on
	// by default we will listen to all interfaces.
	var bindIP net.IP
	switch udpVersionFromIP(ipAddr) {
	case "udp4":
		bindIP = net.IPv4zero
	case "udp6":
		bindIP = net.IPv6zero
	default:
		return nil, errors.New("invalid ip provided")
	}

	// If local ip is specified then use that instead.
	if s.cfg.LocalIP != "" {
		ipAddr = net.ParseIP(s.cfg.LocalIP)
		if ipAddr == nil {
			return nil, errors.New("invalid local ip provided")
		}
		bindIP = ipAddr
	}
	udpAddr := &net.UDPAddr{
		IP:   bindIP,
		Port: int(s.cfg.UDPPort),
	}
	// Listen to all network interfaces
	// for both ip protocols.
	networkVersion := "udp"
	conn, err := net.ListenUDP(networkVersion, udpAddr)
	if err != nil {
		return nil, errors.Wrap(err, "could not listen to UDP")
	}

	localNode, err := s.createLocalNode(
		privKey,
		ipAddr,
		int(s.cfg.UDPPort),
		int(s.cfg.TCPPort),
	)
	if err != nil {
		return nil, errors.Wrap(err, "could not create local node")
	}
	if s.cfg.HostAddress != "" {
		hostIP := net.ParseIP(s.cfg.HostAddress)
		if hostIP.To4() == nil && hostIP.To16() == nil {
			log.Errorf("Invalid host address given: %s", hostIP.String())
		} else {
			localNode.SetFallbackIP(hostIP)
			localNode.SetStaticIP(hostIP)
		}
	}
	if s.cfg.HostDNS != "" {
		host := s.cfg.HostDNS
		ips, err := net.LookupIP(host)
		if err != nil {
			return nil, errors.Wrap(err, "could not resolve host address")
		}
		if len(ips) > 0 {
			// Use first IP returned from the
			// resolver.
			firstIP := ips[0]
			localNode.SetFallbackIP(firstIP)
		}
	}
	dv5Cfg := discover.Config{
		PrivateKey: privKey,
	}
	dv5Cfg.Bootnodes = []*enode.Node{}
	for _, addr := range s.cfg.Discv5BootStrapAddr {
		bootNode, err := enode.Parse(enode.ValidSchemes, addr)
		if err != nil {
			return nil, errors.Wrap(err, "could not bootstrap addr")
		}
		dv5Cfg.Bootnodes = append(dv5Cfg.Bootnodes, bootNode)
	}

	listener, err := discover.ListenV5(conn, localNode, dv5Cfg)
	if err != nil {
		return nil, errors.Wrap(err, "could not listen to discV5")
	}
	return listener, nil
}

func (s *Service) createLocalNode(
	privKey *ecdsa.PrivateKey,
	ipAddr net.IP,
	udpPort, tcpPort int,
) (*enode.LocalNode, error) {
	db, err := enode.OpenDB("")
	if err != nil {
		return nil, errors.Wrap(err, "could not open node's peer database")
	}
	localNode := enode.NewLocalNode(db, privKey)

	ipEntry := enr.IP(ipAddr)
	udpEntry := enr.UDP(udpPort)
	tcpEntry := enr.TCP(tcpPort)
	localNode.Set(ipEntry)
	localNode.Set(udpEntry)
	localNode.Set(tcpEntry)
	localNode.SetFallbackIP(ipAddr)
	localNode.SetFallbackUDP(udpPort)

	localNode, err = addForkEntry(localNode, s.genesisTime, s.genesisValidatorsRoot)
	if err != nil {
		return nil, errors.Wrap(err, "could not add eth2 fork version entry to enr")
	}
	localNode = initializeAttSubnets(localNode)
	return initializeSyncCommSubnets(localNode), nil
}

func (s *Service) startDiscoveryV5(
	addr net.IP,
	privKey *ecdsa.PrivateKey,
) (*discover.UDPv5, error) {
	listener, err := s.createListener(addr, privKey)
	if err != nil {
		return nil, errors.Wrap(err, "could not create listener")
	}
	record := listener.Self()
	log.WithField("ENR", record.String()).Info("Started discovery v5")
	return listener, nil
}

// filterPeer validates each node that we retrieve from our dht. We
// try to ascertain that the peer can be a valid protocol peer.
// Validity Conditions:
//  1. The local node is still actively looking for peers to
//     connect to.
//  2. Peer has a valid IP and TCP port set in their enr.
//  3. Peer hasn't been marked as 'bad'
//  4. Peer is not currently active or connected.
//  5. Peer is ready to receive incoming connections.
//  6. Peer's fork digest in their ENR matches that of
//     our localnodes.
func (s *Service) filterPeer(node *enode.Node) bool {
	// Ignore nil node entries passed in.
	if node == nil {
		return false
	}
	// ignore nodes with no ip address stored.
	if node.IP() == nil {
		return false
	}
	// do not dial nodes with their tcp ports not set
	if err := node.Record().Load(enr.WithEntry("tcp", new(enr.TCP))); err != nil {
		if !enr.IsNotFound(err) {
			log.WithError(err).Debug("Could not retrieve tcp port")
		}
		return false
	}
	peerData, multiAddr, err := convertToAddrInfo(node)
	if err != nil {
		log.WithError(err).Debug("Could not convert to peer data")
		return false
	}
	if s.peers.IsBad(peerData.ID) {
		return false
	}
	if s.peers.IsActive(peerData.ID) {
		return false
	}
	if s.host.Network().Connectedness(peerData.ID) == network.Connected {
		return false
	}
	if !s.peers.IsReadyToDial(peerData.ID) {
		return false
	}
	nodeENR := node.Record()
	// Decide whether or not to connect to peer that does not
	// match the proper fork ENR data with our local node.
	if s.genesisValidatorsRoot != nil {
		if err := s.compareForkENR(nodeENR); err != nil {
			log.WithError(err).Trace("Fork ENR mismatches between peer and local node")
			return false
		}
	}
	// Add peer to peer handler.
	s.peers.Add(nodeENR, peerData.ID, multiAddr, network.DirUnknown)
	return true
}

// This checks our set max peers in our config, and
// determines whether our currently connected and
// active peers are above our set max peer limit.
func (s *Service) isPeerAtLimit(inbound bool) bool {
	numOfConns := len(s.host.Network().Peers())
	maxPeers := int(s.cfg.MaxPeers)
	// If we are measuring the limit for inbound peers
	// we apply the high watermark buffer.
	if inbound {
		maxPeers += highWatermarkBuffer
		maxInbound := s.peers.InboundLimit() + highWatermarkBuffer
		currInbound := len(s.peers.InboundConnected())
		// Exit early if we are at the inbound limit.
		if currInbound >= maxInbound {
			return true
		}
	}
	activePeers := len(s.Peers().Active())
	return activePeers >= maxPeers || numOfConns >= maxPeers
}

// PeersFromStringAddrs converts peer raw ENRs into multiaddrs for p2p.
func PeersFromStringAddrs(addrs []string) ([]ma.Multiaddr, error) {
	var allAddrs []ma.Multiaddr
	enodeString, multiAddrString := parseGenericAddrs(addrs)
	for _, stringAddr := range multiAddrString {
		addr, err := multiAddrFromString(stringAddr)
		if err != nil {
			return nil, errors.Wrapf(err, "Could not get multiaddr from string")
		}
		allAddrs = append(allAddrs, addr)
	}
	for _, stringAddr := range enodeString {
		enodeAddr, err := enode.Parse(enode.ValidSchemes, stringAddr)
		if err != nil {
			return nil, errors.Wrapf(err, "Could not get enode from string")
		}
		addr, err := convertToSingleMultiAddr(enodeAddr)
		if err != nil {
			return nil, errors.Wrapf(err, "Could not get multiaddr")
		}
		allAddrs = append(allAddrs, addr)
	}
	return allAddrs, nil
}

func parseBootStrapAddrs(addrs []string) (discv5Nodes []string) {
	discv5Nodes, _ = parseGenericAddrs(addrs)
	if len(discv5Nodes) == 0 {
		log.Warn("No bootstrap addresses supplied")
	}
	return discv5Nodes
}

func parseGenericAddrs(addrs []string) (enodeString, multiAddrString []string) {
	for _, addr := range addrs {
		if addr == "" {
			// Ignore empty entries
			continue
		}
		_, err := enode.Parse(enode.ValidSchemes, addr)
		if err == nil {
			enodeString = append(enodeString, addr)
			continue
		}
		_, err = multiAddrFromString(addr)
		if err == nil {
			multiAddrString = append(multiAddrString, addr)
			continue
		}
		log.WithError(err).Errorf("Invalid address of %s provided", addr)
	}
	return enodeString, multiAddrString
}

func convertToMultiAddr(nodes []*enode.Node) []ma.Multiaddr {
	var multiAddrs []ma.Multiaddr
	for _, node := range nodes {
		// ignore nodes with no ip address stored
		if node.IP() == nil {
			continue
		}
		multiAddr, err := convertToSingleMultiAddr(node)
		if err != nil {
			log.WithError(err).Error("Could not convert to multiAddr")
			continue
		}
		multiAddrs = append(multiAddrs, multiAddr)
	}
	return multiAddrs
}

func convertToAddrInfo(node *enode.Node) (*peer.AddrInfo, ma.Multiaddr, error) {
	multiAddr, err := convertToSingleMultiAddr(node)
	if err != nil {
		return nil, nil, err
	}
	info, err := peer.AddrInfoFromP2pAddr(multiAddr)
	if err != nil {
		return nil, nil, err
	}
	return info, multiAddr, nil
}

func convertToSingleMultiAddr(node *enode.Node) (ma.Multiaddr, error) {
	pubkey := node.Pubkey()
	assertedKey, err := ecdsaprysm.ConvertToInterfacePubkey(pubkey)
	if err != nil {
		return nil, errors.Wrap(err, "could not get pubkey")
	}
	id, err := peer.IDFromPublicKey(assertedKey)
	if err != nil {
		return nil, errors.Wrap(err, "could not get peer id")
	}
	return multiAddressBuilderWithID(node.IP().String(), "tcp", uint(node.TCP()), id)
}

func convertToUdpMultiAddr(node *enode.Node) ([]ma.Multiaddr, error) {
	pubkey := node.Pubkey()
	assertedKey, err := ecdsaprysm.ConvertToInterfacePubkey(pubkey)
	if err != nil {
		return nil, errors.Wrap(err, "could not get pubkey")
	}
	id, err := peer.IDFromPublicKey(assertedKey)
	if err != nil {
		return nil, errors.Wrap(err, "could not get peer id")
	}

	var addresses []ma.Multiaddr
	var ip4 enr.IPv4
	var ip6 enr.IPv6
	if node.Load(&ip4) == nil {
		address, ipErr := multiAddressBuilderWithID(net.IP(ip4).String(), "udp", uint(node.UDP()), id)
		if ipErr != nil {
			return nil, errors.Wrap(ipErr, "could not build IPv4 address")
		}
		addresses = append(addresses, address)
	}
	if node.Load(&ip6) == nil {
		address, ipErr := multiAddressBuilderWithID(net.IP(ip6).String(), "udp", uint(node.UDP()), id)
		if ipErr != nil {
			return nil, errors.Wrap(ipErr, "could not build IPv6 address")
		}
		addresses = append(addresses, address)
	}

	return addresses, nil
}

func peerIdsFromMultiAddrs(addrs []ma.Multiaddr) []peer.ID {
	var peers []peer.ID
	for _, a := range addrs {
		info, err := peer.AddrInfoFromP2pAddr(a)
		if err != nil {
			log.WithError(err).Errorf("Could not derive peer info from multiaddress %s", a.String())
			continue
		}
		peers = append(peers, info.ID)
	}
	return peers
}

func multiAddrFromString(address string) (ma.Multiaddr, error) {
	return ma.NewMultiaddr(address)
}

func udpVersionFromIP(ipAddr net.IP) string {
	if ipAddr.To4() != nil {
		return "udp4"
	}
	return "udp6"
}
