package p2p

import (
	"net"

	cmtrand "github.com/cometbft/cometbft/internal/rand"
	cmtsync "github.com/cometbft/cometbft/libs/sync"
)

// IPeerSet has a (immutable) subset of the methods of PeerSet.
type IPeerSet interface {
	// Has returns true if the set contains the peer referred to by this key.
	Has(key ID) bool
	// HasIP returns true if the set contains the peer referred to by this IP
	HasIP(ip net.IP) bool
	// Get returns the peer with the given key, or nil if not found.
	Get(key ID) Peer
	// Copy returns a copy of the peers list.
	Copy() []Peer
	// Size returns the number of peers in the PeerSet.
	Size() int
	// ForEach iterates over the PeerSet and calls the given function for each peer.
	ForEach(peer func(Peer))
	// Random returns a random peer from the PeerSet.
	Random() Peer
}

// -----------------------------------------------------------------------------

// PeerSet is a special thread-safe structure for keeping a table of peers.
type PeerSet struct {
	mtx    cmtsync.Mutex
	lookup map[ID]*peerSetItem
	list   []Peer
}

type peerSetItem struct {
	peer  Peer
	index int
}

// NewPeerSet creates a new peerSet with a list of initial capacity of 256 items.
func NewPeerSet() *PeerSet {
	return &PeerSet{
		lookup: make(map[ID]*peerSetItem),
		list:   make([]Peer, 0, 256),
	}
}

// Add adds the peer to the PeerSet.
// It returns an error carrying the reason, if the peer is already present.
func (ps *PeerSet) Add(peer Peer) error {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()

	if ps.lookup[peer.ID()] != nil {
		return ErrSwitchDuplicatePeerID{peer.ID()}
	}
	if peer.GetRemovalFailed() {
		return ErrPeerRemoval{}
	}

	index := len(ps.list)
	// Appending is safe even with other goroutines
	// iterating over the ps.list slice.
	ps.list = append(ps.list, peer)
	ps.lookup[peer.ID()] = &peerSetItem{peer, index}
	return nil
}

// Has returns true if the set contains the peer referred to by this
// peerKey, otherwise false.
func (ps *PeerSet) Has(peerKey ID) bool {
	ps.mtx.Lock()
	_, ok := ps.lookup[peerKey]
	ps.mtx.Unlock()
	return ok
}

// HasIP returns true if the set contains the peer referred to by this IP
// address, otherwise false.
func (ps *PeerSet) HasIP(peerIP net.IP) bool {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()

	for _, peer := range ps.list {
		if peer.RemoteIP().Equal(peerIP) {
			return true
		}
	}

	return false
}

// Get looks up a peer by the provided peerKey. Returns nil if peer is not
// found.
func (ps *PeerSet) Get(peerKey ID) Peer {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()

	item, ok := ps.lookup[peerKey]
	if ok {
		return item.peer
	}
	return nil
}

// Remove removes the peer from the PeerSet.
func (ps *PeerSet) Remove(peer Peer) bool {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()

	item, ok := ps.lookup[peer.ID()]
	if !ok || len(ps.list) == 0 {
		// Removing the peer has failed so we set a flag to mark that a removal was attempted.
		// This can happen when the peer add routine from the switch is running in
		// parallel to the receive routine of MConn.
		// There is an error within MConn but the switch has not actually added the peer to the peer set yet.
		// Setting this flag will prevent a peer from being added to a node's peer set afterwards.
		peer.SetRemovalFailed()
		return false
	}
	index := item.index

	// Remove from ps.lookup.
	delete(ps.lookup, peer.ID())

	// If it's not the last item.
	if index != len(ps.list)-1 {
		// Swap it with the last item.
		lastPeer := ps.list[len(ps.list)-1]
		item := ps.lookup[lastPeer.ID()]
		item.index = index
		ps.list[index] = item.peer
	}

	// Remove the last item from ps.list.
	ps.list[len(ps.list)-1] = nil // nil the last entry of the slice to shorten, so it isn't reachable & can be GC'd.
	ps.list = ps.list[:len(ps.list)-1]

	return true
}

// Size returns the number of unique items in the peerSet.
func (ps *PeerSet) Size() int {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()
	return len(ps.list)
}

// Copy returns the copy of the peers list.
//
// Note: there are no guarantees about the thread-safety of Peer objects.
func (ps *PeerSet) Copy() []Peer {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()

	c := make([]Peer, len(ps.list))
	copy(c, ps.list)
	return c
}

// ForEach iterates over the PeerSet and calls the given function for each peer.
func (ps *PeerSet) ForEach(fn func(peer Peer)) {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()

	for _, item := range ps.lookup {
		fn(item.peer)
	}
}

// Random returns a random peer from the PeerSet.
func (ps *PeerSet) Random() Peer {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()

	if len(ps.list) == 0 {
		return nil
	}

	return ps.list[cmtrand.Int()%len(ps.list)]
}
