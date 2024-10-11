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

package discover

import (
	"bytes"
	"crypto/ecdsa"
	"encoding/binary"
	"fmt"
	"math/rand"
	"net"
	"reflect"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/internal/testlog"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/p2p/discover/v5wire"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/enr"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/stretchr/testify/require"
	"golang.org/x/exp/slices"
)

// Real sockets, real crypto: this test checks end-to-end connectivity for UDPv5.
func TestUDPv5_lookupE2E(t *testing.T) {
	t.Parallel()

	const N = 5
	var nodes []*UDPv5
	for i := 0; i < N; i++ {
		var cfg Config
		if len(nodes) > 0 {
			bn := nodes[0].Self()
			cfg.Bootnodes = []*enode.Node{bn}
		}
		node := startLocalhostV5(t, cfg)
		nodes = append(nodes, node)
		defer node.Close()
	}
	last := nodes[N-1]
	target := nodes[rand.Intn(N-2)].Self()

	// It is expected that all nodes can be found.
	expectedResult := make([]*enode.Node, len(nodes))
	for i := range nodes {
		expectedResult[i] = nodes[i].Self()
	}
	slices.SortFunc(expectedResult, func(a, b *enode.Node) int {
		return enode.DistCmp(target.ID(), a.ID(), b.ID())
	})

	// Do the lookup.
	results := last.Lookup(target.ID())
	if err := checkNodesEqual(results, expectedResult); err != nil {
		t.Fatalf("lookup returned wrong results: %v", err)
	}
}

func startLocalhostV5(t *testing.T, cfg Config) *UDPv5 {
	cfg.PrivateKey = newkey()
	db, _ := enode.OpenDB("")
	ln := enode.NewLocalNode(db, cfg.PrivateKey)

	// Prefix logs with node ID.
	lprefix := fmt.Sprintf("(%s)", ln.ID().TerminalString())
	lfmt := log.TerminalFormat(false)
	cfg.Log = testlog.Logger(t, log.LvlTrace)
	cfg.Log.SetHandler(log.FuncHandler(func(r *log.Record) error {
		t.Logf("%s %s", lprefix, lfmt.Format(r))
		return nil
	}))

	// Listen.
	socket, err := net.ListenUDP("udp4", &net.UDPAddr{IP: net.IP{127, 0, 0, 1}})
	if err != nil {
		t.Fatal(err)
	}
	realaddr := socket.LocalAddr().(*net.UDPAddr)
	ln.SetStaticIP(realaddr.IP)
	ln.Set(enr.UDP(realaddr.Port))
	udp, err := ListenV5(socket, ln, cfg)
	if err != nil {
		t.Fatal(err)
	}
	return udp
}

// This test checks that incoming PING calls are handled correctly.
func TestUDPv5_pingHandling(t *testing.T) {
	t.Parallel()
	test := newUDPV5Test(t)
	defer test.close()

	test.packetIn(&v5wire.Ping{ReqID: []byte("foo")})
	test.waitPacketOut(func(p *v5wire.Pong, addr *net.UDPAddr, _ v5wire.Nonce) {
		if !bytes.Equal(p.ReqID, []byte("foo")) {
			t.Error("wrong request ID in response:", p.ReqID)
		}
		if p.ENRSeq != test.table.self().Seq() {
			t.Error("wrong ENR sequence number in response:", p.ENRSeq)
		}
	})
}

