package indexer

import (
	"context"

	"github.com/cometbft/cometbft/libs/pubsub/query"
	"github.com/cometbft/cometbft/types"
)

//go:generate ../../scripts/mockery_generate.sh BlockIndexer

// BlockIndexer defines an interface contract for indexing block events.
type BlockIndexer interface {
	// Has returns true if the given height has been indexed. An error is returned
	// upon database query failure.
	Has(height int64) (bool, error)

	// Index indexes BeginBlock and EndBlock events for a given block by its height.
	Index(types.EventDataNewBlockHeader) error

	// Search performs a query for block heights that match a given BeginBlock
	// and Endblock event search criteria.
	Search(ctx context.Context, q *query.Query) ([]int64, error)
}
