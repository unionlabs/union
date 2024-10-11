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

package pruner

import (
	"bytes"
	"encoding/binary"
	"errors"
	"fmt"
	"math"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/state/snapshot"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/ethereum/go-ethereum/trie"
)

const (
	// stateBloomFilePrefix is the filename prefix of state bloom filter.
	stateBloomFilePrefix = "statebloom"

	// stateBloomFilePrefix is the filename suffix of state bloom filter.
	stateBloomFileSuffix = "bf.gz"

	// stateBloomFileTempSuffix is the filename suffix of state bloom filter
	// while it is being written out to detect write aborts.
	stateBloomFileTempSuffix = ".tmp"

	// rangeCompactionThreshold is the minimal deleted entry number for
	// triggering range compaction. It's a quite arbitrary number but just
	// to avoid triggering range compaction because of small deletion.
	rangeCompactionThreshold = 100000
)

// Config includes all the configurations for pruning.
type Config struct {
	Datadir   string // The directory of the state database
	BloomSize uint64 // The Megabytes of memory allocated to bloom-filter
}

// Pruner is an offline tool to prune the stale state with the
// help of the snapshot. The workflow of pruner is very simple:
//
//   - iterate the snapshot, reconstruct the relevant state
//   - iterate the database, delete all other state entries which
//     don't belong to the target state and the genesis state
//
// It can take several hours(around 2 hours for mainnet) to finish
// the whole pruning work. It's recommended to run this offline tool
// periodically in order to release the disk usage and improve the
// disk read performance to some extent.
type Pruner struct {
	config      Config
	chainHeader *types.Header
	db          ethdb.Database
	stateBloom  *stateBloom
	snaptree    *snapshot.Tree
}

// NewPruner creates the pruner instance.
func NewPruner(db ethdb.Database, config Config) (*Pruner, error) {
	headBlock := rawdb.ReadHeadBlock(db)
	if headBlock == nil {
		return nil, errors.New("failed to load head block")
	}
	// Offline pruning is only supported in legacy hash based scheme.
	triedb := trie.NewDatabase(db, trie.HashDefaults)

	snapconfig := snapshot.Config{
		CacheSize:  256,
		Recovery:   false,
		NoBuild:    true,
		AsyncBuild: false,
	}
	snaptree, err := snapshot.New(snapconfig, db, triedb, headBlock.Root())
	if err != nil {
		return nil, err // The relevant snapshot(s) might not exist
	}
	// Sanitize the bloom filter size if it's too small.
	if config.BloomSize < 256 {
		log.Warn("Sanitizing bloomfilter size", "provided(MB)", config.BloomSize, "updated(MB)", 256)
		config.BloomSize = 256
	}
	stateBloom, err := newStateBloomWithSize(config.BloomSize)
	if err != nil {
		return nil, err
	}
	return &Pruner{
		config:      config,
		chainHeader: headBlock.Header(),
		db:          db,
		stateBloom:  stateBloom,
		snaptree:    snaptree,
	}, nil
}

