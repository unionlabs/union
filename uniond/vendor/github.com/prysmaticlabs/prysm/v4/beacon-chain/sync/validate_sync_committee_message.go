package sync

import (
	"context"
	"errors"
	"fmt"
	"reflect"
	"strings"

	pubsub "github.com/libp2p/go-libp2p-pubsub"
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/altair"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	p2ptypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/types"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"go.opencensus.io/trace"
)

// Sync committee subnets are used to propagate unaggregated sync committee messages to subsections of the network.
//
// The sync_committee_{subnet_id} topics are used to propagate unaggregated sync committee messages
// to the subnet subnet_id to be aggregated before being gossiped to the
// global sync_committee_contribution_and_proof topic.
//
// The following validations MUST pass before forwarding the sync_committee_message on the network:
//
// [IGNORE] The message's slot is for the current slot (with a MAXIMUM_GOSSIP_CLOCK_DISPARITY allowance),
// i.e. sync_committee_message.slot == current_slot.
// [REJECT] The subnet_id is valid for the given validator, i.e. subnet_id in
// compute_subnets_for_sync_committee(state, sync_committee_message.validator_index).
// Note this validation implies the validator is part of the broader current sync
// committee along with the correct subcommittee.
// [IGNORE] There has been no other valid sync committee message for the declared slot for the
// validator referenced by sync_committee_message.validator_index (this requires maintaining
// a cache of size SYNC_COMMITTEE_SIZE // SYNC_COMMITTEE_SUBNET_COUNT for each subnet that can be
// flushed after each slot). Note this validation is per topic so that for a given slot, multiple
// messages could be forwarded with the same validator_index as long as the subnet_ids are distinct.
// [REJECT] The signature is valid for the message beacon_block_root for the validator referenced by validator_index.
func (s *Service) validateSyncCommitteeMessage(
	ctx context.Context, pid peer.ID, msg *pubsub.Message,
) (pubsub.ValidationResult, error) {
	ctx, span := trace.StartSpan(ctx, "sync.validateSyncCommitteeMessage")
	defer span.End()

	if pid == s.cfg.p2p.PeerID() {
		return pubsub.ValidationAccept, nil
	}

	// Basic validations before proceeding.
	if s.cfg.initialSync.Syncing() {
		return pubsub.ValidationIgnore, nil
	}

	if msg.Topic == nil {
		return pubsub.ValidationReject, errInvalidTopic
	}

	// Read the data from the pubsub message, and reject if there is an error.
	m, err := s.readSyncCommitteeMessage(msg)
	if err != nil {
		tracing.AnnotateError(span, err)
		return pubsub.ValidationReject, err
	}

	// Validate sync message times before proceeding.
	// The message's `slot` is for the current slot (with a MAXIMUM_GOSSIP_CLOCK_DISPARITY allowance).
	if err := altair.ValidateSyncMessageTime(
		m.Slot,
		s.cfg.clock.GenesisTime(),
		params.BeaconConfig().MaximumGossipClockDisparityDuration(),
	); err != nil {
		tracing.AnnotateError(span, err)
		return pubsub.ValidationIgnore, err
	}

	committeeIndices, err := s.cfg.chain.HeadSyncCommitteeIndices(ctx, m.ValidatorIndex, m.Slot)
	if err != nil {
		tracing.AnnotateError(span, err)
		return pubsub.ValidationIgnore, err
	}

	// Validate the message's data according to the p2p specification.
	if result, err := validationPipeline(
		ctx,
		ignoreEmptyCommittee(committeeIndices),
		s.rejectIncorrectSyncCommittee(committeeIndices, *msg.Topic),
		s.ignoreHasSeenSyncMsg(ctx, m, committeeIndices),
		s.rejectInvalidSyncCommitteeSignature(m),
	); result != pubsub.ValidationAccept {
		return result, err
	}

	s.markSyncCommitteeMessagesSeen(committeeIndices, m)

	msg.ValidatorData = m
	return pubsub.ValidationAccept, nil
}

