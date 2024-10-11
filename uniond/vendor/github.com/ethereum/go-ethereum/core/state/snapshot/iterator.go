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
	"fmt"
	"sort"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/ethdb"
)

// Iterator is an iterator to step over all the accounts or the specific
// storage in a snapshot which may or may not be composed of multiple layers.
type Iterator interface {
	// Next steps the iterator forward one element, returning false if exhausted,
	// or an error if iteration failed for some reason (e.g. root being iterated
	// becomes stale and garbage collected).
	Next() bool

	// Error returns any failure that occurred during iteration, which might have
	// caused a premature iteration exit (e.g. snapshot stack becoming stale).
	Error() error

	// Hash returns the hash of the account or storage slot the iterator is
	// currently at.
	Hash() common.Hash

	// Release releases associated resources. Release should always succeed and
	// can be called multiple times without causing error.
	Release()
}

// AccountIterator is an iterator to step over all the accounts in a snapshot,
// which may or may not be composed of multiple layers.
type AccountIterator interface {
	Iterator

	// Account returns the RLP encoded slim account the iterator is currently at.
	// An error will be returned if the iterator becomes invalid
	Account() []byte
}

// StorageIterator is an iterator to step over the specific storage in a snapshot,
// which may or may not be composed of multiple layers.
type StorageIterator interface {
	Iterator

	// Slot returns the storage slot the iterator is currently at. An error will
	// be returned if the iterator becomes invalid
	Slot() []byte
}

// diffAccountIterator is an account iterator that steps over the accounts (both
// live and deleted) contained within a single diff layer. Higher order iterators
// will use the deleted accounts to skip deeper iterators.
type diffAccountIterator struct {
	// curHash is the current hash the iterator is positioned on. The field is
	// explicitly tracked since the referenced diff layer might go stale after
	// the iterator was positioned and we don't want to fail accessing the old
	// hash as long as the iterator is not touched any more.
	curHash common.Hash

	layer *diffLayer    // Live layer to retrieve values from
	keys  []common.Hash // Keys left in the layer to iterate
	fail  error         // Any failures encountered (stale)
}

// AccountIterator creates an account iterator over a single diff layer.
func (dl *diffLayer) AccountIterator(seek common.Hash) AccountIterator {
	// Seek out the requested starting account
	hashes := dl.AccountList()
	index := sort.Search(len(hashes), func(i int) bool {
		return bytes.Compare(seek[:], hashes[i][:]) <= 0
	})
	// Assemble and returned the already seeked iterator
	return &diffAccountIterator{
		layer: dl,
		keys:  hashes[index:],
	}
}

// Next steps the iterator forward one element, returning false if exhausted.
func (it *diffAccountIterator) Next() bool {
	// If the iterator was already stale, consider it a programmer error. Although
	// we could just return false here, triggering this path would probably mean
	// somebody forgot to check for Error, so lets blow up instead of undefined
	// behavior that's hard to debug.
	if it.fail != nil {
		panic(fmt.Sprintf("called Next of failed iterator: %v", it.fail))
	}
	// Stop iterating if all keys were exhausted
	if len(it.keys) == 0 {
		return false
	}
	if it.layer.Stale() {
		it.fail, it.keys = ErrSnapshotStale, nil
		return false
	}
	// Iterator seems to be still alive, retrieve and cache the live hash
	it.curHash = it.keys[0]
	// key cached, shift the iterator and notify the user of success
	it.keys = it.keys[1:]
	return true
}

// Error returns any failure that occurred during iteration, which might have
// caused a premature iteration exit (e.g. snapshot stack becoming stale).
func (it *diffAccountIterator) Error() error {
	return it.fail
}

// Hash returns the hash of the account the iterator is currently at.
func (it *diffAccountIterator) Hash() common.Hash {
	return it.curHash
}

// Account returns the RLP encoded slim account the iterator is currently at.
// This method may _fail_, if the underlying layer has been flattened between
// the call to Next and Account. That type of error will set it.Err.
// This method assumes that flattening does not delete elements from
// the accountdata mapping (writing nil into it is fine though), and will panic
// if elements have been deleted.
//
// Note the returned account is not a copy, please don't modify it.
func (it *diffAccountIterator) Account() []byte {
	it.layer.lock.RLock()
	blob, ok := it.layer.accountData[it.curHash]
	if !ok {
		if _, ok := it.layer.destructSet[it.curHash]; ok {
			it.layer.lock.RUnlock()
			return nil
		}
		panic(fmt.Sprintf("iterator referenced non-existent account: %x", it.curHash))
	}
	it.layer.lock.RUnlock()
	if it.layer.Stale() {
		it.fail, it.keys = ErrSnapshotStale, nil
	}
	return blob
}

