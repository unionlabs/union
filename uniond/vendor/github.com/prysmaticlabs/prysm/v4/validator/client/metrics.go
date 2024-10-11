package client

import (
	"context"
	"fmt"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/sirupsen/logrus"
)

var (
	// ValidatorStatusesGaugeVec used to track validator statuses by public key.
	ValidatorStatusesGaugeVec = promauto.NewGaugeVec(
		prometheus.GaugeOpts{
			Namespace: "validator",
			Name:      "statuses",
			Help:      "validator statuses: 0 UNKNOWN, 1 DEPOSITED, 2 PENDING, 3 ACTIVE, 4 EXITING, 5 SLASHING, 6 EXITED",
		},
		[]string{
			"pubkey",
		},
	)
	// ValidatorAggSuccessVec used to count successful aggregations.
	ValidatorAggSuccessVec = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Namespace: "validator",
			Name:      "successful_aggregations",
		},
		[]string{
			"pubkey",
		},
	)
	// ValidatorAggFailVec used to count failed aggregations.
	ValidatorAggFailVec = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Namespace: "validator",
			Name:      "failed_aggregations",
		},
		[]string{
			"pubkey",
		},
	)
	// ValidatorProposeSuccessVec used to count successful proposals.
	ValidatorProposeSuccessVec = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Namespace: "validator",
			Name:      "successful_proposals",
		},
		[]string{
			"pubkey",
		},
	)
	// ValidatorProposeFailVec used to count failed proposals.
	ValidatorProposeFailVec = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Namespace: "validator",
			Name:      "failed_proposals",
		},
		[]string{
			"pubkey",
		},
	)
	// ValidatorProposeFailVecSlasher used to count failed proposals by slashing protection.
	ValidatorProposeFailVecSlasher = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Name: "validator_proposals_rejected_total",
			Help: "Count the block proposals rejected by slashing protection.",
		},
		[]string{
			"pubkey",
		},
	)
	// ValidatorBalancesGaugeVec used to keep track of validator balances by public key.
	ValidatorBalancesGaugeVec = promauto.NewGaugeVec(
		prometheus.GaugeOpts{
			Namespace: "validator",
			Name:      "balance",
			Help:      "current validator balance.",
		},
		[]string{
			"pubkey",
		},
	)
	// ValidatorAttestedSlotsGaugeVec used to keep track of validator attested slots by public key.
	ValidatorAttestedSlotsGaugeVec = promauto.NewGaugeVec(
		prometheus.GaugeOpts{
			Namespace: "validator",
			Name:      "last_attested_slot",
			Help:      "Last attested slot.",
		},
		[]string{
			"pubkey",
		},
	)
	// ValidatorCorrectlyVotedSourceGaugeVec used to keep track of validator's accuracy on voting source by public key.
	ValidatorCorrectlyVotedSourceGaugeVec = promauto.NewGaugeVec(
		prometheus.GaugeOpts{
			Namespace: "validator",
			Name:      "correctly_voted_source",
			Help: "True if correctly voted source in last attestation. In Altair, this " +
				"value will be false if the attestation was not included within " +
				"integer_squareroot(SLOTS_PER_EPOCH) slots, even if it was a vote for the " +
				"correct source.",
		},
		[]string{
			"pubkey",
		},
	)
	// ValidatorCorrectlyVotedTargetGaugeVec used to keep track of validator's accuracy on voting target by public key.
	ValidatorCorrectlyVotedTargetGaugeVec = promauto.NewGaugeVec(
		prometheus.GaugeOpts{
			Namespace: "validator",
			Name:      "correctly_voted_target",
			Help: "True if correctly voted target in last attestation. In Altair, this " +
				"value will be false if the attestation was not included within " +
				"SLOTS_PER_EPOCH slots, even if it was a vote for the correct target.",
		},
		[]string{
			"pubkey",
		},
	)
	// ValidatorCorrectlyVotedHeadGaugeVec used to keep track of validator's accuracy on voting head by public key.
	ValidatorCorrectlyVotedHeadGaugeVec = promauto.NewGaugeVec(
		prometheus.GaugeOpts{
			Namespace: "validator",
			Name:      "correctly_voted_head",
			Help: "True if correctly voted head in last attestation. In Altair, this value " +
				"will be false if the attestation was not included in the next slot.",
		},
		[]string{
			"pubkey",
		},
	)
	// ValidatorAttestSuccessVec used to count successful attestations.
	ValidatorAttestSuccessVec = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Namespace: "validator",
			Name:      "successful_attestations",
		},
		[]string{
			"pubkey",
		},
	)
	// ValidatorAttestFailVec used to count failed attestations.
	ValidatorAttestFailVec = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Namespace: "validator",
			Name:      "failed_attestations",
		},
		[]string{
			"pubkey",
		},
	)
	// ValidatorAttestFailVecSlasher used to count failed attestations by slashing protection.
	ValidatorAttestFailVecSlasher = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Name: "validator_attestations_rejected_total",
			Help: "Count the attestations rejected by slashing protection.",
		},
		[]string{
			"pubkey",
		},
	)
	// ValidatorNextAttestationSlotGaugeVec used to track validator statuses by public key.
	ValidatorNextAttestationSlotGaugeVec = promauto.NewGaugeVec(
		prometheus.GaugeOpts{
			Namespace: "validator",
			Name:      "next_attestation_slot",
			Help:      "validator next scheduled attestation slot",
		},
		[]string{
			"pubkey",
		},
	)
	// ValidatorNextProposalSlotGaugeVec used to track validator statuses by public key.
	ValidatorNextProposalSlotGaugeVec = promauto.NewGaugeVec(
		prometheus.GaugeOpts{
			Namespace: "validator",
			Name:      "next_proposal_slot",
			Help:      "validator next scheduled proposal slot",
		},
		[]string{
			"pubkey",
		},
	)
	// ValidatorInSyncCommitteeGaugeVec used to track whether validator is in the current sync committee.
	ValidatorInSyncCommitteeGaugeVec = promauto.NewGaugeVec(
		prometheus.GaugeOpts{
			Namespace: "validator",
			Name:      "in_sync_committee",
			Help:      "validator sync committee.New in Altair hardfork",
		},
		[]string{
			"pubkey",
		},
	)
	// ValidatorInNextSyncCommitteeGaugeVec used to track whether validator is in the next sync committee.
	ValidatorInNextSyncCommitteeGaugeVec = promauto.NewGaugeVec(
		prometheus.GaugeOpts{
			Namespace: "validator",
			Name:      "in_next_sync_committee",
			Help:      "validator next sync committee. New in Altair hardfork",
		},
		[]string{
			"pubkey",
		},
	)
	// ValidatorInactivityScoreGaugeVec used to track validator inactivity scores.
	ValidatorInactivityScoreGaugeVec = promauto.NewGaugeVec(
		prometheus.GaugeOpts{
			Namespace: "validator",
			Name:      "inactivity_score",
			Help:      "Validator inactivity score. 0 is optimum number. New in Altair hardfork",
		},
		[]string{
			"pubkey",
		},
	)
)

