package blockchain

import (
	"bytes"
	"context"
	"fmt"
	"time"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/sirupsen/logrus"
	"go.opencensus.io/trace"
)

// reorgLateBlockCountAttestations is the time until the end of the slot in which we count
// attestations to see if we will reorg the incoming block
const reorgLateBlockCountAttestations = 2 * time.Second

// AttestationStateFetcher allows for retrieving a beacon state corresponding to the block
// root of an attestation's target checkpoint.
type AttestationStateFetcher interface {
	AttestationTargetState(ctx context.Context, target *ethpb.Checkpoint) (state.ReadOnlyBeaconState, error)
}

// AttestationReceiver interface defines the methods of chain service receive and processing new attestations.
type AttestationReceiver interface {
	AttestationStateFetcher
	VerifyLmdFfgConsistency(ctx context.Context, att *ethpb.Attestation) error
	InForkchoice([32]byte) bool
}

// AttestationTargetState returns the pre state of attestation.
func (s *Service) AttestationTargetState(ctx context.Context, target *ethpb.Checkpoint) (state.ReadOnlyBeaconState, error) {
	ss, err := slots.EpochStart(target.Epoch)
	if err != nil {
		return nil, err
	}
	if err := slots.ValidateClock(ss, uint64(s.genesisTime.Unix())); err != nil {
		return nil, err
	}
	// We acquire the lock here instead than on gettAttPreState because that function gets called from UpdateHead that holds a write lock
	s.cfg.ForkChoiceStore.RLock()
	defer s.cfg.ForkChoiceStore.RUnlock()
	return s.getAttPreState(ctx, target)
}

// VerifyLmdFfgConsistency verifies that attestation's LMD and FFG votes are consistency to each other.
func (s *Service) VerifyLmdFfgConsistency(ctx context.Context, a *ethpb.Attestation) error {
	r, err := s.TargetRootForEpoch([32]byte(a.Data.BeaconBlockRoot), a.Data.Target.Epoch)
	if err != nil {
		return err
	}
	if !bytes.Equal(a.Data.Target.Root, r[:]) {
		return fmt.Errorf("FFG and LMD votes are not consistent, block root: %#x, target root: %#x, canonical target root: %#x", a.Data.BeaconBlockRoot, a.Data.Target.Root, r)
	}
	return nil
}

// This routine processes fork choice attestations from the pool to account for validator votes and fork choice.
func (s *Service) spawnProcessAttestationsRoutine() {
	go func() {
		_, err := s.clockWaiter.WaitForClock(s.ctx)
		if err != nil {
			log.WithError(err).Error("spawnProcessAttestationsRoutine failed to receive genesis data")
			return
		}
		if s.genesisTime.IsZero() {
			log.Warn("ProcessAttestations routine waiting for genesis time")
			for s.genesisTime.IsZero() {
				if err := s.ctx.Err(); err != nil {
					log.WithError(err).Error("Giving up waiting for genesis time")
					return
				}
				time.Sleep(1 * time.Second)
			}
			log.Warn("Genesis time received, now available to process attestations")
		}
		// Wait for node to be synced before running the routine.
		if err := s.waitForSync(); err != nil {
			log.WithError(err).Error("Could not wait to sync")
			return
		}

		reorgInterval := time.Second*time.Duration(params.BeaconConfig().SecondsPerSlot) - reorgLateBlockCountAttestations
		ticker := slots.NewSlotTickerWithIntervals(s.genesisTime, []time.Duration{0, reorgInterval})
		for {
			select {
			case <-s.ctx.Done():
				return
			case slotInterval := <-ticker.C():
				if slotInterval.Interval > 0 {
					s.UpdateHead(s.ctx, slotInterval.Slot+1)
				} else {
					s.cfg.ForkChoiceStore.Lock()
					if err := s.cfg.ForkChoiceStore.NewSlot(s.ctx, slotInterval.Slot); err != nil {
						log.WithError(err).Error("could not process new slot")
					}
					s.cfg.ForkChoiceStore.Unlock()

					s.UpdateHead(s.ctx, slotInterval.Slot)
				}
			}
		}
	}()
}

