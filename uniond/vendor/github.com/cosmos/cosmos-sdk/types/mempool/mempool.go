package mempool

import (
	"context"
	"errors"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

type Mempool interface {
	// Insert attempts to insert a Tx into the app-side mempool returning
	// an error upon failure.
	Insert(context.Context, sdk.Tx) error

	// Select returns an Iterator over the app-side mempool. If txs are specified,
	// then they shall be incorporated into the Iterator. The Iterator is not thread-safe to use.
	Select(context.Context, []sdk.Tx) Iterator

	// SelectBy use callback to iterate over the mempool, it's thread-safe to use.
	SelectBy(context.Context, []sdk.Tx, func(sdk.Tx) bool)

	// CountTx returns the number of transactions currently in the mempool.
	CountTx() int

	// Remove attempts to remove a transaction from the mempool, returning an error
	// upon failure.
	Remove(sdk.Tx) error
}

// Iterator defines an app-side mempool iterator interface that is as minimal as
// possible. The order of iteration is determined by the app-side mempool
// implementation.
type Iterator interface {
	// Next returns the next transaction from the mempool. If there are no more
	// transactions, it returns nil.
	Next() Iterator

	// Tx returns the transaction at the current position of the iterator.
	Tx() sdk.Tx
}

var (
	ErrTxNotFound           = errors.New("tx not found in mempool")
	ErrMempoolTxMaxCapacity = errors.New("pool reached max tx capacity")
)