// LogValidatorGainsAndLosses logs important metrics related to this validator client's
// responsibilities throughout the beacon chain's lifecycle. It logs absolute accrued rewards
// and penalties over time, percentage gain/loss, and gives the end user a better idea
// of how the validator performs with respect to the rest.
func (v *validator) LogValidatorGainsAndLosses(ctx context.Context, slot primitives.Slot) error {
	if !slots.IsEpochEnd(slot) || slot <= params.BeaconConfig().SlotsPerEpoch {
		// Do nothing unless we are at the end of the epoch, and not in the first epoch.
		return nil
	}
	if !v.logValidatorBalances {
		return nil
	}

	var pks [][fieldparams.BLSPubkeyLength]byte
	var err error
	pks, err = v.keyManager.FetchValidatingPublicKeys(ctx)
	if err != nil {
		return err
	}
	pubKeys := bytesutil.FromBytes48Array(pks)

	req := &ethpb.ValidatorPerformanceRequest{
		PublicKeys: pubKeys,
	}
	resp, err := v.beaconClient.GetValidatorPerformance(ctx, req)
	if err != nil {
		return err
	}

	if v.emitAccountMetrics {
		// There is no distinction between unknown and pending validators here.
		// The balance is recorded as 0, as this metric is the effective balance of a participating validator.
		for _, missingPubKey := range resp.MissingValidators {
			fmtKey := fmt.Sprintf("%#x", missingPubKey)
			ValidatorBalancesGaugeVec.WithLabelValues(fmtKey).Set(0)
		}
	}

	prevEpoch := primitives.Epoch(0)
	if slot >= params.BeaconConfig().SlotsPerEpoch {
		prevEpoch = primitives.Epoch(slot/params.BeaconConfig().SlotsPerEpoch) - 1
		if uint64(v.voteStats.startEpoch) == ^uint64(0) { // Handles unknown first epoch.
			v.voteStats.startEpoch = prevEpoch
		}
	}
	v.prevBalanceLock.Lock()
	for i, pubKey := range resp.PublicKeys {
		v.logForEachValidator(i, pubKey, resp, slot, prevEpoch)
	}
	v.prevBalanceLock.Unlock()

	v.UpdateLogAggregateStats(resp, slot)
	return nil
}

