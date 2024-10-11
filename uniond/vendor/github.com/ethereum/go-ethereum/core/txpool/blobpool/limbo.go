// Copyright 2023 The go-ethereum Authors
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

package blobpool

import (
	"errors"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/holiman/billy"
)

// limboBlob is a wrapper around an opaque blobset that also contains the tx hash
// to which it belongs as well as the block number in which it was included for
// finality eviction.
type limboBlob struct {
	TxHash common.Hash // Owner transaction's hash to support resurrecting reorged txs
	Block  uint64      // Block in which the blob transaction was included
	Tx     *types.Transaction
}

// limbo is a light, indexed database to temporarily store recently included
// blobs until they are finalized. The purpose is to support small reorgs, which
// would require pulling back up old blobs (which aren't part of the chain).
//
// TODO(karalabe): Currently updating the inclusion block of a blob needs a full db rewrite. Can we do without?
type limbo struct {
	store billy.Database // Persistent data store for limboed blobs

	index  map[common.Hash]uint64            // Mappings from tx hashes to datastore ids
	groups map[uint64]map[uint64]common.Hash // Set of txs included in past blocks
}

// newLimbo opens and indexes a set of limboed blob transactions.
func newLimbo(datadir string) (*limbo, error) {
	l := &limbo{
		index:  make(map[common.Hash]uint64),
		groups: make(map[uint64]map[uint64]common.Hash),
	}
	// Index all limboed blobs on disk and delete anything inprocessable
	var fails []uint64
	index := func(id uint64, size uint32, data []byte) {
		if l.parseBlob(id, data) != nil {
			fails = append(fails, id)
		}
	}
	store, err := billy.Open(billy.Options{Path: datadir}, newSlotter(), index)
	if err != nil {
		return nil, err
	}
	l.store = store

	if len(fails) > 0 {
		log.Warn("Dropping invalidated limboed blobs", "ids", fails)
		for _, id := range fails {
			if err := l.store.Delete(id); err != nil {
				l.Close()
				return nil, err
			}
		}
	}
	return l, nil
}

// Close closes down the underlying persistent store.
func (l *limbo) Close() error {
	return l.store.Close()
}

// parseBlob is a callback method on limbo creation that gets called for each
// limboed blob on disk to create the in-memory metadata index.
func (l *limbo) parseBlob(id uint64, data []byte) error {
	item := new(limboBlob)
	if err := rlp.DecodeBytes(data, item); err != nil {
		// This path is impossible unless the disk data representation changes
		// across restarts. For that ever unprobable case, recover gracefully
		// by ignoring this data entry.
		log.Error("Failed to decode blob limbo entry", "id", id, "err", err)
		return err
	}
	if _, ok := l.index[item.TxHash]; ok {
		// This path is impossible, unless due to a programming error a blob gets
		// inserted into the limbo which was already part of if. Recover gracefully
		// by ignoring this data entry.
		log.Error("Dropping duplicate blob limbo entry", "owner", item.TxHash, "id", id)
		return errors.New("duplicate blob")
	}
	l.index[item.TxHash] = id

	if _, ok := l.groups[item.Block]; !ok {
		l.groups[item.Block] = make(map[uint64]common.Hash)
	}
	l.groups[item.Block][id] = item.TxHash

	return nil
}

// finalize evicts all blobs belonging to a recently finalized block or older.
func (l *limbo) finalize(final *types.Header) {
	// Just in case there's no final block yet (network not yet merged, weird
	// restart, sethead, etc), fail gracefully.
	if final == nil {
		log.Error("Nil finalized block cannot evict old blobs")
		return
	}
	for block, ids := range l.groups {
		if block > final.Number.Uint64() {
			continue
		}
		for id, owner := range ids {
			if err := l.store.Delete(id); err != nil {
				log.Error("Failed to drop finalized blob", "block", block, "id", id, "err", err)
			}
			delete(l.index, owner)
		}
		delete(l.groups, block)
	}
}

