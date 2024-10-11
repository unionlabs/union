// Copyright 2021 The go-ethereum Authors
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

package gasprice

import (
	"context"
	"encoding/binary"
	"errors"
	"fmt"
	"math"
	"math/big"
	"sync/atomic"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/consensus/misc/eip1559"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/rpc"
	"golang.org/x/exp/slices"
)

var (
	errInvalidPercentile = errors.New("invalid reward percentile")
	errRequestBeyondHead = errors.New("request beyond head block")
)

const (
	// maxBlockFetchers is the max number of goroutines to spin up to pull blocks
	// for the fee history calculation (mostly relevant for LES).
	maxBlockFetchers = 4
)

// blockFees represents a single block for processing
type blockFees struct {
	// set by the caller
	blockNumber uint64
	header      *types.Header
	block       *types.Block // only set if reward percentiles are requested
	receipts    types.Receipts
	// filled by processBlock
	results processedFees
	err     error
}

type cacheKey struct {
	number      uint64
	percentiles string
}

// processedFees contains the results of a processed block.
type processedFees struct {
	reward               []*big.Int
	baseFee, nextBaseFee *big.Int
	gasUsedRatio         float64
}

// txGasAndReward is sorted in ascending order based on reward
type txGasAndReward struct {
	gasUsed uint64
	reward  *big.Int
}

// processBlock takes a blockFees structure with the blockNumber, the header and optionally
// the block field filled in, retrieves the block from the backend if not present yet and
// fills in the rest of the fields.
func (oracle *Oracle) processBlock(bf *blockFees, percentiles []float64) {
	chainconfig := oracle.backend.ChainConfig()
	if bf.results.baseFee = bf.header.BaseFee; bf.results.baseFee == nil {
		bf.results.baseFee = new(big.Int)
	}
	if chainconfig.IsLondon(big.NewInt(int64(bf.blockNumber + 1))) {
		bf.results.nextBaseFee = eip1559.CalcBaseFee(chainconfig, bf.header)
	} else {
		bf.results.nextBaseFee = new(big.Int)
	}
	bf.results.gasUsedRatio = float64(bf.header.GasUsed) / float64(bf.header.GasLimit)
	if len(percentiles) == 0 {
		// rewards were not requested, return null
		return
	}
	if bf.block == nil || (bf.receipts == nil && len(bf.block.Transactions()) != 0) {
		log.Error("Block or receipts are missing while reward percentiles are requested")
		return
	}

	bf.results.reward = make([]*big.Int, len(percentiles))
	if len(bf.block.Transactions()) == 0 {
		// return an all zero row if there are no transactions to gather data from
		for i := range bf.results.reward {
			bf.results.reward[i] = new(big.Int)
		}
		return
	}

	sorter := make([]txGasAndReward, len(bf.block.Transactions()))
	for i, tx := range bf.block.Transactions() {
		reward, _ := tx.EffectiveGasTip(bf.block.BaseFee())
		sorter[i] = txGasAndReward{gasUsed: bf.receipts[i].GasUsed, reward: reward}
	}
	slices.SortStableFunc(sorter, func(a, b txGasAndReward) int {
		return a.reward.Cmp(b.reward)
	})

	var txIndex int
	sumGasUsed := sorter[0].gasUsed

	for i, p := range percentiles {
		thresholdGasUsed := uint64(float64(bf.block.GasUsed()) * p / 100)
		for sumGasUsed < thresholdGasUsed && txIndex < len(bf.block.Transactions())-1 {
			txIndex++
			sumGasUsed += sorter[txIndex].gasUsed
		}
		bf.results.reward[i] = sorter[txIndex].reward
	}
}

