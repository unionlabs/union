package monitor

import (
	"fmt"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/sirupsen/logrus"
)

// processSyncCommitteeContribution logs the event when tracked validators' aggregated sync contribution has been processed.
// TODO: We do not log if a sync contribution was included in an aggregate (we log them when they are included in blocks)
func (s *Service) processSyncCommitteeContribution(contribution *ethpb.SignedContributionAndProof) {
	idx := contribution.Message.AggregatorIndex
	s.Lock()
	defer s.Unlock()
	if s.trackedIndex(idx) {
		aggPerf := s.aggregatedPerformance[idx]
		aggPerf.totalSyncCommitteeAggregations++
		s.aggregatedPerformance[idx] = aggPerf

		log.WithField("ValidatorIndex", contribution.Message.AggregatorIndex).Info("Sync committee aggregation processed")
	}
}

// processSyncAggregate logs the event when tracked validators is a sync-committee member and its contribution has been included
func (s *Service) processSyncAggregate(state state.BeaconState, blk interfaces.ReadOnlyBeaconBlock) {
	if blk == nil || blk.Body() == nil {
		return
	}
	if blk.Version() == version.Phase0 {
		return
	}
	bits, err := blk.Body().SyncAggregate()
	if err != nil {
		log.WithError(err).Error("Could not get SyncAggregate")
		return
	}
	s.Lock()
	defer s.Unlock()
	for validatorIdx, committeeIndices := range s.trackedSyncCommitteeIndices {
		if len(committeeIndices) > 0 {
			contrib := 0
			for _, idx := range committeeIndices {
				if bits.SyncCommitteeBits.BitAt(uint64(idx)) {
					contrib++
				}
			}

			balance, err := state.BalanceAtIndex(validatorIdx)
			if err != nil {
				log.Error("Could not get balance")
				return
			}

			latestPerf := s.latestPerformance[validatorIdx]
			balanceChg := int64(balance - latestPerf.balance)
			latestPerf.balanceChange = balanceChg
			latestPerf.balance = balance
			s.latestPerformance[validatorIdx] = latestPerf

			aggPerf := s.aggregatedPerformance[validatorIdx]
			aggPerf.totalSyncCommitteeContributions += uint64(contrib)
			s.aggregatedPerformance[validatorIdx] = aggPerf

			syncCommitteeContributionCounter.WithLabelValues(
				fmt.Sprintf("%d", validatorIdx)).Add(float64(contrib))

			log.WithFields(logrus.Fields{
				"ValidatorIndex":       validatorIdx,
				"ExpectedContribCount": len(committeeIndices),
				"ContribCount":         contrib,
				"NewBalance":           balance,
				"BalanceChange":        balanceChg,
			}).Info("Sync committee contribution included")
		}
	}
}
