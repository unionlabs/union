package sync

import (
	"context"

	pubsub "github.com/libp2p/go-libp2p-pubsub"
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/blocks"
	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"go.opencensus.io/trace"
)

func (s *Service) validateBlsToExecutionChange(ctx context.Context, pid peer.ID, msg *pubsub.Message) (pubsub.ValidationResult, error) {
	// Validation runs on publish (not just subscriptions), so we should approve any message from
	// ourselves.
	if pid == s.cfg.p2p.PeerID() {
		return pubsub.ValidationAccept, nil
	}

	// The head state will be too far away to validate any execution change.
	if s.cfg.initialSync.Syncing() {
		return pubsub.ValidationIgnore, nil
	}

	ctx, span := trace.StartSpan(ctx, "sync.validateBlsToExecutionChange")
	defer span.End()

	m, err := s.decodePubsubMessage(msg)
	if err != nil {
		tracing.AnnotateError(span, err)
		return pubsub.ValidationReject, err
	}

	blsChange, ok := m.(*ethpb.SignedBLSToExecutionChange)
	if !ok {
		return pubsub.ValidationReject, errWrongMessage
	}

	// Check that the validator hasn't submitted a previous execution change.
	if blsChange.Message == nil {
		return pubsub.ValidationReject, errNilMessage
	}
	if s.cfg.blsToExecPool.ValidatorExists(blsChange.Message.ValidatorIndex) {
		return pubsub.ValidationIgnore, nil
	}
	st, err := s.cfg.chain.HeadStateReadOnly(ctx)
	if err != nil {
		return pubsub.ValidationIgnore, err
	}
	// Validate that the execution change object is valid.
	_, err = blocks.ValidateBLSToExecutionChange(st, blsChange)
	if err != nil {
		return pubsub.ValidationReject, err
	}
	// Validate the signature of the message using our batch gossip verifier.
	sigBatch, err := blocks.BLSChangesSignatureBatch(st, []*ethpb.SignedBLSToExecutionChange{blsChange})
	if err != nil {
		return pubsub.ValidationReject, err
	}
	res, err := s.validateWithBatchVerifier(ctx, "bls to execution change", sigBatch)
	if res != pubsub.ValidationAccept {
		return res, err
	}
	msg.ValidatorData = blsChange // Used in downstream subscriber
	return pubsub.ValidationAccept, nil
}
