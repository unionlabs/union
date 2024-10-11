package testing

import (
	"context"
	"sync"
	"sync/atomic"

	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"google.golang.org/protobuf/proto"
)

// MockBroadcaster implements p2p.Broadcaster for testing.
type MockBroadcaster struct {
	BroadcastCalled       atomic.Bool
	BroadcastMessages     []proto.Message
	BroadcastAttestations []*ethpb.Attestation
	msgLock               sync.Mutex
	attLock               sync.Mutex
}

// Broadcast records a broadcast occurred.
func (m *MockBroadcaster) Broadcast(_ context.Context, msg proto.Message) error {
	m.BroadcastCalled.Store(true)
	m.msgLock.Lock()
	defer m.msgLock.Unlock()
	m.BroadcastMessages = append(m.BroadcastMessages, msg)
	return nil
}

// BroadcastAttestation records a broadcast occurred.
func (m *MockBroadcaster) BroadcastAttestation(_ context.Context, _ uint64, a *ethpb.Attestation) error {
	m.BroadcastCalled.Store(true)
	m.attLock.Lock()
	defer m.attLock.Unlock()
	m.BroadcastAttestations = append(m.BroadcastAttestations, a)
	return nil
}

// BroadcastSyncCommitteeMessage records a broadcast occurred.
func (m *MockBroadcaster) BroadcastSyncCommitteeMessage(_ context.Context, _ uint64, _ *ethpb.SyncCommitteeMessage) error {
	m.BroadcastCalled.Store(true)
	return nil
}

// BroadcastBlob broadcasts a blob for mock.
func (m *MockBroadcaster) BroadcastBlob(context.Context, uint64, *ethpb.BlobSidecar) error {
	m.BroadcastCalled.Store(true)
	return nil
}

// NumMessages returns the number of messages broadcasted.
func (m *MockBroadcaster) NumMessages() int {
	m.msgLock.Lock()
	defer m.msgLock.Unlock()
	return len(m.BroadcastMessages)
}

// NumAttestations returns the number of attestations broadcasted.
func (m *MockBroadcaster) NumAttestations() int {
	m.attLock.Lock()
	defer m.attLock.Unlock()
	return len(m.BroadcastAttestations)
}
