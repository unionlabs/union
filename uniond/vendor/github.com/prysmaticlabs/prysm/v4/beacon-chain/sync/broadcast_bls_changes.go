package sync

import (
	"context"
	"time"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/blocks"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	types "github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/rand"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

const broadcastBLSChangesRateLimit = 128

// This routine broadcasts known BLS changes at the Capella fork.
func (s *Service) broadcastBLSChanges(currSlot types.Slot) {
	capellaSlotStart, err := slots.EpochStart(params.BeaconConfig().CapellaForkEpoch)
	if err != nil {
		// only possible error is an overflow, so we exit early from the method
		return
	}
	if currSlot != capellaSlotStart {
		return
	}
	changes, err := s.cfg.blsToExecPool.PendingBLSToExecChanges()
	if err != nil {
		log.WithError(err).Error("could not get BLS to execution changes")
	}
	if len(changes) == 0 {
		return
	}
	source := rand.NewGenerator()
	length := len(changes)
	broadcastChanges := make([]*ethpb.SignedBLSToExecutionChange, length)
	for i := 0; i < length; i++ {
		idx := source.Intn(len(changes))
		broadcastChanges[i] = changes[idx]
		changes = append(changes[:idx], changes[idx+1:]...)
	}

	go s.rateBLSChanges(s.ctx, broadcastChanges)
}

func (s *Service) broadcastBLSBatch(ctx context.Context, ptr *[]*ethpb.SignedBLSToExecutionChange) {
	limit := broadcastBLSChangesRateLimit
	if len(*ptr) < broadcastBLSChangesRateLimit {
		limit = len(*ptr)
	}
	st, err := s.cfg.chain.HeadStateReadOnly(ctx)
	if err != nil {
		log.WithError(err).Error("could not get head state")
		return
	}
	for _, ch := range (*ptr)[:limit] {
		if ch != nil {
			_, err := blocks.ValidateBLSToExecutionChange(st, ch)
			if err != nil {
				log.WithError(err).Error("could not validate BLS to execution change")
				continue
			}
			if err := s.cfg.p2p.Broadcast(ctx, ch); err != nil {
				log.WithError(err).Error("could not broadcast BLS to execution changes.")
			}
		}
	}
	*ptr = (*ptr)[limit:]
}

func (s *Service) rateBLSChanges(ctx context.Context, changes []*ethpb.SignedBLSToExecutionChange) {
	s.broadcastBLSBatch(ctx, &changes)
	if len(changes) == 0 {
		return
	}
	ticker := time.NewTicker(500 * time.Millisecond)
	for {
		select {
		case <-s.ctx.Done():
			return
		case <-ticker.C:
			s.broadcastBLSBatch(ctx, &changes)
			if len(changes) == 0 {
				return
			}
		}
	}
}
