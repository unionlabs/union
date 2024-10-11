package slasher

import (
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
)

var (
	attestationDistance = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Name:    "slasher_attestation_distance_epochs",
			Help:    "The number of epochs between att target and source",
			Buckets: []float64{0, 1, 2, 3, 4, 5, 10, 20, 50, 100},
		},
	)
	chunksSavedTotal = promauto.NewCounter(prometheus.CounterOpts{
		Name: "slasher_chunks_saved_total",
		Help: "Total number of slasher chunks saved to disk",
	})
	deferredAttestationsTotal = promauto.NewCounter(prometheus.CounterOpts{
		Name: "slasher_attestations_deferred_total",
		Help: "Total number of attestations deferred by slasher for future processing",
	})
	droppedAttestationsTotal = promauto.NewCounter(prometheus.CounterOpts{
		Name: "slasher_attestations_dropped_total",
		Help: "Total number of attestations dropped by slasher due to invalidity",
	})
	processedAttestationsTotal = promauto.NewCounter(prometheus.CounterOpts{
		Name: "slasher_attestations_processed_total",
		Help: "Total number of attestations successfully processed by slasher",
	})
	receivedBlocksTotal = promauto.NewCounter(prometheus.CounterOpts{
		Name: "slasher_blocks_received_total",
		Help: "Total number of blocks received by slasher",
	})
	processedBlocksTotal = promauto.NewCounter(prometheus.CounterOpts{
		Name: "slasher_blocks_processed_total",
		Help: "Total number of blocks successfully processed by slasher",
	})
	doubleProposalsTotal = promauto.NewCounter(prometheus.CounterOpts{
		Name: "slasher_double_proposals_total",
		Help: "Total slashable proposals successfully detected by slasher",
	})
	doubleVotesTotal = promauto.NewCounter(prometheus.CounterOpts{
		Name: "slasher_double_votes_total",
		Help: "Total slashable double votes successfully detected by slasher",
	})
	surroundingVotesTotal = promauto.NewCounter(prometheus.CounterOpts{
		Name: "slasher_surrounding_votes_total",
		Help: "Total slashable surrounding votes successfully detected by slasher",
	})
	surroundedVotesTotal = promauto.NewCounter(prometheus.CounterOpts{
		Name: "slasher_surrounded_votes_total",
		Help: "Total slashable surrounded votes successfully detected by slasher",
	})
)