// This test checks that incoming 'unknown' packets trigger the handshake.
func TestUDPv5_unknownPacket(t *testing.T) {
	t.Parallel()
	test := newUDPV5Test(t)
	defer test.close()

	nonce := v5wire.Nonce{1, 2, 3}
	check := func(p *v5wire.Whoareyou, wantSeq uint64) {
		t.Helper()
		if p.Nonce != nonce {
			t.Error("wrong nonce in WHOAREYOU:", p.Nonce, nonce)
		}
		if p.IDNonce == ([16]byte{}) {
			t.Error("all zero ID nonce")
		}
		if p.RecordSeq != wantSeq {
			t.Errorf("wrong record seq %d in WHOAREYOU, want %d", p.RecordSeq, wantSeq)
		}
	}

	// Unknown packet from unknown node.
	test.packetIn(&v5wire.Unknown{Nonce: nonce})
	test.waitPacketOut(func(p *v5wire.Whoareyou, addr *net.UDPAddr, _ v5wire.Nonce) {
		check(p, 0)
	})

	// Make node known.
	n := test.getNode(test.remotekey, test.remoteaddr).Node()
	test.table.addSeenNode(wrapNode(n))

	test.packetIn(&v5wire.Unknown{Nonce: nonce})
	test.waitPacketOut(func(p *v5wire.Whoareyou, addr *net.UDPAddr, _ v5wire.Nonce) {
		check(p, n.Seq())
	})
}

// This test checks that incoming FINDNODE calls are handled correctly.
func TestUDPv5_findnodeHandling(t *testing.T) {
	t.Parallel()
	test := newUDPV5Test(t)
	defer test.close()

	// Create test nodes and insert them into the table.
	nodes253 := nodesAtDistance(test.table.self().ID(), 253, 16)
	nodes249 := nodesAtDistance(test.table.self().ID(), 249, 4)
	nodes248 := nodesAtDistance(test.table.self().ID(), 248, 10)
	fillTable(test.table, wrapNodes(nodes253))
	fillTable(test.table, wrapNodes(nodes249))
	fillTable(test.table, wrapNodes(nodes248))

	// Requesting with distance zero should return the node's own record.
	test.packetIn(&v5wire.Findnode{ReqID: []byte{0}, Distances: []uint{0}})
	test.expectNodes([]byte{0}, 1, []*enode.Node{test.udp.Self()})

	// Requesting with distance > 256 shouldn't crash.
	test.packetIn(&v5wire.Findnode{ReqID: []byte{1}, Distances: []uint{4234098}})
	test.expectNodes([]byte{1}, 1, nil)

	// Requesting with empty distance list shouldn't crash either.
	test.packetIn(&v5wire.Findnode{ReqID: []byte{2}, Distances: []uint{}})
	test.expectNodes([]byte{2}, 1, nil)

	// This request gets no nodes because the corresponding bucket is empty.
	test.packetIn(&v5wire.Findnode{ReqID: []byte{3}, Distances: []uint{254}})
	test.expectNodes([]byte{3}, 1, nil)

	// This request gets all the distance-253 nodes.
	test.packetIn(&v5wire.Findnode{ReqID: []byte{4}, Distances: []uint{253}})
	test.expectNodes([]byte{4}, 2, nodes253)

	// This request gets all the distance-249 nodes and some more at 248 because
	// the bucket at 249 is not full.
	test.packetIn(&v5wire.Findnode{ReqID: []byte{5}, Distances: []uint{249, 248}})
	var nodes []*enode.Node
	nodes = append(nodes, nodes249...)
	nodes = append(nodes, nodes248[:10]...)
	test.expectNodes([]byte{5}, 1, nodes)
}

func (test *udpV5Test) expectNodes(wantReqID []byte, wantTotal uint8, wantNodes []*enode.Node) {
	nodeSet := make(map[enode.ID]*enr.Record, len(wantNodes))
	for _, n := range wantNodes {
		nodeSet[n.ID()] = n.Record()
	}

	for {
		test.waitPacketOut(func(p *v5wire.Nodes, addr *net.UDPAddr, _ v5wire.Nonce) {
			if !bytes.Equal(p.ReqID, wantReqID) {
				test.t.Fatalf("wrong request ID %v in response, want %v", p.ReqID, wantReqID)
			}
			if p.RespCount != wantTotal {
				test.t.Fatalf("wrong total response count %d, want %d", p.RespCount, wantTotal)
			}
			for _, record := range p.Nodes {
				n, _ := enode.New(enode.ValidSchemesForTesting, record)
				want := nodeSet[n.ID()]
				if want == nil {
					test.t.Fatalf("unexpected node in response: %v", n)
				}
				if !reflect.DeepEqual(record, want) {
					test.t.Fatalf("wrong record in response: %v", n)
				}
				delete(nodeSet, n.ID())
			}
		})
		if len(nodeSet) == 0 {
			return
		}
	}
}

