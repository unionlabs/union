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

// Package snapshot implements a journalled, dynamic state dump.
package snapshot

import (
	"bytes"
	"errors"
	"fmt"
	"sync"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/metrics"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/ethereum/go-ethereum/trie"
)

var (
	snapshotCleanAccountHitMeter   = metrics.NewRegisteredMeter("state/snapshot/clean/account/hit", nil)
	snapshotCleanAccountMissMeter  = metrics.NewRegisteredMeter("state/snapshot/clean/account/miss", nil)
	snapshotCleanAccountInexMeter  = metrics.NewRegisteredMeter("state/snapshot/clean/account/inex", nil)
	snapshotCleanAccountReadMeter  = metrics.NewRegisteredMeter("state/snapshot/clean/account/read", nil)
	snapshotCleanAccountWriteMeter = metrics.NewRegisteredMeter("state/snapshot/clean/account/write", nil)

	snapshotCleanStorageHitMeter   = metrics.NewRegisteredMeter("state/snapshot/clean/storage/hit", nil)
	snapshotCleanStorageMissMeter  = metrics.NewRegisteredMeter("state/snapshot/clean/storage/miss", nil)
	snapshotCleanStorageInexMeter  = metrics.NewRegisteredMeter("state/snapshot/clean/storage/inex", nil)
	snapshotCleanStorageReadMeter  = metrics.NewRegisteredMeter("state/snapshot/clean/storage/read", nil)
	snapshotCleanStorageWriteMeter = metrics.NewRegisteredMeter("state/snapshot/clean/storage/write", nil)

	snapshotDirtyAccountHitMeter   = metrics.NewRegisteredMeter("state/snapshot/dirty/account/hit", nil)
	snapshotDirtyAccountMissMeter  = metrics.NewRegisteredMeter("state/snapshot/dirty/account/miss", nil)
	snapshotDirtyAccountInexMeter  = metrics.NewRegisteredMeter("state/snapshot/dirty/account/inex", nil)
	snapshotDirtyAccountReadMeter  = metrics.NewRegisteredMeter("state/snapshot/dirty/account/read", nil)
	snapshotDirtyAccountWriteMeter = metrics.NewRegisteredMeter("state/snapshot/dirty/account/write", nil)

	snapshotDirtyStorageHitMeter   = metrics.NewRegisteredMeter("state/snapshot/dirty/storage/hit", nil)
	snapshotDirtyStorageMissMeter  = metrics.NewRegisteredMeter("state/snapshot/dirty/storage/miss", nil)
	snapshotDirtyStorageInexMeter  = metrics.NewRegisteredMeter("state/snapshot/dirty/storage/inex", nil)
	snapshotDirtyStorageReadMeter  = metrics.NewRegisteredMeter("state/snapshot/dirty/storage/read", nil)
	snapshotDirtyStorageWriteMeter = metrics.NewRegisteredMeter("state/snapshot/dirty/storage/write", nil)

	snapshotDirtyAccountHitDepthHist = metrics.NewRegisteredHistogram("state/snapshot/dirty/account/hit/depth", nil, metrics.NewExpDecaySample(1028, 0.015))
	snapshotDirtyStorageHitDepthHist = metrics.NewRegisteredHistogram("state/snapshot/dirty/storage/hit/depth", nil, metrics.NewExpDecaySample(1028, 0.015))

	snapshotFlushAccountItemMeter = metrics.NewRegisteredMeter("state/snapshot/flush/account/item", nil)
	snapshotFlushAccountSizeMeter = metrics.NewRegisteredMeter("state/snapshot/flush/account/size", nil)
	snapshotFlushStorageItemMeter = metrics.NewRegisteredMeter("state/snapshot/flush/storage/item", nil)
	snapshotFlushStorageSizeMeter = metrics.NewRegisteredMeter("state/snapshot/flush/storage/size", nil)

	snapshotBloomIndexTimer = metrics.NewRegisteredResettingTimer("state/snapshot/bloom/index", nil)
	snapshotBloomErrorGauge = metrics.NewRegisteredGaugeFloat64("state/snapshot/bloom/error", nil)

	snapshotBloomAccountTrueHitMeter  = metrics.NewRegisteredMeter("state/snapshot/bloom/account/truehit", nil)
	snapshotBloomAccountFalseHitMeter = metrics.NewRegisteredMeter("state/snapshot/bloom/account/falsehit", nil)
	snapshotBloomAccountMissMeter     = metrics.NewRegisteredMeter("state/snapshot/bloom/account/miss", nil)

	snapshotBloomStorageTrueHitMeter  = metrics.NewRegisteredMeter("state/snapshot/bloom/storage/truehit", nil)
	snapshotBloomStorageFalseHitMeter = metrics.NewRegisteredMeter("state/snapshot/bloom/storage/falsehit", nil)
	snapshotBloomStorageMissMeter     = metrics.NewRegisteredMeter("state/snapshot/bloom/storage/miss", nil)

	// ErrSnapshotStale is returned from data accessors if the underlying snapshot
	// layer had been invalidated due to the chain progressing forward far enough
	// to not maintain the layer's original state.
	ErrSnapshotStale = errors.New("snapshot stale")

	// ErrNotCoveredYet is returned from data accessors if the underlying snapshot
	// is being generated currently and the requested data item is not yet in the
	// range of accounts covered.
	ErrNotCoveredYet = errors.New("not covered yet")

	// ErrNotConstructed is returned if the callers want to iterate the snapshot
	// while the generation is not finished yet.
	ErrNotConstructed = errors.New("snapshot is not constructed")

	// errSnapshotCycle is returned if a snapshot is attempted to be inserted
	// that forms a cycle in the snapshot tree.
	errSnapshotCycle = errors.New("snapshot cycle")
)