// UpdateHead updates the canonical head of the chain based on information from fork-choice attestations and votes.
// The caller of this function MUST hold a lock in forkchoice
func (s *Service) UpdateHead(ctx context.Context, proposingSlot primitives.Slot) {
	ctx, span := trace.StartSpan(ctx, "beacon-chain.blockchain.UpdateHead")
	defer span.End()

	start := time.Now()
	s.cfg.ForkChoiceStore.Lock()
	defer s.cfg.ForkChoiceStore.Unlock()
	// This function is only called at 10 seconds or 0 seconds into the slot
	disparity := params.BeaconConfig().MaximumGossipClockDisparityDuration()
	disparity += reorgLateBlockCountAttestations

	s.processAttestations(ctx, disparity)

	processAttsElapsedTime.Observe(float64(time.Since(start).Milliseconds()))

	start = time.Now()
	// return early if we haven't changed head
	newHeadRoot, err := s.cfg.ForkChoiceStore.Head(ctx)
	if err != nil {
		log.WithError(err).Error("Could not compute head from new attestations")
		return
	}
	if !s.isNewHead(newHeadRoot) {
		return
	}
	log.WithField("newHeadRoot", fmt.Sprintf("%#x", newHeadRoot)).Debug("Head changed due to attestations")
	headState, headBlock, err := s.getStateAndBlock(ctx, newHeadRoot)
	if err != nil {
		log.WithError(err).Error("could not get head block")
		return
	}
	newAttHeadElapsedTime.Observe(float64(time.Since(start).Milliseconds()))
	fcuArgs := &fcuConfig{
		headState:     headState,
		headRoot:      newHeadRoot,
		headBlock:     headBlock,
		proposingSlot: proposingSlot,
	}
	if s.inRegularSync() {
		fcuArgs.attributes = s.getPayloadAttribute(ctx, headState, proposingSlot, newHeadRoot[:])
	}
	if fcuArgs.attributes != nil && s.shouldOverrideFCU(newHeadRoot, proposingSlot) {
		return
	}
	if err := s.forkchoiceUpdateWithExecution(s.ctx, fcuArgs); err != nil {
		log.WithError(err).Error("could not update forkchoice")
	}
}

// This processes fork choice attestations from the pool to account for validator votes and fork choice.
func (s *Service) processAttestations(ctx context.Context, disparity time.Duration) {
	atts := s.cfg.AttPool.ForkchoiceAttestations()
	for _, a := range atts {
		// Based on the spec, don't process the attestation until the subsequent slot.
		// This delays consideration in the fork choice until their slot is in the past.
		// https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/fork-choice.md#validate_on_attestation
		nextSlot := a.Data.Slot + 1
		if err := slots.VerifyTime(uint64(s.genesisTime.Unix()), nextSlot, disparity); err != nil {
			continue
		}

		hasState := s.cfg.BeaconDB.HasStateSummary(ctx, bytesutil.ToBytes32(a.Data.BeaconBlockRoot))
		hasBlock := s.hasBlock(ctx, bytesutil.ToBytes32(a.Data.BeaconBlockRoot))
		if !(hasState && hasBlock) {
			continue
		}

		if err := s.cfg.AttPool.DeleteForkchoiceAttestation(a); err != nil {
			log.WithError(err).Error("Could not delete fork choice attestation in pool")
		}

		if !helpers.VerifyCheckpointEpoch(a.Data.Target, s.genesisTime) {
			continue
		}

		if err := s.receiveAttestationNoPubsub(ctx, a, disparity); err != nil {
			log.WithFields(logrus.Fields{
				"slot":             a.Data.Slot,
				"committeeIndex":   a.Data.CommitteeIndex,
				"beaconBlockRoot":  fmt.Sprintf("%#x", bytesutil.Trunc(a.Data.BeaconBlockRoot)),
				"targetRoot":       fmt.Sprintf("%#x", bytesutil.Trunc(a.Data.Target.Root)),
				"aggregationCount": a.AggregationBits.Count(),
			}).WithError(err).Warn("Could not process attestation for fork choice")
		}
	}
}

// receiveAttestationNoPubsub is a function that defines the operations that are performed on
// attestation that is received from regular sync. The operations consist of:
//  1. Validate attestation, update validator's latest vote
//  2. Apply fork choice to the processed attestation
//  3. Save latest head info
func (s *Service) receiveAttestationNoPubsub(ctx context.Context, att *ethpb.Attestation, disparity time.Duration) error {
	ctx, span := trace.StartSpan(ctx, "beacon-chain.blockchain.receiveAttestationNoPubsub")
	defer span.End()

	if err := s.OnAttestation(ctx, att, disparity); err != nil {
		return errors.Wrap(err, "could not process attestation")
	}

	return nil
}
