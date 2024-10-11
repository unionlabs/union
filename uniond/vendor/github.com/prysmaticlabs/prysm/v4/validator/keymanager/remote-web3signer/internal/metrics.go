package internal

import (
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
)

var (
	signRequestDurationSeconds = promauto.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "remote_web3signer_internal_client_request_duration_seconds",
			Help:    "Time (in seconds) spent doing client HTTP requests",
			Buckets: prometheus.DefBuckets,
		},
		[]string{"method", "status_code"},
	)
)