// Snapshot represents the functionality supported by a snapshot storage layer.
type Snapshot interface {
	// Root returns the root hash for which this snapshot was made.
	Root() common.Hash

	// Account directly retrieves the account associated with a particular hash in
	// the snapshot slim data format.
	Account(hash common.Hash) (*types.SlimAccount, error)

	// AccountRLP directly retrieves the account RLP associated with a particular
	// hash in the snapshot slim data format.
	AccountRLP(hash common.Hash) ([]byte, error)

	// Storage directly retrieves the storage data associated with a particular hash,
	// within a particular account.
	Storage(accountHash, storageHash common.Hash) ([]byte, error)
}

// snapshot is the internal version of the snapshot data layer that supports some
// additional methods compared to the public API.
type snapshot interface {
	Snapshot

	// Parent returns the subsequent layer of a snapshot, or nil if the base was
	// reached.
	//
	// Note, the method is an internal helper to avoid type switching between the
	// disk and diff layers. There is no locking involved.
	Parent() snapshot

	// Update creates a new layer on top of the existing snapshot diff tree with
	// the specified data items.
	//
	// Note, the maps are retained by the method to avoid copying everything.
	Update(blockRoot common.Hash, destructs map[common.Hash]struct{}, accounts map[common.Hash][]byte, storage map[common.Hash]map[common.Hash][]byte) *diffLayer

	// Journal commits an entire diff hierarchy to disk into a single journal entry.
	// This is meant to be used during shutdown to persist the snapshot without
	// flattening everything down (bad for reorgs).
	Journal(buffer *bytes.Buffer) (common.Hash, error)

	// Stale return whether this layer has become stale (was flattened across) or
	// if it's still live.
	Stale() bool

	// AccountIterator creates an account iterator over an arbitrary layer.
	AccountIterator(seek common.Hash) AccountIterator

	// StorageIterator creates a storage iterator over an arbitrary layer.
	StorageIterator(account common.Hash, seek common.Hash) (StorageIterator, bool)
}

// Config includes the configurations for snapshots.
type Config struct {
	CacheSize  int  // Megabytes permitted to use for read caches
	Recovery   bool // Indicator that the snapshots is in the recovery mode
	NoBuild    bool // Indicator that the snapshots generation is disallowed
	AsyncBuild bool // The snapshot generation is allowed to be constructed asynchronously
}

