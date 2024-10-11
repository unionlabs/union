package peerdata

import (
	"context"
	"errors"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/p2p/enr"
	"github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/peer"
	ma "github.com/multiformats/go-multiaddr"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/metadata"
)

var (
	// ErrPeerUnknown is returned when there is an attempt to obtain data from a peer that is not known.
	ErrPeerUnknown = errors.New("peer unknown")
	// ErrNoPeerStatus is returned when there is a map entry for a given peer but there is no chain
	// status for that peer. This should happen in rare circumstances only, but is a very possible
	// scenario in a chaotic and adversarial network.
	ErrNoPeerStatus = errors.New("no chain status for peer")
)

// PeerConnectionState is the state of the connection.
type PeerConnectionState ethpb.ConnectionState

// StoreConfig holds peer store parameters.
type StoreConfig struct {
	MaxPeers int
}

// Store is a container for various peer related data (both protocol and app level).
// Container implements RWMutex, so data access can be restricted on the container level. This allows
// different components rely on the very same peer map container.
// Note: access to data is controlled by clients i.e. client code is responsible for locking/unlocking
// the mutex when accessing data.
type Store struct {
	sync.RWMutex
	ctx          context.Context
	config       *StoreConfig
	peers        map[peer.ID]*PeerData
	trustedPeers map[peer.ID]bool
}

// PeerData aggregates protocol and application level info about a single peer.
type PeerData struct {
	// Network related data.
	Address       ma.Multiaddr
	Direction     network.Direction
	ConnState     PeerConnectionState
	Enr           *enr.Record
	NextValidTime time.Time
	// Chain related data.
	MetaData                  metadata.Metadata
	ChainState                *ethpb.Status
	ChainStateLastUpdated     time.Time
	ChainStateValidationError error
	// Scorers internal data.
	BadResponses         int
	ProcessedBlocks      uint64
	BlockProviderUpdated time.Time
	// Gossip Scoring data.
	TopicScores      map[string]*ethpb.TopicScoreSnapshot
	GossipScore      float64
	BehaviourPenalty float64
}

// NewStore creates new peer data store.
func NewStore(ctx context.Context, config *StoreConfig) *Store {
	return &Store{
		ctx:          ctx,
		config:       config,
		peers:        make(map[peer.ID]*PeerData),
		trustedPeers: make(map[peer.ID]bool),
	}
}

// PeerData returns data associated with a given peer, if any.
// Important: it is assumed that store mutex is locked when calling this method.
func (s *Store) PeerData(pid peer.ID) (*PeerData, bool) {
	peerData, ok := s.peers[pid]
	return peerData, ok
}

// PeerDataGetOrCreate returns data associated with a given peer.
// If no data has been associated yet, newly created and associated data object is returned.
// Important: it is assumed that store mutex is locked when calling this method.
func (s *Store) PeerDataGetOrCreate(pid peer.ID) *PeerData {
	if peerData, ok := s.peers[pid]; ok {
		return peerData
	}
	s.peers[pid] = &PeerData{}
	return s.peers[pid]
}

// SetPeerData updates data associated with a given peer.
// Important: it is assumed that store mutex is locked when calling this method.
func (s *Store) SetPeerData(pid peer.ID, data *PeerData) {
	s.peers[pid] = data
}

// DeletePeerData removes data associated with a given peer.
// Important: it is assumed that store mutex is locked when calling this method.
func (s *Store) DeletePeerData(pid peer.ID) {
	delete(s.peers, pid)
}

// SetTrustedPeers sets our desired trusted peer set.
// Important: it is assumed that store mutex is locked when calling this method.
func (s *Store) SetTrustedPeers(peers []peer.ID) {
	for _, p := range peers {
		s.trustedPeers[p] = true
	}
}

// GetTrustedPeers gets our desired trusted peer ids.
// Important: it is assumed that store mutex is locked when calling this method.
func (s *Store) GetTrustedPeers() []peer.ID {
	var peers []peer.ID
	for p := range s.trustedPeers {
		peers = append(peers, p)
	}
	return peers
}

// DeleteTrustedPeers removes peers from trusted peer set.
// Important: it is assumed that store mutex is locked when calling this method.
func (s *Store) DeleteTrustedPeers(peers []peer.ID) {
	for _, p := range peers {
		delete(s.trustedPeers, p)
	}
}

// Peers returns map of peer data objects.
// Important: it is assumed that store mutex is locked when calling this method.
func (s *Store) Peers() map[peer.ID]*PeerData {
	return s.peers
}

// IsTrustedPeer checks that the provided peer
// is in our trusted peer set.
func (s *Store) IsTrustedPeer(p peer.ID) bool {
	return s.trustedPeers[p]
}

// Config exposes store configuration params.
func (s *Store) Config() *StoreConfig {
	return s.config
}
