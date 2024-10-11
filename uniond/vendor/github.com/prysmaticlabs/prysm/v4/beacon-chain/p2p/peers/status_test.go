package peers_test

import (
	"context"
	"crypto/rand"
	"strconv"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/p2p/enr"
	"github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/peer"
	ma "github.com/multiformats/go-multiaddr"
	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers/peerdata"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers/scorers"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/wrapper"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/eth/v1"
	pb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestStatus(t *testing.T) {
	maxBadResponses := 2
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: maxBadResponses,
			},
		},
	})
	require.NotNil(t, p, "p not created")
	assert.Equal(t, maxBadResponses, p.Scorers().BadResponsesScorer().Params().Threshold, "maxBadResponses incorrect value")
}

func TestPeerExplicitAdd(t *testing.T) {
	maxBadResponses := 2
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: maxBadResponses,
			},
		},
	})

	id, err := peer.Decode("16Uiu2HAkyWZ4Ni1TpvDS8dPxsozmHY85KaiFjodQuV6Tz5tkHVeR")
	require.NoError(t, err, "Failed to create ID")
	address, err := ma.NewMultiaddr("/ip4/213.202.254.180/tcp/13000")
	require.NoError(t, err, "Failed to create address")
	direction := network.DirInbound
	p.Add(new(enr.Record), id, address, direction)

	resAddress, err := p.Address(id)
	require.NoError(t, err)
	assert.Equal(t, address, resAddress, "Unexpected address")

	resDirection, err := p.Direction(id)
	require.NoError(t, err)
	assert.Equal(t, direction, resDirection, "Unexpected direction")

	// Update with another explicit add
	address2, err := ma.NewMultiaddr("/ip4/52.23.23.253/tcp/30000/ipfs/QmfAgkmjiZNZhr2wFN9TwaRgHouMTBT6HELyzE5A3BT2wK/p2p-circuit")
	require.NoError(t, err)
	direction2 := network.DirOutbound
	p.Add(new(enr.Record), id, address2, direction2)

	resAddress2, err := p.Address(id)
	require.NoError(t, err)
	assert.Equal(t, address2, resAddress2, "Unexpected address")

	resDirection2, err := p.Direction(id)
	require.NoError(t, err)
	assert.Equal(t, direction2, resDirection2, "Unexpected direction")
}

func TestPeerNoENR(t *testing.T) {
	maxBadResponses := 2
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: maxBadResponses,
			},
		},
	})

	id, err := peer.Decode("16Uiu2HAkyWZ4Ni1TpvDS8dPxsozmHY85KaiFjodQuV6Tz5tkHVeR")
	require.NoError(t, err, "Failed to create ID")
	address, err := ma.NewMultiaddr("/ip4/213.202.254.180/tcp/13000")
	require.NoError(t, err, "Failed to create address")
	direction := network.DirInbound
	p.Add(nil, id, address, direction)

	retrievedENR, err := p.ENR(id)
	require.NoError(t, err, "Could not retrieve chainstate")
	var nilENR *enr.Record
	assert.Equal(t, nilENR, retrievedENR, "Wanted a nil enr to be saved")
}

func TestPeerNoOverwriteENR(t *testing.T) {
	maxBadResponses := 2
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: maxBadResponses,
			},
		},
	})

	id, err := peer.Decode("16Uiu2HAkyWZ4Ni1TpvDS8dPxsozmHY85KaiFjodQuV6Tz5tkHVeR")
	require.NoError(t, err, "Failed to create ID")
	address, err := ma.NewMultiaddr("/ip4/213.202.254.180/tcp/13000")
	require.NoError(t, err, "Failed to create address")
	direction := network.DirInbound
	record := new(enr.Record)
	record.Set(enr.WithEntry("test", []byte{'a'}))
	p.Add(record, id, address, direction)
	// try to overwrite
	p.Add(nil, id, address, direction)

	retrievedENR, err := p.ENR(id)
	require.NoError(t, err, "Could not retrieve chainstate")
	require.NotNil(t, retrievedENR, "Wanted a non-nil enr")
}

func TestErrUnknownPeer(t *testing.T) {
	maxBadResponses := 2
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: maxBadResponses,
			},
		},
	})

	id, err := peer.Decode("16Uiu2HAkyWZ4Ni1TpvDS8dPxsozmHY85KaiFjodQuV6Tz5tkHVeR")
	require.NoError(t, err)

	_, err = p.Address(id)
	assert.ErrorContains(t, peerdata.ErrPeerUnknown.Error(), err)

	_, err = p.Direction(id)
	assert.ErrorContains(t, peerdata.ErrPeerUnknown.Error(), err)

	_, err = p.ChainState(id)
	assert.ErrorContains(t, peerdata.ErrPeerUnknown.Error(), err)

	_, err = p.ConnectionState(id)
	assert.ErrorContains(t, peerdata.ErrPeerUnknown.Error(), err)

	_, err = p.ChainStateLastUpdated(id)
	assert.ErrorContains(t, peerdata.ErrPeerUnknown.Error(), err)

	_, err = p.Scorers().BadResponsesScorer().Count(id)
	assert.ErrorContains(t, peerdata.ErrPeerUnknown.Error(), err)
}

