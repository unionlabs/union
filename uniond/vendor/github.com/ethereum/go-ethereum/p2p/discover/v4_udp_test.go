// Copyright 2015 The go-ethereum Authors
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
	crand "crypto/rand"
	"encoding/binary"
	"errors"
	"fmt"
	"io"
	"math/rand"
	"net"
	"reflect"
	"sync"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/internal/testlog"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/p2p/discover/v4wire"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/enr"
)

// shared test variables
var (
	futureExp          = uint64(time.Now().Add(10 * time.Hour).Unix())
	testTarget         = v4wire.Pubkey{0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1}
	testRemote         = v4wire.Endpoint{IP: net.ParseIP("1.1.1.1").To4(), UDP: 1, TCP: 2}
	testLocalAnnounced = v4wire.Endpoint{IP: net.ParseIP("2.2.2.2").To4(), UDP: 3, TCP: 4}
	testLocal          = v4wire.Endpoint{IP: net.ParseIP("3.3.3.3").To4(), UDP: 5, TCP: 6}
)

type udpTest struct {
	t                   *testing.T
	pipe                *dgramPipe
	table               *Table
	db                  *enode.DB
	udp                 *UDPv4
	sent                [][]byte
	localkey, remotekey *ecdsa.PrivateKey
	remoteaddr          *net.UDPAddr
}

func newUDPTest(t *testing.T) *udpTest {
	test := &udpTest{
		t:          t,
		pipe:       newpipe(),
		localkey:   newkey(),
		remotekey:  newkey(),
		remoteaddr: &net.UDPAddr{IP: net.IP{10, 0, 1, 99}, Port: 30303},
	}

	test.db, _ = enode.OpenDB("")
	ln := enode.NewLocalNode(test.db, test.localkey)
	test.udp, _ = ListenV4(test.pipe, ln, Config{
		PrivateKey: test.localkey,
		Log:        testlog.Logger(t, log.LvlTrace),
	})
	test.table = test.udp.tab
	// Wait for initial refresh so the table doesn't send unexpected findnode.
	<-test.table.initDone
	return test
}

func (test *udpTest) close() {
	test.udp.Close()
	test.db.Close()
}

// handles a packet as if it had been sent to the transport.
func (test *udpTest) packetIn(wantError error, data v4wire.Packet) {
	test.t.Helper()

	test.packetInFrom(wantError, test.remotekey, test.remoteaddr, data)
}

// handles a packet as if it had been sent to the transport by the key/endpoint.
func (test *udpTest) packetInFrom(wantError error, key *ecdsa.PrivateKey, addr *net.UDPAddr, data v4wire.Packet) {
	test.t.Helper()

	enc, _, err := v4wire.Encode(key, data)
	if err != nil {
		test.t.Errorf("%s encode error: %v", data.Name(), err)
	}
	test.sent = append(test.sent, enc)
	if err = test.udp.handlePacket(addr, enc); err != wantError {
		test.t.Errorf("error mismatch: got %q, want %q", err, wantError)
	}
}

// waits for a packet to be sent by the transport.
// validate should have type func(X, *net.UDPAddr, []byte), where X is a packet type.
func (test *udpTest) waitPacketOut(validate interface{}) (closed bool) {
	test.t.Helper()

	dgram, err := test.pipe.receive()
	if err == errClosed {
		return true
	} else if err != nil {
		test.t.Error("packet receive error:", err)
		return false
	}
	p, _, hash, err := v4wire.Decode(dgram.data)
	if err != nil {
		test.t.Errorf("sent packet decode error: %v", err)
		return false
	}
	fn := reflect.ValueOf(validate)
	exptype := fn.Type().In(0)
	if !reflect.TypeOf(p).AssignableTo(exptype) {
		test.t.Errorf("sent packet type mismatch, got: %v, want: %v", reflect.TypeOf(p), exptype)
		return false
	}
	fn.Call([]reflect.Value{reflect.ValueOf(p), reflect.ValueOf(&dgram.to), reflect.ValueOf(hash)})
	return false
}

