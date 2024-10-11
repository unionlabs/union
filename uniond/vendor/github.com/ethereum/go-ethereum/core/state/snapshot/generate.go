// Copyright 2019 The go-ethereum Authors
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
	"bytes"
	"errors"
	"fmt"
	"time"

	"github.com/VictoriaMetrics/fastcache"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/ethereum/go-ethereum/trie"
	"github.com/ethereum/go-ethereum/trie/trienode"
)

var (
	// accountCheckRange is the upper limit of the number of accounts involved in
	// each range check. This is a value estimated based on experience. If this
	// range is too large, the failure rate of range proof will increase. Otherwise,
	// if the range is too small, the efficiency of the state recovery will decrease.
	accountCheckRange = 128

	// storageCheckRange is the upper limit of the number of storage slots involved
	// in each range check. This is a value estimated based on experience. If this
	// range is too large, the failure rate of range proof will increase. Otherwise,
	// if the range is too small, the efficiency of the state recovery will decrease.
	storageCheckRange = 1024

	// errMissingTrie is returned if the target trie is missing while the generation
	// is running. In this case the generation is aborted and wait the new signal.
	errMissingTrie = errors.New("missing trie")
)

// generateSnapshot regenerates a brand new snapshot based on an existing state
// database and head block asynchronously. The snapshot is returned immediately
// and generation is continued in the background until done.
func generateSnapshot(diskdb ethdb.KeyValueStore, triedb *trie.Database, cache int, root common.Hash) *diskLayer {
	// Create a new disk layer with an initialized state marker at zero
	var (
		stats     = &generatorStats{start: time.Now()}
		batch     = diskdb.NewBatch()
		genMarker = []byte{} // Initialized but empty!
	)
	rawdb.WriteSnapshotRoot(batch, root)
	journalProgress(batch, genMarker, stats)
	if err := batch.Write(); err != nil {
		log.Crit("Failed to write initialized state marker", "err", err)
	}
	base := &diskLayer{
		diskdb:     diskdb,
		triedb:     triedb,
		root:       root,
		cache:      fastcache.New(cache * 1024 * 1024),
		genMarker:  genMarker,
		genPending: make(chan struct{}),
		genAbort:   make(chan chan *generatorStats),
	}
	go base.generate(stats)
	log.Debug("Start snapshot generation", "root", root)
	return base
}

// journalProgress persists the generator stats into the database to resume later.
func journalProgress(db ethdb.KeyValueWriter, marker []byte, stats *generatorStats) {
	// Write out the generator marker. Note it's a standalone disk layer generator
	// which is not mixed with journal. It's ok if the generator is persisted while
	// journal is not.
	entry := journalGenerator{
		Done:   marker == nil,
		Marker: marker,
	}
	if stats != nil {
		entry.Accounts = stats.accounts
		entry.Slots = stats.slots
		entry.Storage = uint64(stats.storage)
	}
	blob, err := rlp.EncodeToBytes(entry)
	if err != nil {
		panic(err) // Cannot happen, here to catch dev errors
	}
	var logstr string
	switch {
	case marker == nil:
		logstr = "done"
	case bytes.Equal(marker, []byte{}):
		logstr = "empty"
	case len(marker) == common.HashLength:
		logstr = fmt.Sprintf("%#x", marker)
	default:
		logstr = fmt.Sprintf("%#x:%#x", marker[:common.HashLength], marker[common.HashLength:])
	}
	log.Debug("Journalled generator progress", "progress", logstr)
	rawdb.WriteSnapshotGenerator(db, blob)
}

// proofResult contains the output of range proving which can be used
// for further processing regardless if it is successful or not.
type proofResult struct {
	keys     [][]byte   // The key set of all elements being iterated, even proving is failed
	vals     [][]byte   // The val set of all elements being iterated, even proving is failed
	diskMore bool       // Set when the database has extra snapshot states since last iteration
	trieMore bool       // Set when the trie has extra snapshot states(only meaningful for successful proving)
	proofErr error      // Indicator whether the given state range is valid or not
	tr       *trie.Trie // The trie, in case the trie was resolved by the prover (may be nil)
}

// valid returns the indicator that range proof is successful or not.
func (result *proofResult) valid() bool {
	return result.proofErr == nil
}

// last returns the last verified element key regardless of whether the range proof is
// successful or not. Nil is returned if nothing involved in the proving.
func (result *proofResult) last() []byte {
	var last []byte
	if len(result.keys) > 0 {
		last = result.keys[len(result.keys)-1]
	}
	return last
}

