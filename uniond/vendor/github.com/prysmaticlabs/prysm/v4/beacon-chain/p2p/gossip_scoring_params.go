package p2p

import (
	"context"
	"math"
	"reflect"
	"strings"
	"time"

	pubsub "github.com/libp2p/go-libp2p-pubsub"
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	coreTime "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/sirupsen/logrus"
)

const (
	// beaconBlockWeight specifies the scoring weight that we apply to
	// our beacon block topic.
	beaconBlockWeight = 0.8
	// aggregateWeight specifies the scoring weight that we apply to
	// our aggregate topic.
	aggregateWeight = 0.5
	// syncContributionWeight specifies the scoring weight that we apply to
	// our sync contribution topic.
	syncContributionWeight = 0.2
	// attestationTotalWeight specifies the scoring weight that we apply to
	// our attestation subnet topic.
	attestationTotalWeight = 1
	// syncCommitteesTotalWeight specifies the scoring weight that we apply to
	// our sync subnet topic.
	syncCommitteesTotalWeight = 0.4
	// attesterSlashingWeight specifies the scoring weight that we apply to
	// our attester slashing topic.
	attesterSlashingWeight = 0.05
	// proposerSlashingWeight specifies the scoring weight that we apply to
	// our proposer slashing topic.
	proposerSlashingWeight = 0.05
	// voluntaryExitWeight specifies the scoring weight that we apply to
	// our voluntary exit topic.
	voluntaryExitWeight = 0.05
	// blsToExecutionChangeWeight specifies the scoring weight that we apply to
	// our bls to execution topic.
	blsToExecutionChangeWeight = 0.05

	// maxInMeshScore describes the max score a peer can attain from being in the mesh.
	maxInMeshScore = 10
	// maxFirstDeliveryScore describes the max score a peer can obtain from first deliveries.
	maxFirstDeliveryScore = 40

	// decayToZero specifies the terminal value that we will use when decaying
	// a value.
	decayToZero = 0.01

	// dampeningFactor reduces the amount by which the various thresholds and caps are created.
	dampeningFactor = 90
)

var (
	// a bool to check if we enable scoring for messages in the mesh sent for near first deliveries.
	meshDeliveryIsScored = false

	// Defines the variables representing the different time periods.
	oneHundredEpochs   = 100 * oneEpochDuration()
	invalidDecayPeriod = 50 * oneEpochDuration()
	twentyEpochs       = 20 * oneEpochDuration()
	tenEpochs          = 10 * oneEpochDuration()
)

func peerScoringParams() (*pubsub.PeerScoreParams, *pubsub.PeerScoreThresholds) {
	thresholds := &pubsub.PeerScoreThresholds{
		GossipThreshold:             -4000,
		PublishThreshold:            -8000,
		GraylistThreshold:           -16000,
		AcceptPXThreshold:           100,
		OpportunisticGraftThreshold: 5,
	}
	scoreParams := &pubsub.PeerScoreParams{
		Topics:        make(map[string]*pubsub.TopicScoreParams),
		TopicScoreCap: 32.72,
		AppSpecificScore: func(p peer.ID) float64 {
			return 0
		},
		AppSpecificWeight:           1,
		IPColocationFactorWeight:    -35.11,
		IPColocationFactorThreshold: 10,
		IPColocationFactorWhitelist: nil,
		BehaviourPenaltyWeight:      -15.92,
		BehaviourPenaltyThreshold:   6,
		BehaviourPenaltyDecay:       scoreDecay(tenEpochs),
		DecayInterval:               oneSlotDuration(),
		DecayToZero:                 decayToZero,
		RetainScore:                 oneHundredEpochs,
	}
	return scoreParams, thresholds
}