// push stores a new blob transaction into the limbo, waiting until finality for
// it to be automatically evicted.
func (l *limbo) push(tx *types.Transaction, block uint64) error {
	// If the blobs are already tracked by the limbo, consider it a programming
	// error. There's not much to do against it, but be loud.
	if _, ok := l.index[tx.Hash()]; ok {
		log.Error("Limbo cannot push already tracked blobs", "tx", tx)
		return errors.New("already tracked blob transaction")
	}
	if err := l.setAndIndex(tx, block); err != nil {
		log.Error("Failed to set and index limboed blobs", "tx", tx, "err", err)
		return err
	}
	return nil
}

// pull retrieves a previously pushed set of blobs back from the limbo, removing
// it at the same time. This method should be used when a previously included blob
// transaction gets reorged out.
func (l *limbo) pull(tx common.Hash) (*types.Transaction, error) {
	// If the blobs are not tracked by the limbo, there's not much to do. This
	// can happen for example if a blob transaction is mined without pushing it
	// into the network first.
	id, ok := l.index[tx]
	if !ok {
		log.Trace("Limbo cannot pull non-tracked blobs", "tx", tx)
		return nil, errors.New("unseen blob transaction")
	}
	item, err := l.getAndDrop(id)
	if err != nil {
		log.Error("Failed to get and drop limboed blobs", "tx", tx, "id", id, "err", err)
		return nil, err
	}
	return item.Tx, nil
}

// update changes the block number under which a blob transaction is tracked. This
// method should be used when a reorg changes a transaction's inclusion block.
//
// The method may log errors for various unexpcted scenarios but will not return
// any of it since there's no clear error case. Some errors may be due to coding
// issues, others caused by signers mining MEV stuff or swapping transactions. In
// all cases, the pool needs to continue operating.
func (l *limbo) update(txhash common.Hash, block uint64) {
	// If the blobs are not tracked by the limbo, there's not much to do. This
	// can happen for example if a blob transaction is mined without pushing it
	// into the network first.
	id, ok := l.index[txhash]
	if !ok {
		log.Trace("Limbo cannot update non-tracked blobs", "tx", txhash)
		return
	}
	// If there was no change in the blob's inclusion block, don't mess around
	// with heavy database operations.
	if _, ok := l.groups[block][id]; ok {
		log.Trace("Blob transaction unchanged in limbo", "tx", txhash, "block", block)
		return
	}
	// Retrieve the old blobs from the data store and write them back with a new
	// block number. IF anything fails, there's not much to do, go on.
	item, err := l.getAndDrop(id)
	if err != nil {
		log.Error("Failed to get and drop limboed blobs", "tx", txhash, "id", id, "err", err)
		return
	}
	if err := l.setAndIndex(item.Tx, block); err != nil {
		log.Error("Failed to set and index limboed blobs", "tx", txhash, "err", err)
		return
	}
	log.Trace("Blob transaction updated in limbo", "tx", txhash, "old-block", item.Block, "new-block", block)
}

// getAndDrop retrieves a blob item from the limbo store and deletes it both from
// the store and indices.
func (l *limbo) getAndDrop(id uint64) (*limboBlob, error) {
	data, err := l.store.Get(id)
	if err != nil {
		return nil, err
	}
	item := new(limboBlob)
	if err = rlp.DecodeBytes(data, item); err != nil {
		return nil, err
	}
	delete(l.index, item.TxHash)
	delete(l.groups[item.Block], id)
	if len(l.groups[item.Block]) == 0 {
		delete(l.groups, item.Block)
	}
	if err := l.store.Delete(id); err != nil {
		return nil, err
	}
	return item, nil
}

// setAndIndex assembles a limbo blob database entry and stores it, also updating
// the in-memory indices.
func (l *limbo) setAndIndex(tx *types.Transaction, block uint64) error {
	txhash := tx.Hash()
	item := &limboBlob{
		TxHash: txhash,
		Block:  block,
		Tx:     tx,
	}
	data, err := rlp.EncodeToBytes(item)
	if err != nil {
		panic(err) // cannot happen runtime, dev error
	}
	id, err := l.store.Put(data)
	if err != nil {
		return err
	}
	l.index[txhash] = id
	if _, ok := l.groups[block]; !ok {
		l.groups[block] = make(map[uint64]common.Hash)
	}
	l.groups[block][id] = txhash
	return nil
}
