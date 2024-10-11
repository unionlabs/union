package sync

import (
	"context"

	pubsub "github.com/libp2p/go-libp2p-pubsub"
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/blocks"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/operation"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"go.opencensus.io/trace"
)

// Clients who receive a proposer slashing on this topic MUST validate the conditions within VerifyProposerSlashing before
// forwarding it across the network.
func (s *Service) validateProposerSlashing(ctx context.Context, pid peer.ID, msg *pubsub.Message) (pubsub.ValidationResult, error) {
	// Validation runs on publish (not just subscriptions), so we should approve any message from
	// ourselves.
	if pid == s.cfg.p2p.PeerID() {
		return pubsub.ValidationAccept, nil
	}

	// The head state will be too far away to validate any slashing.
	if s.cfg.initialSync.Syncing() {
		return pubsub.ValidationIgnore, nil
	}

	ctx, span := trace.StartSpan(ctx, "sync.validateProposerSlashing")
	defer span.End()

	m, err := s.decodePubsubMessage(msg)
	if err != nil {
		tracing.AnnotateError(span, err)
		return pubsub.ValidationReject, err
	}

	slashing, ok := m.(*ethpb.ProposerSlashing)
	if !ok {
		return pubsub.ValidationReject, errWrongMessage
	}

	if slashing.Header_1 == nil || slashing.Header_1.Header == nil {
		return pubsub.ValidationReject, errNilMessage
	}
	if s.hasSeenProposerSlashingIndex(slashing.Header_1.Header.ProposerIndex) {
		return pubsub.ValidationIgnore, nil
	}

	headState, err := s.cfg.chain.HeadState(ctx)
	if err != nil {
		return pubsub.ValidationIgnore, err
	}
	if err := blocks.VerifyProposerSlashing(headState, slashing); err != nil {
		return pubsub.ValidationReject, err
	}

	// notify events
	s.cfg.operationNotifier.OperationFeed().Send(&feed.Event{
		Type: operation.ProposerSlashingReceived,
		Data: &operation.ProposerSlashingReceivedData{
			ProposerSlashing: slashing,
		},
	})

	msg.ValidatorData = slashing // Used in downstream subscriber
	return pubsub.ValidationAccept, nil
}

// Returns true if the node has already received a valid proposer slashing received for the proposer with index
func (s *Service) hasSeenProposerSlashingIndex(i primitives.ValidatorIndex) bool {
	s.seenProposerSlashingLock.RLock()
	defer s.seenProposerSlashingLock.RUnlock()
	_, seen := s.seenProposerSlashingCache.Get(i)
	return seen
}

// Set proposer slashing index in proposer slashing cache.
func (s *Service) setProposerSlashingIndexSeen(i primitives.ValidatorIndex) {
	s.seenProposerSlashingLock.Lock()
	defer s.seenProposerSlashingLock.Unlock()
	s.seenProposerSlashingCache.Add(i, true)
}