// Release is a noop for diff account iterators as there are no held resources.
func (it *diffAccountIterator) Release() {}

// diskAccountIterator is an account iterator that steps over the live accounts
// contained within a disk layer.
type diskAccountIterator struct {
	layer *diskLayer
	it    ethdb.Iterator
}

// AccountIterator creates an account iterator over a disk layer.
func (dl *diskLayer) AccountIterator(seek common.Hash) AccountIterator {
	pos := common.TrimRightZeroes(seek[:])
	return &diskAccountIterator{
		layer: dl,
		it:    dl.diskdb.NewIterator(rawdb.SnapshotAccountPrefix, pos),
	}
}

// Next steps the iterator forward one element, returning false if exhausted.
func (it *diskAccountIterator) Next() bool {
	// If the iterator was already exhausted, don't bother
	if it.it == nil {
		return false
	}
	// Try to advance the iterator and release it if we reached the end
	for {
		if !it.it.Next() {
			it.it.Release()
			it.it = nil
			return false
		}
		if len(it.it.Key()) == len(rawdb.SnapshotAccountPrefix)+common.HashLength {
			break
		}
	}
	return true
}

// Error returns any failure that occurred during iteration, which might have
// caused a premature iteration exit (e.g. snapshot stack becoming stale).
//
// A diff layer is immutable after creation content wise and can always be fully
// iterated without error, so this method always returns nil.
func (it *diskAccountIterator) Error() error {
	if it.it == nil {
		return nil // Iterator is exhausted and released
	}
	return it.it.Error()
}

// Hash returns the hash of the account the iterator is currently at.
func (it *diskAccountIterator) Hash() common.Hash {
	return common.BytesToHash(it.it.Key()) // The prefix will be truncated
}

// Account returns the RLP encoded slim account the iterator is currently at.
func (it *diskAccountIterator) Account() []byte {
	return it.it.Value()
}

// Release releases the database snapshot held during iteration.
func (it *diskAccountIterator) Release() {
	// The iterator is auto-released on exhaustion, so make sure it's still alive
	if it.it != nil {
		it.it.Release()
		it.it = nil
	}
}

// diffStorageIterator is a storage iterator that steps over the specific storage
// (both live and deleted) contained within a single diff layer. Higher order
// iterators will use the deleted slot to skip deeper iterators.
type diffStorageIterator struct {
	// curHash is the current hash the iterator is positioned on. The field is
	// explicitly tracked since the referenced diff layer might go stale after
	// the iterator was positioned and we don't want to fail accessing the old
	// hash as long as the iterator is not touched any more.
	curHash common.Hash
	account common.Hash

	layer *diffLayer    // Live layer to retrieve values from
	keys  []common.Hash // Keys left in the layer to iterate
	fail  error         // Any failures encountered (stale)
}

// StorageIterator creates a storage iterator over a single diff layer.
// Except the storage iterator is returned, there is an additional flag
// "destructed" returned. If it's true then it means the whole storage is
// destructed in this layer(maybe recreated too), don't bother deeper layer
// for storage retrieval.
func (dl *diffLayer) StorageIterator(account common.Hash, seek common.Hash) (StorageIterator, bool) {
	// Create the storage for this account even it's marked
	// as destructed. The iterator is for the new one which
	// just has the same address as the deleted one.
	hashes, destructed := dl.StorageList(account)
	index := sort.Search(len(hashes), func(i int) bool {
		return bytes.Compare(seek[:], hashes[i][:]) <= 0
	})
	// Assemble and returned the already seeked iterator
	return &diffStorageIterator{
		layer:   dl,
		account: account,
		keys:    hashes[index:],
	}, destructed
}