// This test checks that outgoing PING calls work.
func TestUDPv5_pingCall(t *testing.T) {
	t.Parallel()
	test := newUDPV5Test(t)
	defer test.close()

	remote := test.getNode(test.remotekey, test.remoteaddr).Node()
	done := make(chan error, 1)

	// This ping times out.
	go func() {
		_, err := test.udp.ping(remote)
		done <- err
	}()
	test.waitPacketOut(func(p *v5wire.Ping, addr *net.UDPAddr, _ v5wire.Nonce) {})
	if err := <-done; err != errTimeout {
		t.Fatalf("want errTimeout, got %q", err)
	}

	// This ping works.
	go func() {
		_, err := test.udp.ping(remote)
		done <- err
	}()
	test.waitPacketOut(func(p *v5wire.Ping, addr *net.UDPAddr, _ v5wire.Nonce) {
		test.packetInFrom(test.remotekey, test.remoteaddr, &v5wire.Pong{ReqID: p.ReqID})
	})
	if err := <-done; err != nil {
		t.Fatal(err)
	}

	// This ping gets a reply from the wrong endpoint.
	go func() {
		_, err := test.udp.ping(remote)
		done <- err
	}()
	test.waitPacketOut(func(p *v5wire.Ping, addr *net.UDPAddr, _ v5wire.Nonce) {
		wrongAddr := &net.UDPAddr{IP: net.IP{33, 44, 55, 22}, Port: 10101}
		test.packetInFrom(test.remotekey, wrongAddr, &v5wire.Pong{ReqID: p.ReqID})
	})
	if err := <-done; err != errTimeout {
		t.Fatalf("want errTimeout for reply from wrong IP, got %q", err)
	}
}

// This test checks that outgoing FINDNODE calls work and multiple NODES
// replies are aggregated.
func TestUDPv5_findnodeCall(t *testing.T) {
	t.Parallel()
	test := newUDPV5Test(t)
	defer test.close()

	// Launch the request:
	var (
		distances = []uint{230}
		remote    = test.getNode(test.remotekey, test.remoteaddr).Node()
		nodes     = nodesAtDistance(remote.ID(), int(distances[0]), 8)
		done      = make(chan error, 1)
		response  []*enode.Node
	)
	go func() {
		var err error
		response, err = test.udp.findnode(remote, distances)
		done <- err
	}()

	// Serve the responses:
	test.waitPacketOut(func(p *v5wire.Findnode, addr *net.UDPAddr, _ v5wire.Nonce) {
		if !reflect.DeepEqual(p.Distances, distances) {
			t.Fatalf("wrong distances in request: %v", p.Distances)
		}
		test.packetIn(&v5wire.Nodes{
			ReqID:     p.ReqID,
			RespCount: 2,
			Nodes:     nodesToRecords(nodes[:4]),
		})
		test.packetIn(&v5wire.Nodes{
			ReqID:     p.ReqID,
			RespCount: 2,
			Nodes:     nodesToRecords(nodes[4:]),
		})
	})

	// Check results:
	if err := <-done; err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !reflect.DeepEqual(response, nodes) {
		t.Fatalf("wrong nodes in response")
	}

	// TODO: check invalid IPs
	// TODO: check invalid/unsigned record
}