func prune(snaptree *snapshot.Tree, root common.Hash, maindb ethdb.Database, stateBloom *stateBloom, bloomPath string, middleStateRoots map[common.Hash]struct{}, start time.Time) error {
	// Delete all stale trie nodes in the disk. With the help of state bloom
	// the trie nodes(and codes) belong to the active state will be filtered
	// out. A very small part of stale tries will also be filtered because of
	// the false-positive rate of bloom filter. But the assumption is held here
	// that the false-positive is low enough(~0.05%). The probablity of the
	// dangling node is the state root is super low. So the dangling nodes in
	// theory will never ever be visited again.
	var (
		skipped, count int
		size           common.StorageSize
		pstart         = time.Now()
		logged         = time.Now()
		batch          = maindb.NewBatch()
		iter           = maindb.NewIterator(nil, nil)
	)
	for iter.Next() {
		key := iter.Key()

		// All state entries don't belong to specific state and genesis are deleted here
		// - trie node
		// - legacy contract code
		// - new-scheme contract code
		isCode, codeKey := rawdb.IsCodeKey(key)
		if len(key) == common.HashLength || isCode {
			checkKey := key
			if isCode {
				checkKey = codeKey
			}
			if _, exist := middleStateRoots[common.BytesToHash(checkKey)]; exist {
				log.Debug("Forcibly delete the middle state roots", "hash", common.BytesToHash(checkKey))
			} else {
				if stateBloom.Contain(checkKey) {
					skipped += 1
					continue
				}
			}
			count += 1
			size += common.StorageSize(len(key) + len(iter.Value()))
			batch.Delete(key)

			var eta time.Duration // Realistically will never remain uninited
			if done := binary.BigEndian.Uint64(key[:8]); done > 0 {
				var (
					left  = math.MaxUint64 - binary.BigEndian.Uint64(key[:8])
					speed = done/uint64(time.Since(pstart)/time.Millisecond+1) + 1 // +1s to avoid division by zero
				)
				eta = time.Duration(left/speed) * time.Millisecond
			}
			if time.Since(logged) > 8*time.Second {
				log.Info("Pruning state data", "nodes", count, "skipped", skipped, "size", size,
					"elapsed", common.PrettyDuration(time.Since(pstart)), "eta", common.PrettyDuration(eta))
				logged = time.Now()
			}
			// Recreate the iterator after every batch commit in order
			// to allow the underlying compactor to delete the entries.
			if batch.ValueSize() >= ethdb.IdealBatchSize {
				batch.Write()
				batch.Reset()

				iter.Release()
				iter = maindb.NewIterator(nil, key)
			}
		}
	}
	if batch.ValueSize() > 0 {
		batch.Write()
		batch.Reset()
	}
	iter.Release()
	log.Info("Pruned state data", "nodes", count, "size", size, "elapsed", common.PrettyDuration(time.Since(pstart)))

	// Pruning is done, now drop the "useless" layers from the snapshot.
	// Firstly, flushing the target layer into the disk. After that all
	// diff layers below the target will all be merged into the disk.
	if err := snaptree.Cap(root, 0); err != nil {
		return err
	}
	// Secondly, flushing the snapshot journal into the disk. All diff
	// layers upon are dropped silently. Eventually the entire snapshot
	// tree is converted into a single disk layer with the pruning target
	// as the root.
	if _, err := snaptree.Journal(root); err != nil {
		return err
	}
	// Delete the state bloom, it marks the entire pruning procedure is
	// finished. If any crashes or manual exit happens before this,
	// `RecoverPruning` will pick it up in the next restarts to redo all
	// the things.
	os.RemoveAll(bloomPath)

	// Start compactions, will remove the deleted data from the disk immediately.
	// Note for small pruning, the compaction is skipped.
	if count >= rangeCompactionThreshold {
		cstart := time.Now()
		for b := 0x00; b <= 0xf0; b += 0x10 {
			var (
				start = []byte{byte(b)}
				end   = []byte{byte(b + 0x10)}
			)
			if b == 0xf0 {
				end = nil
			}
			log.Info("Compacting database", "range", fmt.Sprintf("%#x-%#x", start, end), "elapsed", common.PrettyDuration(time.Since(cstart)))
			if err := maindb.Compact(start, end); err != nil {
				log.Error("Database compaction failed", "error", err)
				return err
			}
		}
		log.Info("Database compaction finished", "elapsed", common.PrettyDuration(time.Since(cstart)))
	}
	log.Info("State pruning successful", "pruned", size, "elapsed", common.PrettyDuration(time.Since(start)))
	return nil
}