// Next steps the iterator forward one element, returning false if exhausted.
func (it *diffStorageIterator) Next() bool {
	// If the iterator was already stale, consider it a programmer error. Although
	// we could just return false here, triggering this path would probably mean
	// somebody forgot to check for Error, so lets blow up instead of undefined
	// behavior that's hard to debug.
	if it.fail != nil {
		panic(fmt.Sprintf("called Next of failed iterator: %v", it.fail))
	}
	// Stop iterating if all keys were exhausted
	if len(it.keys) == 0 {
		return false
	}
	if it.layer.Stale() {
		it.fail, it.keys = ErrSnapshotStale, nil
		return false
	}
	// Iterator seems to be still alive, retrieve and cache the live hash
	it.curHash = it.keys[0]
	// key cached, shift the iterator and notify the user of success
	it.keys = it.keys[1:]
	return true
}

// Error returns any failure that occurred during iteration, which might have
// caused a premature iteration exit (e.g. snapshot stack becoming stale).
func (it *diffStorageIterator) Error() error {
	return it.fail
}

// Hash returns the hash of the storage slot the iterator is currently at.
func (it *diffStorageIterator) Hash() common.Hash {
	return it.curHash
}

// Slot returns the raw storage slot value the iterator is currently at.
// This method may _fail_, if the underlying layer has been flattened between
// the call to Next and Value. That type of error will set it.Err.
// This method assumes that flattening does not delete elements from
// the storage mapping (writing nil into it is fine though), and will panic
// if elements have been deleted.
//
// Note the returned slot is not a copy, please don't modify it.
func (it *diffStorageIterator) Slot() []byte {
	it.layer.lock.RLock()
	storage, ok := it.layer.storageData[it.account]
	if !ok {
		panic(fmt.Sprintf("iterator referenced non-existent account storage: %x", it.account))
	}
	// Storage slot might be nil(deleted), but it must exist
	blob, ok := storage[it.curHash]
	if !ok {
		panic(fmt.Sprintf("iterator referenced non-existent storage slot: %x", it.curHash))
	}
	it.layer.lock.RUnlock()
	if it.layer.Stale() {
		it.fail, it.keys = ErrSnapshotStale, nil
	}
	return blob
}

// Release is a noop for diff account iterators as there are no held resources.
func (it *diffStorageIterator) Release() {}

// diskStorageIterator is a storage iterator that steps over the live storage
// contained within a disk layer.
type diskStorageIterator struct {
	layer   *diskLayer
	account common.Hash
	it      ethdb.Iterator
}

// StorageIterator creates a storage iterator over a disk layer.
// If the whole storage is destructed, then all entries in the disk
// layer are deleted already. So the "destructed" flag returned here
// is always false.
func (dl *diskLayer) StorageIterator(account common.Hash, seek common.Hash) (StorageIterator, bool) {
	pos := common.TrimRightZeroes(seek[:])
	return &diskStorageIterator{
		layer:   dl,
		account: account,
		it:      dl.diskdb.NewIterator(append(rawdb.SnapshotStoragePrefix, account.Bytes()...), pos),
	}, false
}

// Next steps the iterator forward one element, returning false if exhausted.
func (it *diskStorageIterator) Next() bool {
	// If the iterator was already exhausted, don't bother
	if it.it == nil {
		return false
	}
	// Try to advance the iterator and release it if we reached the end
	for {
		if !it.it.Next() {
			it.it.Release()
			it.it = nil
			return false
		}
		if len(it.it.Key()) == len(rawdb.SnapshotStoragePrefix)+common.HashLength+common.HashLength {
			break
		}
	}
	return true
}

// Error returns any failure that occurred during iteration, which might have
// caused a premature iteration exit (e.g. snapshot stack becoming stale).
//
// A diff layer is immutable after creation content wise and can always be fully
// iterated without error, so this method always returns nil.
func (it *diskStorageIterator) Error() error {
	if it.it == nil {
		return nil // Iterator is exhausted and released
	}
	return it.it.Error()
}

// Hash returns the hash of the storage slot the iterator is currently at.
func (it *diskStorageIterator) Hash() common.Hash {
	return common.BytesToHash(it.it.Key()) // The prefix will be truncated
}

// Slot returns the raw storage slot content the iterator is currently at.
func (it *diskStorageIterator) Slot() []byte {
	return it.it.Value()
}

// Release releases the database snapshot held during iteration.
func (it *diskStorageIterator) Release() {
	// The iterator is auto-released on exhaustion, so make sure it's still alive
	if it.it != nil {
		it.it.Release()
		it.it = nil
	}
}