// Tree is an Ethereum state snapshot tree. It consists of one persistent base
// layer backed by a key-value store, on top of which arbitrarily many in-memory
// diff layers are topped. The memory diffs can form a tree with branching, but
// the disk layer is singleton and common to all. If a reorg goes deeper than the
// disk layer, everything needs to be deleted.
//
// The goal of a state snapshot is twofold: to allow direct access to account and
// storage data to avoid expensive multi-level trie lookups; and to allow sorted,
// cheap iteration of the account/storage tries for sync aid.
type Tree struct {
	config Config                   // Snapshots configurations
	diskdb ethdb.KeyValueStore      // Persistent database to store the snapshot
	triedb *trie.Database           // In-memory cache to access the trie through
	layers map[common.Hash]snapshot // Collection of all known layers
	lock   sync.RWMutex

	// Test hooks
	onFlatten func() // Hook invoked when the bottom most diff layers are flattened
}

// New attempts to load an already existing snapshot from a persistent key-value
// store (with a number of memory layers from a journal), ensuring that the head
// of the snapshot matches the expected one.
//
// If the snapshot is missing or the disk layer is broken, the snapshot will be
// reconstructed using both the existing data and the state trie.
// The repair happens on a background thread.
//
// If the memory layers in the journal do not match the disk layer (e.g. there is
// a gap) or the journal is missing, there are two repair cases:
//
//   - if the 'recovery' parameter is true, memory diff-layers and the disk-layer
//     will all be kept. This case happens when the snapshot is 'ahead' of the
//     state trie.
//   - otherwise, the entire snapshot is considered invalid and will be recreated on
//     a background thread.
func New(config Config, diskdb ethdb.KeyValueStore, triedb *trie.Database, root common.Hash) (*Tree, error) {
	// Create a new, empty snapshot tree
	snap := &Tree{
		config: config,
		diskdb: diskdb,
		triedb: triedb,
		layers: make(map[common.Hash]snapshot),
	}
	// Attempt to load a previously persisted snapshot and rebuild one if failed
	head, disabled, err := loadSnapshot(diskdb, triedb, root, config.CacheSize, config.Recovery, config.NoBuild)
	if disabled {
		log.Warn("Snapshot maintenance disabled (syncing)")
		return snap, nil
	}
	// Create the building waiter iff the background generation is allowed
	if !config.NoBuild && !config.AsyncBuild {
		defer snap.waitBuild()
	}
	if err != nil {
		log.Warn("Failed to load snapshot", "err", err)
		if !config.NoBuild {
			snap.Rebuild(root)
			return snap, nil
		}
		return nil, err // Bail out the error, don't rebuild automatically.
	}
	// Existing snapshot loaded, seed all the layers
	for head != nil {
		snap.layers[head.Root()] = head
		head = head.Parent()
	}
	return snap, nil
}

// waitBuild blocks until the snapshot finishes rebuilding. This method is meant
// to be used by tests to ensure we're testing what we believe we are.
func (t *Tree) waitBuild() {
	// Find the rebuild termination channel
	var done chan struct{}

	t.lock.RLock()
	for _, layer := range t.layers {
		if layer, ok := layer.(*diskLayer); ok {
			done = layer.genPending
			break
		}
	}
	t.lock.RUnlock()

	// Wait until the snapshot is generated
	if done != nil {
		<-done
	}
}

// Disable interrupts any pending snapshot generator, deletes all the snapshot
// layers in memory and marks snapshots disabled globally. In order to resume
// the snapshot functionality, the caller must invoke Rebuild.
func (t *Tree) Disable() {
	// Interrupt any live snapshot layers
	t.lock.Lock()
	defer t.lock.Unlock()

	for _, layer := range t.layers {
		switch layer := layer.(type) {
		case *diskLayer:
			// If the base layer is generating, abort it
			if layer.genAbort != nil {
				abort := make(chan *generatorStats)
				layer.genAbort <- abort
				<-abort
			}
			// Layer should be inactive now, mark it as stale
			layer.lock.Lock()
			layer.stale = true
			layer.lock.Unlock()

		case *diffLayer:
			// If the layer is a simple diff, simply mark as stale
			layer.lock.Lock()
			layer.stale.Store(true)
			layer.lock.Unlock()

		default:
			panic(fmt.Sprintf("unknown layer type: %T", layer))
		}
	}
	t.layers = map[common.Hash]snapshot{}

	// Delete all snapshot liveness information from the database
	batch := t.diskdb.NewBatch()

	rawdb.WriteSnapshotDisabled(batch)
	rawdb.DeleteSnapshotRoot(batch)
	rawdb.DeleteSnapshotJournal(batch)
	rawdb.DeleteSnapshotGenerator(batch)
	rawdb.DeleteSnapshotRecoveryNumber(batch)
	// Note, we don't delete the sync progress

	if err := batch.Write(); err != nil {
		log.Crit("Failed to disable snapshots", "err", err)
	}
}

