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
	"encoding/binary"
	"errors"
	"fmt"
	"io"
	"time"

	"github.com/VictoriaMetrics/fastcache"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/ethereum/go-ethereum/trie"
)

const journalVersion uint64 = 0

// journalGenerator is a disk layer entry containing the generator progress marker.
type journalGenerator struct {
	// Indicator that whether the database was in progress of being wiped.
	// It's deprecated but keep it here for background compatibility.
	Wiping bool

	Done     bool // Whether the generator finished creating the snapshot
	Marker   []byte
	Accounts uint64
	Slots    uint64
	Storage  uint64
}

// journalDestruct is an account deletion entry in a diffLayer's disk journal.
type journalDestruct struct {
	Hash common.Hash
}

// journalAccount is an account entry in a diffLayer's disk journal.
type journalAccount struct {
	Hash common.Hash
	Blob []byte
}

// journalStorage is an account's storage map in a diffLayer's disk journal.
type journalStorage struct {
	Hash common.Hash
	Keys []common.Hash
	Vals [][]byte
}

func ParseGeneratorStatus(generatorBlob []byte) string {
	if len(generatorBlob) == 0 {
		return ""
	}
	var generator journalGenerator
	if err := rlp.DecodeBytes(generatorBlob, &generator); err != nil {
		log.Warn("failed to decode snapshot generator", "err", err)
		return ""
	}
	// Figure out whether we're after or within an account
	var m string
	switch marker := generator.Marker; len(marker) {
	case common.HashLength:
		m = fmt.Sprintf("at %#x", marker)
	case 2 * common.HashLength:
		m = fmt.Sprintf("in %#x at %#x", marker[:common.HashLength], marker[common.HashLength:])
	default:
		m = fmt.Sprintf("%#x", marker)
	}
	return fmt.Sprintf(`Done: %v, Accounts: %d, Slots: %d, Storage: %d, Marker: %s`,
		generator.Done, generator.Accounts, generator.Slots, generator.Storage, m)
}

// loadAndParseJournal tries to parse the snapshot journal in latest format.
func loadAndParseJournal(db ethdb.KeyValueStore, base *diskLayer) (snapshot, journalGenerator, error) {
	// Retrieve the disk layer generator. It must exist, no matter the
	// snapshot is fully generated or not. Otherwise the entire disk
	// layer is invalid.
	generatorBlob := rawdb.ReadSnapshotGenerator(db)
	if len(generatorBlob) == 0 {
		return nil, journalGenerator{}, errors.New("missing snapshot generator")
	}
	var generator journalGenerator
	if err := rlp.DecodeBytes(generatorBlob, &generator); err != nil {
		return nil, journalGenerator{}, fmt.Errorf("failed to decode snapshot generator: %v", err)
	}
	// Retrieve the diff layer journal. It's possible that the journal is
	// not existent, e.g. the disk layer is generating while that the Geth
	// crashes without persisting the diff journal.
	// So if there is no journal, or the journal is invalid(e.g. the journal
	// is not matched with disk layer; or the it's the legacy-format journal,
	// etc.), we just discard all diffs and try to recover them later.
	var current snapshot = base
	err := iterateJournal(db, func(parent common.Hash, root common.Hash, destructSet map[common.Hash]struct{}, accountData map[common.Hash][]byte, storageData map[common.Hash]map[common.Hash][]byte) error {
		current = newDiffLayer(current, root, destructSet, accountData, storageData)
		return nil
	})
	if err != nil {
		return base, generator, nil
	}
	return current, generator, nil
}