func (s *Service) topicScoreParams(topic string) (*pubsub.TopicScoreParams, error) {
	activeValidators, err := s.retrieveActiveValidators()
	if err != nil {
		return nil, err
	}
	switch {
	case strings.Contains(topic, GossipBlockMessage):
		return defaultBlockTopicParams(), nil
	case strings.Contains(topic, GossipAggregateAndProofMessage):
		return defaultAggregateTopicParams(activeValidators), nil
	case strings.Contains(topic, GossipAttestationMessage):
		return defaultAggregateSubnetTopicParams(activeValidators), nil
	case strings.Contains(topic, GossipSyncCommitteeMessage):
		return defaultSyncSubnetTopicParams(activeValidators), nil
	case strings.Contains(topic, GossipContributionAndProofMessage):
		return defaultSyncContributionTopicParams(), nil
	case strings.Contains(topic, GossipExitMessage):
		return defaultVoluntaryExitTopicParams(), nil
	case strings.Contains(topic, GossipProposerSlashingMessage):
		return defaultProposerSlashingTopicParams(), nil
	case strings.Contains(topic, GossipAttesterSlashingMessage):
		return defaultAttesterSlashingTopicParams(), nil
	case strings.Contains(topic, GossipBlsToExecutionChangeMessage):
		return defaultBlsToExecutionChangeTopicParams(), nil
	case strings.Contains(topic, GossipBlobSidecarMessage):
		// TODO(Deneb): Using the default block scoring. But this should be updated.
		return defaultBlockTopicParams(), nil
	default:
		return nil, errors.Errorf("unrecognized topic provided for parameter registration: %s", topic)
	}
}

func (s *Service) retrieveActiveValidators() (uint64, error) {
	if s.activeValidatorCount != 0 {
		return s.activeValidatorCount, nil
	}
	rt := s.cfg.DB.LastArchivedRoot(s.ctx)
	if rt == params.BeaconConfig().ZeroHash {
		genState, err := s.cfg.DB.GenesisState(s.ctx)
		if err != nil {
			return 0, err
		}
		if genState == nil || genState.IsNil() {
			return 0, errors.New("no genesis state exists")
		}
		activeVals, err := helpers.ActiveValidatorCount(context.Background(), genState, coreTime.CurrentEpoch(genState))
		if err != nil {
			return 0, err
		}
		// Cache active validator count
		s.activeValidatorCount = activeVals
		return activeVals, nil
	}
	bState, err := s.cfg.DB.State(s.ctx, rt)
	if err != nil {
		return 0, err
	}
	if bState == nil || bState.IsNil() {
		return 0, errors.Errorf("no state with root %#x exists", rt)
	}
	activeVals, err := helpers.ActiveValidatorCount(context.Background(), bState, coreTime.CurrentEpoch(bState))
	if err != nil {
		return 0, err
	}
	// Cache active validator count
	s.activeValidatorCount = activeVals
	return activeVals, nil
}

// Based on the lighthouse parameters.
// https://gist.github.com/blacktemplar/5c1862cb3f0e32a1a7fb0b25e79e6e2c

func defaultBlockTopicParams() *pubsub.TopicScoreParams {
	decayEpoch := time.Duration(5)
	blocksPerEpoch := uint64(params.BeaconConfig().SlotsPerEpoch)
	meshWeight := -0.717
	if !meshDeliveryIsScored {
		// Set the mesh weight as zero as a temporary measure, so as to prevent
		// the average nodes from being penalised.
		meshWeight = 0
	}
	return &pubsub.TopicScoreParams{
		TopicWeight:                     beaconBlockWeight,
		TimeInMeshWeight:                maxInMeshScore / inMeshCap(),
		TimeInMeshQuantum:               inMeshTime(),
		TimeInMeshCap:                   inMeshCap(),
		FirstMessageDeliveriesWeight:    1,
		FirstMessageDeliveriesDecay:     scoreDecay(twentyEpochs),
		FirstMessageDeliveriesCap:       23,
		MeshMessageDeliveriesWeight:     meshWeight,
		MeshMessageDeliveriesDecay:      scoreDecay(decayEpoch * oneEpochDuration()),
		MeshMessageDeliveriesCap:        float64(blocksPerEpoch * uint64(decayEpoch)),
		MeshMessageDeliveriesThreshold:  float64(blocksPerEpoch*uint64(decayEpoch)) / 10,
		MeshMessageDeliveriesWindow:     2 * time.Second,
		MeshMessageDeliveriesActivation: 4 * oneEpochDuration(),
		MeshFailurePenaltyWeight:        meshWeight,
		MeshFailurePenaltyDecay:         scoreDecay(decayEpoch * oneEpochDuration()),
		InvalidMessageDeliveriesWeight:  -140.4475,
		InvalidMessageDeliveriesDecay:   scoreDecay(invalidDecayPeriod),
	}
}