func TestPeerCommitteeIndices(t *testing.T) {
	maxBadResponses := 2
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: maxBadResponses,
			},
		},
	})

	id, err := peer.Decode("16Uiu2HAkyWZ4Ni1TpvDS8dPxsozmHY85KaiFjodQuV6Tz5tkHVeR")
	require.NoError(t, err, "Failed to create ID")
	address, err := ma.NewMultiaddr("/ip4/213.202.254.180/tcp/13000")
	require.NoError(t, err, "Failed to create address")
	direction := network.DirInbound
	record := new(enr.Record)
	record.Set(enr.WithEntry("test", []byte{'a'}))
	p.Add(record, id, address, direction)
	bitV := bitfield.NewBitvector64()
	for i := 0; i < 64; i++ {
		if i == 2 || i == 8 || i == 9 {
			bitV.SetBitAt(uint64(i), true)
		}
	}
	p.SetMetadata(id, wrapper.WrappedMetadataV0(&pb.MetaDataV0{
		SeqNumber: 2,
		Attnets:   bitV,
	}))

	wantedIndices := []uint64{2, 8, 9}

	indices, err := p.CommitteeIndices(id)
	require.NoError(t, err, "Could not retrieve committee indices")
	assert.DeepEqual(t, wantedIndices, indices)
}

func TestPeerSubscribedToSubnet(t *testing.T) {
	maxBadResponses := 2
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: maxBadResponses,
			},
		},
	})

	// Add some peers with different states
	numPeers := 2
	for i := 0; i < numPeers; i++ {
		addPeer(t, p, peers.PeerConnected)
	}
	expectedPeer := p.All()[1]
	bitV := bitfield.NewBitvector64()
	for i := 0; i < 64; i++ {
		if i == 2 || i == 8 || i == 9 {
			bitV.SetBitAt(uint64(i), true)
		}
	}
	p.SetMetadata(expectedPeer, wrapper.WrappedMetadataV0(&pb.MetaDataV0{
		SeqNumber: 2,
		Attnets:   bitV,
	}))
	numPeers = 3
	for i := 0; i < numPeers; i++ {
		addPeer(t, p, peers.PeerDisconnected)
	}
	ps := p.SubscribedToSubnet(2)
	assert.Equal(t, 1, len(ps), "Unexpected num of peers")
	assert.Equal(t, expectedPeer, ps[0])

	ps = p.SubscribedToSubnet(8)
	assert.Equal(t, 1, len(ps), "Unexpected num of peers")
	assert.Equal(t, expectedPeer, ps[0])

	ps = p.SubscribedToSubnet(9)
	assert.Equal(t, 1, len(ps), "Unexpected num of peers")
	assert.Equal(t, expectedPeer, ps[0])
}

func TestPeerImplicitAdd(t *testing.T) {
	maxBadResponses := 2
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: maxBadResponses,
			},
		},
	})

	id, err := peer.Decode("16Uiu2HAkyWZ4Ni1TpvDS8dPxsozmHY85KaiFjodQuV6Tz5tkHVeR")
	require.NoError(t, err)

	connectionState := peers.PeerConnecting
	p.SetConnectionState(id, connectionState)

	resConnectionState, err := p.ConnectionState(id)
	require.NoError(t, err)

	assert.Equal(t, connectionState, resConnectionState, "Unexpected connection state")
}

func TestPeerChainState(t *testing.T) {
	maxBadResponses := 2
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: maxBadResponses,
			},
		},
	})

	id, err := peer.Decode("16Uiu2HAkyWZ4Ni1TpvDS8dPxsozmHY85KaiFjodQuV6Tz5tkHVeR")
	require.NoError(t, err)
	address, err := ma.NewMultiaddr("/ip4/213.202.254.180/tcp/13000")
	require.NoError(t, err, "Failed to create address")
	direction := network.DirInbound
	p.Add(new(enr.Record), id, address, direction)

	oldChainStartLastUpdated, err := p.ChainStateLastUpdated(id)
	require.NoError(t, err)

	finalizedEpoch := primitives.Epoch(123)
	p.SetChainState(id, &pb.Status{FinalizedEpoch: finalizedEpoch})

	resChainState, err := p.ChainState(id)
	require.NoError(t, err)
	assert.Equal(t, finalizedEpoch, resChainState.FinalizedEpoch, "Unexpected finalized epoch")

	newChainStartLastUpdated, err := p.ChainStateLastUpdated(id)
	require.NoError(t, err)
	if !newChainStartLastUpdated.After(oldChainStartLastUpdated) {
		t.Errorf("Last updated did not increase: old %v new %v", oldChainStartLastUpdated, newChainStartLastUpdated)
	}
}

