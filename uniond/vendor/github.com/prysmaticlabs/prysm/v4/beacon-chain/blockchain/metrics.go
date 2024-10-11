package blockchain

import (
	"context"

	"github.com/pkg/errors"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/altair"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/epoch/precompute"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
)

var (
	beaconSlot = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "beacon_slot",
		Help: "Latest slot of the beacon chain state",
	})
	beaconHeadSlot = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "beacon_head_slot",
		Help: "Slot of the head block of the beacon chain",
	})
	clockTimeSlot = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "beacon_clock_time_slot",
		Help: "The current slot based on the genesis time and current clock",
	})

	headFinalizedEpoch = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "head_finalized_epoch",
		Help: "Last finalized epoch of the head state",
	})
	headFinalizedRoot = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "head_finalized_root",
		Help: "Last finalized root of the head state",
	})
	beaconFinalizedEpoch = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "beacon_finalized_epoch",
		Help: "Last finalized epoch of the processed state",
	})
	beaconFinalizedRoot = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "beacon_finalized_root",
		Help: "Last finalized root of the processed state",
	})
	beaconCurrentJustifiedEpoch = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "beacon_current_justified_epoch",
		Help: "Current justified epoch of the processed state",
	})
	beaconCurrentJustifiedRoot = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "beacon_current_justified_root",
		Help: "Current justified root of the processed state",
	})
	beaconPrevJustifiedEpoch = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "beacon_previous_justified_epoch",
		Help: "Previous justified epoch of the processed state",
	})
	beaconPrevJustifiedRoot = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "beacon_previous_justified_root",
		Help: "Previous justified root of the processed state",
	})
	activeValidatorCount = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "beacon_current_active_validators",
		Help: "Current total active validators",
	})
	validatorsCount = promauto.NewGaugeVec(prometheus.GaugeOpts{
		Name: "validator_count",
		Help: "The total number of validators",
	}, []string{"state"})
	validatorsBalance = promauto.NewGaugeVec(prometheus.GaugeOpts{
		Name: "validators_total_balance",
		Help: "The total balance of validators, in GWei",
	}, []string{"state"})
	validatorsEffectiveBalance = promauto.NewGaugeVec(prometheus.GaugeOpts{
		Name: "validators_total_effective_balance",
		Help: "The total effective balance of validators, in GWei",
	}, []string{"state"})
	currentEth1DataDepositCount = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "current_eth1_data_deposit_count",
		Help: "The current eth1 deposit count in the last processed state eth1data field.",
	})
	processedDepositsCount = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "beacon_processed_deposits_total",
		Help: "Total number of deposits processed",
	})
	stateTrieReferences = promauto.NewGaugeVec(prometheus.GaugeOpts{
		Name: "field_references",
		Help: "The number of states a particular field is shared with.",
	}, []string{"state"})
	prevEpochActiveBalances = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "beacon_prev_epoch_active_gwei",
		Help: "The total amount of ether, in gwei, that was active for voting of previous epoch",
	})
	prevEpochSourceBalances = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "beacon_prev_epoch_source_gwei",
		Help: "The total amount of ether, in gwei, that has been used in voting attestation source of previous epoch",
	})
	prevEpochTargetBalances = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "beacon_prev_epoch_target_gwei",
		Help: "The total amount of ether, in gwei, that has been used in voting attestation target of previous epoch",
	})
	prevEpochHeadBalances = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "beacon_prev_epoch_head_gwei",
		Help: "The total amount of ether, in gwei, that has been used in voting attestation head of previous epoch",
	})
	reorgCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "beacon_reorgs_total",
		Help: "Count the number of times beacon chain has a reorg",
	})
	LateBlockAttemptedReorgCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "beacon_late_block_attempted_reorgs",
		Help: "Count the number of times a proposer served by this beacon has attempted a late block reorg",
	})
	lateBlockFailedAttemptFirstThreshold = promauto.NewCounter(prometheus.CounterOpts{
		Name: "beacon_failed_reorg_attempts_first_threshold",
		Help: "Count the number of times a proposer served by this beacon attempted a late block reorg but desisted in the first threshold",
	})
	lateBlockFailedAttemptSecondThreshold = promauto.NewCounter(prometheus.CounterOpts{
		Name: "beacon_failed_reorg_attempts_second_threshold",
		Help: "Count the number of times a proposer served by this beacon attempted a late block reorg but desisted in the second threshold",
	})
	saveOrphanedAttCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "saved_orphaned_att_total",
		Help: "Count the number of times an orphaned attestation is saved",
	})
	attestationInclusionDelay = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Name:    "attestation_inclusion_delay_slots",
			Help:    "The number of slots between att.Slot and block.Slot",
			Buckets: []float64{1, 2, 3, 4, 6, 32, 64},
		},
	)
	syncHeadStateMiss = promauto.NewCounter(prometheus.CounterOpts{
		Name: "sync_head_state_miss",
		Help: "The number of sync head state requests that are not present in the cache.",
	})
	syncHeadStateHit = promauto.NewCounter(prometheus.CounterOpts{
		Name: "sync_head_state_hit",
		Help: "The number of sync head state requests that are present in the cache.",
	})
	newPayloadValidNodeCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "new_payload_valid_node_count",
		Help: "Count the number of valid nodes after newPayload EE call",
	})
	newPayloadOptimisticNodeCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "new_payload_optimistic_node_count",
		Help: "Count the number of optimistic nodes after newPayload EE call",
	})
	newPayloadInvalidNodeCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "new_payload_invalid_node_count",
		Help: "Count the number of invalid nodes after newPayload EE call",
	})
	forkchoiceUpdatedValidNodeCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "forkchoice_updated_valid_node_count",
		Help: "Count the number of valid nodes after forkchoiceUpdated EE call",
	})
	forkchoiceUpdatedOptimisticNodeCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "forkchoice_updated_optimistic_node_count",
		Help: "Count the number of optimistic nodes after forkchoiceUpdated EE call",
	})
	forkchoiceUpdatedInvalidNodeCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "forkchoice_updated_invalid_node_count",
		Help: "Count the number of invalid nodes after forkchoiceUpdated EE call",
	})
	txsPerSlotCount = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "txs_per_slot_count",
		Help: "Count the number of txs per slot",
	})
	onBlockProcessingTime = promauto.NewSummary(prometheus.SummaryOpts{
		Name: "on_block_processing_milliseconds",
		Help: "Total time in milliseconds to complete a call to postBlockProcess()",
	})
	stateTransitionProcessingTime = promauto.NewSummary(prometheus.SummaryOpts{
		Name: "state_transition_processing_milliseconds",
		Help: "Total time to call a state transition in validateStateTransition()",
	})
	chainServiceProcessingTime = promauto.NewSummary(prometheus.SummaryOpts{
		Name: "chain_service_processing_milliseconds",
		Help: "Total time to call a chain service in ReceiveBlock()",
	})
	processAttsElapsedTime = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Name:    "process_attestations_milliseconds",
			Help:    "Captures latency for process attestations (forkchoice) in milliseconds",
			Buckets: []float64{1, 5, 20, 100, 500, 1000},
		},
	)
	newAttHeadElapsedTime = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Name:    "new_att_head_milliseconds",
			Help:    "Captures latency for new attestation head in milliseconds",
			Buckets: []float64{1, 5, 20, 100, 500, 1000},
		},
	)
	newBlockHeadElapsedTime = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Name:    "new_block_head_milliseconds",
			Help:    "Captures latency for new block head in milliseconds",
			Buckets: []float64{1, 5, 20, 100, 500, 1000},
		},
	)
	reorgDistance = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Name:    "reorg_distance",
			Help:    "Captures distance of reorgs. Distance is defined as the number of blocks between the old head and the new head",
			Buckets: []float64{1, 2, 4, 8, 16, 32, 64},
		},
	)
	reorgDepth = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Name:    "reorg_depth",
			Help:    "Captures depth of reorgs. Depth is defined as the number of blocks between the head and the common ancestor",
			Buckets: []float64{1, 2, 4, 8, 16, 32},
		},
	)
)

