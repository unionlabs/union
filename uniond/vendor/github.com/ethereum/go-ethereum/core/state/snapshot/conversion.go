// Copyright 2020 The go-ethereum Authors
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

package snapshot

import (
	"encoding/binary"
	"errors"
	"fmt"
	"math"
	"runtime"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/ethereum/go-ethereum/trie"
)

// trieKV represents a trie key-value pair
type trieKV struct {
	key   common.Hash
	value []byte
}

type (
	// trieGeneratorFn is the interface of trie generation which can
	// be implemented by different trie algorithm.
	trieGeneratorFn func(db ethdb.KeyValueWriter, scheme string, owner common.Hash, in chan (trieKV), out chan (common.Hash))

	// leafCallbackFn is the callback invoked at the leaves of the trie,
	// returns the subtrie root with the specified subtrie identifier.
	leafCallbackFn func(db ethdb.KeyValueWriter, accountHash, codeHash common.Hash, stat *generateStats) (common.Hash, error)
)

// GenerateAccountTrieRoot takes an account iterator and reproduces the root hash.
func GenerateAccountTrieRoot(it AccountIterator) (common.Hash, error) {
	return generateTrieRoot(nil, "", it, common.Hash{}, stackTrieGenerate, nil, newGenerateStats(), true)
}

// GenerateStorageTrieRoot takes a storage iterator and reproduces the root hash.
func GenerateStorageTrieRoot(account common.Hash, it StorageIterator) (common.Hash, error) {
	return generateTrieRoot(nil, "", it, account, stackTrieGenerate, nil, newGenerateStats(), true)
}

// GenerateTrie takes the whole snapshot tree as the input, traverses all the
// accounts as well as the corresponding storages and regenerate the whole state
// (account trie + all storage tries).
func GenerateTrie(snaptree *Tree, root common.Hash, src ethdb.Database, dst ethdb.KeyValueWriter) error {
	// Traverse all state by snapshot, re-generate the whole state trie
	acctIt, err := snaptree.AccountIterator(root, common.Hash{})
	if err != nil {
		return err // The required snapshot might not exist.
	}
	defer acctIt.Release()

	scheme := snaptree.triedb.Scheme()
	got, err := generateTrieRoot(dst, scheme, acctIt, common.Hash{}, stackTrieGenerate, func(dst ethdb.KeyValueWriter, accountHash, codeHash common.Hash, stat *generateStats) (common.Hash, error) {
		// Migrate the code first, commit the contract code into the tmp db.
		if codeHash != types.EmptyCodeHash {
			code := rawdb.ReadCode(src, codeHash)
			if len(code) == 0 {
				return common.Hash{}, errors.New("failed to read contract code")
			}
			rawdb.WriteCode(dst, codeHash, code)
		}
		// Then migrate all storage trie nodes into the tmp db.
		storageIt, err := snaptree.StorageIterator(root, accountHash, common.Hash{})
		if err != nil {
			return common.Hash{}, err
		}
		defer storageIt.Release()

		hash, err := generateTrieRoot(dst, scheme, storageIt, accountHash, stackTrieGenerate, nil, stat, false)
		if err != nil {
			return common.Hash{}, err
		}
		return hash, nil
	}, newGenerateStats(), true)

	if err != nil {
		return err
	}
	if got != root {
		return fmt.Errorf("state root hash mismatch: got %x, want %x", got, root)
	}
	return nil
}

// generateStats is a collection of statistics gathered by the trie generator
// for logging purposes.
type generateStats struct {
	head  common.Hash
	start time.Time

	accounts uint64 // Number of accounts done (including those being crawled)
	slots    uint64 // Number of storage slots done (including those being crawled)

	slotsStart map[common.Hash]time.Time   // Start time for account slot crawling
	slotsHead  map[common.Hash]common.Hash // Slot head for accounts being crawled

	lock sync.RWMutex
}

// newGenerateStats creates a new generator stats.
func newGenerateStats() *generateStats {
	return &generateStats{
		slotsStart: make(map[common.Hash]time.Time),
		slotsHead:  make(map[common.Hash]common.Hash),
		start:      time.Now(),
	}
}