// loadSnapshot loads a pre-existing state snapshot backed by a key-value store.
func loadSnapshot(diskdb ethdb.KeyValueStore, triedb *trie.Database, root common.Hash, cache int, recovery bool, noBuild bool) (snapshot, bool, error) {
	// If snapshotting is disabled (initial sync in progress), don't do anything,
	// wait for the chain to permit us to do something meaningful
	if rawdb.ReadSnapshotDisabled(diskdb) {
		return nil, true, nil
	}
	// Retrieve the block number and hash of the snapshot, failing if no snapshot
	// is present in the database (or crashed mid-update).
	baseRoot := rawdb.ReadSnapshotRoot(diskdb)
	if baseRoot == (common.Hash{}) {
		return nil, false, errors.New("missing or corrupted snapshot")
	}
	base := &diskLayer{
		diskdb: diskdb,
		triedb: triedb,
		cache:  fastcache.New(cache * 1024 * 1024),
		root:   baseRoot,
	}
	snapshot, generator, err := loadAndParseJournal(diskdb, base)
	if err != nil {
		log.Warn("Failed to load journal", "error", err)
		return nil, false, err
	}
	// Entire snapshot journal loaded, sanity check the head. If the loaded
	// snapshot is not matched with current state root, print a warning log
	// or discard the entire snapshot it's legacy snapshot.
	//
	// Possible scenario: Geth was crashed without persisting journal and then
	// restart, the head is rewound to the point with available state(trie)
	// which is below the snapshot. In this case the snapshot can be recovered
	// by re-executing blocks but right now it's unavailable.
	if head := snapshot.Root(); head != root {
		// If it's legacy snapshot, or it's new-format snapshot but
		// it's not in recovery mode, returns the error here for
		// rebuilding the entire snapshot forcibly.
		if !recovery {
			return nil, false, fmt.Errorf("head doesn't match snapshot: have %#x, want %#x", head, root)
		}
		// It's in snapshot recovery, the assumption is held that
		// the disk layer is always higher than chain head. It can
		// be eventually recovered when the chain head beyonds the
		// disk layer.
		log.Warn("Snapshot is not continuous with chain", "snaproot", head, "chainroot", root)
	}
	// Load the disk layer status from the generator if it's not complete
	if !generator.Done {
		base.genMarker = generator.Marker
		if base.genMarker == nil {
			base.genMarker = []byte{}
		}
	}
	// Everything loaded correctly, resume any suspended operations
	// if the background generation is allowed
	if !generator.Done && !noBuild {
		base.genPending = make(chan struct{})
		base.genAbort = make(chan chan *generatorStats)

		var origin uint64
		if len(generator.Marker) >= 8 {
			origin = binary.BigEndian.Uint64(generator.Marker)
		}
		go base.generate(&generatorStats{
			origin:   origin,
			start:    time.Now(),
			accounts: generator.Accounts,
			slots:    generator.Slots,
			storage:  common.StorageSize(generator.Storage),
		})
	}
	return snapshot, false, nil
}

// Journal terminates any in-progress snapshot generation, also implicitly pushing
// the progress into the database.
func (dl *diskLayer) Journal(buffer *bytes.Buffer) (common.Hash, error) {
	// If the snapshot is currently being generated, abort it
	var stats *generatorStats
	if dl.genAbort != nil {
		abort := make(chan *generatorStats)
		dl.genAbort <- abort

		if stats = <-abort; stats != nil {
			stats.Log("Journalling in-progress snapshot", dl.root, dl.genMarker)
		}
	}
	// Ensure the layer didn't get stale
	dl.lock.RLock()
	defer dl.lock.RUnlock()

	if dl.stale {
		return common.Hash{}, ErrSnapshotStale
	}
	// Ensure the generator stats is written even if none was ran this cycle
	journalProgress(dl.diskdb, dl.genMarker, stats)

	log.Debug("Journalled disk layer", "root", dl.root)
	return dl.root, nil
}

// Journal writes the memory layer contents into a buffer to be stored in the
// database as the snapshot journal.
func (dl *diffLayer) Journal(buffer *bytes.Buffer) (common.Hash, error) {
	// Journal the parent first
	base, err := dl.parent.Journal(buffer)
	if err != nil {
		return common.Hash{}, err
	}
	// Ensure the layer didn't get stale
	dl.lock.RLock()
	defer dl.lock.RUnlock()

	if dl.Stale() {
		return common.Hash{}, ErrSnapshotStale
	}
	// Everything below was journalled, persist this layer too
	if err := rlp.Encode(buffer, dl.root); err != nil {
		return common.Hash{}, err
	}
	destructs := make([]journalDestruct, 0, len(dl.destructSet))
	for hash := range dl.destructSet {
		destructs = append(destructs, journalDestruct{Hash: hash})
	}
	if err := rlp.Encode(buffer, destructs); err != nil {
		return common.Hash{}, err
	}
	accounts := make([]journalAccount, 0, len(dl.accountData))
	for hash, blob := range dl.accountData {
		accounts = append(accounts, journalAccount{Hash: hash, Blob: blob})
	}
	if err := rlp.Encode(buffer, accounts); err != nil {
		return common.Hash{}, err
	}
	storage := make([]journalStorage, 0, len(dl.storageData))
	for hash, slots := range dl.storageData {
		keys := make([]common.Hash, 0, len(slots))
		vals := make([][]byte, 0, len(slots))
		for key, val := range slots {
			keys = append(keys, key)
			vals = append(vals, val)
		}
		storage = append(storage, journalStorage{Hash: hash, Keys: keys, Vals: vals})
	}
	if err := rlp.Encode(buffer, storage); err != nil {
		return common.Hash{}, err
	}
	log.Debug("Journalled diff layer", "root", dl.root, "parent", dl.parent.Root())
	return base, nil
}