// Prune deletes all historical state nodes except the nodes belong to the
// specified state version. If user doesn't specify the state version, use
// the bottom-most snapshot diff layer as the target.
func (p *Pruner) Prune(root common.Hash) error {
	// If the state bloom filter is already committed previously,
	// reuse it for pruning instead of generating a new one. It's
	// mandatory because a part of state may already be deleted,
	// the recovery procedure is necessary.
	_, stateBloomRoot, err := findBloomFilter(p.config.Datadir)
	if err != nil {
		return err
	}
	if stateBloomRoot != (common.Hash{}) {
		return RecoverPruning(p.config.Datadir, p.db)
	}
	// If the target state root is not specified, use the HEAD-127 as the
	// target. The reason for picking it is:
	// - in most of the normal cases, the related state is available
	// - the probability of this layer being reorg is very low
	var layers []snapshot.Snapshot
	if root == (common.Hash{}) {
		// Retrieve all snapshot layers from the current HEAD.
		// In theory there are 128 difflayers + 1 disk layer present,
		// so 128 diff layers are expected to be returned.
		layers = p.snaptree.Snapshots(p.chainHeader.Root, 128, true)
		if len(layers) != 128 {
			// Reject if the accumulated diff layers are less than 128. It
			// means in most of normal cases, there is no associated state
			// with bottom-most diff layer.
			return fmt.Errorf("snapshot not old enough yet: need %d more blocks", 128-len(layers))
		}
		// Use the bottom-most diff layer as the target
		root = layers[len(layers)-1].Root()
	}
	// Ensure the root is really present. The weak assumption
	// is the presence of root can indicate the presence of the
	// entire trie.
	if !rawdb.HasLegacyTrieNode(p.db, root) {
		// The special case is for clique based networks(goerli
		// and some other private networks), it's possible that two
		// consecutive blocks will have same root. In this case snapshot
		// difflayer won't be created. So HEAD-127 may not paired with
		// head-127 layer. Instead the paired layer is higher than the
		// bottom-most diff layer. Try to find the bottom-most snapshot
		// layer with state available.
		//
		// Note HEAD and HEAD-1 is ignored. Usually there is the associated
		// state available, but we don't want to use the topmost state
		// as the pruning target.
		var found bool
		for i := len(layers) - 2; i >= 2; i-- {
			if rawdb.HasLegacyTrieNode(p.db, layers[i].Root()) {
				root = layers[i].Root()
				found = true
				log.Info("Selecting middle-layer as the pruning target", "root", root, "depth", i)
				break
			}
		}
		if !found {
			if len(layers) > 0 {
				return errors.New("no snapshot paired state")
			}
			return fmt.Errorf("associated state[%x] is not present", root)
		}
	} else {
		if len(layers) > 0 {
			log.Info("Selecting bottom-most difflayer as the pruning target", "root", root, "height", p.chainHeader.Number.Uint64()-127)
		} else {
			log.Info("Selecting user-specified state as the pruning target", "root", root)
		}
	}
	// All the state roots of the middle layer should be forcibly pruned,
	// otherwise the dangling state will be left.
	middleRoots := make(map[common.Hash]struct{})
	for _, layer := range layers {
		if layer.Root() == root {
			break
		}
		middleRoots[layer.Root()] = struct{}{}
	}
	// Traverse the target state, re-construct the whole state trie and
	// commit to the given bloom filter.
	start := time.Now()
	if err := snapshot.GenerateTrie(p.snaptree, root, p.db, p.stateBloom); err != nil {
		return err
	}
	// Traverse the genesis, put all genesis state entries into the
	// bloom filter too.
	if err := extractGenesis(p.db, p.stateBloom); err != nil {
		return err
	}
	filterName := bloomFilterName(p.config.Datadir, root)

	log.Info("Writing state bloom to disk", "name", filterName)
	if err := p.stateBloom.Commit(filterName, filterName+stateBloomFileTempSuffix); err != nil {
		return err
	}
	log.Info("State bloom filter committed", "name", filterName)
	return prune(p.snaptree, root, p.db, p.stateBloom, filterName, middleRoots, start)
}

// RecoverPruning will resume the pruning procedure during the system restart.
// This function is used in this case: user tries to prune state data, but the
// system was interrupted midway because of crash or manual-kill. In this case
// if the bloom filter for filtering active state is already constructed, the
// pruning can be resumed. What's more if the bloom filter is constructed, the
// pruning **has to be resumed**. Otherwise a lot of dangling nodes may be left
// in the disk.
func RecoverPruning(datadir string, db ethdb.Database) error {
	stateBloomPath, stateBloomRoot, err := findBloomFilter(datadir)
	if err != nil {
		return err
	}
	if stateBloomPath == "" {
		return nil // nothing to recover
	}
	headBlock := rawdb.ReadHeadBlock(db)
	if headBlock == nil {
		return errors.New("failed to load head block")
	}
	// Initialize the snapshot tree in recovery mode to handle this special case:
	// - Users run the `prune-state` command multiple times
	// - Neither these `prune-state` running is finished(e.g. interrupted manually)
	// - The state bloom filter is already generated, a part of state is deleted,
	//   so that resuming the pruning here is mandatory
	// - The state HEAD is rewound already because of multiple incomplete `prune-state`
	// In this case, even the state HEAD is not exactly matched with snapshot, it
	// still feasible to recover the pruning correctly.
	snapconfig := snapshot.Config{
		CacheSize:  256,
		Recovery:   true,
		NoBuild:    true,
		AsyncBuild: false,
	}
	// Offline pruning is only supported in legacy hash based scheme.
	triedb := trie.NewDatabase(db, trie.HashDefaults)
	snaptree, err := snapshot.New(snapconfig, db, triedb, headBlock.Root())
	if err != nil {
		return err // The relevant snapshot(s) might not exist
	}
	stateBloom, err := NewStateBloomFromDisk(stateBloomPath)
	if err != nil {
		return err
	}
	log.Info("Loaded state bloom filter", "path", stateBloomPath)

	// All the state roots of the middle layers should be forcibly pruned,
	// otherwise the dangling state will be left.
	var (
		found       bool
		layers      = snaptree.Snapshots(headBlock.Root(), 128, true)
		middleRoots = make(map[common.Hash]struct{})
	)
	for _, layer := range layers {
		if layer.Root() == stateBloomRoot {
			found = true
			break
		}
		middleRoots[layer.Root()] = struct{}{}
	}
	if !found {
		log.Error("Pruning target state is not existent")
		return errors.New("non-existent target state")
	}
	return prune(snaptree, stateBloomRoot, db, stateBloom, stateBloomPath, middleRoots, time.Now())
}