func TestPeerWithNilChainState(t *testing.T) {
	maxBadResponses := 2
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: maxBadResponses,
			},
		},
	})

	id, err := peer.Decode("16Uiu2HAkyWZ4Ni1TpvDS8dPxsozmHY85KaiFjodQuV6Tz5tkHVeR")
	require.NoError(t, err)
	address, err := ma.NewMultiaddr("/ip4/213.202.254.180/tcp/13000")
	require.NoError(t, err, "Failed to create address")
	direction := network.DirInbound
	p.Add(new(enr.Record), id, address, direction)

	p.SetChainState(id, nil)

	resChainState, err := p.ChainState(id)
	require.Equal(t, peerdata.ErrNoPeerStatus, err)
	var nothing *pb.Status
	require.Equal(t, resChainState, nothing)
}

func TestPeerBadResponses(t *testing.T) {
	maxBadResponses := 2
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: maxBadResponses,
			},
		},
	})

	id, err := peer.Decode("16Uiu2HAkyWZ4Ni1TpvDS8dPxsozmHY85KaiFjodQuV6Tz5tkHVeR")
	require.NoError(t, err)
	{
		_, err := id.MarshalBinary()
		require.NoError(t, err)
	}

	assert.Equal(t, false, p.IsBad(id), "Peer marked as bad when should be good")

	address, err := ma.NewMultiaddr("/ip4/213.202.254.180/tcp/13000")
	require.NoError(t, err, "Failed to create address")
	direction := network.DirInbound
	p.Add(new(enr.Record), id, address, direction)

	scorer := p.Scorers().BadResponsesScorer()
	resBadResponses, err := scorer.Count(id)
	require.NoError(t, err)
	assert.Equal(t, 0, resBadResponses, "Unexpected bad responses")
	assert.Equal(t, false, p.IsBad(id), "Peer marked as bad when should be good")

	scorer.Increment(id)
	resBadResponses, err = scorer.Count(id)
	require.NoError(t, err)
	assert.Equal(t, 1, resBadResponses, "Unexpected bad responses")
	assert.Equal(t, false, p.IsBad(id), "Peer marked as bad when should be good")

	scorer.Increment(id)
	resBadResponses, err = scorer.Count(id)
	require.NoError(t, err)
	assert.Equal(t, 2, resBadResponses, "Unexpected bad responses")
	assert.Equal(t, true, p.IsBad(id), "Peer not marked as bad when it should be")

	scorer.Increment(id)
	resBadResponses, err = scorer.Count(id)
	require.NoError(t, err)
	assert.Equal(t, 3, resBadResponses, "Unexpected bad responses")
	assert.Equal(t, true, p.IsBad(id), "Peer not marked as bad when it should be")
}

func TestAddMetaData(t *testing.T) {
	maxBadResponses := 2
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: maxBadResponses,
			},
		},
	})

	// Add some peers with different states
	numPeers := 5
	for i := 0; i < numPeers; i++ {
		addPeer(t, p, peers.PeerConnected)
	}
	newPeer := p.All()[2]

	newMetaData := &pb.MetaDataV0{
		SeqNumber: 8,
		Attnets:   bitfield.NewBitvector64(),
	}
	p.SetMetadata(newPeer, wrapper.WrappedMetadataV0(newMetaData))

	md, err := p.Metadata(newPeer)
	require.NoError(t, err)
	assert.Equal(t, newMetaData.SeqNumber, md.SequenceNumber(), "Unexpected sequence number")
}

func TestPeerConnectionStatuses(t *testing.T) {
	maxBadResponses := 2
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: maxBadResponses,
			},
		},
	})

	// Add some peers with different states
	numPeersDisconnected := 11
	for i := 0; i < numPeersDisconnected; i++ {
		addPeer(t, p, peers.PeerDisconnected)
	}
	numPeersConnecting := 7
	for i := 0; i < numPeersConnecting; i++ {
		addPeer(t, p, peers.PeerConnecting)
	}
	numPeersConnected := 43
	for i := 0; i < numPeersConnected; i++ {
		addPeer(t, p, peers.PeerConnected)
	}
	numPeersDisconnecting := 4
	for i := 0; i < numPeersDisconnecting; i++ {
		addPeer(t, p, peers.PeerDisconnecting)
	}

	// Now confirm the states
	assert.Equal(t, numPeersDisconnected, len(p.Disconnected()), "Unexpected number of disconnected peers")
	assert.Equal(t, numPeersConnecting, len(p.Connecting()), "Unexpected number of connecting peers")
	assert.Equal(t, numPeersConnected, len(p.Connected()), "Unexpected number of connected peers")
	assert.Equal(t, numPeersDisconnecting, len(p.Disconnecting()), "Unexpected number of disconnecting peers")
	numPeersActive := numPeersConnecting + numPeersConnected
	assert.Equal(t, numPeersActive, len(p.Active()), "Unexpected number of active peers")
	numPeersInactive := numPeersDisconnecting + numPeersDisconnected
	assert.Equal(t, numPeersInactive, len(p.Inactive()), "Unexpected number of inactive peers")
	numPeersAll := numPeersActive + numPeersInactive
	assert.Equal(t, numPeersAll, len(p.All()), "Unexpected number of peers")
}