func (v *validator) logForEachValidator(index int, pubKey []byte, resp *ethpb.ValidatorPerformanceResponse, slot primitives.Slot, prevEpoch primitives.Epoch) {
	truncatedKey := fmt.Sprintf("%#x", bytesutil.Trunc(pubKey))
	pubKeyBytes := bytesutil.ToBytes48(pubKey)
	if slot < params.BeaconConfig().SlotsPerEpoch {
		v.prevBalance[pubKeyBytes] = params.BeaconConfig().MaxEffectiveBalance
	}

	// Safely load data from response with slice out of bounds checks. The server should return
	// the response with all slices of equal length, but the validator could panic if the server
	// did not do so for whatever reason.
	var balBeforeEpoch uint64
	var balAfterEpoch uint64
	var correctlyVotedSource bool
	var correctlyVotedTarget bool
	var correctlyVotedHead bool
	if index < len(resp.BalancesBeforeEpochTransition) {
		balBeforeEpoch = resp.BalancesBeforeEpochTransition[index]
	} else {
		log.WithField("pubKey", truncatedKey).Warn("Missing balance before epoch transition")
	}
	if index < len(resp.BalancesAfterEpochTransition) {
		balAfterEpoch = resp.BalancesAfterEpochTransition[index]
	}
	if index < len(resp.CorrectlyVotedSource) {
		correctlyVotedSource = resp.CorrectlyVotedSource[index]
	} else {
		log.WithField("pubKey", truncatedKey).Warn("Missing correctly voted source")
	}
	if index < len(resp.CorrectlyVotedTarget) {
		correctlyVotedTarget = resp.CorrectlyVotedTarget[index]
	} else {
		log.WithField("pubKey", truncatedKey).Warn("Missing correctly voted target")
	}
	if index < len(resp.CorrectlyVotedHead) {
		correctlyVotedHead = resp.CorrectlyVotedHead[index]
	} else {
		log.WithField("pubKey", truncatedKey).Warn("Missing correctly voted head")
	}

	if _, ok := v.startBalances[pubKeyBytes]; !ok {
		v.startBalances[pubKeyBytes] = balBeforeEpoch
	}

	fmtKey := fmt.Sprintf("%#x", pubKey)
	gweiPerEth := float64(params.BeaconConfig().GweiPerEth)
	if v.prevBalance[pubKeyBytes] > 0 {
		newBalance := float64(balAfterEpoch) / gweiPerEth
		prevBalance := float64(balBeforeEpoch) / gweiPerEth
		startBalance := float64(v.startBalances[pubKeyBytes]) / gweiPerEth
		percentNet := (newBalance - prevBalance) / prevBalance
		percentSinceStart := (newBalance - startBalance) / startBalance

		previousEpochSummaryFields := logrus.Fields{
			"pubKey":                  truncatedKey,
			"epoch":                   prevEpoch,
			"correctlyVotedSource":    correctlyVotedSource,
			"correctlyVotedTarget":    correctlyVotedTarget,
			"correctlyVotedHead":      correctlyVotedHead,
			"startBalance":            startBalance,
			"oldBalance":              prevBalance,
			"newBalance":              newBalance,
			"percentChange":           fmt.Sprintf("%.5f%%", percentNet*100),
			"percentChangeSinceStart": fmt.Sprintf("%.5f%%", percentSinceStart*100),
		}

		if slots.ToEpoch(slot) >= params.BeaconConfig().AltairForkEpoch {
			if index < len(resp.InactivityScores) {
				previousEpochSummaryFields["inactivityScore"] = resp.InactivityScores[index]
			} else {
				log.WithField("pubKey", truncatedKey).Warn("Missing inactivity score")
			}
		}

		log.WithFields(previousEpochSummaryFields).Info("Previous epoch voting summary")
		if v.emitAccountMetrics {
			ValidatorBalancesGaugeVec.WithLabelValues(fmtKey).Set(newBalance)
			if correctlyVotedSource {
				ValidatorCorrectlyVotedSourceGaugeVec.WithLabelValues(fmtKey).Set(1)
			} else {
				ValidatorCorrectlyVotedSourceGaugeVec.WithLabelValues(fmtKey).Set(0)
			}
			if correctlyVotedTarget {
				ValidatorCorrectlyVotedTargetGaugeVec.WithLabelValues(fmtKey).Set(1)
			} else {
				ValidatorCorrectlyVotedTargetGaugeVec.WithLabelValues(fmtKey).Set(0)
			}
			if correctlyVotedHead {
				ValidatorCorrectlyVotedHeadGaugeVec.WithLabelValues(fmtKey).Set(1)
			} else {
				ValidatorCorrectlyVotedHeadGaugeVec.WithLabelValues(fmtKey).Set(0)
			}

			// Phase0 specific metrics
			if slots.ToEpoch(slot) >= params.BeaconConfig().AltairForkEpoch {
				if index < len(resp.InactivityScores) {
					ValidatorInactivityScoreGaugeVec.WithLabelValues(fmtKey).Set(float64(resp.InactivityScores[index]))
				}
			}
		}
	}
	v.prevBalance[pubKeyBytes] = balBeforeEpoch
}