func defaultAggregateTopicParams(activeValidators uint64) *pubsub.TopicScoreParams {
	// Determine the expected message rate for the particular gossip topic.
	aggPerSlot := aggregatorsPerSlot(activeValidators)
	firstMessageCap, err := decayLimit(scoreDecay(1*oneEpochDuration()), float64(aggPerSlot*2/gossipSubD))
	if err != nil {
		log.WithError(err).Warn("skipping initializing topic scoring")
		return nil
	}
	firstMessageWeight := maxFirstDeliveryScore / firstMessageCap
	meshThreshold, err := decayThreshold(scoreDecay(1*oneEpochDuration()), float64(aggPerSlot)/dampeningFactor)
	if err != nil {
		log.WithError(err).Warn("skipping initializing topic scoring")
		return nil
	}
	meshWeight := -scoreByWeight(aggregateWeight, meshThreshold)
	meshCap := 4 * meshThreshold
	if !meshDeliveryIsScored {
		// Set the mesh weight as zero as a temporary measure, so as to prevent
		// the average nodes from being penalised.
		meshWeight = 0
	}
	return &pubsub.TopicScoreParams{
		TopicWeight:                     aggregateWeight,
		TimeInMeshWeight:                maxInMeshScore / inMeshCap(),
		TimeInMeshQuantum:               inMeshTime(),
		TimeInMeshCap:                   inMeshCap(),
		FirstMessageDeliveriesWeight:    firstMessageWeight,
		FirstMessageDeliveriesDecay:     scoreDecay(1 * oneEpochDuration()),
		FirstMessageDeliveriesCap:       firstMessageCap,
		MeshMessageDeliveriesWeight:     meshWeight,
		MeshMessageDeliveriesDecay:      scoreDecay(1 * oneEpochDuration()),
		MeshMessageDeliveriesCap:        meshCap,
		MeshMessageDeliveriesThreshold:  meshThreshold,
		MeshMessageDeliveriesWindow:     2 * time.Second,
		MeshMessageDeliveriesActivation: 1 * oneEpochDuration(),
		MeshFailurePenaltyWeight:        meshWeight,
		MeshFailurePenaltyDecay:         scoreDecay(1 * oneEpochDuration()),
		InvalidMessageDeliveriesWeight:  -maxScore() / aggregateWeight,
		InvalidMessageDeliveriesDecay:   scoreDecay(invalidDecayPeriod),
	}
}

