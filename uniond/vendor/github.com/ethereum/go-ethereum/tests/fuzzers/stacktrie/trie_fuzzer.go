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

package stacktrie

import (
	"bytes"
	"encoding/binary"
	"errors"
	"fmt"
	"hash"
	"io"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/trie"
	"github.com/ethereum/go-ethereum/trie/trienode"
	"golang.org/x/crypto/sha3"
	"golang.org/x/exp/slices"
)

type fuzzer struct {
	input     io.Reader
	exhausted bool
	debugging bool
}

func (f *fuzzer) read(size int) []byte {
	out := make([]byte, size)
	if _, err := f.input.Read(out); err != nil {
		f.exhausted = true
	}
	return out
}

func (f *fuzzer) readSlice(min, max int) []byte {
	var a uint16
	binary.Read(f.input, binary.LittleEndian, &a)
	size := min + int(a)%(max-min)
	out := make([]byte, size)
	if _, err := f.input.Read(out); err != nil {
		f.exhausted = true
	}
	return out
}

// spongeDb is a dummy db backend which accumulates writes in a sponge
type spongeDb struct {
	sponge hash.Hash
	debug  bool
}

func (s *spongeDb) Has(key []byte) (bool, error)             { panic("implement me") }
func (s *spongeDb) Get(key []byte) ([]byte, error)           { return nil, errors.New("no such elem") }
func (s *spongeDb) Delete(key []byte) error                  { panic("implement me") }
func (s *spongeDb) NewBatch() ethdb.Batch                    { return &spongeBatch{s} }
func (s *spongeDb) NewBatchWithSize(size int) ethdb.Batch    { return &spongeBatch{s} }
func (s *spongeDb) NewSnapshot() (ethdb.Snapshot, error)     { panic("implement me") }
func (s *spongeDb) Stat(property string) (string, error)     { panic("implement me") }
func (s *spongeDb) Compact(start []byte, limit []byte) error { panic("implement me") }
func (s *spongeDb) Close() error                             { return nil }

func (s *spongeDb) Put(key []byte, value []byte) error {
	if s.debug {
		fmt.Printf("db.Put %x : %x\n", key, value)
	}
	s.sponge.Write(key)
	s.sponge.Write(value)
	return nil
}
func (s *spongeDb) NewIterator(prefix []byte, start []byte) ethdb.Iterator { panic("implement me") }

// spongeBatch is a dummy batch which immediately writes to the underlying spongedb
type spongeBatch struct {
	db *spongeDb
}

func (b *spongeBatch) Put(key, value []byte) error {
	b.db.Put(key, value)
	return nil
}
func (b *spongeBatch) Delete(key []byte) error             { panic("implement me") }
func (b *spongeBatch) ValueSize() int                      { return 100 }
func (b *spongeBatch) Write() error                        { return nil }
func (b *spongeBatch) Reset()                              {}
func (b *spongeBatch) Replay(w ethdb.KeyValueWriter) error { return nil }

type kv struct {
	k, v []byte
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
func fuzz(data []byte) int {
	f := fuzzer{
		input:     bytes.NewReader(data),
		exhausted: false,
	}
	return f.fuzz()
}

func Debug(data []byte) int {
	f := fuzzer{
		input:     bytes.NewReader(data),
		exhausted: false,
		debugging: true,
	}
	return f.fuzz()
}

func (f *fuzzer) fuzz() int {
	// This spongeDb is used to check the sequence of disk-db-writes
	var (
		spongeA = &spongeDb{sponge: sha3.NewLegacyKeccak256()}
		dbA     = trie.NewDatabase(rawdb.NewDatabase(spongeA), nil)
		trieA   = trie.NewEmpty(dbA)
		spongeB = &spongeDb{sponge: sha3.NewLegacyKeccak256()}
		dbB     = trie.NewDatabase(rawdb.NewDatabase(spongeB), nil)

		options = trie.NewStackTrieOptions().WithWriter(func(path []byte, hash common.Hash, blob []byte) {
			rawdb.WriteTrieNode(spongeB, common.Hash{}, path, hash, blob, dbB.Scheme())
		})
		trieB       = trie.NewStackTrie(options)
		vals        []kv
		useful      bool
		maxElements = 10000
		// operate on unique keys only
		keys = make(map[string]struct{})
	)
	// Fill the trie with elements
	for i := 0; !f.exhausted && i < maxElements; i++ {
		k := f.read(32)
		v := f.readSlice(1, 500)
		if f.exhausted {
			// If it was exhausted while reading, the value may be all zeroes,
			// thus 'deletion' which is not supported on stacktrie
			break
		}
		if _, present := keys[string(k)]; present {
			// This key is a duplicate, ignore it
			continue
		}
		keys[string(k)] = struct{}{}
		vals = append(vals, kv{k: k, v: v})
		trieA.MustUpdate(k, v)
		useful = true
	}
	if !useful {
		return 0
	}
	// Flush trie -> database
	rootA, nodes, err := trieA.Commit(false)
	if err != nil {
		panic(err)
	}
	if nodes != nil {
		dbA.Update(rootA, types.EmptyRootHash, 0, trienode.NewWithNodeSet(nodes), nil)
	}
	// Flush memdb -> disk (sponge)
	dbA.Commit(rootA, false)

	// Stacktrie requires sorted insertion
	slices.SortFunc(vals, func(a, b kv) int {
		return bytes.Compare(a.k, b.k)
	})
	for _, kv := range vals {
		if f.debugging {
			fmt.Printf("{\"%#x\" , \"%#x\"} // stacktrie.Update\n", kv.k, kv.v)
		}
		trieB.MustUpdate(kv.k, kv.v)
	}
	rootB := trieB.Hash()
	trieB.Commit()
	if rootA != rootB {
		panic(fmt.Sprintf("roots differ: (trie) %x != %x (stacktrie)", rootA, rootB))
	}
	sumA := spongeA.sponge.Sum(nil)
	sumB := spongeB.sponge.Sum(nil)
	if !bytes.Equal(sumA, sumB) {
		panic(fmt.Sprintf("sequence differ: (trie) %x != %x (stacktrie)", sumA, sumB))
	}

	// Ensure all the nodes are persisted correctly
	var (
		nodeset  = make(map[string][]byte) // path -> blob
		optionsC = trie.NewStackTrieOptions().WithWriter(func(path []byte, hash common.Hash, blob []byte) {
			if crypto.Keccak256Hash(blob) != hash {
				panic("invalid node blob")
			}
			nodeset[string(path)] = common.CopyBytes(blob)
		})
		trieC   = trie.NewStackTrie(optionsC)
		checked int
	)
	for _, kv := range vals {
		trieC.MustUpdate(kv.k, kv.v)
	}
	rootC := trieC.Commit()
	if rootA != rootC {
		panic(fmt.Sprintf("roots differ: (trie) %x != %x (stacktrie)", rootA, rootC))
	}
	trieA, _ = trie.New(trie.TrieID(rootA), dbA)
	iterA := trieA.MustNodeIterator(nil)
	for iterA.Next(true) {
		if iterA.Hash() == (common.Hash{}) {
			if _, present := nodeset[string(iterA.Path())]; present {
				panic("unexpected tiny node")
			}
			continue
		}
		nodeBlob, present := nodeset[string(iterA.Path())]
		if !present {
			panic("missing node")
		}
		if !bytes.Equal(nodeBlob, iterA.NodeBlob()) {
			panic("node blob is not matched")
		}
		checked += 1
	}
	if checked != len(nodeset) {
		panic("node number is not matched")
	}
	return 1
}
