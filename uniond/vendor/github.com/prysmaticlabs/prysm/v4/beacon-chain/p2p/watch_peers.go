package p2p

import (
	"context"

	"github.com/libp2p/go-libp2p/core/host"
	"github.com/libp2p/go-libp2p/core/peer"
	ma "github.com/multiformats/go-multiaddr"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers"
)

// ensurePeerConnections will attempt to reestablish connection to the peers
// if there are currently no connections to that peer.
func ensurePeerConnections(ctx context.Context, h host.Host, peers *peers.Status, relayNodes ...string) {
	// every time reset peersToWatch, add RelayNodes and trust peers
	var peersToWatch []*peer.AddrInfo

	// add RelayNodes
	for _, node := range relayNodes {
		if node == "" {
			continue
		}
		peerInfo, err := MakePeer(node)
		if err != nil {
			log.WithError(err).Error("Could not make peer")
			continue
		}
		peersToWatch = append(peersToWatch, peerInfo)
	}

	// add trusted peers
	trustedPeers := peers.GetTrustedPeers()
	for _, trustedPeer := range trustedPeers {
		maddr, err := peers.Address(trustedPeer)

		// avoid invalid trusted peers
		if err != nil || maddr == nil {
			log.WithField("peer", trustedPeers).WithError(err).Error("Could not get peer address")
			continue
		}
		peerInfo := &peer.AddrInfo{ID: trustedPeer}
		peerInfo.Addrs = []ma.Multiaddr{maddr}
		peersToWatch = append(peersToWatch, peerInfo)
	}

	if len(peersToWatch) == 0 {
		return
	}
	for _, p := range peersToWatch {
		c := h.Network().ConnsToPeer(p.ID)
		if len(c) == 0 {
			if err := connectWithTimeout(ctx, h, p); err != nil {
				log.WithField("peer", p.ID).WithField("addrs", p.Addrs).WithError(err).Errorf("Failed to reconnect to peer")
				continue
			}
		}
	}
}

func connectWithTimeout(ctx context.Context, h host.Host, peer *peer.AddrInfo) error {
	log.WithField("peer", peer.ID).Debug("No connections to peer, reconnecting")
	ctx, cancel := context.WithTimeout(ctx, maxDialTimeout)
	defer cancel()
	return h.Connect(ctx, *peer)
}