// forEach iterates all the visited elements and applies the given callback on them.
// The iteration is aborted if the callback returns non-nil error.
func (result *proofResult) forEach(callback func(key []byte, val []byte) error) error {
	for i := 0; i < len(result.keys); i++ {
		key, val := result.keys[i], result.vals[i]
		if err := callback(key, val); err != nil {
			return err
		}
	}
	return nil
}

// proveRange proves the snapshot segment with particular prefix is "valid".
// The iteration start point will be assigned if the iterator is restored from
// the last interruption. Max will be assigned in order to limit the maximum
// amount of data involved in each iteration.
//
// The proof result will be returned if the range proving is finished, otherwise
// the error will be returned to abort the entire procedure.
func (dl *diskLayer) proveRange(ctx *generatorContext, trieId *trie.ID, prefix []byte, kind string, origin []byte, max int, valueConvertFn func([]byte) ([]byte, error)) (*proofResult, error) {
	var (
		keys     [][]byte
		vals     [][]byte
		proof    = rawdb.NewMemoryDatabase()
		diskMore = false
		iter     = ctx.iterator(kind)
		start    = time.Now()
		min      = append(prefix, origin...)
	)
	for iter.Next() {
		// Ensure the iterated item is always equal or larger than the given origin.
		key := iter.Key()
		if bytes.Compare(key, min) < 0 {
			return nil, errors.New("invalid iteration position")
		}
		// Ensure the iterated item still fall in the specified prefix. If
		// not which means the items in the specified area are all visited.
		// Move the iterator a step back since we iterate one extra element
		// out.
		if !bytes.Equal(key[:len(prefix)], prefix) {
			iter.Hold()
			break
		}
		// Break if we've reached the max size, and signal that we're not
		// done yet. Move the iterator a step back since we iterate one
		// extra element out.
		if len(keys) == max {
			iter.Hold()
			diskMore = true
			break
		}
		keys = append(keys, common.CopyBytes(key[len(prefix):]))

		if valueConvertFn == nil {
			vals = append(vals, common.CopyBytes(iter.Value()))
		} else {
			val, err := valueConvertFn(iter.Value())
			if err != nil {
				// Special case, the state data is corrupted (invalid slim-format account),
				// don't abort the entire procedure directly. Instead, let the fallback
				// generation to heal the invalid data.
				//
				// Here append the original value to ensure that the number of key and
				// value are aligned.
				vals = append(vals, common.CopyBytes(iter.Value()))
				log.Error("Failed to convert account state data", "err", err)
			} else {
				vals = append(vals, val)
			}
		}
	}
	// Update metrics for database iteration and merkle proving
	if kind == snapStorage {
		snapStorageSnapReadCounter.Inc(time.Since(start).Nanoseconds())
	} else {
		snapAccountSnapReadCounter.Inc(time.Since(start).Nanoseconds())
	}
	defer func(start time.Time) {
		if kind == snapStorage {
			snapStorageProveCounter.Inc(time.Since(start).Nanoseconds())
		} else {
			snapAccountProveCounter.Inc(time.Since(start).Nanoseconds())
		}
	}(time.Now())

	// The snap state is exhausted, pass the entire key/val set for verification
	root := trieId.Root
	if origin == nil && !diskMore {
		stackTr := trie.NewStackTrie(nil)
		for i, key := range keys {
			if err := stackTr.Update(key, vals[i]); err != nil {
				return nil, err
			}
		}
		if gotRoot := stackTr.Hash(); gotRoot != root {
			return &proofResult{
				keys:     keys,
				vals:     vals,
				proofErr: fmt.Errorf("wrong root: have %#x want %#x", gotRoot, root),
			}, nil
		}
		return &proofResult{keys: keys, vals: vals}, nil
	}
	// Snap state is chunked, generate edge proofs for verification.
	tr, err := trie.New(trieId, dl.triedb)
	if err != nil {
		ctx.stats.Log("Trie missing, state snapshotting paused", dl.root, dl.genMarker)
		return nil, errMissingTrie
	}
	// Generate the Merkle proofs for the first and last element
	if origin == nil {
		origin = common.Hash{}.Bytes()
	}
	if err := tr.Prove(origin, proof); err != nil {
		log.Debug("Failed to prove range", "kind", kind, "origin", origin, "err", err)
		return &proofResult{
			keys:     keys,
			vals:     vals,
			diskMore: diskMore,
			proofErr: err,
			tr:       tr,
		}, nil
	}
	if len(keys) > 0 {
		if err := tr.Prove(keys[len(keys)-1], proof); err != nil {
			log.Debug("Failed to prove range", "kind", kind, "last", keys[len(keys)-1], "err", err)
			return &proofResult{
				keys:     keys,
				vals:     vals,
				diskMore: diskMore,
				proofErr: err,
				tr:       tr,
			}, nil
		}
	}
	// Verify the snapshot segment with range prover, ensure that all flat states
	// in this range correspond to merkle trie.
	cont, err := trie.VerifyRangeProof(root, origin, keys, vals, proof)
	return &proofResult{
			keys:     keys,
			vals:     vals,
			diskMore: diskMore,
			trieMore: cont,
			proofErr: err,
			tr:       tr},
		nil
}

