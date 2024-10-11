// Copyright 2018 The go-ethereum Authors
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

package core

import (
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/log"
)

// insertStats tracks and reports on block insertion.
type insertStats struct {
	queued, processed, ignored int
	usedGas                    uint64
	lastIndex                  int
	startTime                  mclock.AbsTime
}

// statsReportLimit is the time limit during import and export after which we
// always print out progress. This avoids the user wondering what's going on.
const statsReportLimit = 8 * time.Second

// report prints statistics if some number of blocks have been processed
// or more than a few seconds have passed since the last message.
func (st *insertStats) report(chain []*types.Block, index int, snapDiffItems, snapBufItems, trieDiffNodes, triebufNodes common.StorageSize, setHead bool) {
	// Fetch the timings for the batch
	var (
		now     = mclock.Now()
		elapsed = now.Sub(st.startTime)
	)
	// If we're at the last block of the batch or report period reached, log
	if index == len(chain)-1 || elapsed >= statsReportLimit {
		// Count the number of transactions in this segment
		var txs int
		for _, block := range chain[st.lastIndex : index+1] {
			txs += len(block.Transactions())
		}
		end := chain[index]

		// Assemble the log context and send it to the logger
		context := []interface{}{
			"number", end.Number(), "hash", end.Hash(),
			"blocks", st.processed, "txs", txs, "mgas", float64(st.usedGas) / 1000000,
			"elapsed", common.PrettyDuration(elapsed), "mgasps", float64(st.usedGas) * 1000 / float64(elapsed),
		}
		if timestamp := time.Unix(int64(end.Time()), 0); time.Since(timestamp) > time.Minute {
			context = append(context, []interface{}{"age", common.PrettyAge(timestamp)}...)
		}
		if snapDiffItems != 0 || snapBufItems != 0 { // snapshots enabled
			context = append(context, []interface{}{"snapdiffs", snapDiffItems}...)
			if snapBufItems != 0 { // future snapshot refactor
				context = append(context, []interface{}{"snapdirty", snapBufItems}...)
			}
		}
		if trieDiffNodes != 0 { // pathdb
			context = append(context, []interface{}{"triediffs", trieDiffNodes}...)
		}
		context = append(context, []interface{}{"triedirty", triebufNodes}...)

		if st.queued > 0 {
			context = append(context, []interface{}{"queued", st.queued}...)
		}
		if st.ignored > 0 {
			context = append(context, []interface{}{"ignored", st.ignored}...)
		}
		if setHead {
			log.Info("Imported new chain segment", context...)
		} else {
			log.Info("Imported new potential chain segment", context...)
		}
		// Bump the stats reported to the next section
		*st = insertStats{startTime: now, lastIndex: index + 1}
	}
}

// insertIterator is a helper to assist during chain import.
type insertIterator struct {
	chain types.Blocks // Chain of blocks being iterated over

	results <-chan error // Verification result sink from the consensus engine
	errors  []error      // Header verification errors for the blocks

	index     int       // Current offset of the iterator
	validator Validator // Validator to run if verification succeeds
}

// newInsertIterator creates a new iterator based on the given blocks, which are
// assumed to be a contiguous chain.
func newInsertIterator(chain types.Blocks, results <-chan error, validator Validator) *insertIterator {
	return &insertIterator{
		chain:     chain,
		results:   results,
		errors:    make([]error, 0, len(chain)),
		index:     -1,
		validator: validator,
	}
}

// next returns the next block in the iterator, along with any potential validation
// error for that block. When the end is reached, it will return (nil, nil).
func (it *insertIterator) next() (*types.Block, error) {
	// If we reached the end of the chain, abort
	if it.index+1 >= len(it.chain) {
		it.index = len(it.chain)
		return nil, nil
	}
	// Advance the iterator and wait for verification result if not yet done
	it.index++
	if len(it.errors) <= it.index {
		it.errors = append(it.errors, <-it.results)
	}
	if it.errors[it.index] != nil {
		return it.chain[it.index], it.errors[it.index]
	}
	// Block header valid, run body validation and return
	return it.chain[it.index], it.validator.ValidateBody(it.chain[it.index])
}

// peek returns the next block in the iterator, along with any potential validation
// error for that block, but does **not** advance the iterator.
//
// Both header and body validation errors (nil too) is cached into the iterator
// to avoid duplicating work on the following next() call.
func (it *insertIterator) peek() (*types.Block, error) {
	// If we reached the end of the chain, abort
	if it.index+1 >= len(it.chain) {
		return nil, nil
	}
	// Wait for verification result if not yet done
	if len(it.errors) <= it.index+1 {
		it.errors = append(it.errors, <-it.results)
	}
	if it.errors[it.index+1] != nil {
		return it.chain[it.index+1], it.errors[it.index+1]
	}
	// Block header valid, ignore body validation since we don't have a parent anyway
	return it.chain[it.index+1], nil
}

// previous returns the previous header that was being processed, or nil.
func (it *insertIterator) previous() *types.Header {
	if it.index < 1 {
		return nil
	}
	return it.chain[it.index-1].Header()
}

// current returns the current header that is being processed, or nil.
func (it *insertIterator) current() *types.Header {
	if it.index == -1 || it.index >= len(it.chain) {
		return nil
	}
	return it.chain[it.index].Header()
}

// first returns the first block in the it.
func (it *insertIterator) first() *types.Block {
	return it.chain[0]
}

// remaining returns the number of remaining blocks.
func (it *insertIterator) remaining() int {
	return len(it.chain) - it.index
}

// processed returns the number of processed blocks.
func (it *insertIterator) processed() int {
	return it.index + 1
}
