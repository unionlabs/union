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

package discover

import (
	"crypto/ecdsa"
	"fmt"
	"net"
	"testing"

	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/p2p/discover/v4wire"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/enr"
	"golang.org/x/exp/slices"
)

func TestUDPv4_Lookup(t *testing.T) {
	t.Parallel()
	test := newUDPTest(t)

	// Lookup on empty table returns no nodes.
	targetKey, _ := decodePubkey(crypto.S256(), lookupTestnet.target[:])
	if results := test.udp.LookupPubkey(targetKey); len(results) > 0 {
		t.Fatalf("lookup on empty table returned %d results: %#v", len(results), results)
	}

	// Seed table with initial node.
	fillTable(test.table, []*node{wrapNode(lookupTestnet.node(256, 0))})

	// Start the lookup.
	resultC := make(chan []*enode.Node, 1)
	go func() {
		resultC <- test.udp.LookupPubkey(targetKey)
		test.close()
	}()

	// Answer lookup packets.
	serveTestnet(test, lookupTestnet)

	// Verify result nodes.
	results := <-resultC
	t.Logf("results:")
	for _, e := range results {
		t.Logf("  ld=%d, %x", enode.LogDist(lookupTestnet.target.id(), e.ID()), e.ID().Bytes())
	}
	if len(results) != bucketSize {
		t.Errorf("wrong number of results: got %d, want %d", len(results), bucketSize)
	}
	checkLookupResults(t, lookupTestnet, results)
}

func TestUDPv4_LookupIterator(t *testing.T) {
	t.Parallel()
	test := newUDPTest(t)
	defer test.close()

	// Seed table with initial nodes.
	bootnodes := make([]*node, len(lookupTestnet.dists[256]))
	for i := range lookupTestnet.dists[256] {
		bootnodes[i] = wrapNode(lookupTestnet.node(256, i))
	}
	fillTable(test.table, bootnodes)
	go serveTestnet(test, lookupTestnet)

	// Create the iterator and collect the nodes it yields.
	iter := test.udp.RandomNodes()
	seen := make(map[enode.ID]*enode.Node)
	for limit := lookupTestnet.len(); iter.Next() && len(seen) < limit; {
		seen[iter.Node().ID()] = iter.Node()
	}
	iter.Close()

	// Check that all nodes in lookupTestnet were seen by the iterator.
	results := make([]*enode.Node, 0, len(seen))
	for _, n := range seen {
		results = append(results, n)
	}
	sortByID(results)
	want := lookupTestnet.nodes()
	if err := checkNodesEqual(results, want); err != nil {
		t.Fatal(err)
	}
}

// TestUDPv4_LookupIteratorClose checks that lookupIterator ends when its Close
// method is called.
func TestUDPv4_LookupIteratorClose(t *testing.T) {
	t.Parallel()
	test := newUDPTest(t)
	defer test.close()

	// Seed table with initial nodes.
	bootnodes := make([]*node, len(lookupTestnet.dists[256]))
	for i := range lookupTestnet.dists[256] {
		bootnodes[i] = wrapNode(lookupTestnet.node(256, i))
	}
	fillTable(test.table, bootnodes)
	go serveTestnet(test, lookupTestnet)

	it := test.udp.RandomNodes()
	if ok := it.Next(); !ok || it.Node() == nil {
		t.Fatalf("iterator didn't return any node")
	}

	it.Close()

	ncalls := 0
	for ; ncalls < 100 && it.Next(); ncalls++ {
		if it.Node() == nil {
			t.Error("iterator returned Node() == nil node after Next() == true")
		}
	}
	t.Logf("iterator returned %d nodes after close", ncalls)
	if it.Next() {
		t.Errorf("Next() == true after close and %d more calls", ncalls)
	}
	if n := it.Node(); n != nil {
		t.Errorf("iterator returned non-nil node after close and %d more calls", ncalls)
	}
}