// onStateCallback is a function that is called by generateRange, when processing a range of
// accounts or storage slots. For each element, the callback is invoked.
//
// - If 'delete' is true, then this element (and potential slots) needs to be deleted from the snapshot.
// - If 'write' is true, then this element needs to be updated with the 'val'.
// - If 'write' is false, then this element is already correct, and needs no update.
// The 'val' is the canonical encoding of the value (not the slim format for accounts)
//
// However, for accounts, the storage trie of the account needs to be checked. Also,
// dangling storages(storage exists but the corresponding account is missing) need to
// be cleaned up.
type onStateCallback func(key []byte, val []byte, write bool, delete bool) error

// generateRange generates the state segment with particular prefix. Generation can
// either verify the correctness of existing state through range-proof and skip
// generation, or iterate trie to regenerate state on demand.
func (dl *diskLayer) generateRange(ctx *generatorContext, trieId *trie.ID, prefix []byte, kind string, origin []byte, max int, onState onStateCallback, valueConvertFn func([]byte) ([]byte, error)) (bool, []byte, error) {
	// Use range prover to check the validity of the flat state in the range
	result, err := dl.proveRange(ctx, trieId, prefix, kind, origin, max, valueConvertFn)
	if err != nil {
		return false, nil, err
	}
	last := result.last()

	// Construct contextual logger
	logCtx := []interface{}{"kind", kind, "prefix", hexutil.Encode(prefix)}
	if len(origin) > 0 {
		logCtx = append(logCtx, "origin", hexutil.Encode(origin))
	}
	logger := log.New(logCtx...)

	// The range prover says the range is correct, skip trie iteration
	if result.valid() {
		snapSuccessfulRangeProofMeter.Mark(1)
		logger.Trace("Proved state range", "last", hexutil.Encode(last))

		// The verification is passed, process each state with the given
		// callback function. If this state represents a contract, the
		// corresponding storage check will be performed in the callback
		if err := result.forEach(func(key []byte, val []byte) error { return onState(key, val, false, false) }); err != nil {
			return false, nil, err
		}
		// Only abort the iteration when both database and trie are exhausted
		return !result.diskMore && !result.trieMore, last, nil
	}
	logger.Trace("Detected outdated state range", "last", hexutil.Encode(last), "err", result.proofErr)
	snapFailedRangeProofMeter.Mark(1)

	// Special case, the entire trie is missing. In the original trie scheme,
	// all the duplicated subtries will be filtered out (only one copy of data
	// will be stored). While in the snapshot model, all the storage tries
	// belong to different contracts will be kept even they are duplicated.
	// Track it to a certain extent remove the noise data used for statistics.
	if origin == nil && last == nil {
		meter := snapMissallAccountMeter
		if kind == snapStorage {
			meter = snapMissallStorageMeter
		}
		meter.Mark(1)
	}
	// We use the snap data to build up a cache which can be used by the
	// main account trie as a primary lookup when resolving hashes
	var resolver trie.NodeResolver
	if len(result.keys) > 0 {
		mdb := rawdb.NewMemoryDatabase()
		tdb := trie.NewDatabase(mdb, trie.HashDefaults)
		defer tdb.Close()
		snapTrie := trie.NewEmpty(tdb)
		for i, key := range result.keys {
			snapTrie.Update(key, result.vals[i])
		}
		root, nodes, err := snapTrie.Commit(false)
		if err != nil {
			return false, nil, err
		}
		if nodes != nil {
			tdb.Update(root, types.EmptyRootHash, 0, trienode.NewWithNodeSet(nodes), nil)
			tdb.Commit(root, false)
		}
		resolver = func(owner common.Hash, path []byte, hash common.Hash) []byte {
			return rawdb.ReadTrieNode(mdb, owner, path, hash, tdb.Scheme())
		}
	}
	// Construct the trie for state iteration, reuse the trie
	// if it's already opened with some nodes resolved.
	tr := result.tr
	if tr == nil {
		tr, err = trie.New(trieId, dl.triedb)
		if err != nil {
			ctx.stats.Log("Trie missing, state snapshotting paused", dl.root, dl.genMarker)
			return false, nil, errMissingTrie
		}
	}
	var (
		trieMore       bool
		kvkeys, kvvals = result.keys, result.vals

		// counters
		count     = 0 // number of states delivered by iterator
		created   = 0 // states created from the trie
		updated   = 0 // states updated from the trie
		deleted   = 0 // states not in trie, but were in snapshot
		untouched = 0 // states already correct

		// timers
		start    = time.Now()
		internal time.Duration
	)
	nodeIt, err := tr.NodeIterator(origin)
	if err != nil {
		return false, nil, err
	}
	nodeIt.AddResolver(resolver)
	iter := trie.NewIterator(nodeIt)

	for iter.Next() {
		if last != nil && bytes.Compare(iter.Key, last) > 0 {
			trieMore = true
			break
		}
		count++
		write := true
		created++
		for len(kvkeys) > 0 {
			if cmp := bytes.Compare(kvkeys[0], iter.Key); cmp < 0 {
				// delete the key
				istart := time.Now()
				if err := onState(kvkeys[0], nil, false, true); err != nil {
					return false, nil, err
				}
				kvkeys = kvkeys[1:]
				kvvals = kvvals[1:]
				deleted++
				internal += time.Since(istart)
				continue
			} else if cmp == 0 {
				// the snapshot key can be overwritten
				created--
				if write = !bytes.Equal(kvvals[0], iter.Value); write {
					updated++
				} else {
					untouched++
				}
				kvkeys = kvkeys[1:]
				kvvals = kvvals[1:]
			}
			break
		}
		istart := time.Now()
		if err := onState(iter.Key, iter.Value, write, false); err != nil {
			return false, nil, err
		}
		internal += time.Since(istart)
	}
	if iter.Err != nil {
		// Trie errors should never happen. Still, in case of a bug, expose the
		// error here, as the outer code will presume errors are interrupts, not
		// some deeper issues.
		log.Error("State snapshotter failed to iterate trie", "err", err)
		return false, nil, iter.Err
	}
	// Delete all stale snapshot states remaining
	istart := time.Now()
	for _, key := range kvkeys {
		if err := onState(key, nil, false, true); err != nil {
			return false, nil, err
		}
		deleted += 1
	}
	internal += time.Since(istart)

	// Update metrics for counting trie iteration
	if kind == snapStorage {
		snapStorageTrieReadCounter.Inc((time.Since(start) - internal).Nanoseconds())
	} else {
		snapAccountTrieReadCounter.Inc((time.Since(start) - internal).Nanoseconds())
	}
	logger.Debug("Regenerated state range", "root", trieId.Root, "last", hexutil.Encode(last),
		"count", count, "created", created, "updated", updated, "untouched", untouched, "deleted", deleted)

	// If there are either more trie items, or there are more snap items
	// (in the next segment), then we need to keep working
	return !trieMore && !result.diskMore, last, nil
}

