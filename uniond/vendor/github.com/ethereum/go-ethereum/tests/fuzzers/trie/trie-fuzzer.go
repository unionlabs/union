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

package trie

import (
	"bytes"
	"encoding/binary"
	"errors"
	"fmt"

	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/trie"
	"github.com/ethereum/go-ethereum/trie/trienode"
)

// randTest performs random trie operations.
// Instances of this test are created by Generate.
type randTest []randTestStep

type randTestStep struct {
	op    int
	key   []byte // for opUpdate, opDelete, opGet
	value []byte // for opUpdate
	err   error  // for debugging
}

type proofDb struct{}

func (proofDb) Put(key []byte, value []byte) error {
	return nil
}

func (proofDb) Delete(key []byte) error {
	return nil
}

const (
	opUpdate = iota
	opDelete
	opGet
	opHash
	opCommit
	opItercheckhash
	opProve
	opMax // boundary value, not an actual op
)

type dataSource struct {
	input  []byte
	reader *bytes.Reader
}

func newDataSource(input []byte) *dataSource {
	return &dataSource{
		input, bytes.NewReader(input),
	}
}
func (ds *dataSource) readByte() byte {
	if b, err := ds.reader.ReadByte(); err != nil {
		return 0
	} else {
		return b
	}
}
func (ds *dataSource) Read(buf []byte) (int, error) {
	return ds.reader.Read(buf)
}
func (ds *dataSource) Ended() bool {
	return ds.reader.Len() == 0
}

func Generate(input []byte) randTest {
	var allKeys [][]byte
	r := newDataSource(input)
	genKey := func() []byte {
		if len(allKeys) < 2 || r.readByte() < 0x0f {
			// new key
			key := make([]byte, r.readByte()%50)
			r.Read(key)
			allKeys = append(allKeys, key)
			return key
		}
		// use existing key
		return allKeys[int(r.readByte())%len(allKeys)]
	}

	var steps randTest

	for i := 0; !r.Ended(); i++ {
		step := randTestStep{op: int(r.readByte()) % opMax}
		switch step.op {
		case opUpdate:
			step.key = genKey()
			step.value = make([]byte, 8)
			binary.BigEndian.PutUint64(step.value, uint64(i))
		case opGet, opDelete, opProve:
			step.key = genKey()
		}
		steps = append(steps, step)
		if len(steps) > 500 {
			break
		}
	}

	return steps
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
	program := Generate(input)
	if len(program) == 0 {
		return 0
	}
	if err := runRandTest(program); err != nil {
		panic(err)
	}
	return 1
}

func runRandTest(rt randTest) error {
	var (
		triedb = trie.NewDatabase(rawdb.NewMemoryDatabase(), nil)
		tr     = trie.NewEmpty(triedb)
		origin = types.EmptyRootHash
		values = make(map[string]string) // tracks content of the trie
	)
	for i, step := range rt {
		switch step.op {
		case opUpdate:
			tr.MustUpdate(step.key, step.value)
			values[string(step.key)] = string(step.value)
		case opDelete:
			tr.MustDelete(step.key)
			delete(values, string(step.key))
		case opGet:
			v := tr.MustGet(step.key)
			want := values[string(step.key)]
			if string(v) != want {
				rt[i].err = fmt.Errorf("mismatch for key %#x, got %#x want %#x", step.key, v, want)
			}
		case opHash:
			tr.Hash()
		case opCommit:
			hash, nodes, err := tr.Commit(false)
			if err != nil {
				return err
			}
			if nodes != nil {
				if err := triedb.Update(hash, origin, 0, trienode.NewWithNodeSet(nodes), nil); err != nil {
					return err
				}
			}
			newtr, err := trie.New(trie.TrieID(hash), triedb)
			if err != nil {
				return err
			}
			tr = newtr
			origin = hash
		case opItercheckhash:
			checktr := trie.NewEmpty(triedb)
			it := trie.NewIterator(tr.MustNodeIterator(nil))
			for it.Next() {
				checktr.MustUpdate(it.Key, it.Value)
			}
			if tr.Hash() != checktr.Hash() {
				return errors.New("hash mismatch in opItercheckhash")
			}
		case opProve:
			rt[i].err = tr.Prove(step.key, proofDb{})
		}
		// Abort the test on error.
		if rt[i].err != nil {
			return rt[i].err
		}
	}
	return nil
}