func serveTestnet(test *udpTest, testnet *preminedTestnet) {
	for done := false; !done; {
		done = test.waitPacketOut(func(p v4wire.Packet, to *net.UDPAddr, hash []byte) {
			n, key := testnet.nodeByAddr(to)
			switch p.(type) {
			case *v4wire.Ping:
				test.packetInFrom(nil, key, to, &v4wire.Pong{Expiration: futureExp, ReplyTok: hash})
			case *v4wire.Findnode:
				dist := enode.LogDist(n.ID(), testnet.target.id())
				nodes := testnet.nodesAtDistance(dist - 1)
				test.packetInFrom(nil, key, to, &v4wire.Neighbors{Expiration: futureExp, Nodes: nodes})
			}
		})
	}
}

// checkLookupResults verifies that the results of a lookup are the closest nodes to
// the testnet's target.
func checkLookupResults(t *testing.T, tn *preminedTestnet, results []*enode.Node) {
	t.Helper()
	t.Logf("results:")
	for _, e := range results {
		t.Logf("  ld=%d, %x", enode.LogDist(tn.target.id(), e.ID()), e.ID().Bytes())
	}
	if hasDuplicates(wrapNodes(results)) {
		t.Errorf("result set contains duplicate entries")
	}
	if !sortedByDistanceTo(tn.target.id(), wrapNodes(results)) {
		t.Errorf("result set not sorted by distance to target")
	}
	wantNodes := tn.closest(len(results))
	if err := checkNodesEqual(results, wantNodes); err != nil {
		t.Error(err)
	}
}