// Snapshot retrieves a snapshot belonging to the given block root, or nil if no
// snapshot is maintained for that block.
func (t *Tree) Snapshot(blockRoot common.Hash) Snapshot {
	t.lock.RLock()
	defer t.lock.RUnlock()

	return t.layers[blockRoot]
}

// Snapshots returns all visited layers from the topmost layer with specific
// root and traverses downward. The layer amount is limited by the given number.
// If nodisk is set, then disk layer is excluded.
func (t *Tree) Snapshots(root common.Hash, limits int, nodisk bool) []Snapshot {
	t.lock.RLock()
	defer t.lock.RUnlock()

	if limits == 0 {
		return nil
	}
	layer := t.layers[root]
	if layer == nil {
		return nil
	}
	var ret []Snapshot
	for {
		if _, isdisk := layer.(*diskLayer); isdisk && nodisk {
			break
		}
		ret = append(ret, layer)
		limits -= 1
		if limits == 0 {
			break
		}
		parent := layer.Parent()
		if parent == nil {
			break
		}
		layer = parent
	}
	return ret
}

// Update adds a new snapshot into the tree, if that can be linked to an existing
// old parent. It is disallowed to insert a disk layer (the origin of all).
func (t *Tree) Update(blockRoot common.Hash, parentRoot common.Hash, destructs map[common.Hash]struct{}, accounts map[common.Hash][]byte, storage map[common.Hash]map[common.Hash][]byte) error {
	// Reject noop updates to avoid self-loops in the snapshot tree. This is a
	// special case that can only happen for Clique networks where empty blocks
	// don't modify the state (0 block subsidy).
	//
	// Although we could silently ignore this internally, it should be the caller's
	// responsibility to avoid even attempting to insert such a snapshot.
	if blockRoot == parentRoot {
		return errSnapshotCycle
	}
	// Generate a new snapshot on top of the parent
	parent := t.Snapshot(parentRoot)
	if parent == nil {
		return fmt.Errorf("parent [%#x] snapshot missing", parentRoot)
	}
	snap := parent.(snapshot).Update(blockRoot, destructs, accounts, storage)

	// Save the new snapshot for later
	t.lock.Lock()
	defer t.lock.Unlock()

	t.layers[snap.root] = snap
	return nil
}