// This test checks that pending calls are re-sent when a handshake happens.
func TestUDPv5_callResend(t *testing.T) {
	t.Parallel()
	test := newUDPV5Test(t)
	defer test.close()

	remote := test.getNode(test.remotekey, test.remoteaddr).Node()
	done := make(chan error, 2)
	go func() {
		_, err := test.udp.ping(remote)
		done <- err
	}()
	go func() {
		_, err := test.udp.ping(remote)
		done <- err
	}()

	// Ping answered by WHOAREYOU.
	test.waitPacketOut(func(p *v5wire.Ping, addr *net.UDPAddr, nonce v5wire.Nonce) {
		test.packetIn(&v5wire.Whoareyou{Nonce: nonce})
	})
	// Ping should be re-sent.
	test.waitPacketOut(func(p *v5wire.Ping, addr *net.UDPAddr, _ v5wire.Nonce) {
		test.packetIn(&v5wire.Pong{ReqID: p.ReqID})
	})
	// Answer the other ping.
	test.waitPacketOut(func(p *v5wire.Ping, addr *net.UDPAddr, _ v5wire.Nonce) {
		test.packetIn(&v5wire.Pong{ReqID: p.ReqID})
	})
	if err := <-done; err != nil {
		t.Fatalf("unexpected ping error: %v", err)
	}
	if err := <-done; err != nil {
		t.Fatalf("unexpected ping error: %v", err)
	}
}

// This test ensures we don't allow multiple rounds of WHOAREYOU for a single call.
func TestUDPv5_multipleHandshakeRounds(t *testing.T) {
	t.Parallel()
	test := newUDPV5Test(t)
	defer test.close()

	remote := test.getNode(test.remotekey, test.remoteaddr).Node()
	done := make(chan error, 1)
	go func() {
		_, err := test.udp.ping(remote)
		done <- err
	}()

	// Ping answered by WHOAREYOU.
	test.waitPacketOut(func(p *v5wire.Ping, addr *net.UDPAddr, nonce v5wire.Nonce) {
		test.packetIn(&v5wire.Whoareyou{Nonce: nonce})
	})
	// Ping answered by WHOAREYOU again.
	test.waitPacketOut(func(p *v5wire.Ping, addr *net.UDPAddr, nonce v5wire.Nonce) {
		test.packetIn(&v5wire.Whoareyou{Nonce: nonce})
	})
	if err := <-done; err != errTimeout {
		t.Fatalf("unexpected ping error: %q", err)
	}
}

// This test checks that calls with n replies may take up to n * respTimeout.
func TestUDPv5_callTimeoutReset(t *testing.T) {
	t.Parallel()
	test := newUDPV5Test(t)
	defer test.close()

	// Launch the request:
	var (
		distance = uint(230)
		remote   = test.getNode(test.remotekey, test.remoteaddr).Node()
		nodes    = nodesAtDistance(remote.ID(), int(distance), 8)
		done     = make(chan error, 1)
	)
	go func() {
		_, err := test.udp.findnode(remote, []uint{distance})
		done <- err
	}()

	// Serve two responses, slowly.
	test.waitPacketOut(func(p *v5wire.Findnode, addr *net.UDPAddr, _ v5wire.Nonce) {
		time.Sleep(respTimeout - 50*time.Millisecond)
		test.packetIn(&v5wire.Nodes{
			ReqID:     p.ReqID,
			RespCount: 2,
			Nodes:     nodesToRecords(nodes[:4]),
		})

		time.Sleep(respTimeout - 50*time.Millisecond)
		test.packetIn(&v5wire.Nodes{
			ReqID:     p.ReqID,
			RespCount: 2,
			Nodes:     nodesToRecords(nodes[4:]),
		})
	})
	if err := <-done; err != nil {
		t.Fatalf("unexpected error: %q", err)
	}
}