// resolveBlockRange resolves the specified block range to absolute block numbers while also
// enforcing backend specific limitations. The pending block and corresponding receipts are
// also returned if requested and available.
// Note: an error is only returned if retrieving the head header has failed. If there are no
// retrievable blocks in the specified range then zero block count is returned with no error.
func (oracle *Oracle) resolveBlockRange(ctx context.Context, reqEnd rpc.BlockNumber, blocks uint64) (*types.Block, []*types.Receipt, uint64, uint64, error) {
	var (
		headBlock       *types.Header
		pendingBlock    *types.Block
		pendingReceipts types.Receipts
		err             error
	)

	// Get the chain's current head.
	if headBlock, err = oracle.backend.HeaderByNumber(ctx, rpc.LatestBlockNumber); err != nil {
		return nil, nil, 0, 0, err
	}
	head := rpc.BlockNumber(headBlock.Number.Uint64())

	// Fail if request block is beyond the chain's current head.
	if head < reqEnd {
		return nil, nil, 0, 0, fmt.Errorf("%w: requested %d, head %d", errRequestBeyondHead, reqEnd, head)
	}

	// Resolve block tag.
	if reqEnd < 0 {
		var (
			resolved *types.Header
			err      error
		)
		switch reqEnd {
		case rpc.PendingBlockNumber:
			if pendingBlock, pendingReceipts = oracle.backend.PendingBlockAndReceipts(); pendingBlock != nil {
				resolved = pendingBlock.Header()
			} else {
				// Pending block not supported by backend, process only until latest block.
				resolved = headBlock

				// Update total blocks to return to account for this.
				blocks--
			}
		case rpc.LatestBlockNumber:
			// Retrieved above.
			resolved = headBlock
		case rpc.SafeBlockNumber:
			resolved, err = oracle.backend.HeaderByNumber(ctx, rpc.SafeBlockNumber)
		case rpc.FinalizedBlockNumber:
			resolved, err = oracle.backend.HeaderByNumber(ctx, rpc.FinalizedBlockNumber)
		case rpc.EarliestBlockNumber:
			resolved, err = oracle.backend.HeaderByNumber(ctx, rpc.EarliestBlockNumber)
		}
		if resolved == nil || err != nil {
			return nil, nil, 0, 0, err
		}
		// Absolute number resolved.
		reqEnd = rpc.BlockNumber(resolved.Number.Uint64())
	}

	// If there are no blocks to return, short circuit.
	if blocks == 0 {
		return nil, nil, 0, 0, nil
	}
	// Ensure not trying to retrieve before genesis.
	if uint64(reqEnd+1) < blocks {
		blocks = uint64(reqEnd + 1)
	}
	return pendingBlock, pendingReceipts, uint64(reqEnd), blocks, nil
}