// progressAccounts updates the generator stats for the account range.
func (stat *generateStats) progressAccounts(account common.Hash, done uint64) {
	stat.lock.Lock()
	defer stat.lock.Unlock()

	stat.accounts += done
	stat.head = account
}

// finishAccounts updates the generator stats for the finished account range.
func (stat *generateStats) finishAccounts(done uint64) {
	stat.lock.Lock()
	defer stat.lock.Unlock()

	stat.accounts += done
}

// progressContract updates the generator stats for a specific in-progress contract.
func (stat *generateStats) progressContract(account common.Hash, slot common.Hash, done uint64) {
	stat.lock.Lock()
	defer stat.lock.Unlock()

	stat.slots += done
	stat.slotsHead[account] = slot
	if _, ok := stat.slotsStart[account]; !ok {
		stat.slotsStart[account] = time.Now()
	}
}

// finishContract updates the generator stats for a specific just-finished contract.
func (stat *generateStats) finishContract(account common.Hash, done uint64) {
	stat.lock.Lock()
	defer stat.lock.Unlock()

	stat.slots += done
	delete(stat.slotsHead, account)
	delete(stat.slotsStart, account)
}

// report prints the cumulative progress statistic smartly.
func (stat *generateStats) report() {
	stat.lock.RLock()
	defer stat.lock.RUnlock()

	ctx := []interface{}{
		"accounts", stat.accounts,
		"slots", stat.slots,
		"elapsed", common.PrettyDuration(time.Since(stat.start)),
	}
	if stat.accounts > 0 {
		// If there's progress on the account trie, estimate the time to finish crawling it
		if done := binary.BigEndian.Uint64(stat.head[:8]) / stat.accounts; done > 0 {
			var (
				left  = (math.MaxUint64 - binary.BigEndian.Uint64(stat.head[:8])) / stat.accounts
				speed = done/uint64(time.Since(stat.start)/time.Millisecond+1) + 1 // +1s to avoid division by zero
				eta   = time.Duration(left/speed) * time.Millisecond
			)
			// If there are large contract crawls in progress, estimate their finish time
			for acc, head := range stat.slotsHead {
				start := stat.slotsStart[acc]
				if done := binary.BigEndian.Uint64(head[:8]); done > 0 {
					var (
						left  = math.MaxUint64 - binary.BigEndian.Uint64(head[:8])
						speed = done/uint64(time.Since(start)/time.Millisecond+1) + 1 // +1s to avoid division by zero
					)
					// Override the ETA if larger than the largest until now
					if slotETA := time.Duration(left/speed) * time.Millisecond; eta < slotETA {
						eta = slotETA
					}
				}
			}
			ctx = append(ctx, []interface{}{
				"eta", common.PrettyDuration(eta),
			}...)
		}
	}
	log.Info("Iterating state snapshot", ctx...)
}

// reportDone prints the last log when the whole generation is finished.
func (stat *generateStats) reportDone() {
	stat.lock.RLock()
	defer stat.lock.RUnlock()

	var ctx []interface{}
	ctx = append(ctx, []interface{}{"accounts", stat.accounts}...)
	if stat.slots != 0 {
		ctx = append(ctx, []interface{}{"slots", stat.slots}...)
	}
	ctx = append(ctx, []interface{}{"elapsed", common.PrettyDuration(time.Since(stat.start))}...)
	log.Info("Iterated snapshot", ctx...)
}

// runReport periodically prints the progress information.
func runReport(stats *generateStats, stop chan bool) {
	timer := time.NewTimer(0)
	defer timer.Stop()

	for {
		select {
		case <-timer.C:
			stats.report()
			timer.Reset(time.Second * 8)
		case success := <-stop:
			if success {
				stats.reportDone()
			}
			return
		}
	}
}

