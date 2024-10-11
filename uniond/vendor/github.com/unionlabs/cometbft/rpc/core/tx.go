package core

import (
	"errors"
	"fmt"

	cmtquery "github.com/cometbft/cometbft/libs/pubsub/query"
	ctypes "github.com/cometbft/cometbft/rpc/core/types"
	rpctypes "github.com/cometbft/cometbft/rpc/jsonrpc/types"
	"github.com/cometbft/cometbft/state/txindex"
	"github.com/cometbft/cometbft/state/txindex/null"
	"github.com/cometbft/cometbft/types"
)

const (
	Ascending  = "asc"
	Descending = "desc"
)

// Tx allows you to query the transaction results. `nil` could mean the
// transaction is in the mempool, invalidated, or was not sent in the first
// place.
// More: https://docs.cometbft.com/main/rpc/#/Info/tx
func (env *Environment) Tx(_ *rpctypes.Context, hash []byte, prove bool) (*ctypes.ResultTx, error) {
	// if index is disabled, return error
	if _, ok := env.TxIndexer.(*null.TxIndex); ok {
		return nil, errors.New("transaction indexing is disabled")
	}

	r, err := env.TxIndexer.Get(hash)
	if err != nil {
		return nil, err
	}

	if r == nil {
		return nil, fmt.Errorf("tx (%X) not found", hash)
	}

	var proof types.TxProof
	if prove {
		block, _ := env.BlockStore.LoadBlock(r.Height)
		if block != nil {
			proof = block.Data.Txs.Proof(int(r.Index))
		}
	}

	return &ctypes.ResultTx{
		Hash:     hash,
		Height:   r.Height,
		Index:    r.Index,
		TxResult: r.Result,
		Tx:       r.Tx,
		Proof:    proof,
	}, nil
}

// TxSearch allows you to query for multiple transactions results. It returns a
// list of transactions (maximum ?per_page entries) and the total count.
// More: https://docs.cometbft.com/main/rpc/#/Info/tx_search
func (env *Environment) TxSearch(
	ctx *rpctypes.Context,
	query string,
	prove bool,
	pagePtr, perPagePtr *int,
	orderBy string,
) (*ctypes.ResultTxSearch, error) {
	// if index is disabled, return error
	if _, ok := env.TxIndexer.(*null.TxIndex); ok {
		return nil, errors.New("transaction indexing is disabled")
	} else if len(query) > maxQueryLength {
		return nil, errors.New("maximum query length exceeded")
	}

	// if orderBy is not "asc", "desc", or blank, return error
	if orderBy != "" && orderBy != Ascending && orderBy != Descending {
		return nil, ErrInvalidOrderBy{orderBy}
	}

	q, err := cmtquery.New(query)
	if err != nil {
		return nil, err
	}

	// Validate number of results per page
	perPage := env.validatePerPage(perPagePtr)
	if pagePtr == nil {
		// Default to page 1 if not specified
		pagePtr = new(int)
		*pagePtr = 1
	}

	pagSettings := txindex.Pagination{
		OrderDesc:   orderBy == Descending,
		IsPaginated: true,
		Page:        *pagePtr,
		PerPage:     perPage,
	}

	results, totalCount, err := env.TxIndexer.Search(ctx.Context(), q, pagSettings)
	if err != nil {
		return nil, err
	}

	apiResults := make([]*ctypes.ResultTx, 0, len(results))
	for _, r := range results {
		var proof types.TxProof
		if prove {
			block, _ := env.BlockStore.LoadBlock(r.Height)
			if block != nil {
				proof = block.Data.Txs.Proof(int(r.Index))
			}
		}

		apiResults = append(apiResults, &ctypes.ResultTx{
			Hash:     types.Tx(r.Tx).Hash(),
			Height:   r.Height,
			Index:    r.Index,
			TxResult: r.Result,
			Tx:       r.Tx,
			Proof:    proof,
		})
	}

	return &ctypes.ResultTxSearch{Txs: apiResults, TotalCount: totalCount}, nil
}