func TestPeerValidTime(t *testing.T) {
	maxBadResponses := 2
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: maxBadResponses,
			},
		},
	})

	numPeersConnected := 6
	for i := 0; i < numPeersConnected; i++ {
		addPeer(t, p, peers.PeerConnected)
	}

	allPeers := p.All()

	// Add for 1st peer
	p.SetNextValidTime(allPeers[0], time.Now().Add(-1*time.Second))
	p.SetNextValidTime(allPeers[1], time.Now().Add(1*time.Second))
	p.SetNextValidTime(allPeers[2], time.Now().Add(10*time.Second))

	assert.Equal(t, true, p.IsReadyToDial(allPeers[0]))
	assert.Equal(t, false, p.IsReadyToDial(allPeers[1]))
	assert.Equal(t, false, p.IsReadyToDial(allPeers[2]))

	nextVal, err := p.NextValidTime(allPeers[3])
	require.NoError(t, err)
	assert.Equal(t, true, nextVal.IsZero())
	assert.Equal(t, true, p.IsReadyToDial(allPeers[3]))

	nextVal, err = p.NextValidTime(allPeers[4])
	require.NoError(t, err)
	assert.Equal(t, true, nextVal.IsZero())
	assert.Equal(t, true, p.IsReadyToDial(allPeers[4]))

	nextVal, err = p.NextValidTime(allPeers[5])
	require.NoError(t, err)
	assert.Equal(t, true, nextVal.IsZero())
	assert.Equal(t, true, p.IsReadyToDial(allPeers[5]))

	// Now confirm the states
	assert.Equal(t, numPeersConnected, len(p.Connected()), "Unexpected number of connected peers")
}

func TestPrune(t *testing.T) {
	maxBadResponses := 2
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: maxBadResponses,
			},
		},
	})

	for i := 0; i < p.MaxPeerLimit()+100; i++ {
		if i%7 == 0 {
			// Peer added as disconnected.
			_ = addPeer(t, p, peers.PeerDisconnected)
		}
		// Peer added to peer handler.
		_ = addPeer(t, p, peers.PeerConnected)
	}

	disPeers := p.Disconnected()
	firstPID := disPeers[0]
	secondPID := disPeers[1]
	thirdPID := disPeers[2]

	scorer := p.Scorers().BadResponsesScorer()

	// Make first peer a bad peer
	scorer.Increment(firstPID)
	scorer.Increment(firstPID)

	// Add bad response for p2.
	scorer.Increment(secondPID)

	// Prune peers
	p.Prune()

	// Bad peer is expected to still be kept in handler.
	badRes, err := scorer.Count(firstPID)
	assert.NoError(t, err, "error is supposed to be  nil")
	assert.Equal(t, 2, badRes, "Did not get expected amount")

	// Not so good peer is pruned away so that we can reduce the
	// total size of the handler.
	_, err = scorer.Count(secondPID)
	assert.ErrorContains(t, "peer unknown", err)

	// Last peer has been removed.
	_, err = scorer.Count(thirdPID)
	assert.ErrorContains(t, "peer unknown", err)
}

func TestPeerIPTracker(t *testing.T) {
	resetCfg := features.InitWithReset(&features.Flags{
		EnablePeerScorer: false,
	})
	defer resetCfg()
	maxBadResponses := 2
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: maxBadResponses,
			},
		},
	})

	badIP := "211.227.218.116"
	var badPeers []peer.ID
	for i := 0; i < peers.ColocationLimit+10; i++ {
		port := strconv.Itoa(3000 + i)
		addr, err := ma.NewMultiaddr("/ip4/" + badIP + "/tcp/" + port)
		if err != nil {
			t.Fatal(err)
		}
		badPeers = append(badPeers, createPeer(t, p, addr, network.DirUnknown, peerdata.PeerConnectionState(ethpb.ConnectionState_DISCONNECTED)))
	}
	for _, pr := range badPeers {
		assert.Equal(t, true, p.IsBad(pr), "peer with bad ip is not bad")
	}

	// Add in bad peers, so that our records are trimmed out
	// from the peer store.
	for i := 0; i < p.MaxPeerLimit()+100; i++ {
		// Peer added to peer handler.
		pid := addPeer(t, p, peers.PeerDisconnected)
		p.Scorers().BadResponsesScorer().Increment(pid)
	}
	p.Prune()

	for _, pr := range badPeers {
		assert.Equal(t, false, p.IsBad(pr), "peer with good ip is regarded as bad")
	}
}