// generateTrieRoot generates the trie hash based on the snapshot iterator.
// It can be used for generating account trie, storage trie or even the
// whole state which connects the accounts and the corresponding storages.
func generateTrieRoot(db ethdb.KeyValueWriter, scheme string, it Iterator, account common.Hash, generatorFn trieGeneratorFn, leafCallback leafCallbackFn, stats *generateStats, report bool) (common.Hash, error) {
	var (
		in      = make(chan trieKV)         // chan to pass leaves
		out     = make(chan common.Hash, 1) // chan to collect result
		stoplog = make(chan bool, 1)        // 1-size buffer, works when logging is not enabled
		wg      sync.WaitGroup
	)
	// Spin up a go-routine for trie hash re-generation
	wg.Add(1)
	go func() {
		defer wg.Done()
		generatorFn(db, scheme, account, in, out)
	}()
	// Spin up a go-routine for progress logging
	if report && stats != nil {
		wg.Add(1)
		go func() {
			defer wg.Done()
			runReport(stats, stoplog)
		}()
	}
	// Create a semaphore to assign tasks and collect results through. We'll pre-
	// fill it with nils, thus using the same channel for both limiting concurrent
	// processing and gathering results.
	threads := runtime.NumCPU()
	results := make(chan error, threads)
	for i := 0; i < threads; i++ {
		results <- nil // fill the semaphore
	}
	// stop is a helper function to shutdown the background threads
	// and return the re-generated trie hash.
	stop := func(fail error) (common.Hash, error) {
		close(in)
		result := <-out
		for i := 0; i < threads; i++ {
			if err := <-results; err != nil && fail == nil {
				fail = err
			}
		}
		stoplog <- fail == nil

		wg.Wait()
		return result, fail
	}
	var (
		logged    = time.Now()
		processed = uint64(0)
		leaf      trieKV
	)
	// Start to feed leaves
	for it.Next() {
		if account == (common.Hash{}) {
			var (
				err      error
				fullData []byte
			)
			if leafCallback == nil {
				fullData, err = types.FullAccountRLP(it.(AccountIterator).Account())
				if err != nil {
					return stop(err)
				}
			} else {
				// Wait until the semaphore allows us to continue, aborting if
				// a sub-task failed
				if err := <-results; err != nil {
					results <- nil // stop will drain the results, add a noop back for this error we just consumed
					return stop(err)
				}
				// Fetch the next account and process it concurrently
				account, err := types.FullAccount(it.(AccountIterator).Account())
				if err != nil {
					return stop(err)
				}
				go func(hash common.Hash) {
					subroot, err := leafCallback(db, hash, common.BytesToHash(account.CodeHash), stats)
					if err != nil {
						results <- err
						return
					}
					if account.Root != subroot {
						results <- fmt.Errorf("invalid subroot(path %x), want %x, have %x", hash, account.Root, subroot)
						return
					}
					results <- nil
				}(it.Hash())
				fullData, err = rlp.EncodeToBytes(account)
				if err != nil {
					return stop(err)
				}
			}
			leaf = trieKV{it.Hash(), fullData}
		} else {
			leaf = trieKV{it.Hash(), common.CopyBytes(it.(StorageIterator).Slot())}
		}
		in <- leaf

		// Accumulate the generation statistic if it's required.
		processed++
		if time.Since(logged) > 3*time.Second && stats != nil {
			if account == (common.Hash{}) {
				stats.progressAccounts(it.Hash(), processed)
			} else {
				stats.progressContract(account, it.Hash(), processed)
			}
			logged, processed = time.Now(), 0
		}
	}
	// Commit the last part statistic.
	if processed > 0 && stats != nil {
		if account == (common.Hash{}) {
			stats.finishAccounts(processed)
		} else {
			stats.finishContract(account, processed)
		}
	}
	return stop(nil)
}

func stackTrieGenerate(db ethdb.KeyValueWriter, scheme string, owner common.Hash, in chan trieKV, out chan common.Hash) {
	options := trie.NewStackTrieOptions()
	if db != nil {
		options = options.WithWriter(func(path []byte, hash common.Hash, blob []byte) {
			rawdb.WriteTrieNode(db, owner, path, hash, blob, scheme)
		})
	}
	t := trie.NewStackTrie(options)
	for leaf := range in {
		t.Update(leaf.key[:], leaf.value)
	}
	out <- t.Commit()
}