// This test checks that TALKREQ calls the registered handler function.
func TestUDPv5_talkHandling(t *testing.T) {
	t.Parallel()
	test := newUDPV5Test(t)
	defer test.close()

	var recvMessage []byte
	test.udp.RegisterTalkHandler("test", func(id enode.ID, addr *net.UDPAddr, message []byte) []byte {
		recvMessage = message
		return []byte("test response")
	})

	// Successful case:
	test.packetIn(&v5wire.TalkRequest{
		ReqID:    []byte("foo"),
		Protocol: "test",
		Message:  []byte("test request"),
	})
	test.waitPacketOut(func(p *v5wire.TalkResponse, addr *net.UDPAddr, _ v5wire.Nonce) {
		if !bytes.Equal(p.ReqID, []byte("foo")) {
			t.Error("wrong request ID in response:", p.ReqID)
		}
		if string(p.Message) != "test response" {
			t.Errorf("wrong talk response message: %q", p.Message)
		}
		if string(recvMessage) != "test request" {
			t.Errorf("wrong message received in handler: %q", recvMessage)
		}
	})

	// Check that empty response is returned for unregistered protocols.
	recvMessage = nil
	test.packetIn(&v5wire.TalkRequest{
		ReqID:    []byte("2"),
		Protocol: "wrong",
		Message:  []byte("test request"),
	})
	test.waitPacketOut(func(p *v5wire.TalkResponse, addr *net.UDPAddr, _ v5wire.Nonce) {
		if !bytes.Equal(p.ReqID, []byte("2")) {
			t.Error("wrong request ID in response:", p.ReqID)
		}
		if string(p.Message) != "" {
			t.Errorf("wrong talk response message: %q", p.Message)
		}
		if recvMessage != nil {
			t.Errorf("handler was called for wrong protocol: %q", recvMessage)
		}
	})
}

// This test checks that outgoing TALKREQ calls work.
func TestUDPv5_talkRequest(t *testing.T) {
	t.Parallel()
	test := newUDPV5Test(t)
	defer test.close()

	remote := test.getNode(test.remotekey, test.remoteaddr).Node()
	done := make(chan error, 1)

	// This request times out.
	go func() {
		_, err := test.udp.TalkRequest(remote, "test", []byte("test request"))
		done <- err
	}()
	test.waitPacketOut(func(p *v5wire.TalkRequest, addr *net.UDPAddr, _ v5wire.Nonce) {})
	if err := <-done; err != errTimeout {
		t.Fatalf("want errTimeout, got %q", err)
	}

	// This request works.
	go func() {
		_, err := test.udp.TalkRequest(remote, "test", []byte("test request"))
		done <- err
	}()
	test.waitPacketOut(func(p *v5wire.TalkRequest, addr *net.UDPAddr, _ v5wire.Nonce) {
		if p.Protocol != "test" {
			t.Errorf("wrong protocol ID in talk request: %q", p.Protocol)
		}
		if string(p.Message) != "test request" {
			t.Errorf("wrong message talk request: %q", p.Message)
		}
		test.packetInFrom(test.remotekey, test.remoteaddr, &v5wire.TalkResponse{
			ReqID:   p.ReqID,
			Message: []byte("test response"),
		})
	})
	if err := <-done; err != nil {
		t.Fatal(err)
	}

	// Also check requesting without ENR.
	go func() {
		_, err := test.udp.TalkRequestToID(remote.ID(), test.remoteaddr, "test", []byte("test request 2"))
		done <- err
	}()
	test.waitPacketOut(func(p *v5wire.TalkRequest, addr *net.UDPAddr, _ v5wire.Nonce) {
		if p.Protocol != "test" {
			t.Errorf("wrong protocol ID in talk request: %q", p.Protocol)
		}
		if string(p.Message) != "test request 2" {
			t.Errorf("wrong message talk request: %q", p.Message)
		}
		test.packetInFrom(test.remotekey, test.remoteaddr, &v5wire.TalkResponse{
			ReqID:   p.ReqID,
			Message: []byte("test response 2"),
		})
	})
	if err := <-done; err != nil {
		t.Fatal(err)
	}
}

