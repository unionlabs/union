// Copyright 2022 The go-ethereum Authors
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
	"fmt"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/log"
)

// CheckDanglingStorage iterates the snap storage data, and verifies that all
// storage also has corresponding account data.
func CheckDanglingStorage(chaindb ethdb.KeyValueStore) error {
	if err := checkDanglingDiskStorage(chaindb); err != nil {
		log.Error("Database check error", "err", err)
	}
	return checkDanglingMemStorage(chaindb)
}

// checkDanglingDiskStorage checks if there is any 'dangling' storage data in the
// disk-backed snapshot layer.
func checkDanglingDiskStorage(chaindb ethdb.KeyValueStore) error {
	var (
		lastReport = time.Now()
		start      = time.Now()
		lastKey    []byte
		it         = rawdb.NewKeyLengthIterator(chaindb.NewIterator(rawdb.SnapshotStoragePrefix, nil), 1+2*common.HashLength)
	)
	log.Info("Checking dangling snapshot disk storage")

	defer it.Release()
	for it.Next() {
		k := it.Key()
		accKey := k[1:33]
		if bytes.Equal(accKey, lastKey) {
			// No need to look up for every slot
			continue
		}
		lastKey = common.CopyBytes(accKey)
		if time.Since(lastReport) > time.Second*8 {
			log.Info("Iterating snap storage", "at", fmt.Sprintf("%#x", accKey), "elapsed", common.PrettyDuration(time.Since(start)))
			lastReport = time.Now()
		}
		if data := rawdb.ReadAccountSnapshot(chaindb, common.BytesToHash(accKey)); len(data) == 0 {
			log.Warn("Dangling storage - missing account", "account", fmt.Sprintf("%#x", accKey), "storagekey", fmt.Sprintf("%#x", k))
			return fmt.Errorf("dangling snapshot storage account %#x", accKey)
		}
	}
	log.Info("Verified the snapshot disk storage", "time", common.PrettyDuration(time.Since(start)), "err", it.Error())
	return nil
}

// checkDanglingMemStorage checks if there is any 'dangling' storage in the journalled
// snapshot difflayers.
func checkDanglingMemStorage(db ethdb.KeyValueStore) error {
	start := time.Now()
	log.Info("Checking dangling journalled storage")
	err := iterateJournal(db, func(pRoot, root common.Hash, destructs map[common.Hash]struct{}, accounts map[common.Hash][]byte, storage map[common.Hash]map[common.Hash][]byte) error {
		for accHash := range storage {
			if _, ok := accounts[accHash]; !ok {
				log.Error("Dangling storage - missing account", "account", fmt.Sprintf("%#x", accHash), "root", root)
			}
		}
		return nil
	})
	if err != nil {
		log.Info("Failed to resolve snapshot journal", "err", err)
		return err
	}
	log.Info("Verified the snapshot journalled storage", "time", common.PrettyDuration(time.Since(start)))
	return nil
}

// CheckJournalAccount shows information about an account, from the disk layer and
// up through the diff layers.
func CheckJournalAccount(db ethdb.KeyValueStore, hash common.Hash) error {
	// Look up the disk layer first
	baseRoot := rawdb.ReadSnapshotRoot(db)
	fmt.Printf("Disklayer: Root: %x\n", baseRoot)
	if data := rawdb.ReadAccountSnapshot(db, hash); data != nil {
		account, err := types.FullAccount(data)
		if err != nil {
			panic(err)
		}
		fmt.Printf("\taccount.nonce: %d\n", account.Nonce)
		fmt.Printf("\taccount.balance: %x\n", account.Balance)
		fmt.Printf("\taccount.root: %x\n", account.Root)
		fmt.Printf("\taccount.codehash: %x\n", account.CodeHash)
	}
	// Check storage
	{
		it := rawdb.NewKeyLengthIterator(db.NewIterator(append(rawdb.SnapshotStoragePrefix, hash.Bytes()...), nil), 1+2*common.HashLength)
		fmt.Printf("\tStorage:\n")
		for it.Next() {
			slot := it.Key()[33:]
			fmt.Printf("\t\t%x: %x\n", slot, it.Value())
		}
		it.Release()
	}
	var depth = 0

	return iterateJournal(db, func(pRoot, root common.Hash, destructs map[common.Hash]struct{}, accounts map[common.Hash][]byte, storage map[common.Hash]map[common.Hash][]byte) error {
		_, a := accounts[hash]
		_, b := destructs[hash]
		_, c := storage[hash]
		depth++
		if !a && !b && !c {
			return nil
		}
		fmt.Printf("Disklayer+%d: Root: %x, parent %x\n", depth, root, pRoot)
		if data, ok := accounts[hash]; ok {
			account, err := types.FullAccount(data)
			if err != nil {
				panic(err)
			}
			fmt.Printf("\taccount.nonce: %d\n", account.Nonce)
			fmt.Printf("\taccount.balance: %x\n", account.Balance)
			fmt.Printf("\taccount.root: %x\n", account.Root)
			fmt.Printf("\taccount.codehash: %x\n", account.CodeHash)
		}
		if _, ok := destructs[hash]; ok {
			fmt.Printf("\t Destructed!")
		}
		if data, ok := storage[hash]; ok {
			fmt.Printf("\tStorage\n")
			for k, v := range data {
				fmt.Printf("\t\t%x: %x\n", k, v)
			}
		}
		return nil
	})
}
