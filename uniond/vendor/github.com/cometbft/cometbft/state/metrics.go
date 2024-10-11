package state

import (
	"github.com/go-kit/kit/metrics"
)

const (
	// MetricsSubsystem is a subsystem shared by all metrics exposed by this
	// package.
	MetricsSubsystem = "state"
)

//go:generate go run ../scripts/metricsgen -struct=Metrics

// Metrics contains metrics exposed by this package.
type Metrics struct {
	// Time spent processing FinalizeBlock
	BlockProcessingTime metrics.Histogram `metrics_bucketsizes:"1, 10, 10" metrics_buckettype:"lin"`

	// ConsensusParamUpdates is the total number of times the application has
	// updated the consensus params since process start.
	// metrics:Number of consensus parameter updates returned by the application since process start.
	ConsensusParamUpdates metrics.Counter

	// ValidatorSetUpdates is the total number of times the application has
	// updated the validator set since process start.
	// metrics:Number of validator set updates returned by the application since process start.
	ValidatorSetUpdates metrics.Counter

	// PruningServiceBlockRetainHeight is the accepted block
	// retain height set by the data companion
	PruningServiceBlockRetainHeight metrics.Gauge

	// PruningServiceBlockResultsRetainHeight is the accepted block results
	// retain height set by the data companion
	PruningServiceBlockResultsRetainHeight metrics.Gauge

	// PruningServiceTxIndexerRetainHeight is the accepted transactions indices
	// retain height set by the data companion
	PruningServiceTxIndexerRetainHeight metrics.Gauge

	// PruningServiceBlockIndexerRetainHeight is the accepted blocks indices
	// retain height set by the data companion
	PruningServiceBlockIndexerRetainHeight metrics.Gauge

	// ApplicationBlockRetainHeight is the accepted block
	// retain height set by the application
	ApplicationBlockRetainHeight metrics.Gauge

	// BlockStoreBaseHeight shows the first height at which
	// a block is available
	BlockStoreBaseHeight metrics.Gauge

	// ABCIResultsBaseHeight shows the first height at which
	// abci results are available
	ABCIResultsBaseHeight metrics.Gauge

	// TxIndexerBaseHeight shows the first height at which
	// tx indices are available
	TxIndexerBaseHeight metrics.Gauge

	// BlockIndexerBaseHeight shows the first height at which
	// block indices are available
	BlockIndexerBaseHeight metrics.Gauge

	// The duration of accesses to the state store labeled by which method
	// was called on the store.
	StoreAccessDurationSeconds metrics.Histogram `metrics_bucketsizes:"0.0002, 10, 5" metrics_buckettype:"exp" metrics_labels:"method"`
}
