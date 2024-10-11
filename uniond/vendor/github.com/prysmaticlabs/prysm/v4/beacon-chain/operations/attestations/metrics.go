package attestations

import (
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
)

var (
	aggregatedAttsCount = promauto.NewGauge(
		prometheus.GaugeOpts{
			Name: "aggregated_attestations_in_pool_total",
			Help: "The number of aggregated attestations in the pool.",
		},
	)
	unaggregatedAttsCount = promauto.NewGauge(
		prometheus.GaugeOpts{
			Name: "unaggregated_attestations_in_pool_total",
			Help: "The number of unaggregated attestations in the pool.",
		},
	)
	expiredAggregatedAtts = promauto.NewCounter(prometheus.CounterOpts{
		Name: "expired_aggregated_atts_total",
		Help: "The number of expired and deleted aggregated attestations in the pool.",
	})
	expiredUnaggregatedAtts = promauto.NewCounter(prometheus.CounterOpts{
		Name: "expired_unaggregated_atts_total",
		Help: "The number of expired and deleted unaggregated attestations in the pool.",
	})
	expiredBlockAtts = promauto.NewCounter(prometheus.CounterOpts{
		Name: "expired_block_atts_total",
		Help: "The number of expired and deleted block attestations in the pool.",
	})
	batchForkChoiceAttsT1 = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Name:    "aggregate_attestations_t1",
			Help:    "Captures times of attestation aggregation in milliseconds during the first interval per slot",
			Buckets: []float64{100, 200, 500, 1000, 1500, 2000, 2500, 3500},
		},
	)
	batchForkChoiceAttsT2 = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Name:    "aggregate_attestations_t2",
			Help:    "Captures times of attestation aggregation in milliseconds during the second interval per slot",
			Buckets: []float64{10, 40, 100, 200, 600},
		},
	)
)

func (s *Service) updateMetrics() {
	aggregatedAttsCount.Set(float64(s.cfg.Pool.AggregatedAttestationCount()))
	unaggregatedAttsCount.Set(float64(s.cfg.Pool.UnaggregatedAttestationCount()))
}