func TestTrimmedOrderedPeers(t *testing.T) {
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: 1,
			},
		},
	})

	expectedTarget := primitives.Epoch(2)
	maxPeers := 3
	var mockroot2 [32]byte
	var mockroot3 [32]byte
	var mockroot4 [32]byte
	var mockroot5 [32]byte
	copy(mockroot2[:], "two")
	copy(mockroot3[:], "three")
	copy(mockroot4[:], "four")
	copy(mockroot5[:], "five")
	// Peer 1
	pid1 := addPeer(t, p, peers.PeerConnected)
	p.SetChainState(pid1, &pb.Status{
		HeadSlot:       3 * params.BeaconConfig().SlotsPerEpoch,
		FinalizedEpoch: 3,
		FinalizedRoot:  mockroot3[:],
	})
	// Peer 2
	pid2 := addPeer(t, p, peers.PeerConnected)
	p.SetChainState(pid2, &pb.Status{
		HeadSlot:       4 * params.BeaconConfig().SlotsPerEpoch,
		FinalizedEpoch: 4,
		FinalizedRoot:  mockroot4[:],
	})
	// Peer 3
	pid3 := addPeer(t, p, peers.PeerConnected)
	p.SetChainState(pid3, &pb.Status{
		HeadSlot:       5 * params.BeaconConfig().SlotsPerEpoch,
		FinalizedEpoch: 5,
		FinalizedRoot:  mockroot5[:],
	})
	// Peer 4
	pid4 := addPeer(t, p, peers.PeerConnected)
	p.SetChainState(pid4, &pb.Status{
		HeadSlot:       2 * params.BeaconConfig().SlotsPerEpoch,
		FinalizedEpoch: 2,
		FinalizedRoot:  mockroot2[:],
	})
	// Peer 5
	pid5 := addPeer(t, p, peers.PeerConnected)
	p.SetChainState(pid5, &pb.Status{
		HeadSlot:       2 * params.BeaconConfig().SlotsPerEpoch,
		FinalizedEpoch: 2,
		FinalizedRoot:  mockroot2[:],
	})

	target, pids := p.BestFinalized(maxPeers, 0)
	assert.Equal(t, expectedTarget, target, "Incorrect target epoch retrieved")
	assert.Equal(t, maxPeers, len(pids), "Incorrect number of peers retrieved")

	// Expect the returned list to be ordered by finalized epoch and trimmed to max peers.
	assert.Equal(t, pid3, pids[0], "Incorrect first peer")
	assert.Equal(t, pid2, pids[1], "Incorrect second peer")
	assert.Equal(t, pid1, pids[2], "Incorrect third peer")
}

func TestConcurrentPeerLimitHolds(t *testing.T) {
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: 1,
			},
		},
	})
	assert.Equal(t, true, uint64(p.MaxPeerLimit()) > p.ConnectedPeerLimit(), "max peer limit doesn't exceed connected peer limit")
}

func TestAtInboundPeerLimit(t *testing.T) {
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: 1,
			},
		},
	})
	for i := 0; i < 15; i++ {
		// Peer added to peer handler.
		createPeer(t, p, nil, network.DirOutbound, peerdata.PeerConnectionState(ethpb.ConnectionState_CONNECTED))
	}
	assert.Equal(t, false, p.IsAboveInboundLimit(), "Inbound limit exceeded")
	for i := 0; i < 31; i++ {
		// Peer added to peer handler.
		createPeer(t, p, nil, network.DirInbound, peerdata.PeerConnectionState(ethpb.ConnectionState_CONNECTED))
	}
	assert.Equal(t, true, p.IsAboveInboundLimit(), "Inbound limit not exceeded")
}

func TestPrunePeers(t *testing.T) {
	resetCfg := features.InitWithReset(&features.Flags{
		EnablePeerScorer: false,
	})
	defer resetCfg()
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: 1,
			},
		},
	})
	for i := 0; i < 15; i++ {
		// Peer added to peer handler.
		createPeer(t, p, nil, network.DirOutbound, peerdata.PeerConnectionState(ethpb.ConnectionState_CONNECTED))
	}
	// Assert there are no prunable peers.
	peersToPrune := p.PeersToPrune()
	assert.Equal(t, 0, len(peersToPrune))

	for i := 0; i < 18; i++ {
		// Peer added to peer handler.
		createPeer(t, p, nil, network.DirInbound, peerdata.PeerConnectionState(ethpb.ConnectionState_CONNECTED))
	}

	// Assert there are the correct prunable peers.
	peersToPrune = p.PeersToPrune()
	assert.Equal(t, 3, len(peersToPrune))

	// Add in more peers.
	for i := 0; i < 13; i++ {
		// Peer added to peer handler.
		createPeer(t, p, nil, network.DirInbound, peerdata.PeerConnectionState(ethpb.ConnectionState_CONNECTED))
	}

	// Set up bad scores for inbound peers.
	inboundPeers := p.InboundConnected()
	for i, pid := range inboundPeers {
		modulo := i % 5
		// Increment bad scores for peers.
		for j := 0; j < modulo; j++ {
			p.Scorers().BadResponsesScorer().Increment(pid)
		}
	}
	// Assert all peers more than max are prunable.
	peersToPrune = p.PeersToPrune()
	assert.Equal(t, 16, len(peersToPrune))
	for _, pid := range peersToPrune {
		dir, err := p.Direction(pid)
		require.NoError(t, err)
		assert.Equal(t, network.DirInbound, dir)
	}

	// Ensure it is in the descending order.
	currCount, err := p.Scorers().BadResponsesScorer().Count(peersToPrune[0])
	require.NoError(t, err)
	for _, pid := range peersToPrune {
		count, err := p.Scorers().BadResponsesScorer().Count(pid)
		require.NoError(t, err)
		assert.Equal(t, true, currCount >= count)
		currCount = count
	}
}