// This test checks that lookupDistances works.
func TestUDPv5_lookupDistances(t *testing.T) {
	test := newUDPV5Test(t)
	lnID := test.table.self().ID()

	t.Run("target distance of 1", func(t *testing.T) {
		node := nodeAtDistance(lnID, 1, intIP(0))
		dists := lookupDistances(lnID, node.ID())
		require.Equal(t, []uint{1, 2, 3}, dists)
	})

	t.Run("target distance of 2", func(t *testing.T) {
		node := nodeAtDistance(lnID, 2, intIP(0))
		dists := lookupDistances(lnID, node.ID())
		require.Equal(t, []uint{2, 3, 1}, dists)
	})

	t.Run("target distance of 128", func(t *testing.T) {
		node := nodeAtDistance(lnID, 128, intIP(0))
		dists := lookupDistances(lnID, node.ID())
		require.Equal(t, []uint{128, 129, 127}, dists)
	})

	t.Run("target distance of 255", func(t *testing.T) {
		node := nodeAtDistance(lnID, 255, intIP(0))
		dists := lookupDistances(lnID, node.ID())
		require.Equal(t, []uint{255, 256, 254}, dists)
	})

	t.Run("target distance of 256", func(t *testing.T) {
		node := nodeAtDistance(lnID, 256, intIP(0))
		dists := lookupDistances(lnID, node.ID())
		require.Equal(t, []uint{256, 255, 254}, dists)
	})
}

// This test checks that lookup works.
func TestUDPv5_lookup(t *testing.T) {
	t.Parallel()
	test := newUDPV5Test(t)

	// Lookup on empty table returns no nodes.
	if results := test.udp.Lookup(lookupTestnet.target.id()); len(results) > 0 {
		t.Fatalf("lookup on empty table returned %d results: %#v", len(results), results)
	}

	// Ensure the tester knows all nodes in lookupTestnet by IP.
	for d, nn := range lookupTestnet.dists {
		for i, key := range nn {
			n := lookupTestnet.node(d, i)
			test.getNode(key, &net.UDPAddr{IP: n.IP(), Port: n.UDP()})
		}
	}

	// Seed table with initial node.
	initialNode := lookupTestnet.node(256, 0)
	fillTable(test.table, []*node{wrapNode(initialNode)})

	// Start the lookup.
	resultC := make(chan []*enode.Node, 1)
	go func() {
		resultC <- test.udp.Lookup(lookupTestnet.target.id())
		test.close()
	}()

	// Answer lookup packets.
	asked := make(map[enode.ID]bool)
	for done := false; !done; {
		done = test.waitPacketOut(func(p v5wire.Packet, to *net.UDPAddr, _ v5wire.Nonce) {
			recipient, key := lookupTestnet.nodeByAddr(to)
			switch p := p.(type) {
			case *v5wire.Ping:
				test.packetInFrom(key, to, &v5wire.Pong{ReqID: p.ReqID})
			case *v5wire.Findnode:
				if asked[recipient.ID()] {
					t.Error("Asked node", recipient.ID(), "twice")
				}
				asked[recipient.ID()] = true
				nodes := lookupTestnet.neighborsAtDistances(recipient, p.Distances, 16)
				t.Logf("Got FINDNODE for %v, returning %d nodes", p.Distances, len(nodes))
				for _, resp := range packNodes(p.ReqID, nodes) {
					test.packetInFrom(key, to, resp)
				}
			}
		})
	}

	// Verify result nodes.
	results := <-resultC
	checkLookupResults(t, lookupTestnet, results)
}

// This test checks the local node can be utilised to set key-values.
func TestUDPv5_LocalNode(t *testing.T) {
	t.Parallel()
	var cfg Config
	node := startLocalhostV5(t, cfg)
	defer node.Close()
	localNd := node.LocalNode()

	// set value in node's local record
	testVal := [4]byte{'A', 'B', 'C', 'D'}
	localNd.Set(enr.WithEntry("testing", &testVal))

	// retrieve the value from self to make sure it matches.
	outputVal := [4]byte{}
	if err := node.Self().Load(enr.WithEntry("testing", &outputVal)); err != nil {
		t.Errorf("Could not load value from record: %v", err)
	}
	if testVal != outputVal {
		t.Errorf("Wanted %#x to be retrieved from the record but instead got %#x", testVal, outputVal)
	}
}