// checkAndFlush checks if an interruption signal is received or the
// batch size has exceeded the allowance.
func (dl *diskLayer) checkAndFlush(ctx *generatorContext, current []byte) error {
	var abort chan *generatorStats
	select {
	case abort = <-dl.genAbort:
	default:
	}
	if ctx.batch.ValueSize() > ethdb.IdealBatchSize || abort != nil {
		if bytes.Compare(current, dl.genMarker) < 0 {
			log.Error("Snapshot generator went backwards", "current", fmt.Sprintf("%x", current), "genMarker", fmt.Sprintf("%x", dl.genMarker))
		}
		// Flush out the batch anyway no matter it's empty or not.
		// It's possible that all the states are recovered and the
		// generation indeed makes progress.
		journalProgress(ctx.batch, current, ctx.stats)

		if err := ctx.batch.Write(); err != nil {
			return err
		}
		ctx.batch.Reset()

		dl.lock.Lock()
		dl.genMarker = current
		dl.lock.Unlock()

		if abort != nil {
			ctx.stats.Log("Aborting state snapshot generation", dl.root, current)
			return newAbortErr(abort) // bubble up an error for interruption
		}
		// Don't hold the iterators too long, release them to let compactor works
		ctx.reopenIterator(snapAccount)
		ctx.reopenIterator(snapStorage)
	}
	if time.Since(ctx.logged) > 8*time.Second {
		ctx.stats.Log("Generating state snapshot", dl.root, current)
		ctx.logged = time.Now()
	}
	return nil
}

