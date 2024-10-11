package backfill

import (
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
)

var (
	oldestBatch = promauto.NewGauge(
		prometheus.GaugeOpts{
			Name: "backfill_earliest_wip_slot",
			Help: "Earliest slot that has been assigned to a worker.",
		},
	)
	batchesWaiting = promauto.NewGauge(
		prometheus.GaugeOpts{
			Name: "backfill_importable_batches_waiting",
			Help: "Number of batches that are ready to be imported once they can be connected to the existing chain.",
		},
	)
	backfillRemainingBatches = promauto.NewGauge(
		prometheus.GaugeOpts{
			Name: "backfill_remaining_batches",
			Help: "Backfill remaining batches.",
		},
	)
	backfillBatchesImported = promauto.NewCounter(
		prometheus.CounterOpts{
			Name: "backfill_batches_imported",
			Help: "Number of backfill batches downloaded and imported.",
		},
	)
	backfillBatchApproximateBytes = promauto.NewCounter(
		prometheus.CounterOpts{
			Name: "backfill_batch_bytes_downloaded",
			Help: "Count of bytes downloaded from peers",
		},
	)
	backfillBatchTimeRoundtrip = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Name:    "backfill_batch_time_roundtrip",
			Help:    "Total time to import batch, from first scheduled to imported.",
			Buckets: []float64{400, 800, 1600, 3200, 6400, 12800},
		},
	)
	backfillBatchTimeWaiting = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Name:    "backfill_batch_time_waiting",
			Help:    "Time batch waited for a suitable peer.",
			Buckets: []float64{50, 100, 300, 1000, 2000},
		},
	)
	backfillBatchTimeDownloading = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Name:    "backfill_batch_time_download",
			Help:    "Time batch spent downloading blocks from peer.",
			Buckets: []float64{100, 300, 1000, 2000, 4000, 8000},
		},
	)
	backfillBatchTimeVerifying = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Name:    "backfill_batch_time_verify",
			Help:    "Time batch spent downloading blocks from peer.",
			Buckets: []float64{100, 300, 1000, 2000, 4000, 8000},
		},
	)
)