func TestUDPv4_packetErrors(t *testing.T) {
	test := newUDPTest(t)
	defer test.close()

	test.packetIn(errExpired, &v4wire.Ping{From: testRemote, To: testLocalAnnounced, Version: 4})
	test.packetIn(errUnsolicitedReply, &v4wire.Pong{ReplyTok: []byte{}, Expiration: futureExp})
	test.packetIn(errUnknownNode, &v4wire.Findnode{Expiration: futureExp})
	test.packetIn(errUnsolicitedReply, &v4wire.Neighbors{Expiration: futureExp})
}

func TestUDPv4_pingTimeout(t *testing.T) {
	t.Parallel()
	test := newUDPTest(t)
	defer test.close()

	key := newkey()
	toaddr := &net.UDPAddr{IP: net.ParseIP("1.2.3.4"), Port: 2222}
	node := enode.NewV4(&key.PublicKey, toaddr.IP, 0, toaddr.Port)
	if _, err := test.udp.ping(node); err != errTimeout {
		t.Error("expected timeout error, got", err)
	}
}

type testPacket byte

func (req testPacket) Kind() byte   { return byte(req) }
func (req testPacket) Name() string { return "" }

func TestUDPv4_responseTimeouts(t *testing.T) {
	t.Parallel()
	test := newUDPTest(t)
	defer test.close()

	randomDuration := func(max time.Duration) time.Duration {
		return time.Duration(rand.Int63n(int64(max)))
	}

	var (
		nReqs      = 200
		nTimeouts  = 0                       // number of requests with ptype > 128
		nilErr     = make(chan error, nReqs) // for requests that get a reply
		timeoutErr = make(chan error, nReqs) // for requests that time out
	)
	for i := 0; i < nReqs; i++ {
		// Create a matcher for a random request in udp.loop. Requests
		// with ptype <= 128 will not get a reply and should time out.
		// For all other requests, a reply is scheduled to arrive
		// within the timeout window.
		p := &replyMatcher{
			ptype:    byte(rand.Intn(255)),
			callback: func(v4wire.Packet) (bool, bool) { return true, true },
		}
		binary.BigEndian.PutUint64(p.from[:], uint64(i))
		if p.ptype <= 128 {
			p.errc = timeoutErr
			test.udp.addReplyMatcher <- p
			nTimeouts++
		} else {
			p.errc = nilErr
			test.udp.addReplyMatcher <- p
			time.AfterFunc(randomDuration(60*time.Millisecond), func() {
				if !test.udp.handleReply(p.from, p.ip, testPacket(p.ptype)) {
					t.Logf("not matched: %v", p)
				}
			})
		}
		time.Sleep(randomDuration(30 * time.Millisecond))
	}

	// Check that all timeouts were delivered and that the rest got nil errors.
	// The replies must be delivered.
	var (
		recvDeadline        = time.After(20 * time.Second)
		nTimeoutsRecv, nNil = 0, 0
	)
	for i := 0; i < nReqs; i++ {
		select {
		case err := <-timeoutErr:
			if err != errTimeout {
				t.Fatalf("got non-timeout error on timeoutErr %d: %v", i, err)
			}
			nTimeoutsRecv++
		case err := <-nilErr:
			if err != nil {
				t.Fatalf("got non-nil error on nilErr %d: %v", i, err)
			}
			nNil++
		case <-recvDeadline:
			t.Fatalf("exceeded recv deadline")
		}
	}
	if nTimeoutsRecv != nTimeouts {
		t.Errorf("wrong number of timeout errors received: got %d, want %d", nTimeoutsRecv, nTimeouts)
	}
	if nNil != nReqs-nTimeouts {
		t.Errorf("wrong number of successful replies: got %d, want %d", nNil, nReqs-nTimeouts)
	}
}