func TestUDPv5_PingWithIPV4MappedAddress(t *testing.T) {
	t.Parallel()
	test := newUDPV5Test(t)
	defer test.close()

	rawIP := net.IPv4(0xFF, 0x12, 0x33, 0xE5)
	test.remoteaddr = &net.UDPAddr{
		IP:   rawIP.To16(),
		Port: 0,
	}
	remote := test.getNode(test.remotekey, test.remoteaddr).Node()
	done := make(chan struct{}, 1)

	// This handler will truncate the ipv4-mapped in ipv6 address.
	go func() {
		test.udp.handlePing(&v5wire.Ping{ENRSeq: 1}, remote.ID(), test.remoteaddr)
		done <- struct{}{}
	}()
	test.waitPacketOut(func(p *v5wire.Pong, addr *net.UDPAddr, _ v5wire.Nonce) {
		if len(p.ToIP) == net.IPv6len {
			t.Error("Received untruncated ip address")
		}
		if len(p.ToIP) != net.IPv4len {
			t.Errorf("Received ip address with incorrect length: %d", len(p.ToIP))
		}
		if !p.ToIP.Equal(rawIP) {
			t.Errorf("Received incorrect ip address: wanted %s but received %s", rawIP.String(), p.ToIP.String())
		}
	})
	<-done
}

// udpV5Test is the framework for all tests above.
// It runs the UDPv5 transport on a virtual socket and allows testing outgoing packets.
type udpV5Test struct {
	t                   *testing.T
	pipe                *dgramPipe
	table               *Table
	db                  *enode.DB
	udp                 *UDPv5
	localkey, remotekey *ecdsa.PrivateKey
	remoteaddr          *net.UDPAddr
	nodesByID           map[enode.ID]*enode.LocalNode
	nodesByIP           map[string]*enode.LocalNode
}

// testCodec is the packet encoding used by protocol tests. This codec does not perform encryption.
type testCodec struct {
	test *udpV5Test
	id   enode.ID
	ctr  uint64
}

type testCodecFrame struct {
	NodeID  enode.ID
	AuthTag v5wire.Nonce
	Ptype   byte
	Packet  rlp.RawValue
}

func (c *testCodec) Encode(toID enode.ID, addr string, p v5wire.Packet, _ *v5wire.Whoareyou) ([]byte, v5wire.Nonce, error) {
	c.ctr++
	var authTag v5wire.Nonce
	binary.BigEndian.PutUint64(authTag[:], c.ctr)

	penc, _ := rlp.EncodeToBytes(p)
	frame, err := rlp.EncodeToBytes(testCodecFrame{c.id, authTag, p.Kind(), penc})
	return frame, authTag, err
}

func (c *testCodec) Decode(input []byte, addr string) (enode.ID, *enode.Node, v5wire.Packet, error) {
	frame, p, err := c.decodeFrame(input)
	if err != nil {
		return enode.ID{}, nil, nil, err
	}
	return frame.NodeID, nil, p, nil
}

func (c *testCodec) decodeFrame(input []byte) (frame testCodecFrame, p v5wire.Packet, err error) {
	if err = rlp.DecodeBytes(input, &frame); err != nil {
		return frame, nil, fmt.Errorf("invalid frame: %v", err)
	}
	switch frame.Ptype {
	case v5wire.UnknownPacket:
		dec := new(v5wire.Unknown)
		err = rlp.DecodeBytes(frame.Packet, &dec)
		p = dec
	case v5wire.WhoareyouPacket:
		dec := new(v5wire.Whoareyou)
		err = rlp.DecodeBytes(frame.Packet, &dec)
		p = dec
	default:
		p, err = v5wire.DecodeMessage(frame.Ptype, frame.Packet)
	}
	return frame, p, err
}