// Cap traverses downwards the snapshot tree from a head block hash until the
// number of allowed layers are crossed. All layers beyond the permitted number
// are flattened downwards.
//
// Note, the final diff layer count in general will be one more than the amount
// requested. This happens because the bottom-most diff layer is the accumulator
// which may or may not overflow and cascade to disk. Since this last layer's
// survival is only known *after* capping, we need to omit it from the count if
// we want to ensure that *at least* the requested number of diff layers remain.
func (t *Tree) Cap(root common.Hash, layers int) error {
	// Retrieve the head snapshot to cap from
	snap := t.Snapshot(root)
	if snap == nil {
		return fmt.Errorf("snapshot [%#x] missing", root)
	}
	diff, ok := snap.(*diffLayer)
	if !ok {
		return fmt.Errorf("snapshot [%#x] is disk layer", root)
	}
	// If the generator is still running, use a more aggressive cap
	diff.origin.lock.RLock()
	if diff.origin.genMarker != nil && layers > 8 {
		layers = 8
	}
	diff.origin.lock.RUnlock()

	// Run the internal capping and discard all stale layers
	t.lock.Lock()
	defer t.lock.Unlock()

	// Flattening the bottom-most diff layer requires special casing since there's
	// no child to rewire to the grandparent. In that case we can fake a temporary
	// child for the capping and then remove it.
	if layers == 0 {
		// If full commit was requested, flatten the diffs and merge onto disk
		diff.lock.RLock()
		base := diffToDisk(diff.flatten().(*diffLayer))
		diff.lock.RUnlock()

		// Replace the entire snapshot tree with the flat base
		t.layers = map[common.Hash]snapshot{base.root: base}
		return nil
	}
	persisted := t.cap(diff, layers)

	// Remove any layer that is stale or links into a stale layer
	children := make(map[common.Hash][]common.Hash)
	for root, snap := range t.layers {
		if diff, ok := snap.(*diffLayer); ok {
			parent := diff.parent.Root()
			children[parent] = append(children[parent], root)
		}
	}
	var remove func(root common.Hash)
	remove = func(root common.Hash) {
		delete(t.layers, root)
		for _, child := range children[root] {
			remove(child)
		}
		delete(children, root)
	}
	for root, snap := range t.layers {
		if snap.Stale() {
			remove(root)
		}
	}
	// If the disk layer was modified, regenerate all the cumulative blooms
	if persisted != nil {
		var rebloom func(root common.Hash)
		rebloom = func(root common.Hash) {
			if diff, ok := t.layers[root].(*diffLayer); ok {
				diff.rebloom(persisted)
			}
			for _, child := range children[root] {
				rebloom(child)
			}
		}
		rebloom(persisted.root)
	}
	return nil
}

// cap traverses downwards the diff tree until the number of allowed layers are
// crossed. All diffs beyond the permitted number are flattened downwards. If the
// layer limit is reached, memory cap is also enforced (but not before).
//
// The method returns the new disk layer if diffs were persisted into it.
//
// Note, the final diff layer count in general will be one more than the amount
// requested. This happens because the bottom-most diff layer is the accumulator
// which may or may not overflow and cascade to disk. Since this last layer's
// survival is only known *after* capping, we need to omit it from the count if
// we want to ensure that *at least* the requested number of diff layers remain.
func (t *Tree) cap(diff *diffLayer, layers int) *diskLayer {
	// Dive until we run out of layers or reach the persistent database
	for i := 0; i < layers-1; i++ {
		// If we still have diff layers below, continue down
		if parent, ok := diff.parent.(*diffLayer); ok {
			diff = parent
		} else {
			// Diff stack too shallow, return without modifications
			return nil
		}
	}
	// We're out of layers, flatten anything below, stopping if it's the disk or if
	// the memory limit is not yet exceeded.
	switch parent := diff.parent.(type) {
	case *diskLayer:
		return nil

	case *diffLayer:
		// Hold the write lock until the flattened parent is linked correctly.
		// Otherwise, the stale layer may be accessed by external reads in the
		// meantime.
		diff.lock.Lock()
		defer diff.lock.Unlock()

		// Flatten the parent into the grandparent. The flattening internally obtains a
		// write lock on grandparent.
		flattened := parent.flatten().(*diffLayer)
		t.layers[flattened.root] = flattened

		// Invoke the hook if it's registered. Ugly hack.
		if t.onFlatten != nil {
			t.onFlatten()
		}
		diff.parent = flattened
		if flattened.memory < aggregatorMemoryLimit {
			// Accumulator layer is smaller than the limit, so we can abort, unless
			// there's a snapshot being generated currently. In that case, the trie
			// will move from underneath the generator so we **must** merge all the
			// partial data down into the snapshot and restart the generation.
			if flattened.parent.(*diskLayer).genAbort == nil {
				return nil
			}
		}
	default:
		panic(fmt.Sprintf("unknown data layer: %T", parent))
	}
	// If the bottom-most layer is larger than our memory cap, persist to disk
	bottom := diff.parent.(*diffLayer)

	bottom.lock.RLock()
	base := diffToDisk(bottom)
	bottom.lock.RUnlock()

	t.layers[base.root] = base
	diff.parent = base
	return base
}

