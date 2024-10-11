package txindex

import (
	"context"
	"errors"

	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/libs/pubsub/query"
)

// XXX/TODO: These types should be moved to the indexer package.

//go:generate ../../scripts/mockery_generate.sh TxIndexer

// TxIndexer interface defines methods to index and search transactions.
type TxIndexer interface {
	// AddBatch analyzes, indexes and stores a batch of transactions.
	AddBatch(b *Batch) error

	// Index analyzes, indexes and stores a single transaction.
	Index(result *abci.TxResult) error

	// Get returns the transaction specified by hash or nil if the transaction is not indexed
	// or stored.
	Get(hash []byte) (*abci.TxResult, error)

	// Search allows you to query for transactions.
	Search(ctx context.Context, q *query.Query, pagSettings Pagination) ([]*abci.TxResult, int, error)

	// Set Logger
	SetLogger(l log.Logger)

	Prune(retainHeight int64) (int64, int64, error)

	GetRetainHeight() (int64, error)

	SetRetainHeight(retainHeight int64) error
}

// Batch groups together multiple Index operations to be performed at the same time.
// NOTE: Batch is NOT thread-safe and must not be modified after starting its execution.
type Batch struct {
	Ops []*abci.TxResult
}

// Pagination provides pagination information for queries.
// This allows us to use the same TxSearch API for pruning to return all relevant data,
// while still limiting public queries to pagination.
type Pagination struct {
	OrderDesc   bool
	IsPaginated bool
	Page        int
	PerPage     int
}

// NewBatch creates a new Batch.
func NewBatch(n int64) *Batch {
	return &Batch{
		Ops: make([]*abci.TxResult, n),
	}
}

// Add or update an entry for the given result.Index.
func (b *Batch) Add(result *abci.TxResult) error {
	b.Ops[result.Index] = result
	return nil
}

// Size returns the total number of operations inside the batch.
func (b *Batch) Size() int {
	return len(b.Ops)
}

// ErrorEmptyHash indicates empty hash.
var ErrorEmptyHash = errors.New("transaction hash cannot be empty")