// This is the test network for the Lookup test.
// The nodes were obtained by running lookupTestnet.mine with a random NodeID as target.
var lookupTestnet = &preminedTestnet{
	target: hexEncPubkey("5d485bdcbe9bc89314a10ae9231e429d33853e3a8fa2af39f5f827370a2e4185e344ace5d16237491dad41f278f1d3785210d29ace76cd627b9147ee340b1125"),
	dists: [257][]*ecdsa.PrivateKey{
		251: {
			hexEncPrivkey("29738ba0c1a4397d6a65f292eee07f02df8e58d41594ba2be3cf84ce0fc58169"),
			hexEncPrivkey("511b1686e4e58a917f7f848e9bf5539d206a68f5ad6b54b552c2399fe7d174ae"),
			hexEncPrivkey("d09e5eaeec0fd596236faed210e55ef45112409a5aa7f3276d26646080dcfaeb"),
			hexEncPrivkey("c1e20dbbf0d530e50573bd0a260b32ec15eb9190032b4633d44834afc8afe578"),
			hexEncPrivkey("ed5f38f5702d92d306143e5d9154fb21819777da39af325ea359f453d179e80b"),
		},
		252: {
			hexEncPrivkey("1c9b1cafbec00848d2c174b858219914b42a7d5c9359b1ca03fd650e8239ae94"),
			hexEncPrivkey("e0e1e8db4a6f13c1ffdd3e96b72fa7012293ced187c9dcdcb9ba2af37a46fa10"),
			hexEncPrivkey("3d53823e0a0295cb09f3e11d16c1b44d07dd37cec6f739b8df3a590189fe9fb9"),
		},
		253: {
			hexEncPrivkey("2d0511ae9bf590166597eeab86b6f27b1ab761761eaea8965487b162f8703847"),
			hexEncPrivkey("6cfbd7b8503073fc3dbdb746a7c672571648d3bd15197ccf7f7fef3d904f53a2"),
			hexEncPrivkey("a30599b12827b69120633f15b98a7f6bc9fc2e9a0fd6ae2ebb767c0e64d743ab"),
			hexEncPrivkey("14a98db9b46a831d67eff29f3b85b1b485bb12ae9796aea98d91be3dc78d8a91"),
			hexEncPrivkey("2369ff1fc1ff8ca7d20b17e2673adc3365c3674377f21c5d9dafaff21fe12e24"),
			hexEncPrivkey("9ae91101d6b5048607f41ec0f690ef5d09507928aded2410aabd9237aa2727d7"),
			hexEncPrivkey("05e3c59090a3fd1ae697c09c574a36fcf9bedd0afa8fe3946f21117319ca4973"),
			hexEncPrivkey("06f31c5ea632658f718a91a1b1b9ae4b7549d7b3bc61cbc2be5f4a439039f3ad"),
		},
		254: {
			hexEncPrivkey("dec742079ec00ff4ec1284d7905bc3de2366f67a0769431fd16f80fd68c58a7c"),
			hexEncPrivkey("ff02c8861fa12fbd129d2a95ea663492ef9c1e51de19dcfbbfe1c59894a28d2b"),
			hexEncPrivkey("4dded9e4eefcbce4262be4fd9e8a773670ab0b5f448f286ec97dfc8cf681444a"),
			hexEncPrivkey("750d931e2a8baa2c9268cb46b7cd851f4198018bed22f4dceb09dd334a2395f6"),
			hexEncPrivkey("ce1435a956a98ffec484cd11489c4f165cf1606819ab6b521cee440f0c677e9e"),
			hexEncPrivkey("996e7f8d1638be92d7328b4770f47e5420fc4bafecb4324fd33b1f5d9f403a75"),
			hexEncPrivkey("ebdc44e77a6cc0eb622e58cf3bb903c3da4c91ca75b447b0168505d8fc308b9c"),
			hexEncPrivkey("46bd1eddcf6431bea66fc19ebc45df191c1c7d6ed552dcdc7392885009c322f0"),
		},
		255: {
			hexEncPrivkey("da8645f90826e57228d9ea72aff84500060ad111a5d62e4af831ed8e4b5acfb8"),
			hexEncPrivkey("3c944c5d9af51d4c1d43f5d0f3a1a7ef65d5e82744d669b58b5fed242941a566"),
			hexEncPrivkey("5ebcde76f1d579eebf6e43b0ffe9157e65ffaa391175d5b9aa988f47df3e33da"),
			hexEncPrivkey("97f78253a7d1d796e4eaabce721febcc4550dd68fb11cc818378ba807a2cb7de"),
			hexEncPrivkey("a38cd7dc9b4079d1c0406afd0fdb1165c285f2c44f946eca96fc67772c988c7d"),
			hexEncPrivkey("d64cbb3ffdf712c372b7a22a176308ef8f91861398d5dbaf326fd89c6eaeef1c"),
			hexEncPrivkey("d269609743ef29d6446e3355ec647e38d919c82a4eb5837e442efd7f4218944f"),
			hexEncPrivkey("d8f7bcc4a530efde1d143717007179e0d9ace405ddaaf151c4d863753b7fd64c"),
		},
		256: {
			hexEncPrivkey("8c5b422155d33ea8e9d46f71d1ad3e7b24cb40051413ffa1a81cff613d243ba9"),
			hexEncPrivkey("937b1af801def4e8f5a3a8bd225a8bcff1db764e41d3e177f2e9376e8dd87233"),
			hexEncPrivkey("120260dce739b6f71f171da6f65bc361b5fad51db74cf02d3e973347819a6518"),
			hexEncPrivkey("1fa56cf25d4b46c2bf94e82355aa631717b63190785ac6bae545a88aadc304a9"),
			hexEncPrivkey("3c38c503c0376f9b4adcbe935d5f4b890391741c764f61b03cd4d0d42deae002"),
			hexEncPrivkey("3a54af3e9fa162bc8623cdf3e5d9b70bf30ade1d54cc3abea8659aba6cff471f"),
			hexEncPrivkey("6799a02ea1999aefdcbcc4d3ff9544478be7365a328d0d0f37c26bd95ade0cda"),
			hexEncPrivkey("e24a7bc9051058f918646b0f6e3d16884b2a55a15553b89bab910d55ebc36116"),
		},
	},
}