func newUDPV5Test(t *testing.T) *udpV5Test {
	test := &udpV5Test{
		t:          t,
		pipe:       newpipe(),
		localkey:   newkey(),
		remotekey:  newkey(),
		remoteaddr: &net.UDPAddr{IP: net.IP{10, 0, 1, 99}, Port: 30303},
		nodesByID:  make(map[enode.ID]*enode.LocalNode),
		nodesByIP:  make(map[string]*enode.LocalNode),
	}
	test.db, _ = enode.OpenDB("")
	ln := enode.NewLocalNode(test.db, test.localkey)
	ln.SetStaticIP(net.IP{10, 0, 0, 1})
	ln.Set(enr.UDP(30303))
	test.udp, _ = ListenV5(test.pipe, ln, Config{
		PrivateKey:   test.localkey,
		Log:          testlog.Logger(t, log.LvlTrace),
		ValidSchemes: enode.ValidSchemesForTesting,
	})
	test.udp.codec = &testCodec{test: test, id: ln.ID()}
	test.table = test.udp.tab
	test.nodesByID[ln.ID()] = ln
	// Wait for initial refresh so the table doesn't send unexpected findnode.
	<-test.table.initDone
	return test
}

// handles a packet as if it had been sent to the transport.
func (test *udpV5Test) packetIn(packet v5wire.Packet) {
	test.t.Helper()
	test.packetInFrom(test.remotekey, test.remoteaddr, packet)
}

// handles a packet as if it had been sent to the transport by the key/endpoint.
func (test *udpV5Test) packetInFrom(key *ecdsa.PrivateKey, addr *net.UDPAddr, packet v5wire.Packet) {
	test.t.Helper()

	ln := test.getNode(key, addr)
	codec := &testCodec{test: test, id: ln.ID()}
	enc, _, err := codec.Encode(test.udp.Self().ID(), addr.String(), packet, nil)
	if err != nil {
		test.t.Errorf("%s encode error: %v", packet.Name(), err)
	}
	if test.udp.dispatchReadPacket(addr, enc) {
		<-test.udp.readNextCh // unblock UDPv5.dispatch
	}
}

// getNode ensures the test knows about a node at the given endpoint.
func (test *udpV5Test) getNode(key *ecdsa.PrivateKey, addr *net.UDPAddr) *enode.LocalNode {
	id := encodePubkey(&key.PublicKey).id()
	ln := test.nodesByID[id]
	if ln == nil {
		db, _ := enode.OpenDB("")
		ln = enode.NewLocalNode(db, key)
		ln.SetStaticIP(addr.IP)
		ln.Set(enr.UDP(addr.Port))
		test.nodesByID[id] = ln
	}
	test.nodesByIP[string(addr.IP)] = ln
	return ln
}

// waitPacketOut waits for the next output packet and handles it using the given 'validate'
// function. The function must be of type func (X, *net.UDPAddr, v5wire.Nonce) where X is
// assignable to packetV5.
func (test *udpV5Test) waitPacketOut(validate interface{}) (closed bool) {
	test.t.Helper()

	fn := reflect.ValueOf(validate)
	exptype := fn.Type().In(0)

	dgram, err := test.pipe.receive()
	if err == errClosed {
		return true
	}
	if err == errTimeout {
		test.t.Fatalf("timed out waiting for %v", exptype)
		return false
	}
	ln := test.nodesByIP[string(dgram.to.IP)]
	if ln == nil {
		test.t.Fatalf("attempt to send to non-existing node %v", &dgram.to)
		return false
	}
	codec := &testCodec{test: test, id: ln.ID()}
	frame, p, err := codec.decodeFrame(dgram.data)
	if err != nil {
		test.t.Errorf("sent packet decode error: %v", err)
		return false
	}
	if !reflect.TypeOf(p).AssignableTo(exptype) {
		test.t.Errorf("sent packet type mismatch, got: %v, want: %v", reflect.TypeOf(p), exptype)
		return false
	}
	fn.Call([]reflect.Value{reflect.ValueOf(p), reflect.ValueOf(&dgram.to), reflect.ValueOf(frame.AuthTag)})
	return false
}

func (test *udpV5Test) close() {
	test.t.Helper()

	test.udp.Close()
	test.db.Close()
	for id, n := range test.nodesByID {
		if id != test.udp.Self().ID() {
			n.Database().Close()
		}
	}
	if len(test.pipe.queue) != 0 {
		test.t.Fatalf("%d unmatched UDP packets in queue", len(test.pipe.queue))
	}
}
