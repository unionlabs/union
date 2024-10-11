package slasherkv

import (
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
)

var (
	slasherAttestationsPrunedTotal = promauto.NewCounter(prometheus.CounterOpts{
		Name: "slasher_attestations_pruned_total",
		Help: "Total number of old attestations pruned by slasher",
	})
	slasherProposalsPrunedTotal = promauto.NewCounter(prometheus.CounterOpts{
		Name: "slasher_proposals_pruned_total",
		Help: "Total number of old proposals pruned by slasher",
	})
)
