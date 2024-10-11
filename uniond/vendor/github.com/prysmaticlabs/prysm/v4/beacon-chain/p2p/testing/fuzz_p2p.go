package testing

import (
	"context"

	"github.com/ethereum/go-ethereum/p2p/enr"
	pubsub "github.com/libp2p/go-libp2p-pubsub"
	"github.com/libp2p/go-libp2p/core/control"
	"github.com/libp2p/go-libp2p/core/host"
	"github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/multiformats/go-multiaddr"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/encoder"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/metadata"
	"google.golang.org/protobuf/proto"
)

// FakeP2P stack
type FakeP2P struct {
}

// NewFuzzTestP2P - Create a new fake p2p stack.
func NewFuzzTestP2P() *FakeP2P {
	return &FakeP2P{}
}

// Encoding -- fake.
func (_ *FakeP2P) Encoding() encoder.NetworkEncoding {
	return &encoder.SszNetworkEncoder{}
}

// AddConnectionHandler -- fake.
func (_ *FakeP2P) AddConnectionHandler(_, _ func(ctx context.Context, id peer.ID) error) {

}

// AddDisconnectionHandler -- fake.
func (_ *FakeP2P) AddDisconnectionHandler(_ func(ctx context.Context, id peer.ID) error) {
}

// AddPingMethod -- fake.
func (_ *FakeP2P) AddPingMethod(_ func(ctx context.Context, id peer.ID) error) {

}

// PeerID -- fake.
func (_ *FakeP2P) PeerID() peer.ID {
	return "fake"
}

// ENR returns the enr of the local peer.
func (_ *FakeP2P) ENR() *enr.Record {
	return new(enr.Record)
}

// DiscoveryAddresses -- fake
func (_ *FakeP2P) DiscoveryAddresses() ([]multiaddr.Multiaddr, error) {
	return nil, nil
}

// FindPeersWithSubnet mocks the p2p func.
func (_ *FakeP2P) FindPeersWithSubnet(_ context.Context, _ string, _ uint64, _ int) (bool, error) {
	return false, nil
}

// RefreshENR mocks the p2p func.
func (_ *FakeP2P) RefreshENR() {}

// LeaveTopic -- fake.
func (_ *FakeP2P) LeaveTopic(_ string) error {
	return nil
}

// Metadata -- fake.
func (_ *FakeP2P) Metadata() metadata.Metadata {
	return nil
}

// Peers -- fake.
func (_ *FakeP2P) Peers() *peers.Status {
	return nil
}

// PublishToTopic -- fake.
func (_ *FakeP2P) PublishToTopic(_ context.Context, _ string, _ []byte, _ ...pubsub.PubOpt) error {
	return nil
}

// Send -- fake.
func (_ *FakeP2P) Send(_ context.Context, _ interface{}, _ string, _ peer.ID) (network.Stream, error) {
	return nil, nil
}

// PubSub -- fake.
func (_ *FakeP2P) PubSub() *pubsub.PubSub {
	return nil
}

// MetadataSeq -- fake.
func (_ *FakeP2P) MetadataSeq() uint64 {
	return 0
}

// SetStreamHandler -- fake.
func (_ *FakeP2P) SetStreamHandler(_ string, _ network.StreamHandler) {

}

// SubscribeToTopic -- fake.
func (_ *FakeP2P) SubscribeToTopic(_ string, _ ...pubsub.SubOpt) (*pubsub.Subscription, error) {
	return nil, nil
}

// JoinTopic -- fake.
func (_ *FakeP2P) JoinTopic(_ string, _ ...pubsub.TopicOpt) (*pubsub.Topic, error) {
	return nil, nil
}

// Host -- fake.
func (_ *FakeP2P) Host() host.Host {
	return nil
}

// Disconnect -- fake.
func (_ *FakeP2P) Disconnect(_ peer.ID) error {
	return nil
}

// Broadcast -- fake.
func (_ *FakeP2P) Broadcast(_ context.Context, _ proto.Message) error {
	return nil
}

// BroadcastAttestation -- fake.
func (_ *FakeP2P) BroadcastAttestation(_ context.Context, _ uint64, _ *ethpb.Attestation) error {
	return nil
}

// BroadcastSyncCommitteeMessage -- fake.
func (_ *FakeP2P) BroadcastSyncCommitteeMessage(_ context.Context, _ uint64, _ *ethpb.SyncCommitteeMessage) error {
	return nil
}

// BroadcastBlob -- fake.
func (_ *FakeP2P) BroadcastBlob(_ context.Context, _ uint64, _ *ethpb.BlobSidecar) error {
	return nil
}

// InterceptPeerDial -- fake.
func (_ *FakeP2P) InterceptPeerDial(peer.ID) (allow bool) {
	return true
}

// InterceptAddrDial -- fake.
func (_ *FakeP2P) InterceptAddrDial(peer.ID, multiaddr.Multiaddr) (allow bool) {
	return true
}

// InterceptAccept -- fake.
func (_ *FakeP2P) InterceptAccept(_ network.ConnMultiaddrs) (allow bool) {
	return true
}

// InterceptSecured -- fake.
func (_ *FakeP2P) InterceptSecured(network.Direction, peer.ID, network.ConnMultiaddrs) (allow bool) {
	return true
}

// InterceptUpgraded -- fake.
func (_ *FakeP2P) InterceptUpgraded(network.Conn) (allow bool, reason control.DisconnectReason) {
	return true, 0
}