func TestPrunePeers_TrustedPeers(t *testing.T) {
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: 1,
			},
		},
	})

	for i := 0; i < 15; i++ {
		// Peer added to peer handler.
		createPeer(t, p, nil, network.DirOutbound, peerdata.PeerConnectionState(ethpb.ConnectionState_CONNECTED))
	}
	// Assert there are no prunable peers.
	peersToPrune := p.PeersToPrune()
	assert.Equal(t, 0, len(peersToPrune))

	for i := 0; i < 18; i++ {
		// Peer added to peer handler.
		createPeer(t, p, nil, network.DirInbound, peerdata.PeerConnectionState(ethpb.ConnectionState_CONNECTED))
	}

	// Assert there are the correct prunable peers.
	peersToPrune = p.PeersToPrune()
	assert.Equal(t, 3, len(peersToPrune))

	// Add in more peers.
	for i := 0; i < 13; i++ {
		// Peer added to peer handler.
		createPeer(t, p, nil, network.DirInbound, peerdata.PeerConnectionState(ethpb.ConnectionState_CONNECTED))
	}

	var trustedPeers []peer.ID
	// Set up bad scores for inbound peers.
	inboundPeers := p.InboundConnected()
	for i, pid := range inboundPeers {
		modulo := i % 5
		// Increment bad scores for peers.
		for j := 0; j < modulo; j++ {
			p.Scorers().BadResponsesScorer().Increment(pid)
		}
		if modulo == 4 {
			trustedPeers = append(trustedPeers, pid)
		}
	}
	p.SetTrustedPeers(trustedPeers)

	// Assert we have correct trusted peers
	trustedPeers = p.GetTrustedPeers()
	assert.Equal(t, 6, len(trustedPeers))

	// Assert all peers more than max are prunable.
	peersToPrune = p.PeersToPrune()
	assert.Equal(t, 16, len(peersToPrune))

	// Check that trusted peers are not pruned.
	for _, pid := range peersToPrune {
		for _, tPid := range trustedPeers {
			assert.NotEqual(t, pid.String(), tPid.String())
		}
	}

	// Add more peers to check if trusted peers can be pruned after they are deleted from trusted peer set.
	for i := 0; i < 9; i++ {
		// Peer added to peer handler.
		createPeer(t, p, nil, network.DirInbound, peerdata.PeerConnectionState(ethpb.ConnectionState_CONNECTED))
	}

	// Delete trusted peers.
	p.DeleteTrustedPeers(trustedPeers)

	peersToPrune = p.PeersToPrune()
	assert.Equal(t, 25, len(peersToPrune))

	// Check that trusted peers are pruned.
	for _, tPid := range trustedPeers {
		pruned := false
		for _, pid := range peersToPrune {
			if pid.String() == tPid.String() {
				pruned = true
			}
		}
		assert.Equal(t, true, pruned)
	}

	// Assert have zero trusted peers
	trustedPeers = p.GetTrustedPeers()
	assert.Equal(t, 0, len(trustedPeers))

	for _, pid := range peersToPrune {
		dir, err := p.Direction(pid)
		require.NoError(t, err)
		assert.Equal(t, network.DirInbound, dir)
	}

	// Ensure it is in the descending order.
	currScore := p.Scorers().Score(peersToPrune[0])
	for _, pid := range peersToPrune {
		score := p.Scorers().Score(pid)
		assert.Equal(t, true, currScore <= score)
		currScore = score
	}
}