// UpdateLogAggregateStats updates and logs the voteStats struct of a validator using the RPC response obtained from LogValidatorGainsAndLosses.
func (v *validator) UpdateLogAggregateStats(resp *ethpb.ValidatorPerformanceResponse, slot primitives.Slot) {
	summary := &v.voteStats
	currentEpoch := primitives.Epoch(slot / params.BeaconConfig().SlotsPerEpoch)
	var attested, correctSource, correctTarget, correctHead, inactivityScore int

	for i := range resp.PublicKeys {
		included := false
		if i < len(resp.CorrectlyVotedSource) && resp.CorrectlyVotedSource[i] {
			included = true
			correctSource++
			summary.totalCorrectSource++
		}
		if i < len(resp.CorrectlyVotedTarget) && resp.CorrectlyVotedTarget[i] {
			included = true
			correctTarget++
			summary.totalCorrectTarget++
		}
		if i < len(resp.CorrectlyVotedHead) && resp.CorrectlyVotedHead[i] {
			// in altair, head == true also means source == true, hence no need to mark included true
			correctHead++
			summary.totalCorrectHead++
		}
		if included {
			attested++
			summary.totalAttestedCount++
		}
		// Altair metrics
		if slots.ToEpoch(slot) > params.BeaconConfig().AltairForkEpoch && i < len(resp.InactivityScores) {
			inactivityScore += int(resp.InactivityScores[i])
		}
	}

	// Return early if no attestation got included from previous epoch.
	// This happens when validators joined halfway through epoch and already passed its assigned slot.
	if attested == 0 {
		return
	}

	epochSummaryFields := logrus.Fields{
		"epoch":                   currentEpoch - 1,
		"attestationInclusionPct": fmt.Sprintf("%.0f%%", (float64(attested)/float64(len(resp.PublicKeys)))*100),
		"correctlyVotedSourcePct": fmt.Sprintf("%.0f%%", (float64(correctSource)/float64(attested))*100),
		"correctlyVotedTargetPct": fmt.Sprintf("%.0f%%", (float64(correctTarget)/float64(attested))*100),
		"correctlyVotedHeadPct":   fmt.Sprintf("%.0f%%", (float64(correctHead)/float64(attested))*100),
	}

	// Altair summary fields.
	if slots.ToEpoch(slot) > params.BeaconConfig().AltairForkEpoch && attested > 0 {
		epochSummaryFields["averageInactivityScore"] = fmt.Sprintf("%.0f", float64(inactivityScore)/float64(len(resp.PublicKeys)))
	}

	log.WithFields(epochSummaryFields).Info("Previous epoch aggregated voting summary")

	var totalStartBal, totalPrevBal uint64
	v.prevBalanceLock.RLock()
	for i, val := range v.startBalances {
		totalStartBal += val
		totalPrevBal += v.prevBalance[i]
	}
	v.prevBalanceLock.RUnlock()

	if totalStartBal == 0 || summary.totalAttestedCount == 0 {
		log.Error("Failed to print launch summary: one or more divisors is 0")
		return
	}

	summary.totalRequestedCount += uint64(len(resp.PublicKeys))

	launchSummaryFields := logrus.Fields{
		"numberOfEpochs":           fmt.Sprintf("%d", currentEpoch-summary.startEpoch),
		"attestationsInclusionPct": fmt.Sprintf("%.0f%%", (float64(summary.totalAttestedCount)/float64(summary.totalRequestedCount))*100),
		"correctlyVotedSourcePct":  fmt.Sprintf("%.0f%%", (float64(summary.totalCorrectSource)/float64(summary.totalAttestedCount))*100),
		"correctlyVotedTargetPct":  fmt.Sprintf("%.0f%%", (float64(summary.totalCorrectTarget)/float64(summary.totalAttestedCount))*100),
		"correctlyVotedHeadPct":    fmt.Sprintf("%.0f%%", (float64(summary.totalCorrectHead)/float64(summary.totalAttestedCount))*100),
		"pctChangeCombinedBalance": fmt.Sprintf("%.5f%%", (float64(totalPrevBal)-float64(totalStartBal))/float64(totalStartBal)*100),
	}

	// Add phase0 specific fields
	if slots.ToEpoch(slot) < params.BeaconConfig().AltairForkEpoch {
		launchSummaryFields["averageInclusionDistance"] = fmt.Sprintf("%.2f slots", float64(summary.totalDistance)/float64(summary.totalAttestedCount))
	}

	log.WithFields(launchSummaryFields).Info("Vote summary since launch")
}