// reportSlotMetrics reports slot related metrics.
func reportSlotMetrics(stateSlot, headSlot, clockSlot primitives.Slot, finalizedCheckpoint *ethpb.Checkpoint) {
	clockTimeSlot.Set(float64(clockSlot))
	beaconSlot.Set(float64(stateSlot))
	beaconHeadSlot.Set(float64(headSlot))
	if finalizedCheckpoint != nil {
		headFinalizedEpoch.Set(float64(finalizedCheckpoint.Epoch))
		headFinalizedRoot.Set(float64(bytesutil.ToLowInt64(finalizedCheckpoint.Root)))
	}
}

// reportEpochMetrics reports epoch related metrics.
func reportEpochMetrics(ctx context.Context, postState, headState state.BeaconState) error {
	currentEpoch := primitives.Epoch(postState.Slot() / params.BeaconConfig().SlotsPerEpoch)

	// Validator instances
	pendingInstances := 0
	activeInstances := 0
	slashingInstances := 0
	slashedInstances := 0
	exitingInstances := 0
	exitedInstances := 0
	// Validator balances
	pendingBalance := uint64(0)
	activeBalance := uint64(0)
	activeEffectiveBalance := uint64(0)
	exitingBalance := uint64(0)
	exitingEffectiveBalance := uint64(0)
	slashingBalance := uint64(0)
	slashingEffectiveBalance := uint64(0)

	for i := 0; i < postState.NumValidators(); i++ {
		validator, err := postState.ValidatorAtIndexReadOnly(primitives.ValidatorIndex(i))
		if err != nil {
			log.WithError(err).Error("Could not load validator")
			continue
		}
		bal, err := postState.BalanceAtIndex(primitives.ValidatorIndex(i))
		if err != nil {
			log.WithError(err).Error("Could not load validator balance")
			continue
		}
		if validator.Slashed() {
			if currentEpoch < validator.ExitEpoch() {
				slashingInstances++
				slashingBalance += bal
				slashingEffectiveBalance += validator.EffectiveBalance()
			} else {
				slashedInstances++
			}
			continue
		}
		if validator.ExitEpoch() != params.BeaconConfig().FarFutureEpoch {
			if currentEpoch < validator.ExitEpoch() {
				exitingInstances++
				exitingBalance += bal
				exitingEffectiveBalance += validator.EffectiveBalance()
			} else {
				exitedInstances++
			}
			continue
		}
		if currentEpoch < validator.ActivationEpoch() {
			pendingInstances++
			pendingBalance += bal
			continue
		}
		activeInstances++
		activeBalance += bal
		activeEffectiveBalance += validator.EffectiveBalance()
	}
	activeInstances += exitingInstances + slashingInstances
	activeBalance += exitingBalance + slashingBalance
	activeEffectiveBalance += exitingEffectiveBalance + slashingEffectiveBalance

	activeValidatorCount.Set(float64(activeInstances))
	validatorsCount.WithLabelValues("Pending").Set(float64(pendingInstances))
	validatorsCount.WithLabelValues("Active").Set(float64(activeInstances))
	validatorsCount.WithLabelValues("Exiting").Set(float64(exitingInstances))
	validatorsCount.WithLabelValues("Exited").Set(float64(exitedInstances))
	validatorsCount.WithLabelValues("Slashing").Set(float64(slashingInstances))
	validatorsCount.WithLabelValues("Slashed").Set(float64(slashedInstances))
	validatorsBalance.WithLabelValues("Pending").Set(float64(pendingBalance))
	validatorsBalance.WithLabelValues("Active").Set(float64(activeBalance))
	validatorsBalance.WithLabelValues("Exiting").Set(float64(exitingBalance))
	validatorsBalance.WithLabelValues("Slashing").Set(float64(slashingBalance))
	validatorsEffectiveBalance.WithLabelValues("Active").Set(float64(activeEffectiveBalance))
	validatorsEffectiveBalance.WithLabelValues("Exiting").Set(float64(exitingEffectiveBalance))
	validatorsEffectiveBalance.WithLabelValues("Slashing").Set(float64(slashingEffectiveBalance))

	// Last justified slot
	beaconCurrentJustifiedEpoch.Set(float64(postState.CurrentJustifiedCheckpoint().Epoch))
	beaconCurrentJustifiedRoot.Set(float64(bytesutil.ToLowInt64(postState.CurrentJustifiedCheckpoint().Root)))

	// Last previous justified slot
	beaconPrevJustifiedEpoch.Set(float64(postState.PreviousJustifiedCheckpoint().Epoch))
	beaconPrevJustifiedRoot.Set(float64(bytesutil.ToLowInt64(postState.PreviousJustifiedCheckpoint().Root)))

	// Last finalized slot
	beaconFinalizedEpoch.Set(float64(postState.FinalizedCheckpointEpoch()))
	beaconFinalizedRoot.Set(float64(bytesutil.ToLowInt64(postState.FinalizedCheckpoint().Root)))
	currentEth1DataDepositCount.Set(float64(postState.Eth1Data().DepositCount))
	processedDepositsCount.Set(float64(postState.Eth1DepositIndex() + 1))

	var b *precompute.Balance
	var v []*precompute.Validator
	var err error

	if headState.Version() == version.Phase0 {
		v, b, err = precompute.New(ctx, headState)
		if err != nil {
			return err
		}
		_, b, err = precompute.ProcessAttestations(ctx, headState, v, b)
		if err != nil {
			return err
		}
	} else if headState.Version() >= version.Altair {
		v, b, err = altair.InitializePrecomputeValidators(ctx, headState)
		if err != nil {
			return err
		}
		_, b, err = altair.ProcessEpochParticipation(ctx, headState, b, v)
		if err != nil {
			return err
		}
	} else {
		return errors.Errorf("invalid state type provided: %T", headState.ToProtoUnsafe())
	}

	prevEpochActiveBalances.Set(float64(b.ActivePrevEpoch))
	prevEpochSourceBalances.Set(float64(b.PrevEpochAttested))
	prevEpochTargetBalances.Set(float64(b.PrevEpochTargetAttested))
	prevEpochHeadBalances.Set(float64(b.PrevEpochHeadAttested))

	refMap := postState.FieldReferencesCount()
	for name, val := range refMap {
		stateTrieReferences.WithLabelValues(name).Set(float64(val))
	}
	postState.RecordStateMetrics()

	return nil
}

func reportAttestationInclusion(blk interfaces.ReadOnlyBeaconBlock) {
	for _, att := range blk.Body().Attestations() {
		attestationInclusionDelay.Observe(float64(blk.Slot() - att.Data.Slot))
	}
}