// diffToDisk merges a bottom-most diff into the persistent disk layer underneath
// it. The method will panic if called onto a non-bottom-most diff layer.
//
// The disk layer persistence should be operated in an atomic way. All updates should
// be discarded if the whole transition if not finished.
func diffToDisk(bottom *diffLayer) *diskLayer {
	var (
		base  = bottom.parent.(*diskLayer)
		batch = base.diskdb.NewBatch()
		stats *generatorStats
	)
	// If the disk layer is running a snapshot generator, abort it
	if base.genAbort != nil {
		abort := make(chan *generatorStats)
		base.genAbort <- abort
		stats = <-abort
	}
	// Put the deletion in the batch writer, flush all updates in the final step.
	rawdb.DeleteSnapshotRoot(batch)

	// Mark the original base as stale as we're going to create a new wrapper
	base.lock.Lock()
	if base.stale {
		panic("parent disk layer is stale") // we've committed into the same base from two children, boo
	}
	base.stale = true
	base.lock.Unlock()

	// Destroy all the destructed accounts from the database
	for hash := range bottom.destructSet {
		// Skip any account not covered yet by the snapshot
		if base.genMarker != nil && bytes.Compare(hash[:], base.genMarker) > 0 {
			continue
		}
		// Remove all storage slots
		rawdb.DeleteAccountSnapshot(batch, hash)
		base.cache.Set(hash[:], nil)

		it := rawdb.IterateStorageSnapshots(base.diskdb, hash)
		for it.Next() {
			key := it.Key()
			batch.Delete(key)
			base.cache.Del(key[1:])
			snapshotFlushStorageItemMeter.Mark(1)

			// Ensure we don't delete too much data blindly (contract can be
			// huge). It's ok to flush, the root will go missing in case of a
			// crash and we'll detect and regenerate the snapshot.
			if batch.ValueSize() > 64*1024*1024 {
				if err := batch.Write(); err != nil {
					log.Crit("Failed to write storage deletions", "err", err)
				}
				batch.Reset()
			}
		}
		it.Release()
	}
	// Push all updated accounts into the database
	for hash, data := range bottom.accountData {
		// Skip any account not covered yet by the snapshot
		if base.genMarker != nil && bytes.Compare(hash[:], base.genMarker) > 0 {
			continue
		}
		// Push the account to disk
		rawdb.WriteAccountSnapshot(batch, hash, data)
		base.cache.Set(hash[:], data)
		snapshotCleanAccountWriteMeter.Mark(int64(len(data)))

		snapshotFlushAccountItemMeter.Mark(1)
		snapshotFlushAccountSizeMeter.Mark(int64(len(data)))

		// Ensure we don't write too much data blindly. It's ok to flush, the
		// root will go missing in case of a crash and we'll detect and regen
		// the snapshot.
		if batch.ValueSize() > 64*1024*1024 {
			if err := batch.Write(); err != nil {
				log.Crit("Failed to write storage deletions", "err", err)
			}
			batch.Reset()
		}
	}
	// Push all the storage slots into the database
	for accountHash, storage := range bottom.storageData {
		// Skip any account not covered yet by the snapshot
		if base.genMarker != nil && bytes.Compare(accountHash[:], base.genMarker) > 0 {
			continue
		}
		// Generation might be mid-account, track that case too
		midAccount := base.genMarker != nil && bytes.Equal(accountHash[:], base.genMarker[:common.HashLength])

		for storageHash, data := range storage {
			// Skip any slot not covered yet by the snapshot
			if midAccount && bytes.Compare(storageHash[:], base.genMarker[common.HashLength:]) > 0 {
				continue
			}
			if len(data) > 0 {
				rawdb.WriteStorageSnapshot(batch, accountHash, storageHash, data)
				base.cache.Set(append(accountHash[:], storageHash[:]...), data)
				snapshotCleanStorageWriteMeter.Mark(int64(len(data)))
			} else {
				rawdb.DeleteStorageSnapshot(batch, accountHash, storageHash)
				base.cache.Set(append(accountHash[:], storageHash[:]...), nil)
			}
			snapshotFlushStorageItemMeter.Mark(1)
			snapshotFlushStorageSizeMeter.Mark(int64(len(data)))
		}
	}
	// Update the snapshot block marker and write any remainder data
	rawdb.WriteSnapshotRoot(batch, bottom.root)

	// Write out the generator progress marker and report
	journalProgress(batch, base.genMarker, stats)

	// Flush all the updates in the single db operation. Ensure the
	// disk layer transition is atomic.
	if err := batch.Write(); err != nil {
		log.Crit("Failed to write leftover snapshot", "err", err)
	}
	log.Debug("Journalled disk layer", "root", bottom.root, "complete", base.genMarker == nil)
	res := &diskLayer{
		root:       bottom.root,
		cache:      base.cache,
		diskdb:     base.diskdb,
		triedb:     base.triedb,
		genMarker:  base.genMarker,
		genPending: base.genPending,
	}
	// If snapshot generation hasn't finished yet, port over all the starts and
	// continue where the previous round left off.
	//
	// Note, the `base.genAbort` comparison is not used normally, it's checked
	// to allow the tests to play with the marker without triggering this path.
	if base.genMarker != nil && base.genAbort != nil {
		res.genMarker = base.genMarker
		res.genAbort = make(chan chan *generatorStats)
		go res.generate(stats)
	}
	return res
}