// FeeHistory returns data relevant for fee estimation based on the specified range of blocks.
// The range can be specified either with absolute block numbers or ending with the latest
// or pending block. Backends may or may not support gathering data from the pending block
// or blocks older than a certain age (specified in maxHistory). The first block of the
// actually processed range is returned to avoid ambiguity when parts of the requested range
// are not available or when the head has changed during processing this request.
// Three arrays are returned based on the processed blocks:
//   - reward: the requested percentiles of effective priority fees per gas of transactions in each
//     block, sorted in ascending order and weighted by gas used.
//   - baseFee: base fee per gas in the given block
//   - gasUsedRatio: gasUsed/gasLimit in the given block
//
// Note: baseFee includes the next block after the newest of the returned range, because this
// value can be derived from the newest block.
func (oracle *Oracle) FeeHistory(ctx context.Context, blocks uint64, unresolvedLastBlock rpc.BlockNumber, rewardPercentiles []float64) (*big.Int, [][]*big.Int, []*big.Int, []float64, error) {
	if blocks < 1 {
		return common.Big0, nil, nil, nil, nil // returning with no data and no error means there are no retrievable blocks
	}
	maxFeeHistory := oracle.maxHeaderHistory
	if len(rewardPercentiles) != 0 {
		maxFeeHistory = oracle.maxBlockHistory
	}
	if blocks > maxFeeHistory {
		log.Warn("Sanitizing fee history length", "requested", blocks, "truncated", maxFeeHistory)
		blocks = maxFeeHistory
	}
	for i, p := range rewardPercentiles {
		if p < 0 || p > 100 {
			return common.Big0, nil, nil, nil, fmt.Errorf("%w: %f", errInvalidPercentile, p)
		}
		if i > 0 && p < rewardPercentiles[i-1] {
			return common.Big0, nil, nil, nil, fmt.Errorf("%w: #%d:%f > #%d:%f", errInvalidPercentile, i-1, rewardPercentiles[i-1], i, p)
		}
	}
	var (
		pendingBlock    *types.Block
		pendingReceipts []*types.Receipt
		err             error
	)
	pendingBlock, pendingReceipts, lastBlock, blocks, err := oracle.resolveBlockRange(ctx, unresolvedLastBlock, blocks)
	if err != nil || blocks == 0 {
		return common.Big0, nil, nil, nil, err
	}
	oldestBlock := lastBlock + 1 - blocks

	var next atomic.Uint64
	next.Store(oldestBlock)
	results := make(chan *blockFees, blocks)

	percentileKey := make([]byte, 8*len(rewardPercentiles))
	for i, p := range rewardPercentiles {
		binary.LittleEndian.PutUint64(percentileKey[i*8:(i+1)*8], math.Float64bits(p))
	}
	for i := 0; i < maxBlockFetchers && i < int(blocks); i++ {
		go func() {
			for {
				// Retrieve the next block number to fetch with this goroutine
				blockNumber := next.Add(1) - 1
				if blockNumber > lastBlock {
					return
				}

				fees := &blockFees{blockNumber: blockNumber}
				if pendingBlock != nil && blockNumber >= pendingBlock.NumberU64() {
					fees.block, fees.receipts = pendingBlock, pendingReceipts
					fees.header = fees.block.Header()
					oracle.processBlock(fees, rewardPercentiles)
					results <- fees
				} else {
					cacheKey := cacheKey{number: blockNumber, percentiles: string(percentileKey)}

					if p, ok := oracle.historyCache.Get(cacheKey); ok {
						fees.results = p
						results <- fees
					} else {
						if len(rewardPercentiles) != 0 {
							fees.block, fees.err = oracle.backend.BlockByNumber(ctx, rpc.BlockNumber(blockNumber))
							if fees.block != nil && fees.err == nil {
								fees.receipts, fees.err = oracle.backend.GetReceipts(ctx, fees.block.Hash())
								fees.header = fees.block.Header()
							}
						} else {
							fees.header, fees.err = oracle.backend.HeaderByNumber(ctx, rpc.BlockNumber(blockNumber))
						}
						if fees.header != nil && fees.err == nil {
							oracle.processBlock(fees, rewardPercentiles)
							if fees.err == nil {
								oracle.historyCache.Add(cacheKey, fees.results)
							}
						}
						// send to results even if empty to guarantee that blocks items are sent in total
						results <- fees
					}
				}
			}
		}()
	}
	var (
		reward       = make([][]*big.Int, blocks)
		baseFee      = make([]*big.Int, blocks+1)
		gasUsedRatio = make([]float64, blocks)
		firstMissing = blocks
	)
	for ; blocks > 0; blocks-- {
		fees := <-results
		if fees.err != nil {
			return common.Big0, nil, nil, nil, fees.err
		}
		i := fees.blockNumber - oldestBlock
		if fees.results.baseFee != nil {
			reward[i], baseFee[i], baseFee[i+1], gasUsedRatio[i] = fees.results.reward, fees.results.baseFee, fees.results.nextBaseFee, fees.results.gasUsedRatio
		} else {
			// getting no block and no error means we are requesting into the future (might happen because of a reorg)
			if i < firstMissing {
				firstMissing = i
			}
		}
	}
	if firstMissing == 0 {
		return common.Big0, nil, nil, nil, nil
	}
	if len(rewardPercentiles) != 0 {
		reward = reward[:firstMissing]
	} else {
		reward = nil
	}
	baseFee, gasUsedRatio = baseFee[:firstMissing+1], gasUsedRatio[:firstMissing]
	return new(big.Int).SetUint64(oldestBlock), reward, baseFee, gasUsedRatio, nil
}
