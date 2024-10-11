package sync

import (
	"context"
	"fmt"
	"strings"

	pubsub "github.com/libp2p/go-libp2p-pubsub"
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/verification"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/rand"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	prysmTime "github.com/prysmaticlabs/prysm/v4/time"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/sirupsen/logrus"
)

func (s *Service) validateBlob(ctx context.Context, pid peer.ID, msg *pubsub.Message) (pubsub.ValidationResult, error) {
	receivedTime := prysmTime.Now()

	if pid == s.cfg.p2p.PeerID() {
		return pubsub.ValidationAccept, nil
	}
	if s.cfg.initialSync.Syncing() {
		return pubsub.ValidationIgnore, nil
	}
	if msg.Topic == nil {
		return pubsub.ValidationReject, errInvalidTopic
	}
	m, err := s.decodePubsubMessage(msg)
	if err != nil {
		log.WithError(err).Error("Failed to decode message")
		return pubsub.ValidationReject, err
	}

	bpb, ok := m.(*eth.BlobSidecar)
	if !ok {
		log.WithField("message", m).Error("Message is not of type *eth.BlobSidecar")
		return pubsub.ValidationReject, errWrongMessage
	}
	blob, err := blocks.NewROBlob(bpb)
	if err != nil {
		return pubsub.ValidationReject, errors.Wrap(err, "roblob conversion failure")
	}
	vf := s.newBlobVerifier(blob, verification.GossipSidecarRequirements)

	if err := vf.BlobIndexInBounds(); err != nil {
		return pubsub.ValidationReject, err
	}

	// [REJECT] The sidecar is for the correct subnet -- i.e. compute_subnet_for_blob_sidecar(sidecar.index) == subnet_id.
	want := fmt.Sprintf("blob_sidecar_%d", computeSubnetForBlobSidecar(blob.Index))
	if !strings.Contains(*msg.Topic, want) {
		log.WithFields(blobFields(blob)).Debug("Sidecar index does not match topic")
		return pubsub.ValidationReject, fmt.Errorf("wrong topic name: %s", *msg.Topic)
	}

	if err := vf.NotFromFutureSlot(); err != nil {
		return pubsub.ValidationIgnore, err
	}

	startTime, err := slots.ToTime(uint64(s.cfg.chain.GenesisTime().Unix()), blob.Slot())
	if err != nil {
		return pubsub.ValidationIgnore, err
	}

	// [IGNORE] The sidecar is the first sidecar for the tuple (block_header.slot, block_header.proposer_index, sidecar.index) with valid header signature and sidecar inclusion proof
	if s.hasSeenBlobIndex(blob.Slot(), blob.ProposerIndex(), blob.Index) {
		return pubsub.ValidationIgnore, nil
	}

	if err := vf.SlotAboveFinalized(); err != nil {
		return pubsub.ValidationIgnore, err
	}

	if err := vf.SidecarParentSeen(s.hasBadBlock); err != nil {
		go func() {
			if err := s.sendBatchRootRequest(context.Background(), [][32]byte{blob.ParentRoot()}, rand.NewGenerator()); err != nil {
				log.WithError(err).WithFields(blobFields(blob)).Debug("Failed to send batch root request")
			}
		}()
		missingParentBlobSidecarCount.Inc()
		return pubsub.ValidationIgnore, err
	}

	if err := vf.ValidProposerSignature(ctx); err != nil {
		return pubsub.ValidationReject, err
	}

	if err := vf.SidecarParentValid(s.hasBadBlock); err != nil {
		return pubsub.ValidationReject, err
	}

	if err := vf.SidecarParentSlotLower(); err != nil {
		return pubsub.ValidationReject, err
	}

	if err := vf.SidecarDescendsFromFinalized(); err != nil {
		return pubsub.ValidationReject, err
	}

	if err := vf.SidecarInclusionProven(); err != nil {
		return pubsub.ValidationReject, err
	}

	if err := vf.SidecarKzgProofVerified(); err != nil {
		return pubsub.ValidationReject, err
	}

	if err := vf.SidecarProposerExpected(ctx); err != nil {
		return pubsub.ValidationReject, err
	}

	fields := blobFields(blob)
	sinceSlotStartTime := receivedTime.Sub(startTime)
	fields["sinceSlotStartTime"] = sinceSlotStartTime
	fields["validationTime"] = s.cfg.clock.Now().Sub(receivedTime)
	log.WithFields(fields).Debug("Received blob sidecar gossip")

	blobSidecarArrivalGossipSummary.Observe(float64(sinceSlotStartTime.Milliseconds()))

	vBlobData, err := vf.VerifiedROBlob()
	if err != nil {
		return pubsub.ValidationReject, err
	}
	msg.ValidatorData = vBlobData

	return pubsub.ValidationAccept, nil
}

// Returns true if the blob with the same slot, proposer index, and blob index has been seen before.
func (s *Service) hasSeenBlobIndex(slot primitives.Slot, proposerIndex primitives.ValidatorIndex, index uint64) bool {
	s.seenBlobLock.RLock()
	defer s.seenBlobLock.RUnlock()
	b := append(bytesutil.Bytes32(uint64(slot)), bytesutil.Bytes32(uint64(proposerIndex))...)
	b = append(b, bytesutil.Bytes32(index)...)
	_, seen := s.seenBlobCache.Get(string(b))
	return seen
}

// Sets the blob with the same slot, proposer index, and blob index as seen.
func (s *Service) setSeenBlobIndex(slot primitives.Slot, proposerIndex primitives.ValidatorIndex, index uint64) {
	s.seenBlobLock.Lock()
	defer s.seenBlobLock.Unlock()
	b := append(bytesutil.Bytes32(uint64(slot)), bytesutil.Bytes32(uint64(proposerIndex))...)
	b = append(b, bytesutil.Bytes32(index)...)
	s.seenBlobCache.Add(string(b), true)
}

func blobFields(b blocks.ROBlob) logrus.Fields {
	return logrus.Fields{
		"slot":          b.Slot(),
		"proposerIndex": b.ProposerIndex(),
		"blockRoot":     fmt.Sprintf("%#x", b.BlockRoot()),
		"kzgCommitment": fmt.Sprintf("%#x", b.KzgCommitment),
		"index":         b.Index,
	}
}

func computeSubnetForBlobSidecar(index uint64) uint64 {
	return index % params.BeaconConfig().BlobsidecarSubnetCount
}
