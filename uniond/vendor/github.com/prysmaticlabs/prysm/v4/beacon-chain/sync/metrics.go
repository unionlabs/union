package sync

import (
	"fmt"
	"reflect"
	"strings"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	pb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

var (
	topicPeerCount = promauto.NewGaugeVec(
		prometheus.GaugeOpts{
			Name: "p2p_topic_peer_count",
			Help: "The number of peers subscribed to a given topic.",
		}, []string{"topic"},
	)
	subscribedTopicPeerCount = promauto.NewGaugeVec(
		prometheus.GaugeOpts{
			Name: "p2p_subscribed_topic_peer_total",
			Help: "The number of peers subscribed to topics that a host node is also subscribed to.",
		}, []string{"topic"},
	)
	messageReceivedCounter = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Name: "p2p_message_received_total",
			Help: "Count of messages received.",
		},
		[]string{"topic"},
	)
	messageFailedValidationCounter = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Name: "p2p_message_failed_validation_total",
			Help: "Count of messages that failed validation.",
		},
		[]string{"topic"},
	)
	messageIgnoredValidationCounter = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Name: "p2p_message_ignored_validation_total",
			Help: "Count of messages that were ignored in validation.",
		},
		[]string{"topic"},
	)
	messageFailedProcessingCounter = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Name: "p2p_message_failed_processing_total",
			Help: "Count of messages that passed validation but failed processing.",
		},
		[]string{"topic"},
	)
	numberOfTimesResyncedCounter = promauto.NewCounter(
		prometheus.CounterOpts{
			Name: "number_of_times_resynced",
			Help: "Count the number of times a node resyncs.",
		},
	)
	duplicatesRemovedCounter = promauto.NewCounter(
		prometheus.CounterOpts{
			Name: "number_of_duplicates_removed",
			Help: "Count the number of times a duplicate signature set has been removed.",
		},
	)
	numberOfSetsAggregated = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Name:    "number_of_sets_aggregated",
			Help:    "Count the number of times different sets have been successfully aggregated in a batch.",
			Buckets: []float64{10, 50, 100, 200, 400, 800, 1600, 3200},
		},
	)
	rpcBlocksByRangeResponseLatency = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Name:    "rpc_blocks_by_range_response_latency_milliseconds",
			Help:    "Captures total time to respond to rpc blocks by range requests in a milliseconds distribution",
			Buckets: []float64{5, 10, 50, 100, 150, 250, 500, 1000, 2000},
		},
	)
	rpcBlobsByRangeResponseLatency = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Name:    "rpc_blobs_by_range_response_latency_milliseconds",
			Help:    "Captures total time to respond to rpc BlobsByRange requests in a milliseconds distribution",
			Buckets: []float64{5, 10, 50, 100, 150, 250, 500, 1000, 2000},
		},
	)
	arrivalBlockPropagationHistogram = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Name:    "block_arrival_latency_milliseconds",
			Help:    "Captures blocks propagation time. Blocks arrival in milliseconds distribution",
			Buckets: []float64{100, 250, 500, 750, 1000, 1500, 2000, 4000, 8000, 12000, 16000, 20000, 24000},
		},
	)
	arrivalBlockPropagationGauge = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "block_arrival_latency_milliseconds_gauge",
		Help: "Captures blocks propagation time. Blocks arrival in milliseconds",
	})

	// Attestation processing granular error tracking.
	attBadBlockCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "gossip_attestation_bad_block_total",
		Help: "Increased when a gossip attestation references a bad block",
	})
	attBadLmdConsistencyCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "gossip_attestation_bad_lmd_consistency_total",
		Help: "Increased when a gossip attestation has bad LMD GHOST consistency",
	})
	attBadSelectionProofCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "gossip_attestation_bad_selection_proof_total",
		Help: "Increased when a gossip attestation has a bad selection proof",
	})
	attBadSignatureBatchCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "gossip_attestation_bad_signature_batch_total",
		Help: "Increased when a gossip attestation has a bad signature batch",
	})

	// Attestation and block gossip verification performance.
	aggregateAttestationVerificationGossipSummary = promauto.NewSummary(
		prometheus.SummaryOpts{
			Name: "gossip_aggregate_attestation_verification_milliseconds",
			Help: "Time to verify gossiped attestations",
		},
	)
	blockVerificationGossipSummary = promauto.NewSummary(
		prometheus.SummaryOpts{
			Name: "gossip_block_verification_milliseconds",
			Help: "Time to verify gossiped blocks",
		},
	)
	blockArrivalGossipSummary = promauto.NewSummary(
		prometheus.SummaryOpts{
			Name: "gossip_block_arrival_milliseconds",
			Help: "Time for gossiped blocks to arrive",
		},
	)
	blobSidecarArrivalGossipSummary = promauto.NewSummary(
		prometheus.SummaryOpts{
			Name: "gossip_blob_sidecar_arrival_milliseconds",
			Help: "Time for gossiped blob sidecars to arrive",
		},
	)

	// Sync committee verification performance.
	syncMessagesForUnknownBlocks = promauto.NewCounter(
		prometheus.CounterOpts{
			Name: "sync_committee_messages_unknown_root",
			Help: "The number of sync committee messages that are checked against DB to see if there vote is for an unknown root",
		},
	)

	// Dropped blob sidecars due to missing parent block.
	missingParentBlobSidecarCount = promauto.NewCounter(
		prometheus.CounterOpts{
			Name: "gossip_missing_parent_blob_sidecar_total",
			Help: "The number of blob sidecars that were dropped due to missing parent block",
		},
	)
)