// extractGenesis loads the genesis state and commits all the state entries
// into the given bloomfilter.
func extractGenesis(db ethdb.Database, stateBloom *stateBloom) error {
	genesisHash := rawdb.ReadCanonicalHash(db, 0)
	if genesisHash == (common.Hash{}) {
		return errors.New("missing genesis hash")
	}
	genesis := rawdb.ReadBlock(db, genesisHash, 0)
	if genesis == nil {
		return errors.New("missing genesis block")
	}
	t, err := trie.NewStateTrie(trie.StateTrieID(genesis.Root()), trie.NewDatabase(db, trie.HashDefaults))
	if err != nil {
		return err
	}
	accIter, err := t.NodeIterator(nil)
	if err != nil {
		return err
	}
	for accIter.Next(true) {
		hash := accIter.Hash()

		// Embedded nodes don't have hash.
		if hash != (common.Hash{}) {
			stateBloom.Put(hash.Bytes(), nil)
		}
		// If it's a leaf node, yes we are touching an account,
		// dig into the storage trie further.
		if accIter.Leaf() {
			var acc types.StateAccount
			if err := rlp.DecodeBytes(accIter.LeafBlob(), &acc); err != nil {
				return err
			}
			if acc.Root != types.EmptyRootHash {
				id := trie.StorageTrieID(genesis.Root(), common.BytesToHash(accIter.LeafKey()), acc.Root)
				storageTrie, err := trie.NewStateTrie(id, trie.NewDatabase(db, trie.HashDefaults))
				if err != nil {
					return err
				}
				storageIter, err := storageTrie.NodeIterator(nil)
				if err != nil {
					return err
				}
				for storageIter.Next(true) {
					hash := storageIter.Hash()
					if hash != (common.Hash{}) {
						stateBloom.Put(hash.Bytes(), nil)
					}
				}
				if storageIter.Error() != nil {
					return storageIter.Error()
				}
			}
			if !bytes.Equal(acc.CodeHash, types.EmptyCodeHash.Bytes()) {
				stateBloom.Put(acc.CodeHash, nil)
			}
		}
	}
	return accIter.Error()
}

func bloomFilterName(datadir string, hash common.Hash) string {
	return filepath.Join(datadir, fmt.Sprintf("%s.%s.%s", stateBloomFilePrefix, hash.Hex(), stateBloomFileSuffix))
}

func isBloomFilter(filename string) (bool, common.Hash) {
	filename = filepath.Base(filename)
	if strings.HasPrefix(filename, stateBloomFilePrefix) && strings.HasSuffix(filename, stateBloomFileSuffix) {
		return true, common.HexToHash(filename[len(stateBloomFilePrefix)+1 : len(filename)-len(stateBloomFileSuffix)-1])
	}
	return false, common.Hash{}
}

func findBloomFilter(datadir string) (string, common.Hash, error) {
	var (
		stateBloomPath string
		stateBloomRoot common.Hash
	)
	if err := filepath.Walk(datadir, func(path string, info os.FileInfo, err error) error {
		if info != nil && !info.IsDir() {
			ok, root := isBloomFilter(path)
			if ok {
				stateBloomPath = path
				stateBloomRoot = root
			}
		}
		return nil
	}); err != nil {
		return "", common.Hash{}, err
	}
	return stateBloomPath, stateBloomRoot, nil
}