type preminedTestnet struct {
	target encPubkey
	dists  [hashBits + 1][]*ecdsa.PrivateKey
}

func (tn *preminedTestnet) len() int {
	n := 0
	for _, keys := range tn.dists {
		n += len(keys)
	}
	return n
}

func (tn *preminedTestnet) nodes() []*enode.Node {
	result := make([]*enode.Node, 0, tn.len())
	for dist, keys := range tn.dists {
		for index := range keys {
			result = append(result, tn.node(dist, index))
		}
	}
	sortByID(result)
	return result
}

func (tn *preminedTestnet) node(dist, index int) *enode.Node {
	key := tn.dists[dist][index]
	rec := new(enr.Record)
	rec.Set(enr.IP{127, byte(dist >> 8), byte(dist), byte(index)})
	rec.Set(enr.UDP(5000))
	enode.SignV4(rec, key)
	n, _ := enode.New(enode.ValidSchemes, rec)
	return n
}

func (tn *preminedTestnet) nodeByAddr(addr *net.UDPAddr) (*enode.Node, *ecdsa.PrivateKey) {
	dist := int(addr.IP[1])<<8 + int(addr.IP[2])
	index := int(addr.IP[3])
	key := tn.dists[dist][index]
	return tn.node(dist, index), key
}

func (tn *preminedTestnet) nodesAtDistance(dist int) []v4wire.Node {
	result := make([]v4wire.Node, len(tn.dists[dist]))
	for i := range result {
		result[i] = nodeToRPC(wrapNode(tn.node(dist, i)))
	}
	return result
}

func (tn *preminedTestnet) neighborsAtDistances(base *enode.Node, distances []uint, elems int) []*enode.Node {
	var result []*enode.Node
	for d := range lookupTestnet.dists {
		for i := range lookupTestnet.dists[d] {
			n := lookupTestnet.node(d, i)
			d := enode.LogDist(base.ID(), n.ID())
			if containsUint(uint(d), distances) {
				result = append(result, n)
				if len(result) >= elems {
					return result
				}
			}
		}
	}
	return result
}

func (tn *preminedTestnet) closest(n int) (nodes []*enode.Node) {
	for d := range tn.dists {
		for i := range tn.dists[d] {
			nodes = append(nodes, tn.node(d, i))
		}
	}
	slices.SortFunc(nodes, func(a, b *enode.Node) int {
		return enode.DistCmp(tn.target.id(), a.ID(), b.ID())
	})
	return nodes[:n]
}

var _ = (*preminedTestnet).mine // avoid linter warning about mine being dead code.

// mine generates a testnet struct literal with nodes at
// various distances to the network's target.
func (tn *preminedTestnet) mine() {
	// Clear existing slices first (useful when re-mining).
	for i := range tn.dists {
		tn.dists[i] = nil
	}

	targetSha := tn.target.id()
	found, need := 0, 40
	for found < need {
		k := newkey()
		ld := enode.LogDist(targetSha, encodePubkey(&k.PublicKey).id())
		if len(tn.dists[ld]) < 8 {
			tn.dists[ld] = append(tn.dists[ld], k)
			found++
			fmt.Printf("found ID with ld %d (%d/%d)\n", ld, found, need)
		}
	}
	fmt.Printf("&preminedTestnet{\n")
	fmt.Printf("	target: hexEncPubkey(\"%x\"),\n", tn.target[:])
	fmt.Printf("	dists: [%d][]*ecdsa.PrivateKey{\n", len(tn.dists))
	for ld, ns := range tn.dists {
		if len(ns) == 0 {
			continue
		}
		fmt.Printf("		%d: {\n", ld)
		for _, key := range ns {
			fmt.Printf("			hexEncPrivkey(\"%x\"),\n", crypto.FromECDSA(key))
		}
		fmt.Printf("		},\n")
	}
	fmt.Printf("	},\n")
	fmt.Printf("}\n")
}