func TestUDPv4_findnodeTimeout(t *testing.T) {
	t.Parallel()
	test := newUDPTest(t)
	defer test.close()

	toaddr := &net.UDPAddr{IP: net.ParseIP("1.2.3.4"), Port: 2222}
	toid := enode.ID{1, 2, 3, 4}
	target := v4wire.Pubkey{4, 5, 6, 7}
	result, err := test.udp.findnode(toid, toaddr, target)
	if err != errTimeout {
		t.Error("expected timeout error, got", err)
	}
	if len(result) > 0 {
		t.Error("expected empty result, got", result)
	}
}

func TestUDPv4_findnode(t *testing.T) {
	test := newUDPTest(t)
	defer test.close()

	// put a few nodes into the table. their exact
	// distribution shouldn't matter much, although we need to
	// take care not to overflow any bucket.
	nodes := &nodesByDistance{target: testTarget.ID()}
	live := make(map[enode.ID]bool)
	numCandidates := 2 * bucketSize
	for i := 0; i < numCandidates; i++ {
		key := newkey()
		ip := net.IP{10, 13, 0, byte(i)}
		n := wrapNode(enode.NewV4(&key.PublicKey, ip, 0, 2000))
		// Ensure half of table content isn't verified live yet.
		if i > numCandidates/2 {
			n.livenessChecks = 1
			live[n.ID()] = true
		}
		nodes.push(n, numCandidates)
	}
	fillTable(test.table, nodes.entries)

	// ensure there's a bond with the test node,
	// findnode won't be accepted otherwise.
	remoteID := v4wire.EncodePubkey(&test.remotekey.PublicKey).ID()
	test.table.db.UpdateLastPongReceived(remoteID, test.remoteaddr.IP, time.Now())

	// check that closest neighbors are returned.
	expected := test.table.findnodeByID(testTarget.ID(), bucketSize, true)
	test.packetIn(nil, &v4wire.Findnode{Target: testTarget, Expiration: futureExp})
	waitNeighbors := func(want []*node) {
		test.waitPacketOut(func(p *v4wire.Neighbors, to *net.UDPAddr, hash []byte) {
			if len(p.Nodes) != len(want) {
				t.Errorf("wrong number of results: got %d, want %d", len(p.Nodes), bucketSize)
				return
			}
			for i, n := range p.Nodes {
				if n.ID.ID() != want[i].ID() {
					t.Errorf("result mismatch at %d:\n  got:  %v\n  want: %v", i, n, expected.entries[i])
				}
				if !live[n.ID.ID()] {
					t.Errorf("result includes dead node %v", n.ID.ID())
				}
			}
		})
	}
	// Receive replies.
	want := expected.entries
	if len(want) > v4wire.MaxNeighbors {
		waitNeighbors(want[:v4wire.MaxNeighbors])
		want = want[v4wire.MaxNeighbors:]
	}
	waitNeighbors(want)
}