func defaultSyncContributionTopicParams() *pubsub.TopicScoreParams {
	// Determine the expected message rate for the particular gossip topic.
	aggPerSlot := params.BeaconConfig().SyncCommitteeSubnetCount * params.BeaconConfig().TargetAggregatorsPerSyncSubcommittee
	firstMessageCap, err := decayLimit(scoreDecay(1*oneEpochDuration()), float64(aggPerSlot*2/gossipSubD))
	if err != nil {
		log.WithError(err).Warn("skipping initializing topic scoring")
		return nil
	}
	firstMessageWeight := maxFirstDeliveryScore / firstMessageCap
	meshThreshold, err := decayThreshold(scoreDecay(1*oneEpochDuration()), float64(aggPerSlot)/dampeningFactor)
	if err != nil {
		log.WithError(err).Warn("skipping initializing topic scoring")
		return nil
	}
	meshWeight := -scoreByWeight(syncContributionWeight, meshThreshold)
	meshCap := 4 * meshThreshold
	if !meshDeliveryIsScored {
		// Set the mesh weight as zero as a temporary measure, so as to prevent
		// the average nodes from being penalised.
		meshWeight = 0
	}
	return &pubsub.TopicScoreParams{
		TopicWeight:                     syncContributionWeight,
		TimeInMeshWeight:                maxInMeshScore / inMeshCap(),
		TimeInMeshQuantum:               inMeshTime(),
		TimeInMeshCap:                   inMeshCap(),
		FirstMessageDeliveriesWeight:    firstMessageWeight,
		FirstMessageDeliveriesDecay:     scoreDecay(1 * oneEpochDuration()),
		FirstMessageDeliveriesCap:       firstMessageCap,
		MeshMessageDeliveriesWeight:     meshWeight,
		MeshMessageDeliveriesDecay:      scoreDecay(1 * oneEpochDuration()),
		MeshMessageDeliveriesCap:        meshCap,
		MeshMessageDeliveriesThreshold:  meshThreshold,
		MeshMessageDeliveriesWindow:     2 * time.Second,
		MeshMessageDeliveriesActivation: 1 * oneEpochDuration(),
		MeshFailurePenaltyWeight:        meshWeight,
		MeshFailurePenaltyDecay:         scoreDecay(1 * oneEpochDuration()),
		InvalidMessageDeliveriesWeight:  -maxScore() / syncContributionWeight,
		InvalidMessageDeliveriesDecay:   scoreDecay(invalidDecayPeriod),
	}
}

func defaultAggregateSubnetTopicParams(activeValidators uint64) *pubsub.TopicScoreParams {
	subnetCount := params.BeaconConfig().AttestationSubnetCount
	// Get weight for each specific subnet.
	topicWeight := attestationTotalWeight / float64(subnetCount)
	subnetWeight := activeValidators / subnetCount
	if subnetWeight == 0 {
		log.Warn("Subnet weight is 0, skipping initializing topic scoring")
		return nil
	}
	// Determine the amount of validators expected in a subnet in a single slot.
	numPerSlot := time.Duration(subnetWeight / uint64(params.BeaconConfig().SlotsPerEpoch))
	if numPerSlot == 0 {
		log.Warn("numPerSlot is 0, skipping initializing topic scoring")
		return nil
	}
	comsPerSlot := committeeCountPerSlot(activeValidators)
	exceedsThreshold := comsPerSlot >= 2*subnetCount/uint64(params.BeaconConfig().SlotsPerEpoch)
	firstDecay := time.Duration(1)
	meshDecay := time.Duration(4)
	if exceedsThreshold {
		firstDecay = 4
		meshDecay = 16
	}
	rate := numPerSlot * 2 / gossipSubD
	if rate == 0 {
		log.Warn("rate is 0, skipping initializing topic scoring")
		return nil
	}
	// Determine expected first deliveries based on the message rate.
	firstMessageCap, err := decayLimit(scoreDecay(firstDecay*oneEpochDuration()), float64(rate))
	if err != nil {
		log.WithError(err).Warn("skipping initializing topic scoring")
		return nil
	}
	firstMessageWeight := maxFirstDeliveryScore / firstMessageCap
	// Determine expected mesh deliveries based on message rate applied with a dampening factor.
	meshThreshold, err := decayThreshold(scoreDecay(meshDecay*oneEpochDuration()), float64(numPerSlot)/dampeningFactor)
	if err != nil {
		log.WithError(err).Warn("skipping initializing topic scoring")
		return nil
	}
	meshWeight := -scoreByWeight(topicWeight, meshThreshold)
	meshCap := 4 * meshThreshold
	if !meshDeliveryIsScored {
		// Set the mesh weight as zero as a temporary measure, so as to prevent
		// the average nodes from being penalised.
		meshWeight = 0
	}
	return &pubsub.TopicScoreParams{
		TopicWeight:                     topicWeight,
		TimeInMeshWeight:                maxInMeshScore / inMeshCap(),
		TimeInMeshQuantum:               inMeshTime(),
		TimeInMeshCap:                   inMeshCap(),
		FirstMessageDeliveriesWeight:    firstMessageWeight,
		FirstMessageDeliveriesDecay:     scoreDecay(firstDecay * oneEpochDuration()),
		FirstMessageDeliveriesCap:       firstMessageCap,
		MeshMessageDeliveriesWeight:     meshWeight,
		MeshMessageDeliveriesDecay:      scoreDecay(meshDecay * oneEpochDuration()),
		MeshMessageDeliveriesCap:        meshCap,
		MeshMessageDeliveriesThreshold:  meshThreshold,
		MeshMessageDeliveriesWindow:     2 * time.Second,
		MeshMessageDeliveriesActivation: 1 * oneEpochDuration(),
		MeshFailurePenaltyWeight:        meshWeight,
		MeshFailurePenaltyDecay:         scoreDecay(meshDecay * oneEpochDuration()),
		InvalidMessageDeliveriesWeight:  -maxScore() / topicWeight,
		InvalidMessageDeliveriesDecay:   scoreDecay(invalidDecayPeriod),
	}
}

