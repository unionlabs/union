package p2p

import (
	"net"
	"sync"
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/cometbft/cometbft/crypto/ed25519"
	"github.com/cometbft/cometbft/libs/service"
)

// mockPeer for testing the PeerSet.
type mockPeer struct {
	service.BaseService
	ip net.IP
	id ID
}

func (mp *mockPeer) FlushStop()            { mp.Stop() } //nolint:errcheck // ignore error
func (*mockPeer) TrySend(Envelope) bool    { return true }
func (*mockPeer) Send(Envelope) bool       { return true }
func (*mockPeer) NodeInfo() NodeInfo       { return DefaultNodeInfo{} }
func (*mockPeer) Status() ConnectionStatus { return ConnectionStatus{} }
func (mp *mockPeer) ID() ID                { return mp.id }
func (*mockPeer) IsOutbound() bool         { return false }
func (*mockPeer) IsPersistent() bool       { return true }
func (*mockPeer) Get(s string) any         { return s }
func (*mockPeer) Set(string, any)          {}
func (mp *mockPeer) RemoteIP() net.IP      { return mp.ip }
func (*mockPeer) SocketAddr() *NetAddress  { return nil }
func (mp *mockPeer) RemoteAddr() net.Addr  { return &net.TCPAddr{IP: mp.ip, Port: 8800} }
func (*mockPeer) CloseConn() error         { return nil }
func (*mockPeer) SetRemovalFailed()        {}
func (*mockPeer) GetRemovalFailed() bool   { return false }

// Returns a mock peer.
func newMockPeer(ip net.IP) *mockPeer {
	if ip == nil {
		ip = net.IP{127, 0, 0, 1}
	}
	nodeKey := NodeKey{PrivKey: ed25519.GenPrivKey()}
	return &mockPeer{
		ip: ip,
		id: nodeKey.ID(),
	}
}

func TestPeerSetAddRemoveOne(t *testing.T) {
	peerSet := NewPeerSet()

	var peerList []Peer
	for i := 0; i < 5; i++ {
		p := newMockPeer(net.IP{127, 0, 0, byte(i)})
		if err := peerSet.Add(p); err != nil {
			t.Error(err)
		}
		peerList = append(peerList, p)
	}

	n := len(peerList)
	// 1. Test removing from the front
	for i, peerAtFront := range peerList {
		removed := peerSet.Remove(peerAtFront)
		assert.True(t, removed)
		wantSize := n - i - 1
		for j := 0; j < 2; j++ {
			assert.False(t, false, peerSet.Has(peerAtFront.ID()), "#%d Run #%d: failed to remove peer", i, j)
			assert.Equal(t, wantSize, peerSet.Size(), "#%d Run #%d: failed to remove peer and decrement size", i, j)
			// Test the route of removing the now non-existent element
			removed := peerSet.Remove(peerAtFront)
			assert.False(t, removed)
		}
	}

	// 2. Next we are testing removing the peer at the end
	// a) Replenish the peerSet
	for _, peer := range peerList {
		if err := peerSet.Add(peer); err != nil {
			t.Error(err)
		}
	}

	// b) In reverse, remove each element
	for i := n - 1; i >= 0; i-- {
		peerAtEnd := peerList[i]
		removed := peerSet.Remove(peerAtEnd)
		assert.True(t, removed)
		assert.False(t, false, peerSet.Has(peerAtEnd.ID()), "#%d: failed to remove item at end", i)
		assert.Equal(t, i, peerSet.Size(), "#%d: differing sizes after peerSet.Remove(atEndPeer)", i)
	}
}

func TestPeerSetAddRemoveMany(t *testing.T) {
	peerSet := NewPeerSet()

	peers := []Peer{}
	n := 100
	for i := 0; i < n; i++ {
		peer := newMockPeer(net.IP{127, 0, 0, byte(i)})
		if err := peerSet.Add(peer); err != nil {
			t.Errorf("failed to add new peer")
		}
		if peerSet.Size() != i+1 {
			t.Errorf("failed to add new peer and increment size")
		}
		peers = append(peers, peer)
	}

	for i, peer := range peers {
		removed := peerSet.Remove(peer)
		assert.True(t, removed)
		if peerSet.Has(peer.ID()) {
			t.Errorf("failed to remove peer")
		}
		if peerSet.Size() != len(peers)-i-1 {
			t.Errorf("failed to remove peer and decrement size")
		}
	}
}

func TestPeerSetAddDuplicate(t *testing.T) {
	peerSet := NewPeerSet()
	peer := newMockPeer(nil)

	n := 20
	errsChan := make(chan error)
	// Add the same asynchronously to test the
	// concurrent guarantees of our APIs, and
	// our expectation in the end is that only
	// one addition succeeded, but the rest are
	// instances of ErrSwitchDuplicatePeer.
	for i := 0; i < n; i++ {
		go func() {
			errsChan <- peerSet.Add(peer)
		}()
	}

	// Now collect and tally the results
	errsTally := make(map[string]int)
	for i := 0; i < n; i++ {
		err := <-errsChan

		switch err.(type) {
		case ErrSwitchDuplicatePeerID:
			errsTally["duplicateID"]++
		default:
			errsTally["other"]++
		}
	}

	// Our next procedure is to ensure that only one addition
	// succeeded and that the rest are each ErrSwitchDuplicatePeer.
	wantErrCount, gotErrCount := n-1, errsTally["duplicateID"]
	assert.Equal(t, wantErrCount, gotErrCount, "invalid ErrSwitchDuplicatePeer count")

	wantNilErrCount, gotNilErrCount := 1, errsTally["other"]
	assert.Equal(t, wantNilErrCount, gotNilErrCount, "invalid nil errCount")
}

func TestPeerSetGet(t *testing.T) {
	var (
		peerSet = NewPeerSet()
		peer    = newMockPeer(nil)
	)

	assert.Nil(t, peerSet.Get(peer.ID()), "expecting a nil lookup, before .Add")

	if err := peerSet.Add(peer); err != nil {
		t.Fatalf("Failed to add new peer: %v", err)
	}

	var wg sync.WaitGroup
	for i := 0; i < 10; i++ {
		// Add them asynchronously to test the
		// concurrent guarantees of our APIs.
		wg.Add(1)
		go func(i int) {
			defer wg.Done()
			have, want := peerSet.Get(peer.ID()), peer
			assert.Equal(t, want, have, "%d: have %v, want %v", i, want, have)
		}(i)
	}
	wg.Wait()
}
