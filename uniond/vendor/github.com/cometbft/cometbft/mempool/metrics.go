package mempool

import (
	"github.com/go-kit/kit/metrics"
)

const (
	// MetricsSubsystem is a subsystem shared by all metrics exposed by this
	// package.
	MetricsSubsystem = "mempool"
)

//go:generate go run ../scripts/metricsgen -struct=Metrics

// Metrics contains metrics exposed by this package.
// see MetricsProvider for descriptions.
type Metrics struct {
	// Number of uncommitted transactions in the mempool.
	Size metrics.Gauge

	// Total size of the mempool in bytes.
	SizeBytes metrics.Gauge

	// Histogram of transaction sizes in bytes.
	TxSizeBytes metrics.Histogram `metrics_bucketsizes:"1,3,7" metrics_buckettype:"exp"`

	// Number of failed transactions.
	FailedTxs metrics.Counter

	// RejectedTxs defines the number of rejected transactions. These are
	// transactions that passed CheckTx but failed to make it into the mempool
	// due to resource limits, e.g. mempool is full and no lower priority
	// transactions exist in the mempool.
	// metrics:Number of rejected transactions.
	RejectedTxs metrics.Counter

	// Number of times transactions are rechecked in the mempool.
	RecheckTimes metrics.Counter

	// Number of times transactions were received more than once.
	// metrics:Number of duplicate transaction reception.
	AlreadyReceivedTxs metrics.Counter

	// Number of connections being actively used for gossiping transactions
	// (experimental feature).
	ActiveOutboundConnections metrics.Gauge
}
