package p2p

import (
	"bytes"
	"context"
	"fmt"
	"reflect"
	"time"

	"github.com/pkg/errors"
	ssz "github.com/prysmaticlabs/fastssz"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/altair"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"go.opencensus.io/trace"
	"google.golang.org/protobuf/proto"
)

// ErrMessageNotMapped occurs on a Broadcast attempt when a message has not been defined in the
// GossipTypeMapping.
var ErrMessageNotMapped = errors.New("message type is not mapped to a PubSub topic")

// Broadcast a message to the p2p network, the message is assumed to be
// broadcasted to the current fork.
func (s *Service) Broadcast(ctx context.Context, msg proto.Message) error {
	ctx, span := trace.StartSpan(ctx, "p2p.Broadcast")
	defer span.End()

	twoSlots := time.Duration(2*params.BeaconConfig().SecondsPerSlot) * time.Second
	ctx, cancel := context.WithTimeout(ctx, twoSlots)
	defer cancel()

	forkDigest, err := s.currentForkDigest()
	if err != nil {
		err := errors.Wrap(err, "could not retrieve fork digest")
		tracing.AnnotateError(span, err)
		return err
	}

	topic, ok := GossipTypeMapping[reflect.TypeOf(msg)]
	if !ok {
		tracing.AnnotateError(span, ErrMessageNotMapped)
		return ErrMessageNotMapped
	}
	castMsg, ok := msg.(ssz.Marshaler)
	if !ok {
		return errors.Errorf("message of %T does not support marshaller interface", msg)
	}
	return s.broadcastObject(ctx, castMsg, fmt.Sprintf(topic, forkDigest))
}

// BroadcastAttestation broadcasts an attestation to the p2p network, the message is assumed to be
// broadcasted to the current fork.
func (s *Service) BroadcastAttestation(ctx context.Context, subnet uint64, att *ethpb.Attestation) error {
	if att == nil {
		return errors.New("attempted to broadcast nil attestation")
	}
	ctx, span := trace.StartSpan(ctx, "p2p.BroadcastAttestation")
	defer span.End()
	forkDigest, err := s.currentForkDigest()
	if err != nil {
		err := errors.Wrap(err, "could not retrieve fork digest")
		tracing.AnnotateError(span, err)
		return err
	}

	// Non-blocking broadcast, with attempts to discover a subnet peer if none available.
	go s.broadcastAttestation(ctx, subnet, att, forkDigest)

	return nil
}

// BroadcastSyncCommitteeMessage broadcasts a sync committee message to the p2p network, the message is assumed to be
// broadcasted to the current fork.
func (s *Service) BroadcastSyncCommitteeMessage(ctx context.Context, subnet uint64, sMsg *ethpb.SyncCommitteeMessage) error {
	if sMsg == nil {
		return errors.New("attempted to broadcast nil sync committee message")
	}
	ctx, span := trace.StartSpan(ctx, "p2p.BroadcastSyncCommitteeMessage")
	defer span.End()
	forkDigest, err := s.currentForkDigest()
	if err != nil {
		err := errors.Wrap(err, "could not retrieve fork digest")
		tracing.AnnotateError(span, err)
		return err
	}

	// Non-blocking broadcast, with attempts to discover a subnet peer if none available.
	go s.broadcastSyncCommittee(ctx, subnet, sMsg, forkDigest)

	return nil
}

