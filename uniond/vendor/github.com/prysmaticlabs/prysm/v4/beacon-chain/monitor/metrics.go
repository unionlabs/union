package monitor

import (
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
	"github.com/sirupsen/logrus"
)

var (
	log = logrus.WithField("prefix", "monitor")
	// TODO: The Prometheus gauge vectors and counters in this package deprecate the
	// corresponding gauge vectors and counters in the validator client.

	// inclusionSlotGauge used to track attestation inclusion distance
	inclusionSlotGauge = promauto.NewGaugeVec(
		prometheus.GaugeOpts{
			Namespace: "monitor",
			Name:      "inclusion_slot",
			Help:      "Attestations inclusion slot",
		},
		[]string{
			"validator_index",
		},
	)
	// timelyHeadCounter used to track attestation timely head flags
	timelyHeadCounter = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Namespace: "monitor",
			Name:      "timely_head",
			Help:      "Attestation timely Head flag",
		},
		[]string{
			"validator_index",
		},
	)
	// timelyTargetCounter used to track attestation timely head flags
	timelyTargetCounter = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Namespace: "monitor",
			Name:      "timely_target",
			Help:      "Attestation timely Target flag",
		},
		[]string{
			"validator_index",
		},
	)
	// timelySourceCounter used to track attestation timely head flags
	timelySourceCounter = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Namespace: "monitor",
			Name:      "timely_source",
			Help:      "Attestation timely Source flag",
		},
		[]string{
			"validator_index",
		},
	)

	// proposedSlotsCounter used to track proposed blocks
	proposedSlotsCounter = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Namespace: "monitor",
			Name:      "proposed_slots_total",
			Help:      "Number of proposed blocks included",
		},
		[]string{
			"validator_index",
		},
	)
	// aggregationCounter used to track aggregations
	aggregationCounter = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Namespace: "monitor",
			Name:      "aggregations",
			Help:      "Number of aggregation duties performed",
		},
		[]string{
			"validator_index",
		},
	)
	// syncCommitteeContributionCounter used to track sync committee
	// contributions
	syncCommitteeContributionCounter = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Namespace: "monitor",
			Name:      "sync_committee_contributions_total",
			Help:      "Number of Sync committee contributions performed",
		},
		[]string{
			"validator_index",
		},
	)
)
