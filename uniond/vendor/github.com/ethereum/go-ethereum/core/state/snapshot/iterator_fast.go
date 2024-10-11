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
	"golang.org/x/exp/slices"
)

// weightedIterator is a iterator with an assigned weight. It is used to prioritise
// which account or storage slot is the correct one if multiple iterators find the
// same one (modified in multiple consecutive blocks).
type weightedIterator struct {
	it       Iterator
	priority int
}

func (it *weightedIterator) Cmp(other *weightedIterator) int {
	// Order the iterators primarily by the account hashes
	hashI := it.it.Hash()
	hashJ := other.it.Hash()

	switch bytes.Compare(hashI[:], hashJ[:]) {
	case -1:
		return -1
	case 1:
		return 1
	}
	// Same account/storage-slot in multiple layers, split by priority
	if it.priority < other.priority {
		return -1
	}
	if it.priority > other.priority {
		return 1
	}
	return 0
}

// fastIterator is a more optimized multi-layer iterator which maintains a
// direct mapping of all iterators leading down to the bottom layer.
type fastIterator struct {
	tree *Tree       // Snapshot tree to reinitialize stale sub-iterators with
	root common.Hash // Root hash to reinitialize stale sub-iterators through

	curAccount []byte
	curSlot    []byte

	iterators []*weightedIterator
	initiated bool
	account   bool
	fail      error
}

// newFastIterator creates a new hierarchical account or storage iterator with one
// element per diff layer. The returned combo iterator can be used to walk over
// the entire snapshot diff stack simultaneously.
func newFastIterator(tree *Tree, root common.Hash, account common.Hash, seek common.Hash, accountIterator bool) (*fastIterator, error) {
	snap := tree.Snapshot(root)
	if snap == nil {
		return nil, fmt.Errorf("unknown snapshot: %x", root)
	}
	fi := &fastIterator{
		tree:    tree,
		root:    root,
		account: accountIterator,
	}
	current := snap.(snapshot)
	for depth := 0; current != nil; depth++ {
		if accountIterator {
			fi.iterators = append(fi.iterators, &weightedIterator{
				it:       current.AccountIterator(seek),
				priority: depth,
			})
		} else {
			// If the whole storage is destructed in this layer, don't
			// bother deeper layer anymore. But we should still keep
			// the iterator for this layer, since the iterator can contain
			// some valid slots which belongs to the re-created account.
			it, destructed := current.StorageIterator(account, seek)
			fi.iterators = append(fi.iterators, &weightedIterator{
				it:       it,
				priority: depth,
			})
			if destructed {
				break
			}
		}
		current = current.Parent()
	}
	fi.init()
	return fi, nil
}

// init walks over all the iterators and resolves any clashes between them, after
// which it prepares the stack for step-by-step iteration.
func (fi *fastIterator) init() {
	// Track which account hashes are iterators positioned on
	var positioned = make(map[common.Hash]int)

	// Position all iterators and track how many remain live
	for i := 0; i < len(fi.iterators); i++ {
		// Retrieve the first element and if it clashes with a previous iterator,
		// advance either the current one or the old one. Repeat until nothing is
		// clashing any more.
		it := fi.iterators[i]
		for {
			// If the iterator is exhausted, drop it off the end
			if !it.it.Next() {
				it.it.Release()
				last := len(fi.iterators) - 1

				fi.iterators[i] = fi.iterators[last]
				fi.iterators[last] = nil
				fi.iterators = fi.iterators[:last]

				i--
				break
			}
			// The iterator is still alive, check for collisions with previous ones
			hash := it.it.Hash()
			if other, exist := positioned[hash]; !exist {
				positioned[hash] = i
				break
			} else {
				// Iterators collide, one needs to be progressed, use priority to
				// determine which.
				//
				// This whole else-block can be avoided, if we instead
				// do an initial priority-sort of the iterators. If we do that,
				// then we'll only wind up here if a lower-priority (preferred) iterator
				// has the same value, and then we will always just continue.
				// However, it costs an extra sort, so it's probably not better
				if fi.iterators[other].priority < it.priority {
					// The 'it' should be progressed
					continue
				} else {
					// The 'other' should be progressed, swap them
					it = fi.iterators[other]
					fi.iterators[other], fi.iterators[i] = fi.iterators[i], fi.iterators[other]
					continue
				}
			}
		}
	}
	// Re-sort the entire list
	slices.SortFunc(fi.iterators, func(a, b *weightedIterator) int { return a.Cmp(b) })
	fi.initiated = false
}

