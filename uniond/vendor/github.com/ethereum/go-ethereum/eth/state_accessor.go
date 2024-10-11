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

package eth

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/state"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/eth/tracers"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/trie"
)

// noopReleaser is returned in case there is no operation expected
// for releasing state.
var noopReleaser = tracers.StateReleaseFunc(func() {})

func (eth *Ethereum) hashState(ctx context.Context, block *types.Block, reexec uint64, base *state.StateDB, readOnly bool, preferDisk bool) (statedb *state.StateDB, release tracers.StateReleaseFunc, err error) {
	var (
		current  *types.Block
		database state.Database
		triedb   *trie.Database
		report   = true
		origin   = block.NumberU64()
	)
	// The state is only for reading purposes, check the state presence in
	// live database.
	if readOnly {
		// The state is available in live database, create a reference
		// on top to prevent garbage collection and return a release
		// function to deref it.
		if statedb, err = eth.blockchain.StateAt(block.Root()); err == nil {
			eth.blockchain.TrieDB().Reference(block.Root(), common.Hash{})
			return statedb, func() {
				eth.blockchain.TrieDB().Dereference(block.Root())
			}, nil
		}
	}
	// The state is both for reading and writing, or it's unavailable in disk,
	// try to construct/recover the state over an ephemeral trie.Database for
	// isolating the live one.
	if base != nil {
		if preferDisk {
			// Create an ephemeral trie.Database for isolating the live one. Otherwise
			// the internal junks created by tracing will be persisted into the disk.
			// TODO(rjl493456442), clean cache is disabled to prevent memory leak,
			// please re-enable it for better performance.
			database = state.NewDatabaseWithConfig(eth.chainDb, trie.HashDefaults)
			if statedb, err = state.New(block.Root(), database, nil); err == nil {
				log.Info("Found disk backend for state trie", "root", block.Root(), "number", block.Number())
				return statedb, noopReleaser, nil
			}
		}
		// The optional base statedb is given, mark the start point as parent block
		statedb, database, triedb, report = base, base.Database(), base.Database().TrieDB(), false
		current = eth.blockchain.GetBlock(block.ParentHash(), block.NumberU64()-1)
	} else {
		// Otherwise, try to reexec blocks until we find a state or reach our limit
		current = block

		// Create an ephemeral trie.Database for isolating the live one. Otherwise
		// the internal junks created by tracing will be persisted into the disk.
		// TODO(rjl493456442), clean cache is disabled to prevent memory leak,
		// please re-enable it for better performance.
		triedb = trie.NewDatabase(eth.chainDb, trie.HashDefaults)
		database = state.NewDatabaseWithNodeDB(eth.chainDb, triedb)

		// If we didn't check the live database, do check state over ephemeral database,
		// otherwise we would rewind past a persisted block (specific corner case is
		// chain tracing from the genesis).
		if !readOnly {
			statedb, err = state.New(current.Root(), database, nil)
			if err == nil {
				return statedb, noopReleaser, nil
			}
		}
		// Database does not have the state for the given block, try to regenerate
		for i := uint64(0); i < reexec; i++ {
			if err := ctx.Err(); err != nil {
				return nil, nil, err
			}
			if current.NumberU64() == 0 {
				return nil, nil, errors.New("genesis state is missing")
			}
			parent := eth.blockchain.GetBlock(current.ParentHash(), current.NumberU64()-1)
			if parent == nil {
				return nil, nil, fmt.Errorf("missing block %v %d", current.ParentHash(), current.NumberU64()-1)
			}
			current = parent

			statedb, err = state.New(current.Root(), database, nil)
			if err == nil {
				break
			}
		}
		if err != nil {
			switch err.(type) {
			case *trie.MissingNodeError:
				return nil, nil, fmt.Errorf("required historical state unavailable (reexec=%d)", reexec)
			default:
				return nil, nil, err
			}
		}
	}
	// State is available at historical point, re-execute the blocks on top for
	// the desired state.
	var (
		start  = time.Now()
		logged time.Time
		parent common.Hash
	)
	for current.NumberU64() < origin {
		if err := ctx.Err(); err != nil {
			return nil, nil, err
		}
		// Print progress logs if long enough time elapsed
		if time.Since(logged) > 8*time.Second && report {
			log.Info("Regenerating historical state", "block", current.NumberU64()+1, "target", origin, "remaining", origin-current.NumberU64()-1, "elapsed", time.Since(start))
			logged = time.Now()
		}
		// Retrieve the next block to regenerate and process it
		next := current.NumberU64() + 1
		if current = eth.blockchain.GetBlockByNumber(next); current == nil {
			return nil, nil, fmt.Errorf("block #%d not found", next)
		}
		_, _, _, err := eth.blockchain.Processor().Process(current, statedb, vm.Config{})
		if err != nil {
			return nil, nil, fmt.Errorf("processing block %d failed: %v", current.NumberU64(), err)
		}
		// Finalize the state so any modifications are written to the trie
		root, err := statedb.Commit(current.NumberU64(), eth.blockchain.Config().IsEIP158(current.Number()))
		if err != nil {
			return nil, nil, fmt.Errorf("stateAtBlock commit failed, number %d root %v: %w",
				current.NumberU64(), current.Root().Hex(), err)
		}
		statedb, err = state.New(root, database, nil)
		if err != nil {
			return nil, nil, fmt.Errorf("state reset after block %d failed: %v", current.NumberU64(), err)
		}
		// Hold the state reference and also drop the parent state
		// to prevent accumulating too many nodes in memory.
		triedb.Reference(root, common.Hash{})
		if parent != (common.Hash{}) {
			triedb.Dereference(parent)
		}
		parent = root
	}
	if report {
		_, nodes, imgs := triedb.Size() // all memory is contained within the nodes return in hashdb
		log.Info("Historical state regenerated", "block", current.NumberU64(), "elapsed", time.Since(start), "nodes", nodes, "preimages", imgs)
	}
	return statedb, func() { triedb.Dereference(block.Root()) }, nil
}

