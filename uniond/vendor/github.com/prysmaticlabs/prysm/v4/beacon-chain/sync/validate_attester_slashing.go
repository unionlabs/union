package sync

import (
	"context"

	pubsub "github.com/libp2p/go-libp2p-pubsub"
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/blocks"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/operation"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/container/slice"
	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"go.opencensus.io/trace"
)

// Clients who receive an attester slashing on this topic MUST validate the conditions within VerifyAttesterSlashing before
// forwarding it across the network.
func (s *Service) validateAttesterSlashing(ctx context.Context, pid peer.ID, msg *pubsub.Message) (pubsub.ValidationResult, error) {
	// Validation runs on publish (not just subscriptions), so we should approve any message from
	// ourselves.
	if pid == s.cfg.p2p.PeerID() {
		return pubsub.ValidationAccept, nil
	}

	// The head state will be too far away to validate any slashing.
	if s.cfg.initialSync.Syncing() {
		return pubsub.ValidationIgnore, nil
	}

	ctx, span := trace.StartSpan(ctx, "sync.validateAttesterSlashing")
	defer span.End()

	m, err := s.decodePubsubMessage(msg)
	if err != nil {
		tracing.AnnotateError(span, err)
		return pubsub.ValidationReject, err
	}
	slashing, ok := m.(*ethpb.AttesterSlashing)
	if !ok {
		return pubsub.ValidationReject, errWrongMessage
	}

	slashedVals := blocks.SlashableAttesterIndices(slashing)
	if slashedVals == nil {
		return pubsub.ValidationReject, errNilMessage
	}
	if s.hasSeenAttesterSlashingIndices(slashedVals) {
		return pubsub.ValidationIgnore, nil
	}

	headState, err := s.cfg.chain.HeadState(ctx)
	if err != nil {
		return pubsub.ValidationIgnore, err
	}
	if err := blocks.VerifyAttesterSlashing(ctx, headState, slashing); err != nil {
		return pubsub.ValidationReject, err
	}
	isSlashable := false
	for _, v := range slashedVals {
		val, err := headState.ValidatorAtIndexReadOnly(primitives.ValidatorIndex(v))
		if err != nil {
			return pubsub.ValidationIgnore, err
		}
		if helpers.IsSlashableValidator(val.ActivationEpoch(), val.WithdrawableEpoch(), val.Slashed(), slots.ToEpoch(headState.Slot())) {
			isSlashable = true
			break
		}
	}
	if !isSlashable {
		return pubsub.ValidationReject, errors.Errorf("none of the validators are slashable: %v", slashedVals)
	}
	s.cfg.chain.ReceiveAttesterSlashing(ctx, slashing)

	// notify events
	s.cfg.operationNotifier.OperationFeed().Send(&feed.Event{
		Type: operation.AttesterSlashingReceived,
		Data: &operation.AttesterSlashingReceivedData{
			AttesterSlashing: slashing,
		},
	})

	msg.ValidatorData = slashing // Used in downstream subscriber
	return pubsub.ValidationAccept, nil
}

// Returns true if the node has already received a valid attester slashing with the attesting indices.
func (s *Service) hasSeenAttesterSlashingIndices(slashableIndices []uint64) bool {
	s.seenAttesterSlashingLock.RLock()
	defer s.seenAttesterSlashingLock.RUnlock()

	// Return false if any of the slashing index has not been seen before. (i.e. not in cache)
	for _, index := range slashableIndices {
		if !s.seenAttesterSlashingCache[index] {
			return false
		}
	}

	return true
}

// Set attester slashing indices in attester slashing cache.
func (s *Service) setAttesterSlashingIndicesSeen(indices1, indices2 []uint64) {
	slashableIndices := slice.IntersectionUint64(indices1, indices2)

	s.seenAttesterSlashingLock.Lock()
	defer s.seenAttesterSlashingLock.Unlock()

	for _, index := range slashableIndices {
		s.seenAttesterSlashingCache[index] = true
	}
}
