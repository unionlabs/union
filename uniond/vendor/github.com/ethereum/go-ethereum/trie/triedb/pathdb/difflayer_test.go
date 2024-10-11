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

package pathdb

import (
	"bytes"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/trie/testutil"
	"github.com/ethereum/go-ethereum/trie/trienode"
)

func emptyLayer() *diskLayer {
	return &diskLayer{
		db:     New(rawdb.NewMemoryDatabase(), nil),
		buffer: newNodeBuffer(DefaultBufferSize, nil, 0),
	}
}

// goos: darwin
// goarch: arm64
// pkg: github.com/ethereum/go-ethereum/trie
// BenchmarkSearch128Layers
// BenchmarkSearch128Layers-8   	  243826	      4755 ns/op
func BenchmarkSearch128Layers(b *testing.B) { benchmarkSearch(b, 0, 128) }

// goos: darwin
// goarch: arm64
// pkg: github.com/ethereum/go-ethereum/trie
// BenchmarkSearch512Layers
// BenchmarkSearch512Layers-8   	   49686	     24256 ns/op
func BenchmarkSearch512Layers(b *testing.B) { benchmarkSearch(b, 0, 512) }

// goos: darwin
// goarch: arm64
// pkg: github.com/ethereum/go-ethereum/trie
// BenchmarkSearch1Layer
// BenchmarkSearch1Layer-8   	14062725	        88.40 ns/op
func BenchmarkSearch1Layer(b *testing.B) { benchmarkSearch(b, 127, 128) }

func benchmarkSearch(b *testing.B, depth int, total int) {
	var (
		npath []byte
		nhash common.Hash
		nblob []byte
	)
	// First, we set up 128 diff layers, with 3K items each
	fill := func(parent layer, index int) *diffLayer {
		nodes := make(map[common.Hash]map[string]*trienode.Node)
		nodes[common.Hash{}] = make(map[string]*trienode.Node)
		for i := 0; i < 3000; i++ {
			var (
				path = testutil.RandBytes(32)
				node = testutil.RandomNode()
			)
			nodes[common.Hash{}][string(path)] = trienode.New(node.Hash, node.Blob)
			if npath == nil && depth == index {
				npath = common.CopyBytes(path)
				nblob = common.CopyBytes(node.Blob)
				nhash = node.Hash
			}
		}
		return newDiffLayer(parent, common.Hash{}, 0, 0, nodes, nil)
	}
	var layer layer
	layer = emptyLayer()
	for i := 0; i < total; i++ {
		layer = fill(layer, i)
	}
	b.ResetTimer()

	var (
		have []byte
		err  error
	)
	for i := 0; i < b.N; i++ {
		have, err = layer.Node(common.Hash{}, npath, nhash)
		if err != nil {
			b.Fatal(err)
		}
	}
	if !bytes.Equal(have, nblob) {
		b.Fatalf("have %x want %x", have, nblob)
	}
}

// goos: darwin
// goarch: arm64
// pkg: github.com/ethereum/go-ethereum/trie
// BenchmarkPersist
// BenchmarkPersist-8   	      10	 111252975 ns/op
func BenchmarkPersist(b *testing.B) {
	// First, we set up 128 diff layers, with 3K items each
	fill := func(parent layer) *diffLayer {
		nodes := make(map[common.Hash]map[string]*trienode.Node)
		nodes[common.Hash{}] = make(map[string]*trienode.Node)
		for i := 0; i < 3000; i++ {
			var (
				path = testutil.RandBytes(32)
				node = testutil.RandomNode()
			)
			nodes[common.Hash{}][string(path)] = trienode.New(node.Hash, node.Blob)
		}
		return newDiffLayer(parent, common.Hash{}, 0, 0, nodes, nil)
	}
	for i := 0; i < b.N; i++ {
		b.StopTimer()
		var layer layer
		layer = emptyLayer()
		for i := 1; i < 128; i++ {
			layer = fill(layer)
		}
		b.StartTimer()

		dl, ok := layer.(*diffLayer)
		if !ok {
			break
		}
		dl.persist(false)
	}
}

// BenchmarkJournal benchmarks the performance for journaling the layers.
//
// BenchmarkJournal
// BenchmarkJournal-8   	      10	 110969279 ns/op
func BenchmarkJournal(b *testing.B) {
	b.SkipNow()

	// First, we set up 128 diff layers, with 3K items each
	fill := func(parent layer) *diffLayer {
		nodes := make(map[common.Hash]map[string]*trienode.Node)
		nodes[common.Hash{}] = make(map[string]*trienode.Node)
		for i := 0; i < 3000; i++ {
			var (
				path = testutil.RandBytes(32)
				node = testutil.RandomNode()
			)
			nodes[common.Hash{}][string(path)] = trienode.New(node.Hash, node.Blob)
		}
		// TODO(rjl493456442) a non-nil state set is expected.
		return newDiffLayer(parent, common.Hash{}, 0, 0, nodes, nil)
	}
	var layer layer
	layer = emptyLayer()
	for i := 0; i < 128; i++ {
		layer = fill(layer)
	}
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		layer.journal(new(bytes.Buffer))
	}
}