func (eth *Ethereum) pathState(block *types.Block) (*state.StateDB, func(), error) {
	// Check if the requested state is available in the live chain.
	statedb, err := eth.blockchain.StateAt(block.Root())
	if err == nil {
		return statedb, noopReleaser, nil
	}
	// TODO historic state is not supported in path-based scheme.
	// Fully archive node in pbss will be implemented by relying
	// on state history, but needs more work on top.
	return nil, nil, errors.New("historical state not available in path scheme yet")
}

// stateAtBlock retrieves the state database associated with a certain block.
// If no state is locally available for the given block, a number of blocks
// are attempted to be reexecuted to generate the desired state. The optional
// base layer statedb can be provided which is regarded as the statedb of the
// parent block.
//
// An additional release function will be returned if the requested state is
// available. Release is expected to be invoked when the returned state is no
// longer needed. Its purpose is to prevent resource leaking. Though it can be
// noop in some cases.
//
// Parameters:
//   - block:      The block for which we want the state(state = block.Root)
//   - reexec:     The maximum number of blocks to reprocess trying to obtain the desired state
//   - base:       If the caller is tracing multiple blocks, the caller can provide the parent
//     state continuously from the callsite.
//   - readOnly:   If true, then the live 'blockchain' state database is used. No mutation should
//     be made from caller, e.g. perform Commit or other 'save-to-disk' changes.
//     Otherwise, the trash generated by caller may be persisted permanently.
//   - preferDisk: This arg can be used by the caller to signal that even though the 'base' is
//     provided, it would be preferable to start from a fresh state, if we have it
//     on disk.
func (eth *Ethereum) stateAtBlock(ctx context.Context, block *types.Block, reexec uint64, base *state.StateDB, readOnly bool, preferDisk bool) (statedb *state.StateDB, release tracers.StateReleaseFunc, err error) {
	if eth.blockchain.TrieDB().Scheme() == rawdb.HashScheme {
		return eth.hashState(ctx, block, reexec, base, readOnly, preferDisk)
	}
	return eth.pathState(block)
}

// stateAtTransaction returns the execution environment of a certain transaction.
func (eth *Ethereum) stateAtTransaction(ctx context.Context, block *types.Block, txIndex int, reexec uint64) (*core.Message, vm.BlockContext, *state.StateDB, tracers.StateReleaseFunc, error) {
	// Short circuit if it's genesis block.
	if block.NumberU64() == 0 {
		return nil, vm.BlockContext{}, nil, nil, errors.New("no transaction in genesis")
	}
	// Create the parent state database
	parent := eth.blockchain.GetBlock(block.ParentHash(), block.NumberU64()-1)
	if parent == nil {
		return nil, vm.BlockContext{}, nil, nil, fmt.Errorf("parent %#x not found", block.ParentHash())
	}
	// Lookup the statedb of parent block from the live database,
	// otherwise regenerate it on the flight.
	statedb, release, err := eth.stateAtBlock(ctx, parent, reexec, nil, true, false)
	if err != nil {
		return nil, vm.BlockContext{}, nil, nil, err
	}
	if txIndex == 0 && len(block.Transactions()) == 0 {
		return nil, vm.BlockContext{}, statedb, release, nil
	}
	// Recompute transactions up to the target index.
	signer := types.MakeSigner(eth.blockchain.Config(), block.Number(), block.Time())
	for idx, tx := range block.Transactions() {
		// Assemble the transaction call message and return if the requested offset
		msg, _ := core.TransactionToMessage(tx, signer, block.BaseFee())
		txContext := core.NewEVMTxContext(msg)
		context := core.NewEVMBlockContext(block.Header(), eth.blockchain, nil)
		if idx == txIndex {
			return msg, context, statedb, release, nil
		}
		// Not yet the searched for transaction, execute on top of the current state
		vmenv := vm.NewEVM(context, txContext, statedb, eth.blockchain.Config(), vm.Config{})
		statedb.SetTxContext(tx.Hash(), idx)
		if _, err := core.ApplyMessage(vmenv, msg, new(core.GasPool).AddGas(tx.Gas())); err != nil {
			return nil, vm.BlockContext{}, nil, nil, fmt.Errorf("transaction %#x failed: %v", tx.Hash(), err)
		}
		// Ensure any modifications are committed to the state
		// Only delete empty objects if EIP158/161 (a.k.a Spurious Dragon) is in effect
		statedb.Finalise(vmenv.ChainConfig().IsEIP158(block.Number()))
	}
	return nil, vm.BlockContext{}, nil, nil, fmt.Errorf("transaction index %d out of range for block %#x", txIndex, block.Hash())
}