func TestStatus_BestPeer(t *testing.T) {
	type peerConfig struct {
		headSlot       primitives.Slot
		finalizedEpoch primitives.Epoch
	}
	tests := []struct {
		name              string
		peers             []*peerConfig
		limitPeers        int
		ourFinalizedEpoch primitives.Epoch
		targetEpoch       primitives.Epoch
		// targetEpochSupport denotes how many peers support returned epoch.
		targetEpochSupport int
	}{
		{
			name: "head slot matches finalized epoch",
			peers: []*peerConfig{
				{finalizedEpoch: 4, headSlot: 4 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 4, headSlot: 4 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 3, headSlot: 3 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 4, headSlot: 4 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 4, headSlot: 4 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 3, headSlot: 3 * params.BeaconConfig().SlotsPerEpoch},
			},
			limitPeers:         15,
			targetEpoch:        4,
			targetEpochSupport: 4,
		},
		{
			// Peers are compared using their finalized epoch, head should not affect peer selection.
			// Test case below is a regression case: to ensure that only epoch is used indeed.
			// (Function sorts peers, and on equal head slot, produced incorrect results).
			name: "head slots equal for peers with different finalized epochs",
			peers: []*peerConfig{
				{finalizedEpoch: 4, headSlot: 4 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 4, headSlot: 4 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 3, headSlot: 4 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 4, headSlot: 4 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 4, headSlot: 4 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 3, headSlot: 4 * params.BeaconConfig().SlotsPerEpoch},
			},
			limitPeers:         15,
			targetEpoch:        4,
			targetEpochSupport: 4,
		},
		{
			name: "head slot significantly ahead of finalized epoch (long period of non-finality)",
			peers: []*peerConfig{
				{finalizedEpoch: 4, headSlot: 42 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 4, headSlot: 42 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 3, headSlot: 42 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 4, headSlot: 42 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 4, headSlot: 42 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 3, headSlot: 42 * params.BeaconConfig().SlotsPerEpoch},
			},
			limitPeers:         15,
			targetEpoch:        4,
			targetEpochSupport: 4,
		},
		{
			name: "ignore lower epoch peers",
			peers: []*peerConfig{
				{finalizedEpoch: 4, headSlot: 41 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 4, headSlot: 42 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 3, headSlot: 43 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 4, headSlot: 44 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 4, headSlot: 45 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 3, headSlot: 46 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 6, headSlot: 6 * params.BeaconConfig().SlotsPerEpoch},
			},
			ourFinalizedEpoch:  5,
			limitPeers:         15,
			targetEpoch:        6,
			targetEpochSupport: 1,
		},
		{
			name: "combine peers from several epochs starting from epoch higher than ours",
			peers: []*peerConfig{
				{finalizedEpoch: 4, headSlot: 41 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 4, headSlot: 42 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 3, headSlot: 43 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 4, headSlot: 44 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 4, headSlot: 45 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 3, headSlot: 46 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 6, headSlot: 6 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 6, headSlot: 6 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 6, headSlot: 6 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 7, headSlot: 7 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 8, headSlot: 8 * params.BeaconConfig().SlotsPerEpoch},
			},
			ourFinalizedEpoch:  5,
			limitPeers:         15,
			targetEpoch:        6,
			targetEpochSupport: 5,
		},
		{
			name: "limit number of returned peers",
			peers: []*peerConfig{
				{finalizedEpoch: 4, headSlot: 41 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 4, headSlot: 42 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 3, headSlot: 43 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 4, headSlot: 44 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 4, headSlot: 45 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 3, headSlot: 46 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 6, headSlot: 6 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 6, headSlot: 6 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 6, headSlot: 6 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 7, headSlot: 7 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 8, headSlot: 8 * params.BeaconConfig().SlotsPerEpoch},
			},
			ourFinalizedEpoch:  5,
			limitPeers:         4,
			targetEpoch:        6,
			targetEpochSupport: 4,
		},
		{
			name: "handle epoch ties",
			peers: []*peerConfig{
				{finalizedEpoch: 6, headSlot: 6 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 6, headSlot: 6 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 6, headSlot: 6 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 7, headSlot: 7 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 8, headSlot: 8 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 8, headSlot: 8 * params.BeaconConfig().SlotsPerEpoch},
				{finalizedEpoch: 8, headSlot: 8 * params.BeaconConfig().SlotsPerEpoch},
			},
			ourFinalizedEpoch:  5,
			limitPeers:         15,
			targetEpoch:        8,
			targetEpochSupport: 3,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			p := peers.NewStatus(context.Background(), &peers.StatusConfig{
				PeerLimit: 30,
				ScorerParams: &scorers.Config{
					BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{Threshold: 2},
				},
			})
			for _, peerConfig := range tt.peers {
				p.SetChainState(addPeer(t, p, peers.PeerConnected), &pb.Status{
					FinalizedEpoch: peerConfig.finalizedEpoch,
					HeadSlot:       peerConfig.headSlot,
				})
			}
			epoch, pids := p.BestFinalized(tt.limitPeers, tt.ourFinalizedEpoch)
			assert.Equal(t, tt.targetEpoch, epoch, "Unexpected epoch retrieved")
			assert.Equal(t, tt.targetEpochSupport, len(pids), "Unexpected number of peers supporting retrieved epoch")
		})
	}
}

