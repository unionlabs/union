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

package rangeproof

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"io"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/ethdb/memorydb"
	"github.com/ethereum/go-ethereum/trie"
	"golang.org/x/exp/slices"
)

type kv struct {
	k, v []byte
	t    bool
}

type fuzzer struct {
	input     io.Reader
	exhausted bool
}

func (f *fuzzer) randBytes(n int) []byte {
	r := make([]byte, n)
	if _, err := f.input.Read(r); err != nil {
		f.exhausted = true
	}
	return r
}

func (f *fuzzer) readInt() uint64 {
	var x uint64
	if err := binary.Read(f.input, binary.LittleEndian, &x); err != nil {
		f.exhausted = true
	}
	return x
}

func (f *fuzzer) randomTrie(n int) (*trie.Trie, map[string]*kv) {
	trie := trie.NewEmpty(trie.NewDatabase(rawdb.NewMemoryDatabase(), nil))
	vals := make(map[string]*kv)
	size := f.readInt()
	// Fill it with some fluff
	for i := byte(0); i < byte(size); i++ {
		value := &kv{common.LeftPadBytes([]byte{i}, 32), []byte{i}, false}
		value2 := &kv{common.LeftPadBytes([]byte{i + 10}, 32), []byte{i}, false}
		trie.MustUpdate(value.k, value.v)
		trie.MustUpdate(value2.k, value2.v)
		vals[string(value.k)] = value
		vals[string(value2.k)] = value2
	}
	if f.exhausted {
		return nil, nil
	}
	// And now fill with some random
	for i := 0; i < n; i++ {
		k := f.randBytes(32)
		v := f.randBytes(20)
		value := &kv{k, v, false}
		trie.MustUpdate(k, v)
		vals[string(k)] = value
		if f.exhausted {
			return nil, nil
		}
	}
	return trie, vals
}

func (f *fuzzer) fuzz() int {
	maxSize := 200
	tr, vals := f.randomTrie(1 + int(f.readInt())%maxSize)
	if f.exhausted {
		return 0 // input too short
	}
	var entries []*kv
	for _, kv := range vals {
		entries = append(entries, kv)
	}
	if len(entries) <= 1 {
		return 0
	}
	slices.SortFunc(entries, func(a, b *kv) int {
		return bytes.Compare(a.k, b.k)
	})

	var ok = 0
	for {
		start := int(f.readInt() % uint64(len(entries)))
		end := 1 + int(f.readInt()%uint64(len(entries)-1))
		testcase := int(f.readInt() % uint64(6))
		index := int(f.readInt() & 0xFFFFFFFF)
		index2 := int(f.readInt() & 0xFFFFFFFF)
		if f.exhausted {
			break
		}
		proof := memorydb.New()
		if err := tr.Prove(entries[start].k, proof); err != nil {
			panic(fmt.Sprintf("Failed to prove the first node %v", err))
		}
		if err := tr.Prove(entries[end-1].k, proof); err != nil {
			panic(fmt.Sprintf("Failed to prove the last node %v", err))
		}
		var keys [][]byte
		var vals [][]byte
		for i := start; i < end; i++ {
			keys = append(keys, entries[i].k)
			vals = append(vals, entries[i].v)
		}
		if len(keys) == 0 {
			return 0
		}
		var first = keys[0]
		testcase %= 6
		switch testcase {
		case 0:
			// Modified key
			keys[index%len(keys)] = f.randBytes(32) // In theory it can't be same
		case 1:
			// Modified val
			vals[index%len(vals)] = f.randBytes(20) // In theory it can't be same
		case 2:
			// Gapped entry slice
			index = index % len(keys)
			keys = append(keys[:index], keys[index+1:]...)
			vals = append(vals[:index], vals[index+1:]...)
		case 3:
			// Out of order
			index1 := index % len(keys)
			index2 := index2 % len(keys)
			keys[index1], keys[index2] = keys[index2], keys[index1]
			vals[index1], vals[index2] = vals[index2], vals[index1]
		case 4:
			// Set random key to nil, do nothing
			keys[index%len(keys)] = nil
		case 5:
			// Set random value to nil, deletion
			vals[index%len(vals)] = nil

			// Other cases:
			// Modify something in the proof db
			// add stuff to proof db
			// drop stuff from proof db
		}
		if f.exhausted {
			break
		}
		ok = 1
		//nodes, subtrie
		hasMore, err := trie.VerifyRangeProof(tr.Hash(), first, keys, vals, proof)
		if err != nil {
			if hasMore {
				panic("err != nil && hasMore == true")
			}
		}
	}
	return ok
}

// Fuzz is the fuzzing entry-point.
// The function must return
//
//   - 1 if the fuzzer should increase priority of the
//     given input during subsequent fuzzing (for example, the input is lexically
//     correct and was parsed successfully);
//   - -1 if the input must not be added to corpus even if gives new coverage; and
//   - 0 otherwise
//
// other values are reserved for future use.
func fuzz(input []byte) int {
	if len(input) < 100 {
		return 0
	}
	r := bytes.NewReader(input)
	f := fuzzer{
		input:     r,
		exhausted: false,
	}
	return f.fuzz()
}
