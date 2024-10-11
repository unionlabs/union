// Copyright 2014 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package miner

import (
	"container/heap"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/math"
	"github.com/ethereum/go-ethereum/core/txpool"
	"github.com/ethereum/go-ethereum/core/types"
)

// txWithMinerFee wraps a transaction with its gas price or effective miner gasTipCap
type txWithMinerFee struct {
	tx   *txpool.LazyTransaction
	from common.Address
	fees *big.Int
}

// newTxWithMinerFee creates a wrapped transaction, calculating the effective
// miner gasTipCap if a base fee is provided.
// Returns error in case of a negative effective miner gasTipCap.
func newTxWithMinerFee(tx *txpool.LazyTransaction, from common.Address, baseFee *big.Int) (*txWithMinerFee, error) {
	tip := new(big.Int).Set(tx.GasTipCap)
	if baseFee != nil {
		if tx.GasFeeCap.Cmp(baseFee) < 0 {
			return nil, types.ErrGasFeeCapTooLow
		}
		tip = math.BigMin(tx.GasTipCap, new(big.Int).Sub(tx.GasFeeCap, baseFee))
	}
	return &txWithMinerFee{
		tx:   tx,
		from: from,
		fees: tip,
	}, nil
}

// txByPriceAndTime implements both the sort and the heap interface, making it useful
// for all at once sorting as well as individually adding and removing elements.
type txByPriceAndTime []*txWithMinerFee

func (s txByPriceAndTime) Len() int { return len(s) }
func (s txByPriceAndTime) Less(i, j int) bool {
	// If the prices are equal, use the time the transaction was first seen for
	// deterministic sorting
	cmp := s[i].fees.Cmp(s[j].fees)
	if cmp == 0 {
		return s[i].tx.Time.Before(s[j].tx.Time)
	}
	return cmp > 0
}
func (s txByPriceAndTime) Swap(i, j int) { s[i], s[j] = s[j], s[i] }

func (s *txByPriceAndTime) Push(x interface{}) {
	*s = append(*s, x.(*txWithMinerFee))
}

func (s *txByPriceAndTime) Pop() interface{} {
	old := *s
	n := len(old)
	x := old[n-1]
	old[n-1] = nil
	*s = old[0 : n-1]
	return x
}

// transactionsByPriceAndNonce represents a set of transactions that can return
// transactions in a profit-maximizing sorted order, while supporting removing
// entire batches of transactions for non-executable accounts.
type transactionsByPriceAndNonce struct {
	txs     map[common.Address][]*txpool.LazyTransaction // Per account nonce-sorted list of transactions
	heads   txByPriceAndTime                             // Next transaction for each unique account (price heap)
	signer  types.Signer                                 // Signer for the set of transactions
	baseFee *big.Int                                     // Current base fee
}

// newTransactionsByPriceAndNonce creates a transaction set that can retrieve
// price sorted transactions in a nonce-honouring way.
//
// Note, the input map is reowned so the caller should not interact any more with
// if after providing it to the constructor.
func newTransactionsByPriceAndNonce(signer types.Signer, txs map[common.Address][]*txpool.LazyTransaction, baseFee *big.Int) *transactionsByPriceAndNonce {
	// Initialize a price and received time based heap with the head transactions
	heads := make(txByPriceAndTime, 0, len(txs))
	for from, accTxs := range txs {
		wrapped, err := newTxWithMinerFee(accTxs[0], from, baseFee)
		if err != nil {
			delete(txs, from)
			continue
		}
		heads = append(heads, wrapped)
		txs[from] = accTxs[1:]
	}
	heap.Init(&heads)

	// Assemble and return the transaction set
	return &transactionsByPriceAndNonce{
		txs:     txs,
		heads:   heads,
		signer:  signer,
		baseFee: baseFee,
	}
}

// Peek returns the next transaction by price.
func (t *transactionsByPriceAndNonce) Peek() *txpool.LazyTransaction {
	if len(t.heads) == 0 {
		return nil
	}
	return t.heads[0].tx
}

// Shift replaces the current best head with the next one from the same account.
func (t *transactionsByPriceAndNonce) Shift() {
	acc := t.heads[0].from
	if txs, ok := t.txs[acc]; ok && len(txs) > 0 {
		if wrapped, err := newTxWithMinerFee(txs[0], acc, t.baseFee); err == nil {
			t.heads[0], t.txs[acc] = wrapped, txs[1:]
			heap.Fix(&t.heads, 0)
			return
		}
	}
	heap.Pop(&t.heads)
}

// Pop removes the best transaction, *not* replacing it with the next one from
// the same account. This should be used when a transaction cannot be executed
// and hence all subsequent ones should be discarded from the same account.
func (t *transactionsByPriceAndNonce) Pop() {
	heap.Pop(&t.heads)
}