func TestUDPv4_findnodeMultiReply(t *testing.T) {
	test := newUDPTest(t)
	defer test.close()

	rid := enode.PubkeyToIDV4(&test.remotekey.PublicKey)
	test.table.db.UpdateLastPingReceived(rid, test.remoteaddr.IP, time.Now())

	// queue a pending findnode request
	resultc, errc := make(chan []*node, 1), make(chan error, 1)
	go func() {
		rid := encodePubkey(&test.remotekey.PublicKey).id()
		ns, err := test.udp.findnode(rid, test.remoteaddr, testTarget)
		if err != nil && len(ns) == 0 {
			errc <- err
		} else {
			resultc <- ns
		}
	}()

	// wait for the findnode to be sent.
	// after it is sent, the transport is waiting for a reply
	test.waitPacketOut(func(p *v4wire.Findnode, to *net.UDPAddr, hash []byte) {
		if p.Target != testTarget {
			t.Errorf("wrong target: got %v, want %v", p.Target, testTarget)
		}
	})

	// send the reply as two packets.
	list := []*node{
		wrapNode(enode.MustParse("enode://ba85011c70bcc5c04d8607d3a0ed29aa6179c092cbdda10d5d32684fb33ed01bd94f588ca8f91ac48318087dcb02eaf36773a7a453f0eedd6742af668097b29c@10.0.1.16:30303?discport=30304")),
		wrapNode(enode.MustParse("enode://81fa361d25f157cd421c60dcc28d8dac5ef6a89476633339c5df30287474520caca09627da18543d9079b5b288698b542d56167aa5c09111e55acdbbdf2ef799@10.0.1.16:30303")),
		wrapNode(enode.MustParse("enode://9bffefd833d53fac8e652415f4973bee289e8b1a5c6c4cbe70abf817ce8a64cee11b823b66a987f51aaa9fba0d6a91b3e6bf0d5a5d1042de8e9eeea057b217f8@10.0.1.36:30301?discport=17")),
		wrapNode(enode.MustParse("enode://1b5b4aa662d7cb44a7221bfba67302590b643028197a7d5214790f3bac7aaa4a3241be9e83c09cf1f6c69d007c634faae3dc1b1221793e8446c0b3a09de65960@10.0.1.16:30303")),
	}
	rpclist := make([]v4wire.Node, len(list))
	for i := range list {
		rpclist[i] = nodeToRPC(list[i])
	}
	test.packetIn(nil, &v4wire.Neighbors{Expiration: futureExp, Nodes: rpclist[:2]})
	test.packetIn(nil, &v4wire.Neighbors{Expiration: futureExp, Nodes: rpclist[2:]})

	// check that the sent neighbors are all returned by findnode
	select {
	case result := <-resultc:
		want := append(list[:2], list[3:]...)
		if !reflect.DeepEqual(result, want) {
			t.Errorf("neighbors mismatch:\n  got:  %v\n  want: %v", result, want)
		}
	case err := <-errc:
		t.Errorf("findnode error: %v", err)
	case <-time.After(5 * time.Second):
		t.Error("findnode did not return within 5 seconds")
	}
}

// This test checks that reply matching of pong verifies the ping hash.
func TestUDPv4_pingMatch(t *testing.T) {
	test := newUDPTest(t)
	defer test.close()

	randToken := make([]byte, 32)
	crand.Read(randToken)

	test.packetIn(nil, &v4wire.Ping{From: testRemote, To: testLocalAnnounced, Version: 4, Expiration: futureExp})
	test.waitPacketOut(func(*v4wire.Pong, *net.UDPAddr, []byte) {})
	test.waitPacketOut(func(*v4wire.Ping, *net.UDPAddr, []byte) {})
	test.packetIn(errUnsolicitedReply, &v4wire.Pong{ReplyTok: randToken, To: testLocalAnnounced, Expiration: futureExp})
}

// This test checks that reply matching of pong verifies the sender IP address.
func TestUDPv4_pingMatchIP(t *testing.T) {
	test := newUDPTest(t)
	defer test.close()

	test.packetIn(nil, &v4wire.Ping{From: testRemote, To: testLocalAnnounced, Version: 4, Expiration: futureExp})
	test.waitPacketOut(func(*v4wire.Pong, *net.UDPAddr, []byte) {})

	test.waitPacketOut(func(p *v4wire.Ping, to *net.UDPAddr, hash []byte) {
		wrongAddr := &net.UDPAddr{IP: net.IP{33, 44, 1, 2}, Port: 30000}
		test.packetInFrom(errUnsolicitedReply, test.remotekey, wrongAddr, &v4wire.Pong{
			ReplyTok:   hash,
			To:         testLocalAnnounced,
			Expiration: futureExp,
		})
	})
}