// Parse a sync committee message from a pubsub message.
func (s *Service) readSyncCommitteeMessage(msg *pubsub.Message) (*ethpb.SyncCommitteeMessage, error) {
	raw, err := s.decodePubsubMessage(msg)
	if err != nil {
		return nil, err
	}
	m, ok := raw.(*ethpb.SyncCommitteeMessage)
	if !ok {
		return nil, errWrongMessage
	}
	if m == nil {
		return nil, errNilMessage
	}
	return m, nil
}

// Mark all a slot and validator index as seen for every index in a committee and subnet.
func (s *Service) markSyncCommitteeMessagesSeen(committeeIndices []primitives.CommitteeIndex, m *ethpb.SyncCommitteeMessage) {
	subCommitteeSize := params.BeaconConfig().SyncCommitteeSize / params.BeaconConfig().SyncCommitteeSubnetCount
	for _, idx := range committeeIndices {
		subnet := uint64(idx) / subCommitteeSize
		s.setSeenSyncMessageIndexSlot(m, subnet)
	}
}

// Returns true if the node has received sync committee for the validator with index and slot.
func (s *Service) hasSeenSyncMessageIndexSlot(ctx context.Context, m *ethpb.SyncCommitteeMessage, subCommitteeIndex uint64) bool {
	s.seenSyncMessageLock.RLock()
	defer s.seenSyncMessageLock.RUnlock()
	rt, seen := s.seenSyncMessageCache.Get(seenSyncCommitteeKey(m.Slot, m.ValidatorIndex, subCommitteeIndex))
	if !seen {
		// return early if this is the first message
		return false
	}
	root, ok := rt.([32]byte)
	if !ok {
		return true // Impossible. Return true to be safe
	}
	if !s.cfg.chain.InForkchoice(root) && !s.cfg.beaconDB.HasBlock(ctx, root) {
		syncMessagesForUnknownBlocks.Inc()
		return true
	}
	msgRoot := [32]byte(m.BlockRoot)
	if !s.cfg.chain.InForkchoice(msgRoot) && !s.cfg.beaconDB.HasBlock(ctx, msgRoot) {
		syncMessagesForUnknownBlocks.Inc()
		return false
	}
	headRoot := s.cfg.chain.CachedHeadRoot()
	if root == headRoot {
		return true
	}
	return msgRoot != headRoot
}

// Set sync committee message validator index and slot as seen.
func (s *Service) setSeenSyncMessageIndexSlot(m *ethpb.SyncCommitteeMessage, subCommitteeIndex uint64) {
	s.seenSyncMessageLock.Lock()
	defer s.seenSyncMessageLock.Unlock()
	key := seenSyncCommitteeKey(m.Slot, m.ValidatorIndex, subCommitteeIndex)
	s.seenSyncMessageCache.Add(key, [32]byte(m.BlockRoot))
}

// The `subnet_id` is valid for the given validator. This implies the validator is part of the broader
// current sync committee along with the correct subcommittee.
// We are trying to validate that whatever committee indices that were retrieved from our state for this
// particular validator are indeed valid for this particular topic. Ex: the topic name can be
// /eth2/b5303f2a/sync_committee_2/ssz_snappy
// This would mean that only messages meant for subnet 2 are valid. If a validator creates this sync
// message and broadcasts it into subnet 2, we need to make sure that whatever committee index and
// resultant subnet that the validator has is valid for this particular topic.
func (s *Service) rejectIncorrectSyncCommittee(
	committeeIndices []primitives.CommitteeIndex, topic string,
) validationFn {
	return func(ctx context.Context) (pubsub.ValidationResult, error) {
		_, span := trace.StartSpan(ctx, "sync.rejectIncorrectSyncCommittee")
		defer span.End()
		isValid := false
		digest, err := s.currentForkDigest()
		if err != nil {
			tracing.AnnotateError(span, err)
			return pubsub.ValidationIgnore, err
		}

		format := p2p.GossipTypeMapping[reflect.TypeOf(&ethpb.SyncCommitteeMessage{})]
		// Validate that the validator is in the correct committee.
		subCommitteeSize := params.BeaconConfig().SyncCommitteeSize / params.BeaconConfig().SyncCommitteeSubnetCount
		for _, idx := range committeeIndices {
			subnet := uint64(idx) / subCommitteeSize
			if strings.HasPrefix(topic, fmt.Sprintf(format, digest, subnet)) {
				isValid = true
				break
			}
		}
		if !isValid {
			return pubsub.ValidationReject, errors.New("sync committee message references a different subnet")
		}
		return pubsub.ValidationAccept, nil
	}
}

