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

package server

import (
	"bytes"
	"encoding/binary"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/lru"
	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/les/utils"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/rlp"
)

const (
	balanceCacheLimit = 8192 // the maximum number of cached items in service token balance queue

	// nodeDBVersion is the version identifier of the node data in db
	//
	// Changelog:
	// Version 0 => 1
	// * Replace `lastTotal` with `meta` in positive balance: version 0=>1
	//
	// Version 1 => 2
	// * Positive Balance and negative balance is changed:
	// * Cumulative time is replaced with expiration
	nodeDBVersion = 2

	// dbCleanupCycle is the cycle of db for useless data cleanup
	dbCleanupCycle = time.Hour
)

var (
	positiveBalancePrefix = []byte("pb:")         // dbVersion(uint16 big endian) + positiveBalancePrefix + id -> balance
	negativeBalancePrefix = []byte("nb:")         // dbVersion(uint16 big endian) + negativeBalancePrefix + ip -> balance
	expirationKey         = []byte("expiration:") // dbVersion(uint16 big endian) + expirationKey -> posExp, negExp
)

type nodeDB struct {
	db            ethdb.KeyValueStore
	cache         *lru.Cache[string, utils.ExpiredValue]
	auxbuf        []byte                                              // 37-byte auxiliary buffer for key encoding
	verbuf        [2]byte                                             // 2-byte auxiliary buffer for db version
	evictCallBack func(mclock.AbsTime, bool, utils.ExpiredValue) bool // Callback to determine whether the balance can be evicted.
	clock         mclock.Clock
	closeCh       chan struct{}
	cleanupHook   func() // Test hook used for testing
}

func newNodeDB(db ethdb.KeyValueStore, clock mclock.Clock) *nodeDB {
	ndb := &nodeDB{
		db:      db,
		cache:   lru.NewCache[string, utils.ExpiredValue](balanceCacheLimit),
		auxbuf:  make([]byte, 37),
		clock:   clock,
		closeCh: make(chan struct{}),
	}
	binary.BigEndian.PutUint16(ndb.verbuf[:], uint16(nodeDBVersion))
	go ndb.expirer()
	return ndb
}

func (db *nodeDB) close() {
	close(db.closeCh)
}

func (db *nodeDB) getPrefix(neg bool) []byte {
	prefix := positiveBalancePrefix
	if neg {
		prefix = negativeBalancePrefix
	}
	return append(db.verbuf[:], prefix...)
}

func (db *nodeDB) key(id []byte, neg bool) []byte {
	prefix := positiveBalancePrefix
	if neg {
		prefix = negativeBalancePrefix
	}
	if len(prefix)+len(db.verbuf)+len(id) > len(db.auxbuf) {
		db.auxbuf = append(db.auxbuf, make([]byte, len(prefix)+len(db.verbuf)+len(id)-len(db.auxbuf))...)
	}
	copy(db.auxbuf[:len(db.verbuf)], db.verbuf[:])
	copy(db.auxbuf[len(db.verbuf):len(db.verbuf)+len(prefix)], prefix)
	copy(db.auxbuf[len(prefix)+len(db.verbuf):len(prefix)+len(db.verbuf)+len(id)], id)
	return db.auxbuf[:len(prefix)+len(db.verbuf)+len(id)]
}

func (db *nodeDB) getExpiration() (utils.Fixed64, utils.Fixed64) {
	blob, err := db.db.Get(append(db.verbuf[:], expirationKey...))
	if err != nil || len(blob) != 16 {
		return 0, 0
	}
	return utils.Fixed64(binary.BigEndian.Uint64(blob[:8])), utils.Fixed64(binary.BigEndian.Uint64(blob[8:16]))
}

func (db *nodeDB) setExpiration(pos, neg utils.Fixed64) {
	var buff [16]byte
	binary.BigEndian.PutUint64(buff[:8], uint64(pos))
	binary.BigEndian.PutUint64(buff[8:16], uint64(neg))
	db.db.Put(append(db.verbuf[:], expirationKey...), buff[:16])
}