func (s *Service) broadcastAttestation(ctx context.Context, subnet uint64, att *ethpb.Attestation, forkDigest [4]byte) {
	_, span := trace.StartSpan(ctx, "p2p.broadcastAttestation")
	defer span.End()
	ctx = trace.NewContext(context.Background(), span) // clear parent context / deadline.

	oneEpoch := time.Duration(1*params.BeaconConfig().SlotsPerEpoch.Mul(params.BeaconConfig().SecondsPerSlot)) * time.Second
	ctx, cancel := context.WithTimeout(ctx, oneEpoch)
	defer cancel()

	// Ensure we have peers with this subnet.
	s.subnetLocker(subnet).RLock()
	hasPeer := s.hasPeerWithSubnet(attestationToTopic(subnet, forkDigest))
	s.subnetLocker(subnet).RUnlock()

	span.AddAttributes(
		trace.BoolAttribute("hasPeer", hasPeer),
		trace.Int64Attribute("slot", int64(att.Data.Slot)), // lint:ignore uintcast -- It's safe to do this for tracing.
		trace.Int64Attribute("subnet", int64(subnet)),      // lint:ignore uintcast -- It's safe to do this for tracing.
	)

	if !hasPeer {
		attestationBroadcastAttempts.Inc()
		if err := func() error {
			s.subnetLocker(subnet).Lock()
			defer s.subnetLocker(subnet).Unlock()
			ok, err := s.FindPeersWithSubnet(ctx, attestationToTopic(subnet, forkDigest), subnet, 1)
			if err != nil {
				return err
			}
			if ok {
				savedAttestationBroadcasts.Inc()
				return nil
			}
			return errors.New("failed to find peers for subnet")
		}(); err != nil {
			log.WithError(err).Error("Failed to find peers")
			tracing.AnnotateError(span, err)
		}
	}
	// In the event our attestation is outdated and beyond the
	// acceptable threshold, we exit early and do not broadcast it.
	currSlot := slots.CurrentSlot(uint64(s.genesisTime.Unix()))
	if att.Data.Slot+params.BeaconConfig().SlotsPerEpoch < currSlot {
		log.Warnf("Attestation is too old to broadcast, discarding it. Current Slot: %d , Attestation Slot: %d", currSlot, att.Data.Slot)
		return
	}

	if err := s.broadcastObject(ctx, att, attestationToTopic(subnet, forkDigest)); err != nil {
		log.WithError(err).Error("Failed to broadcast attestation")
		tracing.AnnotateError(span, err)
	}
}

func (s *Service) broadcastSyncCommittee(ctx context.Context, subnet uint64, sMsg *ethpb.SyncCommitteeMessage, forkDigest [4]byte) {
	_, span := trace.StartSpan(ctx, "p2p.broadcastSyncCommittee")
	defer span.End()
	ctx = trace.NewContext(context.Background(), span) // clear parent context / deadline.

	oneSlot := time.Duration(1*params.BeaconConfig().SecondsPerSlot) * time.Second
	ctx, cancel := context.WithTimeout(ctx, oneSlot)
	defer cancel()

	// Ensure we have peers with this subnet.
	// This adds in a special value to the subnet
	// to ensure that we can re-use the same subnet locker.
	wrappedSubIdx := subnet + syncLockerVal
	s.subnetLocker(wrappedSubIdx).RLock()
	hasPeer := s.hasPeerWithSubnet(syncCommitteeToTopic(subnet, forkDigest))
	s.subnetLocker(wrappedSubIdx).RUnlock()

	span.AddAttributes(
		trace.BoolAttribute("hasPeer", hasPeer),
		trace.Int64Attribute("slot", int64(sMsg.Slot)), // lint:ignore uintcast -- It's safe to do this for tracing.
		trace.Int64Attribute("subnet", int64(subnet)),  // lint:ignore uintcast -- It's safe to do this for tracing.
	)

	if !hasPeer {
		syncCommitteeBroadcastAttempts.Inc()
		if err := func() error {
			s.subnetLocker(wrappedSubIdx).Lock()
			defer s.subnetLocker(wrappedSubIdx).Unlock()
			ok, err := s.FindPeersWithSubnet(ctx, syncCommitteeToTopic(subnet, forkDigest), subnet, 1)
			if err != nil {
				return err
			}
			if ok {
				savedSyncCommitteeBroadcasts.Inc()
				return nil
			}
			return errors.New("failed to find peers for subnet")
		}(); err != nil {
			log.WithError(err).Error("Failed to find peers")
			tracing.AnnotateError(span, err)
		}
	}
	// In the event our sync message is outdated and beyond the
	// acceptable threshold, we exit early and do not broadcast it.
	if err := altair.ValidateSyncMessageTime(sMsg.Slot, s.genesisTime, params.BeaconConfig().MaximumGossipClockDisparityDuration()); err != nil {
		log.WithError(err).Warn("Sync Committee Message is too old to broadcast, discarding it")
		return
	}

	if err := s.broadcastObject(ctx, sMsg, syncCommitteeToTopic(subnet, forkDigest)); err != nil {
		log.WithError(err).Error("Failed to broadcast sync committee message")
		tracing.AnnotateError(span, err)
	}
}