func (s *Service) updateMetrics() {
	// do not update metrics if genesis time
	// has not been initialized
	if s.cfg.clock.GenesisTime().IsZero() {
		return
	}
	// We update the dynamic subnet topics.
	digest, err := s.currentForkDigest()
	if err != nil {
		log.WithError(err).Debugf("Could not compute fork digest")
	}
	indices := s.aggregatorSubnetIndices(s.cfg.clock.CurrentSlot())
	syncIndices := cache.SyncSubnetIDs.GetAllSubnets(slots.ToEpoch(s.cfg.clock.CurrentSlot()))
	attTopic := p2p.GossipTypeMapping[reflect.TypeOf(&pb.Attestation{})]
	syncTopic := p2p.GossipTypeMapping[reflect.TypeOf(&pb.SyncCommitteeMessage{})]
	attTopic += s.cfg.p2p.Encoding().ProtocolSuffix()
	syncTopic += s.cfg.p2p.Encoding().ProtocolSuffix()
	if flags.Get().SubscribeToAllSubnets {
		for i := uint64(0); i < params.BeaconConfig().AttestationSubnetCount; i++ {
			s.collectMetricForSubnet(attTopic, digest, i)
		}
		for i := uint64(0); i < params.BeaconConfig().SyncCommitteeSubnetCount; i++ {
			s.collectMetricForSubnet(syncTopic, digest, i)
		}
	} else {
		for _, committeeIdx := range indices {
			s.collectMetricForSubnet(attTopic, digest, committeeIdx)
		}
		for _, committeeIdx := range syncIndices {
			s.collectMetricForSubnet(syncTopic, digest, committeeIdx)
		}
	}

	// We update all other gossip topics.
	for _, topic := range p2p.AllTopics() {
		// We already updated attestation subnet topics.
		if strings.Contains(topic, p2p.GossipAttestationMessage) || strings.Contains(topic, p2p.GossipSyncCommitteeMessage) {
			continue
		}
		topic += s.cfg.p2p.Encoding().ProtocolSuffix()
		if !strings.Contains(topic, "%x") {
			topicPeerCount.WithLabelValues(topic).Set(float64(len(s.cfg.p2p.PubSub().ListPeers(topic))))
			continue
		}
		formattedTopic := fmt.Sprintf(topic, digest)
		topicPeerCount.WithLabelValues(formattedTopic).Set(float64(len(s.cfg.p2p.PubSub().ListPeers(formattedTopic))))
	}

	for _, topic := range s.cfg.p2p.PubSub().GetTopics() {
		subscribedTopicPeerCount.WithLabelValues(topic).Set(float64(len(s.cfg.p2p.PubSub().ListPeers(topic))))
	}
}

func (s *Service) collectMetricForSubnet(topic string, digest [4]byte, index uint64) {
	formattedTopic := fmt.Sprintf(topic, digest, index)
	topicPeerCount.WithLabelValues(formattedTopic).Set(float64(len(s.cfg.p2p.PubSub().ListPeers(formattedTopic))))
}
