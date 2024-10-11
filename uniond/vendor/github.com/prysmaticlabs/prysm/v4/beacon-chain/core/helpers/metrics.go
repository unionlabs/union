package helpers

import (
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
)

var (
	attReceivedTooEarlyCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "attestation_too_early_total",
		Help: "Increased when an attestation is considered too early",
	})
	attReceivedTooLateCount = promauto.NewCounter(prometheus.CounterOpts{
		Name: "attestation_too_late_total",
		Help: "Increased when an attestation is considered too late",
	})
)
