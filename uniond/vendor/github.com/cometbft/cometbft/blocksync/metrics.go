package blocksync

import (
	"github.com/cometbft/cometbft/types"
	"github.com/go-kit/kit/metrics"
)

const (
	// MetricsSubsystem is a subsystem shared by all metrics exposed by this
	// package.
	MetricsSubsystem = "blocksync"
)

//go:generate go run ../scripts/metricsgen -struct=Metrics

// Metrics contains metrics exposed by this package.
type Metrics struct {
	// Whether or not a node is block syncing. 1 if yes, 0 if no.
	Syncing metrics.Gauge
	// Number of transactions in the latest block.
	NumTxs metrics.Gauge
	// Total number of transactions.
	TotalTxs metrics.Gauge
	// Size of the latest block.
	BlockSizeBytes metrics.Gauge
	// The height of the latest block.
	LatestBlockHeight metrics.Gauge
}

func (m *Metrics) recordBlockMetrics(block *types.Block) {
	m.NumTxs.Set(float64(len(block.Data.Txs)))
	m.TotalTxs.Add(float64(len(block.Data.Txs)))
	m.BlockSizeBytes.Set(float64(block.Size()))
	m.LatestBlockHeight.Set(float64(block.Height))
}
