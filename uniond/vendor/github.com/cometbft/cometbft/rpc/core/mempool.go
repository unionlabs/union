package core

import (
	"context"
	"errors"
	"fmt"
	"time"

	abci "github.com/cometbft/cometbft/abci/types"
	mempl "github.com/cometbft/cometbft/mempool"
	ctypes "github.com/cometbft/cometbft/rpc/core/types"
	rpctypes "github.com/cometbft/cometbft/rpc/jsonrpc/types"
	"github.com/cometbft/cometbft/types"
)

//-----------------------------------------------------------------------------
// NOTE: tx should be signed, but this is only checked at the app level (not by CometBFT!)

// BroadcastTxAsync returns right away, with no response. Does not wait for
// CheckTx nor transaction results.
// More: https://docs.cometbft.com/v0.38.x/rpc/#/Tx/broadcast_tx_async
func (env *Environment) BroadcastTxAsync(_ *rpctypes.Context, tx types.Tx) (*ctypes.ResultBroadcastTx, error) {
	err := env.Mempool.CheckTx(tx, nil, mempl.TxInfo{})
	if err != nil {
		return nil, err
	}
	return &ctypes.ResultBroadcastTx{Hash: tx.Hash()}, nil
}

// BroadcastTxSync returns with the response from CheckTx. Does not wait for
// the transaction result.
// More: https://docs.cometbft.com/v0.38.x/rpc/#/Tx/broadcast_tx_sync
func (env *Environment) BroadcastTxSync(ctx *rpctypes.Context, tx types.Tx) (*ctypes.ResultBroadcastTx, error) {
	resCh := make(chan *abci.ResponseCheckTx, 1)
	err := env.Mempool.CheckTx(tx, func(res *abci.ResponseCheckTx) {
		select {
		case <-ctx.Context().Done():
		case resCh <- res:
		}
	}, mempl.TxInfo{})
	if err != nil {
		return nil, err
	}

	select {
	case <-ctx.Context().Done():
		return nil, fmt.Errorf("broadcast confirmation not received: %w", ctx.Context().Err())
	case res := <-resCh:
		return &ctypes.ResultBroadcastTx{
			Code:      res.Code,
			Data:      res.Data,
			Log:       res.Log,
			Codespace: res.Codespace,
			Hash:      tx.Hash(),
		}, nil
	}
}