func defaultSyncSubnetTopicParams(activeValidators uint64) *pubsub.TopicScoreParams {
	subnetCount := params.BeaconConfig().SyncCommitteeSubnetCount
	// Get weight for each specific subnet.
	topicWeight := syncCommitteesTotalWeight / float64(subnetCount)
	syncComSize := params.BeaconConfig().SyncCommitteeSize
	// Set the max as the sync committee size
	if activeValidators > syncComSize {
		activeValidators = syncComSize
	}
	subnetWeight := activeValidators / subnetCount
	if subnetWeight == 0 {
		log.Warn("Subnet weight is 0, skipping initializing topic scoring")
		return nil
	}
	firstDecay := time.Duration(1)
	meshDecay := time.Duration(4)

	rate := subnetWeight * 2 / gossipSubD
	if rate == 0 {
		log.Warn("rate is 0, skipping initializing topic scoring")
		return nil
	}
	// Determine expected first deliveries based on the message rate.
	firstMessageCap, err := decayLimit(scoreDecay(firstDecay*oneEpochDuration()), float64(rate))
	if err != nil {
		log.WithError(err).Warn("Skipping initializing topic scoring")
		return nil
	}
	firstMessageWeight := maxFirstDeliveryScore / firstMessageCap
	// Determine expected mesh deliveries based on message rate applied with a dampening factor.
	meshThreshold, err := decayThreshold(scoreDecay(meshDecay*oneEpochDuration()), float64(subnetWeight)/dampeningFactor)
	if err != nil {
		log.WithError(err).Warn("Skipping initializing topic scoring")
		return nil
	}
	meshWeight := -scoreByWeight(topicWeight, meshThreshold)
	meshCap := 4 * meshThreshold
	if !meshDeliveryIsScored {
		// Set the mesh weight as zero as a temporary measure, so as to prevent
		// the average nodes from being penalised.
		meshWeight = 0
	}
	return &pubsub.TopicScoreParams{
		TopicWeight:                     topicWeight,
		TimeInMeshWeight:                maxInMeshScore / inMeshCap(),
		TimeInMeshQuantum:               inMeshTime(),
		TimeInMeshCap:                   inMeshCap(),
		FirstMessageDeliveriesWeight:    firstMessageWeight,
		FirstMessageDeliveriesDecay:     scoreDecay(firstDecay * oneEpochDuration()),
		FirstMessageDeliveriesCap:       firstMessageCap,
		MeshMessageDeliveriesWeight:     meshWeight,
		MeshMessageDeliveriesDecay:      scoreDecay(meshDecay * oneEpochDuration()),
		MeshMessageDeliveriesCap:        meshCap,
		MeshMessageDeliveriesThreshold:  meshThreshold,
		MeshMessageDeliveriesWindow:     2 * time.Second,
		MeshMessageDeliveriesActivation: 1 * oneEpochDuration(),
		MeshFailurePenaltyWeight:        meshWeight,
		MeshFailurePenaltyDecay:         scoreDecay(meshDecay * oneEpochDuration()),
		InvalidMessageDeliveriesWeight:  -maxScore() / topicWeight,
		InvalidMessageDeliveriesDecay:   scoreDecay(invalidDecayPeriod),
	}
}

