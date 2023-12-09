package psql

// This file adds code to the psql package that is needed for integration with
// v0.34, but which is not part of the original implementation.
//
// In v0.35, ADR 65 was implemented in which the TxIndexer and BlockIndexer
// interfaces were merged into a hybrid EventSink interface. The Backport*
// types defined here bridge the psql EventSink (which was built in terms of
// the v0.35 interface) to the old interfaces.
//
// We took this narrower approach to backporting to avoid pulling in a much
// wider-reaching set of changes in v0.35 that would have broken several of the
// v0.34.x APIs. The result is sufficient to work with the node plumbing as it
// exists in the v0.34 branch.

import (
	"context"
	"errors"

	"github.com/cometbft/cometbft/libs/log"

	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/libs/pubsub/query"
	"github.com/cometbft/cometbft/state/txindex"
	"github.com/cometbft/cometbft/types"
)

const (
	eventTypeFinalizeBlock = "finalize_block"
)

// TxIndexer returns a bridge from es to the CometBFT v0.34 transaction indexer.
func (es *EventSink) TxIndexer() BackportTxIndexer {
	return BackportTxIndexer{psql: es}
}

// BackportTxIndexer implements the txindex.TxIndexer interface by delegating
// indexing operations to an underlying PostgreSQL event sink.
type BackportTxIndexer struct{ psql *EventSink }

// AddBatch indexes a batch of transactions in Postgres, as part of TxIndexer.
func (b BackportTxIndexer) AddBatch(batch *txindex.Batch) error {
	return b.psql.IndexTxEvents(batch.Ops)
}

// Index indexes a single transaction result in Postgres, as part of TxIndexer.
func (b BackportTxIndexer) Index(txr *abci.TxResult) error {
	return b.psql.IndexTxEvents([]*abci.TxResult{txr})
}

// Get is implemented to satisfy the TxIndexer interface, but is not supported
// by the psql event sink and reports an error for all inputs.
func (BackportTxIndexer) Get([]byte) (*abci.TxResult, error) {
	return nil, errors.New("the TxIndexer.Get method is not supported")
}

// Search is implemented to satisfy the TxIndexer interface, but it is not
// supported by the psql event sink and reports an error for all inputs.
func (BackportTxIndexer) Search(context.Context, *query.Query) ([]*abci.TxResult, error) {
	return nil, errors.New("the TxIndexer.Search method is not supported")
}

func (BackportTxIndexer) SetLogger(log.Logger) {}

// BlockIndexer returns a bridge that implements the CometBFT v0.34 block
// indexer interface, using the Postgres event sink as a backing store.
func (es *EventSink) BlockIndexer() BackportBlockIndexer {
	return BackportBlockIndexer{psql: es}
}

// BackportBlockIndexer implements the indexer.BlockIndexer interface by
// delegating indexing operations to an underlying PostgreSQL event sink.
type BackportBlockIndexer struct{ psql *EventSink }

// Has is implemented to satisfy the BlockIndexer interface, but it is not
// supported by the psql event sink and reports an error for all inputs.
func (BackportBlockIndexer) Has(_ int64) (bool, error) {
	return false, errors.New("the BlockIndexer.Has method is not supported")
}

// Index indexes block begin and end events for the specified block.  It is
// part of the BlockIndexer interface.
func (b BackportBlockIndexer) Index(block types.EventDataNewBlockEvents) error {
	return b.psql.IndexBlockEvents(block)
}

// Search is implemented to satisfy the BlockIndexer interface, but it is not
// supported by the psql event sink and reports an error for all inputs.
func (BackportBlockIndexer) Search(context.Context, *query.Query) ([]int64, error) {
	return nil, errors.New("the BlockIndexer.Search method is not supported")
}

func (BackportBlockIndexer) SetLogger(log.Logger) {}