func (db *nodeDB) getOrNewBalance(id []byte, neg bool) utils.ExpiredValue {
	key := db.key(id, neg)
	item, exist := db.cache.Get(string(key))
	if exist {
		return item
	}

	var b utils.ExpiredValue
	enc, err := db.db.Get(key)
	if err != nil || len(enc) == 0 {
		return b
	}
	if err := rlp.DecodeBytes(enc, &b); err != nil {
		log.Crit("Failed to decode positive balance", "err", err)
	}
	db.cache.Add(string(key), b)
	return b
}

func (db *nodeDB) setBalance(id []byte, neg bool, b utils.ExpiredValue) {
	key := db.key(id, neg)
	enc, err := rlp.EncodeToBytes(&(b))
	if err != nil {
		log.Crit("Failed to encode positive balance", "err", err)
	}
	db.db.Put(key, enc)
	db.cache.Add(string(key), b)
}

func (db *nodeDB) delBalance(id []byte, neg bool) {
	key := db.key(id, neg)
	db.db.Delete(key)
	db.cache.Remove(string(key))
}

// getPosBalanceIDs returns a lexicographically ordered list of IDs of accounts
// with a positive balance
func (db *nodeDB) getPosBalanceIDs(start, stop enode.ID, maxCount int) (result []enode.ID) {
	if maxCount <= 0 {
		return
	}
	prefix := db.getPrefix(false)
	keylen := len(prefix) + len(enode.ID{})

	it := db.db.NewIterator(prefix, start.Bytes())
	defer it.Release()

	for it.Next() {
		var id enode.ID
		if len(it.Key()) != keylen {
			return
		}
		copy(id[:], it.Key()[keylen-len(id):])
		if bytes.Compare(id.Bytes(), stop.Bytes()) >= 0 {
			return
		}
		result = append(result, id)
		if len(result) == maxCount {
			return
		}
	}
	return
}

// forEachBalance iterates all balances and passes values to callback.
func (db *nodeDB) forEachBalance(neg bool, callback func(id enode.ID, balance utils.ExpiredValue) bool) {
	prefix := db.getPrefix(neg)
	keylen := len(prefix) + len(enode.ID{})

	it := db.db.NewIterator(prefix, nil)
	defer it.Release()

	for it.Next() {
		var id enode.ID
		if len(it.Key()) != keylen {
			return
		}
		copy(id[:], it.Key()[keylen-len(id):])

		var b utils.ExpiredValue
		if err := rlp.DecodeBytes(it.Value(), &b); err != nil {
			continue
		}
		if !callback(id, b) {
			return
		}
	}
}

func (db *nodeDB) expirer() {
	for {
		select {
		case <-db.clock.After(dbCleanupCycle):
			db.expireNodes()
		case <-db.closeCh:
			return
		}
	}
}

// expireNodes iterates the whole node db and checks whether the
// token balances can be deleted.
func (db *nodeDB) expireNodes() {
	var (
		visited int
		deleted int
		start   = time.Now()
	)
	for _, neg := range []bool{false, true} {
		iter := db.db.NewIterator(db.getPrefix(neg), nil)
		for iter.Next() {
			visited++
			var balance utils.ExpiredValue
			if err := rlp.DecodeBytes(iter.Value(), &balance); err != nil {
				log.Crit("Failed to decode negative balance", "err", err)
			}
			if db.evictCallBack != nil && db.evictCallBack(db.clock.Now(), neg, balance) {
				deleted++
				db.db.Delete(iter.Key())
			}
		}
	}
	// Invoke testing hook if it's not nil.
	if db.cleanupHook != nil {
		db.cleanupHook()
	}
	log.Debug("Expire nodes", "visited", visited, "deleted", deleted, "elapsed", common.PrettyDuration(time.Since(start)))
}