func TestUDPv4_successfulPing(t *testing.T) {
	test := newUDPTest(t)
	added := make(chan *node, 1)
	test.table.nodeAddedHook = func(b *bucket, n *node) { added <- n }
	defer test.close()

	// The remote side sends a ping packet to initiate the exchange.
	go test.packetIn(nil, &v4wire.Ping{From: testRemote, To: testLocalAnnounced, Version: 4, Expiration: futureExp})

	// The ping is replied to.
	test.waitPacketOut(func(p *v4wire.Pong, to *net.UDPAddr, hash []byte) {
		pinghash := test.sent[0][:32]
		if !bytes.Equal(p.ReplyTok, pinghash) {
			t.Errorf("got pong.ReplyTok %x, want %x", p.ReplyTok, pinghash)
		}
		wantTo := v4wire.Endpoint{
			// The mirrored UDP address is the UDP packet sender
			IP: test.remoteaddr.IP, UDP: uint16(test.remoteaddr.Port),
			// The mirrored TCP port is the one from the ping packet
			TCP: testRemote.TCP,
		}
		if !reflect.DeepEqual(p.To, wantTo) {
			t.Errorf("got pong.To %v, want %v", p.To, wantTo)
		}
	})

	// Remote is unknown, the table pings back.
	test.waitPacketOut(func(p *v4wire.Ping, to *net.UDPAddr, hash []byte) {
		if !reflect.DeepEqual(p.From, test.udp.ourEndpoint()) {
			t.Errorf("got ping.From %#v, want %#v", p.From, test.udp.ourEndpoint())
		}
		wantTo := v4wire.Endpoint{
			// The mirrored UDP address is the UDP packet sender.
			IP:  test.remoteaddr.IP,
			UDP: uint16(test.remoteaddr.Port),
			TCP: 0,
		}
		if !reflect.DeepEqual(p.To, wantTo) {
			t.Errorf("got ping.To %v, want %v", p.To, wantTo)
		}
		test.packetIn(nil, &v4wire.Pong{ReplyTok: hash, Expiration: futureExp})
	})

	// The node should be added to the table shortly after getting the
	// pong packet.
	select {
	case n := <-added:
		rid := encodePubkey(&test.remotekey.PublicKey).id()
		if n.ID() != rid {
			t.Errorf("node has wrong ID: got %v, want %v", n.ID(), rid)
		}
		if !n.IP().Equal(test.remoteaddr.IP) {
			t.Errorf("node has wrong IP: got %v, want: %v", n.IP(), test.remoteaddr.IP)
		}
		if n.UDP() != test.remoteaddr.Port {
			t.Errorf("node has wrong UDP port: got %v, want: %v", n.UDP(), test.remoteaddr.Port)
		}
		if n.TCP() != int(testRemote.TCP) {
			t.Errorf("node has wrong TCP port: got %v, want: %v", n.TCP(), testRemote.TCP)
		}
	case <-time.After(2 * time.Second):
		t.Errorf("node was not added within 2 seconds")
	}
}

// This test checks that EIP-868 requests work.
func TestUDPv4_EIP868(t *testing.T) {
	test := newUDPTest(t)
	defer test.close()

	test.udp.localNode.Set(enr.WithEntry("foo", "bar"))
	wantNode := test.udp.localNode.Node()

	// ENR requests aren't allowed before endpoint proof.
	test.packetIn(errUnknownNode, &v4wire.ENRRequest{Expiration: futureExp})

	// Perform endpoint proof and check for sequence number in packet tail.
	test.packetIn(nil, &v4wire.Ping{Expiration: futureExp})
	test.waitPacketOut(func(p *v4wire.Pong, addr *net.UDPAddr, hash []byte) {
		if p.ENRSeq != wantNode.Seq() {
			t.Errorf("wrong sequence number in pong: %d, want %d", p.ENRSeq, wantNode.Seq())
		}
	})
	test.waitPacketOut(func(p *v4wire.Ping, addr *net.UDPAddr, hash []byte) {
		if p.ENRSeq != wantNode.Seq() {
			t.Errorf("wrong sequence number in ping: %d, want %d", p.ENRSeq, wantNode.Seq())
		}
		test.packetIn(nil, &v4wire.Pong{Expiration: futureExp, ReplyTok: hash})
	})

	// Request should work now.
	test.packetIn(nil, &v4wire.ENRRequest{Expiration: futureExp})
	test.waitPacketOut(func(p *v4wire.ENRResponse, addr *net.UDPAddr, hash []byte) {
		n, err := enode.New(enode.ValidSchemes, &p.Record)
		if err != nil {
			t.Fatalf("invalid record: %v", err)
		}
		if !reflect.DeepEqual(n, wantNode) {
			t.Fatalf("wrong node in ENRResponse: %v", n)
		}
	})
}