// BroadcastBlob broadcasts a blob to the p2p network, the message is assumed to be
// broadcasted to the current fork and to the input subnet.
func (s *Service) BroadcastBlob(ctx context.Context, subnet uint64, blob *ethpb.BlobSidecar) error {
	ctx, span := trace.StartSpan(ctx, "p2p.BroadcastBlob")
	defer span.End()
	if blob == nil {
		return errors.New("attempted to broadcast nil blob sidecar")
	}
	forkDigest, err := s.currentForkDigest()
	if err != nil {
		err := errors.Wrap(err, "could not retrieve fork digest")
		tracing.AnnotateError(span, err)
		return err
	}

	// Non-blocking broadcast, with attempts to discover a subnet peer if none available.
	go s.broadcastBlob(ctx, subnet, blob, forkDigest)

	return nil
}

func (s *Service) broadcastBlob(ctx context.Context, subnet uint64, blobSidecar *ethpb.BlobSidecar, forkDigest [4]byte) {
	_, span := trace.StartSpan(ctx, "p2p.broadcastBlob")
	defer span.End()
	ctx = trace.NewContext(context.Background(), span) // clear parent context / deadline.

	oneSlot := time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Second
	ctx, cancel := context.WithTimeout(ctx, oneSlot)
	defer cancel()

	wrappedSubIdx := subnet + blobSubnetLockerVal
	s.subnetLocker(wrappedSubIdx).RLock()
	hasPeer := s.hasPeerWithSubnet(blobSubnetToTopic(subnet, forkDigest))
	s.subnetLocker(wrappedSubIdx).RUnlock()

	if !hasPeer {
		blobSidecarCommitteeBroadcastAttempts.Inc()
		if err := func() error {
			s.subnetLocker(wrappedSubIdx).Lock()
			defer s.subnetLocker(wrappedSubIdx).Unlock()
			ok, err := s.FindPeersWithSubnet(ctx, blobSubnetToTopic(subnet, forkDigest), subnet, 1)
			if err != nil {
				return err
			}
			if ok {
				blobSidecarCommitteeBroadcasts.Inc()
				return nil
			}
			return errors.New("failed to find peers for subnet")
		}(); err != nil {
			log.WithError(err).Error("Failed to find peers")
			tracing.AnnotateError(span, err)
		}
	}

	if err := s.broadcastObject(ctx, blobSidecar, blobSubnetToTopic(subnet, forkDigest)); err != nil {
		log.WithError(err).Error("Failed to broadcast blob sidecar")
		tracing.AnnotateError(span, err)
	}
}

// method to broadcast messages to other peers in our gossip mesh.
func (s *Service) broadcastObject(ctx context.Context, obj ssz.Marshaler, topic string) error {
	ctx, span := trace.StartSpan(ctx, "p2p.broadcastObject")
	defer span.End()

	span.AddAttributes(trace.StringAttribute("topic", topic))

	buf := new(bytes.Buffer)
	if _, err := s.Encoding().EncodeGossip(buf, obj); err != nil {
		err := errors.Wrap(err, "could not encode message")
		tracing.AnnotateError(span, err)
		return err
	}

	if span.IsRecordingEvents() {
		id := hash.FastSum64(buf.Bytes())
		messageLen := int64(buf.Len())
		// lint:ignore uintcast -- It's safe to do this for tracing.
		iid := int64(id)
		span.AddMessageSendEvent(iid, messageLen /*uncompressed*/, messageLen /*compressed*/)
	}
	if err := s.PublishToTopic(ctx, topic+s.Encoding().ProtocolSuffix(), buf.Bytes()); err != nil {
		err := errors.Wrap(err, "could not publish message")
		tracing.AnnotateError(span, err)
		return err
	}
	return nil
}

func attestationToTopic(subnet uint64, forkDigest [4]byte) string {
	return fmt.Sprintf(AttestationSubnetTopicFormat, forkDigest, subnet)
}

func syncCommitteeToTopic(subnet uint64, forkDigest [4]byte) string {
	return fmt.Sprintf(SyncCommitteeSubnetTopicFormat, forkDigest, subnet)
}

func blobSubnetToTopic(subnet uint64, forkDigest [4]byte) string {
	return fmt.Sprintf(BlobSubnetTopicFormat, forkDigest, subnet)
}
