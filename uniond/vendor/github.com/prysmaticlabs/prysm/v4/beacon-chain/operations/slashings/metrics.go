package slashings

import (
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
)

var (
	numPendingAttesterSlashings = promauto.NewGauge(
		prometheus.GaugeOpts{
			Name: "num_pending_attester_slashings",
			Help: "Number of pending attester slashings in the pool",
		},
	)
	numAttesterSlashingsIncluded = promauto.NewCounter(
		prometheus.CounterOpts{
			Name: "attester_slashings_included_total",
			Help: "Number of attester slashings included in blocks",
		},
	)
	numPendingProposerSlashings = promauto.NewGauge(
		prometheus.GaugeOpts{
			Name: "num_pending_proposer_slashings",
			Help: "Number of pending proposer slashings in the pool",
		},
	)
	numProposerSlashingsIncluded = promauto.NewCounter(
		prometheus.CounterOpts{
			Name: "proposer_slashings_included_total",
			Help: "Number of proposer slashings included in blocks",
		},
	)
)
