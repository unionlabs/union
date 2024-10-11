// Copyright 2016 The go-ethereum Authors
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
	"testing"

	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/rlp"
)

func newTestFullNode(v []byte) []interface{} {
	fullNodeData := []interface{}{}
	for i := 0; i < 16; i++ {
		k := bytes.Repeat([]byte{byte(i + 1)}, 32)
		fullNodeData = append(fullNodeData, k)
	}
	fullNodeData = append(fullNodeData, v)
	return fullNodeData
}

func TestDecodeNestedNode(t *testing.T) {
	fullNodeData := newTestFullNode([]byte("fullnode"))

	data := [][]byte{}
	for i := 0; i < 16; i++ {
		data = append(data, nil)
	}
	data = append(data, []byte("subnode"))
	fullNodeData[15] = data

	buf := bytes.NewBuffer([]byte{})
	rlp.Encode(buf, fullNodeData)

	if _, err := decodeNode([]byte("testdecode"), buf.Bytes()); err != nil {
		t.Fatalf("decode nested full node err: %v", err)
	}
}

func TestDecodeFullNodeWrongSizeChild(t *testing.T) {
	fullNodeData := newTestFullNode([]byte("wrongsizechild"))
	fullNodeData[0] = []byte("00")
	buf := bytes.NewBuffer([]byte{})
	rlp.Encode(buf, fullNodeData)

	_, err := decodeNode([]byte("testdecode"), buf.Bytes())
	if _, ok := err.(*decodeError); !ok {
		t.Fatalf("decodeNode returned wrong err: %v", err)
	}
}

func TestDecodeFullNodeWrongNestedFullNode(t *testing.T) {
	fullNodeData := newTestFullNode([]byte("fullnode"))

	data := [][]byte{}
	for i := 0; i < 16; i++ {
		data = append(data, []byte("123456"))
	}
	data = append(data, []byte("subnode"))
	fullNodeData[15] = data

	buf := bytes.NewBuffer([]byte{})
	rlp.Encode(buf, fullNodeData)

	_, err := decodeNode([]byte("testdecode"), buf.Bytes())
	if _, ok := err.(*decodeError); !ok {
		t.Fatalf("decodeNode returned wrong err: %v", err)
	}
}

func TestDecodeFullNode(t *testing.T) {
	fullNodeData := newTestFullNode([]byte("decodefullnode"))
	buf := bytes.NewBuffer([]byte{})
	rlp.Encode(buf, fullNodeData)

	_, err := decodeNode([]byte("testdecode"), buf.Bytes())
	if err != nil {
		t.Fatalf("decode full node err: %v", err)
	}
}

// goos: darwin
// goarch: arm64
// pkg: github.com/ethereum/go-ethereum/trie
// BenchmarkEncodeShortNode
// BenchmarkEncodeShortNode-8   	16878850	        70.81 ns/op	      48 B/op	       1 allocs/op
func BenchmarkEncodeShortNode(b *testing.B) {
	node := &shortNode{
		Key: []byte{0x1, 0x2},
		Val: hashNode(randBytes(32)),
	}
	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		nodeToBytes(node)
	}
}

// goos: darwin
// goarch: arm64
// pkg: github.com/ethereum/go-ethereum/trie
// BenchmarkEncodeFullNode
// BenchmarkEncodeFullNode-8   	 4323273	       284.4 ns/op	     576 B/op	       1 allocs/op
func BenchmarkEncodeFullNode(b *testing.B) {
	node := &fullNode{}
	for i := 0; i < 16; i++ {
		node.Children[i] = hashNode(randBytes(32))
	}
	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		nodeToBytes(node)
	}
}

// goos: darwin
// goarch: arm64
// pkg: github.com/ethereum/go-ethereum/trie
// BenchmarkDecodeShortNode
// BenchmarkDecodeShortNode-8   	 7925638	       151.0 ns/op	     157 B/op	       4 allocs/op
func BenchmarkDecodeShortNode(b *testing.B) {
	node := &shortNode{
		Key: []byte{0x1, 0x2},
		Val: hashNode(randBytes(32)),
	}
	blob := nodeToBytes(node)
	hash := crypto.Keccak256(blob)

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		mustDecodeNode(hash, blob)
	}
}

// goos: darwin
// goarch: arm64
// pkg: github.com/ethereum/go-ethereum/trie
// BenchmarkDecodeShortNodeUnsafe
// BenchmarkDecodeShortNodeUnsafe-8   	 9027476	       128.6 ns/op	     109 B/op	       3 allocs/op
func BenchmarkDecodeShortNodeUnsafe(b *testing.B) {
	node := &shortNode{
		Key: []byte{0x1, 0x2},
		Val: hashNode(randBytes(32)),
	}
	blob := nodeToBytes(node)
	hash := crypto.Keccak256(blob)

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		mustDecodeNodeUnsafe(hash, blob)
	}
}

// goos: darwin
// goarch: arm64
// pkg: github.com/ethereum/go-ethereum/trie
// BenchmarkDecodeFullNode
// BenchmarkDecodeFullNode-8   	 1597462	       761.9 ns/op	    1280 B/op	      18 allocs/op
func BenchmarkDecodeFullNode(b *testing.B) {
	node := &fullNode{}
	for i := 0; i < 16; i++ {
		node.Children[i] = hashNode(randBytes(32))
	}
	blob := nodeToBytes(node)
	hash := crypto.Keccak256(blob)

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		mustDecodeNode(hash, blob)
	}
}

// goos: darwin
// goarch: arm64
// pkg: github.com/ethereum/go-ethereum/trie
// BenchmarkDecodeFullNodeUnsafe
// BenchmarkDecodeFullNodeUnsafe-8   	 1789070	       687.1 ns/op	     704 B/op	      17 allocs/op
func BenchmarkDecodeFullNodeUnsafe(b *testing.B) {
	node := &fullNode{}
	for i := 0; i < 16; i++ {
		node.Children[i] = hashNode(randBytes(32))
	}
	blob := nodeToBytes(node)
	hash := crypto.Keccak256(blob)

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		mustDecodeNodeUnsafe(hash, blob)
	}
}