// Release releases resources
func (t *Tree) Release() {
	if dl := t.disklayer(); dl != nil {
		dl.Release()
	}
}

// Journal commits an entire diff hierarchy to disk into a single journal entry.
// This is meant to be used during shutdown to persist the snapshot without
// flattening everything down (bad for reorgs).
//
// The method returns the root hash of the base layer that needs to be persisted
// to disk as a trie too to allow continuing any pending generation op.
func (t *Tree) Journal(root common.Hash) (common.Hash, error) {
	// Retrieve the head snapshot to journal from var snap snapshot
	snap := t.Snapshot(root)
	if snap == nil {
		return common.Hash{}, fmt.Errorf("snapshot [%#x] missing", root)
	}
	// Run the journaling
	t.lock.Lock()
	defer t.lock.Unlock()

	// Firstly write out the metadata of journal
	journal := new(bytes.Buffer)
	if err := rlp.Encode(journal, journalVersion); err != nil {
		return common.Hash{}, err
	}
	diskroot := t.diskRoot()
	if diskroot == (common.Hash{}) {
		return common.Hash{}, errors.New("invalid disk root")
	}
	// Secondly write out the disk layer root, ensure the
	// diff journal is continuous with disk.
	if err := rlp.Encode(journal, diskroot); err != nil {
		return common.Hash{}, err
	}
	// Finally write out the journal of each layer in reverse order.
	base, err := snap.(snapshot).Journal(journal)
	if err != nil {
		return common.Hash{}, err
	}
	// Store the journal into the database and return
	rawdb.WriteSnapshotJournal(t.diskdb, journal.Bytes())
	return base, nil
}

// Rebuild wipes all available snapshot data from the persistent database and
// discard all caches and diff layers. Afterwards, it starts a new snapshot
// generator with the given root hash.
func (t *Tree) Rebuild(root common.Hash) {
	t.lock.Lock()
	defer t.lock.Unlock()

	// Firstly delete any recovery flag in the database. Because now we are
	// building a brand new snapshot. Also reenable the snapshot feature.
	rawdb.DeleteSnapshotRecoveryNumber(t.diskdb)
	rawdb.DeleteSnapshotDisabled(t.diskdb)

	// Iterate over and mark all layers stale
	for _, layer := range t.layers {
		switch layer := layer.(type) {
		case *diskLayer:
			// If the base layer is generating, abort it and save
			if layer.genAbort != nil {
				abort := make(chan *generatorStats)
				layer.genAbort <- abort
				<-abort
			}
			// Layer should be inactive now, mark it as stale
			layer.lock.Lock()
			layer.stale = true
			layer.lock.Unlock()

		case *diffLayer:
			// If the layer is a simple diff, simply mark as stale
			layer.lock.Lock()
			layer.stale.Store(true)
			layer.lock.Unlock()

		default:
			panic(fmt.Sprintf("unknown layer type: %T", layer))
		}
	}
	// Start generating a new snapshot from scratch on a background thread. The
	// generator will run a wiper first if there's not one running right now.
	log.Info("Rebuilding state snapshot")
	t.layers = map[common.Hash]snapshot{
		root: generateSnapshot(t.diskdb, t.triedb, t.config.CacheSize, root),
	}
}