// BroadcastTxCommit returns with the responses from CheckTx and ExecTxResult.
// More: https://docs.cometbft.com/v0.38.x/rpc/#/Tx/broadcast_tx_commit
func (env *Environment) BroadcastTxCommit(ctx *rpctypes.Context, tx types.Tx) (*ctypes.ResultBroadcastTxCommit, error) {
	subscriber := ctx.RemoteAddr()

	if env.EventBus.NumClients() >= env.Config.MaxSubscriptionClients {
		return nil, fmt.Errorf("max_subscription_clients %d reached", env.Config.MaxSubscriptionClients)
	} else if env.EventBus.NumClientSubscriptions(subscriber) >= env.Config.MaxSubscriptionsPerClient {
		return nil, fmt.Errorf("max_subscriptions_per_client %d reached", env.Config.MaxSubscriptionsPerClient)
	}

	// Subscribe to tx being committed in block.
	subCtx, cancel := context.WithTimeout(ctx.Context(), SubscribeTimeout)
	defer cancel()
	q := types.EventQueryTxFor(tx)
	txSub, err := env.EventBus.Subscribe(subCtx, subscriber, q)
	if err != nil {
		err = fmt.Errorf("failed to subscribe to tx: %w", err)
		env.Logger.Error("Error on broadcast_tx_commit", "err", err)
		return nil, err
	}
	defer func() {
		if err := env.EventBus.Unsubscribe(context.Background(), subscriber, q); err != nil {
			env.Logger.Error("Error unsubscribing from eventBus", "err", err)
		}
	}()

	// Broadcast tx and wait for CheckTx result
	checkTxResCh := make(chan *abci.ResponseCheckTx, 1)
	err = env.Mempool.CheckTx(tx, func(res *abci.ResponseCheckTx) {
		select {
		case <-ctx.Context().Done():
		case checkTxResCh <- res:
		}
	}, mempl.TxInfo{})
	if err != nil {
		env.Logger.Error("Error on broadcastTxCommit", "err", err)
		return nil, fmt.Errorf("error on broadcastTxCommit: %v", err)
	}
	select {
	case <-ctx.Context().Done():
		return nil, fmt.Errorf("broadcast confirmation not received: %w", ctx.Context().Err())
	case checkTxRes := <-checkTxResCh:
		if checkTxRes.Code != abci.CodeTypeOK {
			return &ctypes.ResultBroadcastTxCommit{
				CheckTx:  *checkTxRes,
				TxResult: abci.ExecTxResult{},
				Hash:     tx.Hash(),
			}, nil
		}

		// Wait for the tx to be included in a block or timeout.
		select {
		case msg := <-txSub.Out(): // The tx was included in a block.
			txResultEvent := msg.Data().(types.EventDataTx)
			return &ctypes.ResultBroadcastTxCommit{
				CheckTx:  *checkTxRes,
				TxResult: txResultEvent.Result,
				Hash:     tx.Hash(),
				Height:   txResultEvent.Height,
			}, nil
		case <-txSub.Canceled():
			var reason string
			if txSub.Err() == nil {
				reason = "CometBFT exited"
			} else {
				reason = txSub.Err().Error()
			}
			err = fmt.Errorf("txSub was canceled (reason: %s)", reason)
			env.Logger.Error("Error on broadcastTxCommit", "err", err)
			return &ctypes.ResultBroadcastTxCommit{
				CheckTx:  *checkTxRes,
				TxResult: abci.ExecTxResult{},
				Hash:     tx.Hash(),
			}, err
		case <-time.After(env.Config.TimeoutBroadcastTxCommit):
			err = errors.New("timed out waiting for tx to be included in a block")
			env.Logger.Error("Error on broadcastTxCommit", "err", err)
			return &ctypes.ResultBroadcastTxCommit{
				CheckTx:  *checkTxRes,
				TxResult: abci.ExecTxResult{},
				Hash:     tx.Hash(),
			}, err
		}
	}
}

// UnconfirmedTxs gets unconfirmed transactions (maximum ?limit entries)
// including their number.
// More: https://docs.cometbft.com/v0.38.x/rpc/#/Info/unconfirmed_txs
func (env *Environment) UnconfirmedTxs(_ *rpctypes.Context, limitPtr *int) (*ctypes.ResultUnconfirmedTxs, error) {
	// reuse per_page validator
	limit := env.validatePerPage(limitPtr)

	txs := env.Mempool.ReapMaxTxs(limit)
	return &ctypes.ResultUnconfirmedTxs{
		Count:      len(txs),
		Total:      env.Mempool.Size(),
		TotalBytes: env.Mempool.SizeBytes(),
		Txs:        txs,
	}, nil
}

// NumUnconfirmedTxs gets number of unconfirmed transactions.
// More: https://docs.cometbft.com/v0.38.x/rpc/#/Info/num_unconfirmed_txs
func (env *Environment) NumUnconfirmedTxs(*rpctypes.Context) (*ctypes.ResultUnconfirmedTxs, error) {
	return &ctypes.ResultUnconfirmedTxs{
		Count:      env.Mempool.Size(),
		Total:      env.Mempool.Size(),
		TotalBytes: env.Mempool.SizeBytes(),
	}, nil
}

// CheckTx checks the transaction without executing it. The transaction won't
// be added to the mempool either.
// More: https://docs.cometbft.com/v0.38.x/rpc/#/Tx/check_tx
func (env *Environment) CheckTx(_ *rpctypes.Context, tx types.Tx) (*ctypes.ResultCheckTx, error) {
	res, err := env.ProxyAppMempool.CheckTx(context.TODO(), &abci.RequestCheckTx{Tx: tx})
	if err != nil {
		return nil, err
	}
	return &ctypes.ResultCheckTx{ResponseCheckTx: *res}, nil
}