func TestBestFinalized_returnsMaxValue(t *testing.T) {
	maxBadResponses := 2
	maxPeers := 10
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: maxBadResponses,
			},
		},
	})

	for i := 0; i <= maxPeers+100; i++ {
		p.Add(new(enr.Record), peer.ID(rune(i)), nil, network.DirOutbound)
		p.SetConnectionState(peer.ID(rune(i)), peers.PeerConnected)
		p.SetChainState(peer.ID(rune(i)), &pb.Status{
			FinalizedEpoch: 10,
		})
	}

	_, pids := p.BestFinalized(maxPeers, 0)
	assert.Equal(t, maxPeers, len(pids), "Wrong number of peers returned")
}

func TestStatus_BestNonFinalized(t *testing.T) {
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: 2,
			},
		},
	})

	peerSlots := []primitives.Slot{32, 32, 32, 32, 235, 233, 258, 268, 270}
	for i, headSlot := range peerSlots {
		p.Add(new(enr.Record), peer.ID(rune(i)), nil, network.DirOutbound)
		p.SetConnectionState(peer.ID(rune(i)), peers.PeerConnected)
		p.SetChainState(peer.ID(rune(i)), &pb.Status{
			HeadSlot: headSlot,
		})
	}

	expectedEpoch := primitives.Epoch(8)
	retEpoch, pids := p.BestNonFinalized(3, 5)
	assert.Equal(t, expectedEpoch, retEpoch, "Incorrect Finalized epoch retrieved")
	assert.Equal(t, 3, len(pids), "Unexpected number of peers")
}

func TestStatus_CurrentEpoch(t *testing.T) {
	maxBadResponses := 2
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: maxBadResponses,
			},
		},
	})
	// Peer 1
	pid1 := addPeer(t, p, peers.PeerConnected)
	p.SetChainState(pid1, &pb.Status{
		HeadSlot: params.BeaconConfig().SlotsPerEpoch * 4,
	})
	// Peer 2
	pid2 := addPeer(t, p, peers.PeerConnected)
	p.SetChainState(pid2, &pb.Status{
		HeadSlot: params.BeaconConfig().SlotsPerEpoch * 5,
	})
	// Peer 3
	pid3 := addPeer(t, p, peers.PeerConnected)
	p.SetChainState(pid3, &pb.Status{
		HeadSlot: params.BeaconConfig().SlotsPerEpoch * 4,
	})

	assert.Equal(t, primitives.Epoch(5), p.HighestEpoch(), "Expected current epoch to be 5")
}

func TestInbound(t *testing.T) {
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: 0,
			},
		},
	})
	addr, err := ma.NewMultiaddr("/ip4/127.0.0.1/tcp/33333")
	require.NoError(t, err)
	inbound := createPeer(t, p, addr, network.DirInbound, peers.PeerConnected)
	createPeer(t, p, addr, network.DirOutbound, peers.PeerConnected)

	result := p.Inbound()
	require.Equal(t, 1, len(result))
	assert.Equal(t, inbound.String(), result[0].String())
}

func TestOutbound(t *testing.T) {
	p := peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: 0,
			},
		},
	})
	addr, err := ma.NewMultiaddr("/ip4/127.0.0.1/tcp/33333")
	require.NoError(t, err)
	createPeer(t, p, addr, network.DirInbound, peers.PeerConnected)
	outbound := createPeer(t, p, addr, network.DirOutbound, peers.PeerConnected)

	result := p.Outbound()
	require.Equal(t, 1, len(result))
	assert.Equal(t, outbound.String(), result[0].String())
}

// addPeer is a helper to add a peer with a given connection state)
func addPeer(t *testing.T, p *peers.Status, state peerdata.PeerConnectionState) peer.ID {
	// Set up some peers with different states
	mhBytes := []byte{0x11, 0x04}
	idBytes := make([]byte, 4)
	_, err := rand.Read(idBytes)
	require.NoError(t, err)
	mhBytes = append(mhBytes, idBytes...)
	id, err := peer.IDFromBytes(mhBytes)
	require.NoError(t, err)
	p.Add(new(enr.Record), id, nil, network.DirUnknown)
	p.SetConnectionState(id, state)
	p.SetMetadata(id, wrapper.WrappedMetadataV0(&pb.MetaDataV0{
		SeqNumber: 0,
		Attnets:   bitfield.NewBitvector64(),
	}))
	return id
}

func createPeer(t *testing.T, p *peers.Status, addr ma.Multiaddr,
	dir network.Direction, state peerdata.PeerConnectionState) peer.ID {
	mhBytes := []byte{0x11, 0x04}
	idBytes := make([]byte, 4)
	_, err := rand.Read(idBytes)
	require.NoError(t, err)
	mhBytes = append(mhBytes, idBytes...)
	id, err := peer.IDFromBytes(mhBytes)
	require.NoError(t, err)
	p.Add(new(enr.Record), id, addr, dir)
	p.SetConnectionState(id, state)
	return id
}