// Next steps the iterator forward one element, returning false if exhausted.
func (fi *fastIterator) Next() bool {
	if len(fi.iterators) == 0 {
		return false
	}
	if !fi.initiated {
		// Don't forward first time -- we had to 'Next' once in order to
		// do the sorting already
		fi.initiated = true
		if fi.account {
			fi.curAccount = fi.iterators[0].it.(AccountIterator).Account()
		} else {
			fi.curSlot = fi.iterators[0].it.(StorageIterator).Slot()
		}
		if innerErr := fi.iterators[0].it.Error(); innerErr != nil {
			fi.fail = innerErr
			return false
		}
		if fi.curAccount != nil || fi.curSlot != nil {
			return true
		}
		// Implicit else: we've hit a nil-account or nil-slot, and need to
		// fall through to the loop below to land on something non-nil
	}
	// If an account or a slot is deleted in one of the layers, the key will
	// still be there, but the actual value will be nil. However, the iterator
	// should not export nil-values (but instead simply omit the key), so we
	// need to loop here until we either
	//  - get a non-nil value,
	//  - hit an error,
	//  - or exhaust the iterator
	for {
		if !fi.next(0) {
			return false // exhausted
		}
		if fi.account {
			fi.curAccount = fi.iterators[0].it.(AccountIterator).Account()
		} else {
			fi.curSlot = fi.iterators[0].it.(StorageIterator).Slot()
		}
		if innerErr := fi.iterators[0].it.Error(); innerErr != nil {
			fi.fail = innerErr
			return false // error
		}
		if fi.curAccount != nil || fi.curSlot != nil {
			break // non-nil value found
		}
	}
	return true
}

// next handles the next operation internally and should be invoked when we know
// that two elements in the list may have the same value.
//
// For example, if the iterated hashes become [2,3,5,5,8,9,10], then we should
// invoke next(3), which will call Next on elem 3 (the second '5') and will
// cascade along the list, applying the same operation if needed.
func (fi *fastIterator) next(idx int) bool {
	// If this particular iterator got exhausted, remove it and return true (the
	// next one is surely not exhausted yet, otherwise it would have been removed
	// already).
	if it := fi.iterators[idx].it; !it.Next() {
		it.Release()

		fi.iterators = append(fi.iterators[:idx], fi.iterators[idx+1:]...)
		return len(fi.iterators) > 0
	}
	// If there's no one left to cascade into, return
	if idx == len(fi.iterators)-1 {
		return true
	}
	// We next-ed the iterator at 'idx', now we may have to re-sort that element
	var (
		cur, next         = fi.iterators[idx], fi.iterators[idx+1]
		curHash, nextHash = cur.it.Hash(), next.it.Hash()
	)
	if diff := bytes.Compare(curHash[:], nextHash[:]); diff < 0 {
		// It is still in correct place
		return true
	} else if diff == 0 && cur.priority < next.priority {
		// So still in correct place, but we need to iterate on the next
		fi.next(idx + 1)
		return true
	}
	// At this point, the iterator is in the wrong location, but the remaining
	// list is sorted. Find out where to move the item.
	clash := -1
	index := sort.Search(len(fi.iterators), func(n int) bool {
		// The iterator always advances forward, so anything before the old slot
		// is known to be behind us, so just skip them altogether. This actually
		// is an important clause since the sort order got invalidated.
		if n < idx {
			return false
		}
		if n == len(fi.iterators)-1 {
			// Can always place an elem last
			return true
		}
		nextHash := fi.iterators[n+1].it.Hash()
		if diff := bytes.Compare(curHash[:], nextHash[:]); diff < 0 {
			return true
		} else if diff > 0 {
			return false
		}
		// The elem we're placing it next to has the same value,
		// so whichever winds up on n+1 will need further iteration
		clash = n + 1

		return cur.priority < fi.iterators[n+1].priority
	})
	fi.move(idx, index)
	if clash != -1 {
		fi.next(clash)
	}
	return true
}

// move advances an iterator to another position in the list.
func (fi *fastIterator) move(index, newpos int) {
	elem := fi.iterators[index]
	copy(fi.iterators[index:], fi.iterators[index+1:newpos+1])
	fi.iterators[newpos] = elem
}

// Error returns any failure that occurred during iteration, which might have
// caused a premature iteration exit (e.g. snapshot stack becoming stale).
func (fi *fastIterator) Error() error {
	return fi.fail
}

// Hash returns the current key
func (fi *fastIterator) Hash() common.Hash {
	return fi.iterators[0].it.Hash()
}

// Account returns the current account blob.
// Note the returned account is not a copy, please don't modify it.
func (fi *fastIterator) Account() []byte {
	return fi.curAccount
}

// Slot returns the current storage slot.
// Note the returned slot is not a copy, please don't modify it.
func (fi *fastIterator) Slot() []byte {
	return fi.curSlot
}

// Release iterates over all the remaining live layer iterators and releases each
// of them individually.
func (fi *fastIterator) Release() {
	for _, it := range fi.iterators {
		it.it.Release()
	}
	fi.iterators = nil
}

// Debug is a convenience helper during testing
func (fi *fastIterator) Debug() {
	for _, it := range fi.iterators {
		fmt.Printf("[p=%v v=%v] ", it.priority, it.it.Hash()[0])
	}
	fmt.Println()
}

// newFastAccountIterator creates a new hierarchical account iterator with one
// element per diff layer. The returned combo iterator can be used to walk over
// the entire snapshot diff stack simultaneously.
func newFastAccountIterator(tree *Tree, root common.Hash, seek common.Hash) (AccountIterator, error) {
	return newFastIterator(tree, root, common.Hash{}, seek, true)
}

// newFastStorageIterator creates a new hierarchical storage iterator with one
// element per diff layer. The returned combo iterator can be used to walk over
// the entire snapshot diff stack simultaneously.
func newFastStorageIterator(tree *Tree, root common.Hash, account common.Hash, seek common.Hash) (StorageIterator, error) {
	return newFastIterator(tree, root, account, seek, false)
}