func defaultAttesterSlashingTopicParams() *pubsub.TopicScoreParams {
	return &pubsub.TopicScoreParams{
		TopicWeight:                     attesterSlashingWeight,
		TimeInMeshWeight:                maxInMeshScore / inMeshCap(),
		TimeInMeshQuantum:               inMeshTime(),
		TimeInMeshCap:                   inMeshCap(),
		FirstMessageDeliveriesWeight:    36,
		FirstMessageDeliveriesDecay:     scoreDecay(oneHundredEpochs),
		FirstMessageDeliveriesCap:       1,
		MeshMessageDeliveriesWeight:     0,
		MeshMessageDeliveriesDecay:      0,
		MeshMessageDeliveriesCap:        0,
		MeshMessageDeliveriesThreshold:  0,
		MeshMessageDeliveriesWindow:     0,
		MeshMessageDeliveriesActivation: 0,
		MeshFailurePenaltyWeight:        0,
		MeshFailurePenaltyDecay:         0,
		InvalidMessageDeliveriesWeight:  -2000,
		InvalidMessageDeliveriesDecay:   scoreDecay(invalidDecayPeriod),
	}
}

func defaultProposerSlashingTopicParams() *pubsub.TopicScoreParams {
	return &pubsub.TopicScoreParams{
		TopicWeight:                     proposerSlashingWeight,
		TimeInMeshWeight:                maxInMeshScore / inMeshCap(),
		TimeInMeshQuantum:               inMeshTime(),
		TimeInMeshCap:                   inMeshCap(),
		FirstMessageDeliveriesWeight:    36,
		FirstMessageDeliveriesDecay:     scoreDecay(oneHundredEpochs),
		FirstMessageDeliveriesCap:       1,
		MeshMessageDeliveriesWeight:     0,
		MeshMessageDeliveriesDecay:      0,
		MeshMessageDeliveriesCap:        0,
		MeshMessageDeliveriesThreshold:  0,
		MeshMessageDeliveriesWindow:     0,
		MeshMessageDeliveriesActivation: 0,
		MeshFailurePenaltyWeight:        0,
		MeshFailurePenaltyDecay:         0,
		InvalidMessageDeliveriesWeight:  -2000,
		InvalidMessageDeliveriesDecay:   scoreDecay(invalidDecayPeriod),
	}
}

func defaultVoluntaryExitTopicParams() *pubsub.TopicScoreParams {
	return &pubsub.TopicScoreParams{
		TopicWeight:                     voluntaryExitWeight,
		TimeInMeshWeight:                maxInMeshScore / inMeshCap(),
		TimeInMeshQuantum:               inMeshTime(),
		TimeInMeshCap:                   inMeshCap(),
		FirstMessageDeliveriesWeight:    2,
		FirstMessageDeliveriesDecay:     scoreDecay(oneHundredEpochs),
		FirstMessageDeliveriesCap:       5,
		MeshMessageDeliveriesWeight:     0,
		MeshMessageDeliveriesDecay:      0,
		MeshMessageDeliveriesCap:        0,
		MeshMessageDeliveriesThreshold:  0,
		MeshMessageDeliveriesWindow:     0,
		MeshMessageDeliveriesActivation: 0,
		MeshFailurePenaltyWeight:        0,
		MeshFailurePenaltyDecay:         0,
		InvalidMessageDeliveriesWeight:  -2000,
		InvalidMessageDeliveriesDecay:   scoreDecay(invalidDecayPeriod),
	}
}

