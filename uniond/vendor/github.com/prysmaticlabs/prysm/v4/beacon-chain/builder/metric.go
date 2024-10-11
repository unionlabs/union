package builder

import (
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
)

var (
	submitBlindedBlockLatency = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Name:    "submit_blinded_block_latency_milliseconds",
			Help:    "Captures RPC latency for submitting blinded block in milliseconds",
			Buckets: []float64{1, 2, 5, 10, 20, 50, 100, 200, 500, 1000},
		},
	)
	getHeaderLatency = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Name:    "get_header_latency_milliseconds",
			Help:    "Captures RPC latency for get header in milliseconds",
			Buckets: []float64{1, 2, 5, 10, 20, 50, 100, 200, 500, 1000},
		},
	)
	registerValidatorLatency = promauto.NewHistogram(
		prometheus.HistogramOpts{
			Name:    "register_validator_latency_milliseconds",
			Help:    "Captures RPC latency for register validator in milliseconds",
			Buckets: []float64{1, 2, 5, 10, 20, 50, 100, 200, 500, 1000},
		},
	)
)
