package mempool

import (
	"context"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

var _ Mempool = (*NoOpMempool)(nil)

// NoOpMempool defines a no-op mempool. Transactions are completely discarded and
// ignored when BaseApp interacts with the mempool.
//
// Note: When this mempool is used, it assumed that an application will rely
// on CometBFT's transaction ordering defined in `RequestPrepareProposal`, which
// is FIFO-ordered by default.
type NoOpMempool struct{}

func (NoOpMempool) Insert(context.Context, sdk.Tx) error                  { return nil }
func (NoOpMempool) Select(context.Context, []sdk.Tx) Iterator             { return nil }
func (NoOpMempool) SelectBy(context.Context, []sdk.Tx, func(sdk.Tx) bool) {}
func (NoOpMempool) CountTx() int                                          { return 0 }
func (NoOpMempool) Remove(sdk.Tx) error                                   { return nil }