// generateStorages generates the missing storage slots of the specific contract.
// It's supposed to restart the generation from the given origin position.
func generateStorages(ctx *generatorContext, dl *diskLayer, stateRoot common.Hash, account common.Hash, storageRoot common.Hash, storeMarker []byte) error {
	onStorage := func(key []byte, val []byte, write bool, delete bool) error {
		defer func(start time.Time) {
			snapStorageWriteCounter.Inc(time.Since(start).Nanoseconds())
		}(time.Now())

		if delete {
			rawdb.DeleteStorageSnapshot(ctx.batch, account, common.BytesToHash(key))
			snapWipedStorageMeter.Mark(1)
			return nil
		}
		if write {
			rawdb.WriteStorageSnapshot(ctx.batch, account, common.BytesToHash(key), val)
			snapGeneratedStorageMeter.Mark(1)
		} else {
			snapRecoveredStorageMeter.Mark(1)
		}
		ctx.stats.storage += common.StorageSize(1 + 2*common.HashLength + len(val))
		ctx.stats.slots++

		// If we've exceeded our batch allowance or termination was requested, flush to disk
		if err := dl.checkAndFlush(ctx, append(account[:], key...)); err != nil {
			return err
		}
		return nil
	}
	// Loop for re-generating the missing storage slots.
	var origin = common.CopyBytes(storeMarker)
	for {
		id := trie.StorageTrieID(stateRoot, account, storageRoot)
		exhausted, last, err := dl.generateRange(ctx, id, append(rawdb.SnapshotStoragePrefix, account.Bytes()...), snapStorage, origin, storageCheckRange, onStorage, nil)
		if err != nil {
			return err // The procedure it aborted, either by external signal or internal error.
		}
		// Abort the procedure if the entire contract storage is generated
		if exhausted {
			break
		}
		if origin = increaseKey(last); origin == nil {
			break // special case, the last is 0xffffffff...fff
		}
	}
	return nil
}

// generateAccounts generates the missing snapshot accounts as well as their
// storage slots in the main trie. It's supposed to restart the generation
// from the given origin position.
func generateAccounts(ctx *generatorContext, dl *diskLayer, accMarker []byte) error {
	onAccount := func(key []byte, val []byte, write bool, delete bool) error {
		// Make sure to clear all dangling storages before this account
		account := common.BytesToHash(key)
		ctx.removeStorageBefore(account)

		start := time.Now()
		if delete {
			rawdb.DeleteAccountSnapshot(ctx.batch, account)
			snapWipedAccountMeter.Mark(1)
			snapAccountWriteCounter.Inc(time.Since(start).Nanoseconds())

			ctx.removeStorageAt(account)
			return nil
		}
		// Retrieve the current account and flatten it into the internal format
		var acc types.StateAccount
		if err := rlp.DecodeBytes(val, &acc); err != nil {
			log.Crit("Invalid account encountered during snapshot creation", "err", err)
		}
		// If the account is not yet in-progress, write it out
		if accMarker == nil || !bytes.Equal(account[:], accMarker) {
			dataLen := len(val) // Approximate size, saves us a round of RLP-encoding
			if !write {
				if bytes.Equal(acc.CodeHash, types.EmptyCodeHash[:]) {
					dataLen -= 32
				}
				if acc.Root == types.EmptyRootHash {
					dataLen -= 32
				}
				snapRecoveredAccountMeter.Mark(1)
			} else {
				data := types.SlimAccountRLP(acc)
				dataLen = len(data)
				rawdb.WriteAccountSnapshot(ctx.batch, account, data)
				snapGeneratedAccountMeter.Mark(1)
			}
			ctx.stats.storage += common.StorageSize(1 + common.HashLength + dataLen)
			ctx.stats.accounts++
		}
		// If the snap generation goes here after interrupted, genMarker may go backward
		// when last genMarker is consisted of accountHash and storageHash
		marker := account[:]
		if accMarker != nil && bytes.Equal(marker, accMarker) && len(dl.genMarker) > common.HashLength {
			marker = dl.genMarker[:]
		}
		// If we've exceeded our batch allowance or termination was requested, flush to disk
		if err := dl.checkAndFlush(ctx, marker); err != nil {
			return err
		}
		snapAccountWriteCounter.Inc(time.Since(start).Nanoseconds()) // let's count flush time as well

		// If the iterated account is the contract, create a further loop to
		// verify or regenerate the contract storage.
		if acc.Root == types.EmptyRootHash {
			ctx.removeStorageAt(account)
		} else {
			var storeMarker []byte
			if accMarker != nil && bytes.Equal(account[:], accMarker) && len(dl.genMarker) > common.HashLength {
				storeMarker = dl.genMarker[common.HashLength:]
			}
			if err := generateStorages(ctx, dl, dl.root, account, acc.Root, storeMarker); err != nil {
				return err
			}
		}
		// Some account processed, unmark the marker
		accMarker = nil
		return nil
	}
	// Always reset the initial account range as 1 whenever recover from the
	// interruption. TODO(rjl493456442) can we remove it?
	var accountRange = accountCheckRange
	if len(accMarker) > 0 {
		accountRange = 1
	}
	origin := common.CopyBytes(accMarker)
	for {
		id := trie.StateTrieID(dl.root)
		exhausted, last, err := dl.generateRange(ctx, id, rawdb.SnapshotAccountPrefix, snapAccount, origin, accountRange, onAccount, types.FullAccountRLP)
		if err != nil {
			return err // The procedure it aborted, either by external signal or internal error.
		}
		origin = increaseKey(last)

		// Last step, cleanup the storages after the last account.
		// All the left storages should be treated as dangling.
		if origin == nil || exhausted {
			ctx.removeStorageLeft()
			break
		}
		accountRange = accountCheckRange
	}
	return nil
}