func defaultBlsToExecutionChangeTopicParams() *pubsub.TopicScoreParams {
	return &pubsub.TopicScoreParams{
		TopicWeight:                     blsToExecutionChangeWeight,
		TimeInMeshWeight:                maxInMeshScore / inMeshCap(),
		TimeInMeshQuantum:               inMeshTime(),
		TimeInMeshCap:                   inMeshCap(),
		FirstMessageDeliveriesWeight:    2,
		FirstMessageDeliveriesDecay:     scoreDecay(oneHundredEpochs),
		FirstMessageDeliveriesCap:       5,
		MeshMessageDeliveriesWeight:     0,
		MeshMessageDeliveriesDecay:      0,
		MeshMessageDeliveriesCap:        0,
		MeshMessageDeliveriesThreshold:  0,
		MeshMessageDeliveriesWindow:     0,
		MeshMessageDeliveriesActivation: 0,
		MeshFailurePenaltyWeight:        0,
		MeshFailurePenaltyDecay:         0,
		InvalidMessageDeliveriesWeight:  -2000,
		InvalidMessageDeliveriesDecay:   scoreDecay(invalidDecayPeriod),
	}
}

func oneSlotDuration() time.Duration {
	return time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Second
}

func oneEpochDuration() time.Duration {
	return time.Duration(params.BeaconConfig().SlotsPerEpoch) * oneSlotDuration()
}

// determines the decay rate from the provided time period till
// the decayToZero value. Ex: ( 1 -> 0.01)
func scoreDecay(totalDurationDecay time.Duration) float64 {
	numOfTimes := totalDurationDecay / oneSlotDuration()
	return math.Pow(decayToZero, 1/float64(numOfTimes))
}

// is used to determine the threshold from the decay limit with
// a provided growth rate. This applies the decay rate to a
// computed limit.
func decayThreshold(decayRate, rate float64) (float64, error) {
	d, err := decayLimit(decayRate, rate)
	if err != nil {
		return 0, err
	}
	return d * decayRate, nil
}

// decayLimit provides the value till which a decay process will
// limit till provided with an expected growth rate.
func decayLimit(decayRate, rate float64) (float64, error) {
	if 1 <= decayRate {
		return 0, errors.Errorf("got an invalid decayLimit rate: %f", decayRate)
	}
	return rate / (1 - decayRate), nil
}

func committeeCountPerSlot(activeValidators uint64) uint64 {
	// Use a static parameter for now rather than a dynamic one, we can use
	// the actual parameter later when we have figured out how to fix a circular
	// dependency in service startup order.
	return helpers.SlotCommitteeCount(activeValidators)
}

// Uses a very rough gauge for total aggregator size per slot.
func aggregatorsPerSlot(activeValidators uint64) uint64 {
	comms := committeeCountPerSlot(activeValidators)
	totalAggs := comms * params.BeaconConfig().TargetAggregatorsPerCommittee
	return totalAggs
}

// provides the relevant score by the provided weight and threshold.
func scoreByWeight(weight, threshold float64) float64 {
	return maxScore() / (weight * threshold * threshold)
}

// maxScore attainable by a peer.
func maxScore() float64 {
	totalWeight := beaconBlockWeight + aggregateWeight + syncContributionWeight +
		attestationTotalWeight + syncCommitteesTotalWeight + attesterSlashingWeight +
		proposerSlashingWeight + voluntaryExitWeight + blsToExecutionChangeWeight
	return (maxInMeshScore + maxFirstDeliveryScore) * totalWeight
}

// denotes the unit time in mesh for scoring tallying.
func inMeshTime() time.Duration {
	return 1 * oneSlotDuration()
}

// the cap for `inMesh` time scoring.
func inMeshCap() float64 {
	return float64((3600 * time.Second) / inMeshTime())
}

func logGossipParameters(topic string, params *pubsub.TopicScoreParams) {
	// Exit early in the event the parameter struct is nil.
	if params == nil {
		return
	}
	rawParams := reflect.ValueOf(params).Elem()
	numOfFields := rawParams.NumField()

	fields := make(logrus.Fields, numOfFields)
	for i := 0; i < numOfFields; i++ {
		fields[reflect.TypeOf(params).Elem().Field(i).Name] = rawParams.Field(i).Interface()
	}
	log.WithFields(fields).Debugf("Topic Parameters for %s", topic)
}