// This test verifies that a small network of nodes can boot up into a healthy state.
func TestUDPv4_smallNetConvergence(t *testing.T) {
	t.Parallel()

	// Start the network.
	nodes := make([]*UDPv4, 4)
	for i := range nodes {
		var cfg Config
		if i > 0 {
			bn := nodes[0].Self()
			cfg.Bootnodes = []*enode.Node{bn}
		}
		nodes[i] = startLocalhostV4(t, cfg)
		defer nodes[i].Close()
	}

	// Run through the iterator on all nodes until
	// they have all found each other.
	status := make(chan error, len(nodes))
	for i := range nodes {
		node := nodes[i]
		go func() {
			found := make(map[enode.ID]bool, len(nodes))
			it := node.RandomNodes()
			for it.Next() {
				found[it.Node().ID()] = true
				if len(found) == len(nodes) {
					status <- nil
					return
				}
			}
			status <- fmt.Errorf("node %s didn't find all nodes", node.Self().ID().TerminalString())
		}()
	}

	// Wait for all status reports.
	timeout := time.NewTimer(30 * time.Second)
	defer timeout.Stop()
	for received := 0; received < len(nodes); {
		select {
		case <-timeout.C:
			for _, node := range nodes {
				node.Close()
			}
		case err := <-status:
			received++
			if err != nil {
				t.Error("ERROR:", err)
				return
			}
		}
	}
}

func startLocalhostV4(t *testing.T, cfg Config) *UDPv4 {
	t.Helper()

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
	ln.SetFallbackUDP(realaddr.Port)
	udp, err := ListenV4(socket, ln, cfg)
	if err != nil {
		t.Fatal(err)
	}
	return udp
}

// dgramPipe is a fake UDP socket. It queues all sent datagrams.
type dgramPipe struct {
	mu      *sync.Mutex
	cond    *sync.Cond
	closing chan struct{}
	closed  bool
	queue   []dgram
}

type dgram struct {
	to   net.UDPAddr
	data []byte
}

func newpipe() *dgramPipe {
	mu := new(sync.Mutex)
	return &dgramPipe{
		closing: make(chan struct{}),
		cond:    &sync.Cond{L: mu},
		mu:      mu,
	}
}

// WriteToUDP queues a datagram.
func (c *dgramPipe) WriteToUDP(b []byte, to *net.UDPAddr) (n int, err error) {
	msg := make([]byte, len(b))
	copy(msg, b)
	c.mu.Lock()
	defer c.mu.Unlock()
	if c.closed {
		return 0, errors.New("closed")
	}
	c.queue = append(c.queue, dgram{*to, b})
	c.cond.Signal()
	return len(b), nil
}

// ReadFromUDP just hangs until the pipe is closed.
func (c *dgramPipe) ReadFromUDP(b []byte) (n int, addr *net.UDPAddr, err error) {
	<-c.closing
	return 0, nil, io.EOF
}

func (c *dgramPipe) Close() error {
	c.mu.Lock()
	defer c.mu.Unlock()
	if !c.closed {
		close(c.closing)
		c.closed = true
	}
	c.cond.Broadcast()
	return nil
}

func (c *dgramPipe) LocalAddr() net.Addr {
	return &net.UDPAddr{IP: testLocal.IP, Port: int(testLocal.UDP)}
}

func (c *dgramPipe) receive() (dgram, error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	var timedOut bool
	timer := time.AfterFunc(3*time.Second, func() {
		c.mu.Lock()
		timedOut = true
		c.mu.Unlock()
		c.cond.Broadcast()
	})
	defer timer.Stop()

	for len(c.queue) == 0 && !c.closed && !timedOut {
		c.cond.Wait()
	}
	if c.closed {
		return dgram{}, errClosed
	}
	if timedOut {
		return dgram{}, errTimeout
	}
	p := c.queue[0]
	copy(c.queue, c.queue[1:])
	c.queue = c.queue[:len(c.queue)-1]
	return p, nil
}