// There has been no other valid sync committee signature for the declared `slot`, `validator_index`,
// and `subcommittee_index`. In the event of `validator_index` belongs to multiple subnets, as long
// as one subnet has not been seen, we should let it in.
func (s *Service) ignoreHasSeenSyncMsg(ctx context.Context,
	m *ethpb.SyncCommitteeMessage, committeeIndices []primitives.CommitteeIndex,
) validationFn {
	return func(ctx context.Context) (pubsub.ValidationResult, error) {
		var isValid bool
		subCommitteeSize := params.BeaconConfig().SyncCommitteeSize / params.BeaconConfig().SyncCommitteeSubnetCount
		for _, idx := range committeeIndices {
			subnet := uint64(idx) / subCommitteeSize
			if !s.hasSeenSyncMessageIndexSlot(ctx, m, subnet) {
				isValid = true
				break
			}
		}
		if !isValid {
			return pubsub.ValidationIgnore, nil
		}
		return pubsub.ValidationAccept, nil
	}
}

func (s *Service) rejectInvalidSyncCommitteeSignature(m *ethpb.SyncCommitteeMessage) validationFn {
	return func(ctx context.Context) (pubsub.ValidationResult, error) {
		ctx, span := trace.StartSpan(ctx, "sync.rejectInvalidSyncCommitteeSignature")
		defer span.End()

		// Ignore the message if it is not possible to retrieve the signing root.
		// For internal errors, the correct behaviour is to ignore rather than reject outright,
		// since the failure is locally derived.
		d, err := s.cfg.chain.HeadSyncCommitteeDomain(ctx, m.Slot)
		if err != nil {
			tracing.AnnotateError(span, err)
			return pubsub.ValidationIgnore, err
		}
		rawBytes := p2ptypes.SSZBytes(m.BlockRoot)
		sigRoot, err := signing.ComputeSigningRoot(&rawBytes, d)
		if err != nil {
			tracing.AnnotateError(span, err)
			return pubsub.ValidationIgnore, err
		}

		// Reject for a validator index that is not found, as we should not remain peered with a node
		// that is on such a different fork than our chain.
		pubKey, err := s.cfg.chain.HeadValidatorIndexToPublicKey(ctx, m.ValidatorIndex)
		if err != nil {
			tracing.AnnotateError(span, err)
			return pubsub.ValidationReject, err
		}

		// Ignore a malformed public key from bytes according to the p2p specification.
		pKey, err := bls.PublicKeyFromBytes(pubKey[:])
		if err != nil {
			tracing.AnnotateError(span, err)
			return pubsub.ValidationIgnore, err
		}

		// Batch verify message signature before unmarshalling
		// the signature to a G2 point if batch verification is
		// enabled.
		set := &bls.SignatureBatch{
			Messages:     [][32]byte{sigRoot},
			PublicKeys:   []bls.PublicKey{pKey},
			Signatures:   [][]byte{m.Signature},
			Descriptions: []string{signing.SyncCommitteeSignature},
		}
		return s.validateWithBatchVerifier(ctx, "sync committee message", set)
	}
}

func ignoreEmptyCommittee(indices []primitives.CommitteeIndex) validationFn {
	return func(ctx context.Context) (pubsub.ValidationResult, error) {
		if len(indices) == 0 {
			return pubsub.ValidationIgnore, nil
		}
		return pubsub.ValidationAccept, nil
	}
}

func seenSyncCommitteeKey(slot primitives.Slot, valIndex primitives.ValidatorIndex, subCommitteeIndex uint64) string {
	b := append(bytesutil.Bytes32(uint64(slot)), bytesutil.Bytes32(uint64(valIndex))...)
	b = append(b, bytesutil.Bytes32(subCommitteeIndex)...)
	return string(b)
}

func validationPipeline(ctx context.Context, fns ...validationFn) (pubsub.ValidationResult, error) {
	for _, fn := range fns {
		if result, err := fn(ctx); result != pubsub.ValidationAccept {
			return result, err
		}
	}
	return pubsub.ValidationAccept, nil
}
