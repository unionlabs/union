package testing

import (
	"context"
	"sync"

	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/enr"
	"github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/peer"
	ma "github.com/multiformats/go-multiaddr"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers/scorers"
	pb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	log "github.com/sirupsen/logrus"
)

// MockPeersProvider implements PeersProvider for testing.
type MockPeersProvider struct {
	lock  sync.Mutex
	peers *peers.Status
}

// ClearPeers removes all known peers.
func (m *MockPeersProvider) ClearPeers() {
	m.lock.Lock()
	defer m.lock.Unlock()
	m.peers = peers.NewStatus(context.Background(), &peers.StatusConfig{
		PeerLimit: 30,
		ScorerParams: &scorers.Config{
			BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
				Threshold: 5,
			},
		},
	})
}

// Peers provides access the peer status.
func (m *MockPeersProvider) Peers() *peers.Status {
	m.lock.Lock()
	defer m.lock.Unlock()
	if m.peers == nil {
		m.peers = peers.NewStatus(context.Background(), &peers.StatusConfig{
			PeerLimit: 30,
			ScorerParams: &scorers.Config{
				BadResponsesScorerConfig: &scorers.BadResponsesScorerConfig{
					Threshold: 5,
				},
			},
		})
		// Pretend we are connected to two peers
		id0, err := peer.Decode("16Uiu2HAkyWZ4Ni1TpvDS8dPxsozmHY85KaiFjodQuV6Tz5tkHVeR")
		if err != nil {
			log.WithError(err).Debug("Cannot decode")
		}
		ma0, err := ma.NewMultiaddr("/ip4/213.202.254.180/tcp/13000")
		if err != nil {
			log.WithError(err).Debug("Cannot decode")
		}
		m.peers.Add(createENR(), id0, ma0, network.DirInbound)
		m.peers.SetConnectionState(id0, peers.PeerConnected)
		m.peers.SetChainState(id0, &pb.Status{FinalizedEpoch: 10})
		id1, err := peer.Decode("16Uiu2HAm4HgJ9N1o222xK61o7LSgToYWoAy1wNTJRkh9gLZapVAy")
		if err != nil {
			log.WithError(err).Debug("Cannot decode")
		}
		ma1, err := ma.NewMultiaddr("/ip4/52.23.23.253/tcp/30000/ipfs/QmfAgkmjiZNZhr2wFN9TwaRgHouMTBT6HELyzE5A3BT2wK/p2p-circuit")
		if err != nil {
			log.WithError(err).Debug("Cannot decode")
		}
		m.peers.Add(createENR(), id1, ma1, network.DirOutbound)
		m.peers.SetConnectionState(id1, peers.PeerConnected)
		m.peers.SetChainState(id1, &pb.Status{FinalizedEpoch: 11})
	}
	return m.peers
}

func createENR() *enr.Record {
	key, err := crypto.GenerateKey()
	if err != nil {
		log.Error(err)
	}
	db, err := enode.OpenDB("")
	if err != nil {
		log.Error("could not open node's peer database")
	}
	lNode := enode.NewLocalNode(db, key)
	return lNode.Node().Record()
}