// generate is a background thread that iterates over the state and storage tries,
// constructing the state snapshot. All the arguments are purely for statistics
// gathering and logging, since the method surfs the blocks as they arrive, often
// being restarted.
func (dl *diskLayer) generate(stats *generatorStats) {
	var (
		accMarker []byte
		abort     chan *generatorStats
	)
	if len(dl.genMarker) > 0 { // []byte{} is the start, use nil for that
		accMarker = dl.genMarker[:common.HashLength]
	}
	stats.Log("Resuming state snapshot generation", dl.root, dl.genMarker)

	// Initialize the global generator context. The snapshot iterators are
	// opened at the interrupted position because the assumption is held
	// that all the snapshot data are generated correctly before the marker.
	// Even if the snapshot data is updated during the interruption (before
	// or at the marker), the assumption is still held.
	// For the account or storage slot at the interruption, they will be
	// processed twice by the generator(they are already processed in the
	// last run) but it's fine.
	ctx := newGeneratorContext(stats, dl.diskdb, accMarker, dl.genMarker)
	defer ctx.close()

	if err := generateAccounts(ctx, dl, accMarker); err != nil {
		// Extract the received interruption signal if exists
		if aerr, ok := err.(*abortErr); ok {
			abort = aerr.abort
		}
		// Aborted by internal error, wait the signal
		if abort == nil {
			abort = <-dl.genAbort
		}
		abort <- stats
		return
	}
	// Snapshot fully generated, set the marker to nil.
	// Note even there is nothing to commit, persist the
	// generator anyway to mark the snapshot is complete.
	journalProgress(ctx.batch, nil, stats)
	if err := ctx.batch.Write(); err != nil {
		log.Error("Failed to flush batch", "err", err)

		abort = <-dl.genAbort
		abort <- stats
		return
	}
	ctx.batch.Reset()

	log.Info("Generated state snapshot", "accounts", stats.accounts, "slots", stats.slots,
		"storage", stats.storage, "dangling", stats.dangling, "elapsed", common.PrettyDuration(time.Since(stats.start)))

	dl.lock.Lock()
	dl.genMarker = nil
	close(dl.genPending)
	dl.lock.Unlock()

	// Someone will be looking for us, wait it out
	abort = <-dl.genAbort
	abort <- nil
}

// increaseKey increase the input key by one bit. Return nil if the entire
// addition operation overflows.
func increaseKey(key []byte) []byte {
	for i := len(key) - 1; i >= 0; i-- {
		key[i]++
		if key[i] != 0x0 {
			return key
		}
	}
	return nil
}

// abortErr wraps an interruption signal received to represent the
// generation is aborted by external processes.
type abortErr struct {
	abort chan *generatorStats
}

func newAbortErr(abort chan *generatorStats) error {
	return &abortErr{abort: abort}
}

func (err *abortErr) Error() string {
	return "aborted"
}
