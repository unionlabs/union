package peerdata_test

import (
	"context"
	"testing"

	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers/peerdata"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestStore_GetSetDelete(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	store := peerdata.NewStore(ctx, &peerdata.StoreConfig{
		MaxPeers: 12,
	})
	assert.NotNil(t, store)
	assert.Equal(t, 12, store.Config().MaxPeers)

	// Check non-existent data.
	pid := peer.ID("00001")
	peerData, ok := store.PeerData(pid)
	assert.Equal(t, false, ok)
	assert.Equal(t, (*peerdata.PeerData)(nil), peerData)
	assert.Equal(t, 0, len(store.Peers()))

	// Associate data.
	store.SetPeerData(pid, &peerdata.PeerData{
		BadResponses:    3,
		ProcessedBlocks: 42,
	})
	peerData, ok = store.PeerData(pid)
	assert.Equal(t, true, ok)
	assert.Equal(t, 3, peerData.BadResponses)
	assert.Equal(t, uint64(42), peerData.ProcessedBlocks)
	require.Equal(t, 1, len(store.Peers()))
	peers := store.Peers()
	_, ok = peers[pid]
	require.Equal(t, true, ok)
	assert.Equal(t, 3, peers[pid].BadResponses)
	assert.Equal(t, uint64(42), peers[pid].ProcessedBlocks)

	// Remove data from peer.
	store.DeletePeerData(pid)
	peerData, ok = store.PeerData(pid)
	assert.Equal(t, false, ok)
	assert.Equal(t, (*peerdata.PeerData)(nil), peerData)
	assert.Equal(t, 0, len(store.Peers()))
}

func TestStore_PeerDataGetOrCreate(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	store := peerdata.NewStore(ctx, &peerdata.StoreConfig{
		MaxPeers: 12,
	})
	assert.NotNil(t, store)
	assert.Equal(t, 12, store.Config().MaxPeers)

	pid := peer.ID("00001")
	peerData, ok := store.PeerData(pid)
	assert.Equal(t, false, ok)
	assert.Equal(t, (*peerdata.PeerData)(nil), peerData)
	assert.Equal(t, 0, len(store.Peers()))

	peerData = store.PeerDataGetOrCreate(pid)
	assert.NotNil(t, peerData)
	assert.Equal(t, 0, peerData.BadResponses)
	assert.Equal(t, uint64(0), peerData.ProcessedBlocks)
	require.Equal(t, 1, len(store.Peers()))

	// Method must be idempotent, check.
	peerData = store.PeerDataGetOrCreate(pid)
	assert.NotNil(t, peerData)
	assert.Equal(t, 0, peerData.BadResponses)
	assert.Equal(t, uint64(0), peerData.ProcessedBlocks)
	require.Equal(t, 1, len(store.Peers()))
}

func TestStore_TrustedPeers(t *testing.T) {
	store := peerdata.NewStore(context.Background(), &peerdata.StoreConfig{
		MaxPeers: 12,
	})

	pid1 := peer.ID("00001")
	pid2 := peer.ID("00002")
	pid3 := peer.ID("00003")

	tPeers := []peer.ID{pid1, pid2, pid3}
	store.SetTrustedPeers(tPeers)

	assert.Equal(t, true, store.IsTrustedPeer(pid1))
	assert.Equal(t, true, store.IsTrustedPeer(pid2))
	assert.Equal(t, true, store.IsTrustedPeer(pid3))

	tPeers = store.GetTrustedPeers()
	assert.Equal(t, 3, len(tPeers))

	store.DeleteTrustedPeers(tPeers)
	tPeers = store.GetTrustedPeers()
	assert.Equal(t, 0, len(tPeers))

	assert.Equal(t, false, store.IsTrustedPeer(pid1))
	assert.Equal(t, false, store.IsTrustedPeer(pid2))
	assert.Equal(t, false, store.IsTrustedPeer(pid3))

}