// AccountIterator creates a new account iterator for the specified root hash and
// seeks to a starting account hash.
func (t *Tree) AccountIterator(root common.Hash, seek common.Hash) (AccountIterator, error) {
	ok, err := t.generating()
	if err != nil {
		return nil, err
	}
	if ok {
		return nil, ErrNotConstructed
	}
	return newFastAccountIterator(t, root, seek)
}

// StorageIterator creates a new storage iterator for the specified root hash and
// account. The iterator will be move to the specific start position.
func (t *Tree) StorageIterator(root common.Hash, account common.Hash, seek common.Hash) (StorageIterator, error) {
	ok, err := t.generating()
	if err != nil {
		return nil, err
	}
	if ok {
		return nil, ErrNotConstructed
	}
	return newFastStorageIterator(t, root, account, seek)
}

// Verify iterates the whole state(all the accounts as well as the corresponding storages)
// with the specific root and compares the re-computed hash with the original one.
func (t *Tree) Verify(root common.Hash) error {
	acctIt, err := t.AccountIterator(root, common.Hash{})
	if err != nil {
		return err
	}
	defer acctIt.Release()

	got, err := generateTrieRoot(nil, "", acctIt, common.Hash{}, stackTrieGenerate, func(db ethdb.KeyValueWriter, accountHash, codeHash common.Hash, stat *generateStats) (common.Hash, error) {
		storageIt, err := t.StorageIterator(root, accountHash, common.Hash{})
		if err != nil {
			return common.Hash{}, err
		}
		defer storageIt.Release()

		hash, err := generateTrieRoot(nil, "", storageIt, accountHash, stackTrieGenerate, nil, stat, false)
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

// disklayer is an internal helper function to return the disk layer.
// The lock of snapTree is assumed to be held already.
func (t *Tree) disklayer() *diskLayer {
	var snap snapshot
	for _, s := range t.layers {
		snap = s
		break
	}
	if snap == nil {
		return nil
	}
	switch layer := snap.(type) {
	case *diskLayer:
		return layer
	case *diffLayer:
		return layer.origin
	default:
		panic(fmt.Sprintf("%T: undefined layer", snap))
	}
}

// diskRoot is a internal helper function to return the disk layer root.
// The lock of snapTree is assumed to be held already.
func (t *Tree) diskRoot() common.Hash {
	disklayer := t.disklayer()
	if disklayer == nil {
		return common.Hash{}
	}
	return disklayer.Root()
}

// generating is an internal helper function which reports whether the snapshot
// is still under the construction.
func (t *Tree) generating() (bool, error) {
	t.lock.Lock()
	defer t.lock.Unlock()

	layer := t.disklayer()
	if layer == nil {
		return false, errors.New("disk layer is missing")
	}
	layer.lock.RLock()
	defer layer.lock.RUnlock()
	return layer.genMarker != nil, nil
}

// DiskRoot is a external helper function to return the disk layer root.
func (t *Tree) DiskRoot() common.Hash {
	t.lock.Lock()
	defer t.lock.Unlock()

	return t.diskRoot()
}

// Size returns the memory usage of the diff layers above the disk layer and the
// dirty nodes buffered in the disk layer. Currently, the implementation uses a
// special diff layer (the first) as an aggregator simulating a dirty buffer, so
// the second return will always be 0. However, this will be made consistent with
// the pathdb, which will require a second return.
func (t *Tree) Size() (diffs common.StorageSize, buf common.StorageSize) {
	t.lock.RLock()
	defer t.lock.RUnlock()

	var size common.StorageSize
	for _, layer := range t.layers {
		if layer, ok := layer.(*diffLayer); ok {
			size += common.StorageSize(layer.memory)
		}
	}
	return size, 0
}