// journalCallback is a function which is invoked by iterateJournal, every
// time a difflayer is loaded from disk.
type journalCallback = func(parent common.Hash, root common.Hash, destructs map[common.Hash]struct{}, accounts map[common.Hash][]byte, storage map[common.Hash]map[common.Hash][]byte) error

// iterateJournal iterates through the journalled difflayers, loading them from
// the database, and invoking the callback for each loaded layer.
// The order is incremental; starting with the bottom-most difflayer, going towards
// the most recent layer.
// This method returns error either if there was some error reading from disk,
// OR if the callback returns an error when invoked.
func iterateJournal(db ethdb.KeyValueReader, callback journalCallback) error {
	journal := rawdb.ReadSnapshotJournal(db)
	if len(journal) == 0 {
		log.Warn("Loaded snapshot journal", "diffs", "missing")
		return nil
	}
	r := rlp.NewStream(bytes.NewReader(journal), 0)
	// Firstly, resolve the first element as the journal version
	version, err := r.Uint64()
	if err != nil {
		log.Warn("Failed to resolve the journal version", "error", err)
		return errors.New("failed to resolve journal version")
	}
	if version != journalVersion {
		log.Warn("Discarded the snapshot journal with wrong version", "required", journalVersion, "got", version)
		return errors.New("wrong journal version")
	}
	// Secondly, resolve the disk layer root, ensure it's continuous
	// with disk layer. Note now we can ensure it's the snapshot journal
	// correct version, so we expect everything can be resolved properly.
	var parent common.Hash
	if err := r.Decode(&parent); err != nil {
		return errors.New("missing disk layer root")
	}
	if baseRoot := rawdb.ReadSnapshotRoot(db); baseRoot != parent {
		log.Warn("Loaded snapshot journal", "diskroot", baseRoot, "diffs", "unmatched")
		return errors.New("mismatched disk and diff layers")
	}
	for {
		var (
			root        common.Hash
			destructs   []journalDestruct
			accounts    []journalAccount
			storage     []journalStorage
			destructSet = make(map[common.Hash]struct{})
			accountData = make(map[common.Hash][]byte)
			storageData = make(map[common.Hash]map[common.Hash][]byte)
		)
		// Read the next diff journal entry
		if err := r.Decode(&root); err != nil {
			// The first read may fail with EOF, marking the end of the journal
			if errors.Is(err, io.EOF) {
				return nil
			}
			return fmt.Errorf("load diff root: %v", err)
		}
		if err := r.Decode(&destructs); err != nil {
			return fmt.Errorf("load diff destructs: %v", err)
		}
		if err := r.Decode(&accounts); err != nil {
			return fmt.Errorf("load diff accounts: %v", err)
		}
		if err := r.Decode(&storage); err != nil {
			return fmt.Errorf("load diff storage: %v", err)
		}
		for _, entry := range destructs {
			destructSet[entry.Hash] = struct{}{}
		}
		for _, entry := range accounts {
			if len(entry.Blob) > 0 { // RLP loses nil-ness, but `[]byte{}` is not a valid item, so reinterpret that
				accountData[entry.Hash] = entry.Blob
			} else {
				accountData[entry.Hash] = nil
			}
		}
		for _, entry := range storage {
			slots := make(map[common.Hash][]byte)
			for i, key := range entry.Keys {
				if len(entry.Vals[i]) > 0 { // RLP loses nil-ness, but `[]byte{}` is not a valid item, so reinterpret that
					slots[key] = entry.Vals[i]
				} else {
					slots[key] = nil
				}
			}
			storageData[entry.Hash] = slots
		}
		if err := callback(parent, root, destructSet, accountData, storageData); err != nil {
			return err
		}
		parent = root
	}
}
