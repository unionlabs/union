package synccommittee

import (
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
)

var (
	savedSyncCommitteeMessageTotal = promauto.NewCounter(prometheus.CounterOpts{
		Name: "saved_sync_committee_message_total",
		Help: "The number of saved sync committee message total.",
	})
	savedSyncCommitteeContributionTotal = promauto.NewCounter(prometheus.CounterOpts{
		Name: "saved_sync_committee_contribution_total",
		Help: "The number of saved sync committee contribution total.",
	})
)
