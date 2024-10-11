// Copyright 2014 The go-ethereum Authors
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

package p2p

import (
	"encoding/binary"
	"errors"
	"fmt"
	"math/rand"
	"net"
	"reflect"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/enr"
)

var discard = Protocol{
	Name:   "discard",
	Length: 1,
	Run: func(p *Peer, rw MsgReadWriter) error {
		for {
			msg, err := rw.ReadMsg()
			if err != nil {
				return err
			}
			fmt.Printf("discarding %d\n", msg.Code)
			if err = msg.Discard(); err != nil {
				return err
			}
		}
	},
}

// uintID encodes i into a node ID.
func uintID(i uint16) enode.ID {
	var id enode.ID
	binary.BigEndian.PutUint16(id[:], i)
	return id
}

// newNode creates a node record with the given address.
func newNode(id enode.ID, addr string) *enode.Node {
	var r enr.Record
	if addr != "" {
		// Set the port if present.
		if strings.Contains(addr, ":") {
			hs, ps, err := net.SplitHostPort(addr)
			if err != nil {
				panic(fmt.Errorf("invalid address %q", addr))
			}
			port, err := strconv.Atoi(ps)
			if err != nil {
				panic(fmt.Errorf("invalid port in %q", addr))
			}
			r.Set(enr.TCP(port))
			r.Set(enr.UDP(port))
			addr = hs
		}
		// Set the IP.
		ip := net.ParseIP(addr)
		if ip == nil {
			panic(fmt.Errorf("invalid IP %q", addr))
		}
		r.Set(enr.IP(ip))
	}
	return enode.SignNull(&r, id)
}

func testPeer(protos []Protocol) (func(), *conn, *Peer, <-chan error) {
	var (
		fd1, fd2   = net.Pipe()
		key1, key2 = newkey(), newkey()
		t1         = newTestTransport(&key2.PublicKey, fd1, nil)
		t2         = newTestTransport(&key1.PublicKey, fd2, &key1.PublicKey)
	)

	c1 := &conn{fd: fd1, node: newNode(uintID(1), ""), transport: t1}
	c2 := &conn{fd: fd2, node: newNode(uintID(2), ""), transport: t2}
	for _, p := range protos {
		c1.caps = append(c1.caps, p.cap())
		c2.caps = append(c2.caps, p.cap())
	}

	peer := newPeer(log.Root(), c1, protos)
	errc := make(chan error, 1)
	go func() {
		_, err := peer.run()
		errc <- err
	}()

	closer := func() { c2.close(errors.New("close func called")) }
	return closer, c2, peer, errc
}

func TestPeerProtoReadMsg(t *testing.T) {
	proto := Protocol{
		Name:   "a",
		Length: 5,
		Run: func(peer *Peer, rw MsgReadWriter) error {
			if err := ExpectMsg(rw, 2, []uint{1}); err != nil {
				t.Error(err)
			}
			if err := ExpectMsg(rw, 3, []uint{2}); err != nil {
				t.Error(err)
			}
			if err := ExpectMsg(rw, 4, []uint{3}); err != nil {
				t.Error(err)
			}
			return nil
		},
	}

	closer, rw, _, errc := testPeer([]Protocol{proto})
	defer closer()

	Send(rw, baseProtocolLength+2, []uint{1})
	Send(rw, baseProtocolLength+3, []uint{2})
	Send(rw, baseProtocolLength+4, []uint{3})

	select {
	case err := <-errc:
		if err != errProtocolReturned {
			t.Errorf("peer returned error: %v", err)
		}
	case <-time.After(2 * time.Second):
		t.Errorf("receive timeout")
	}
}

func TestPeerProtoEncodeMsg(t *testing.T) {
	proto := Protocol{
		Name:   "a",
		Length: 2,
		Run: func(peer *Peer, rw MsgReadWriter) error {
			if err := SendItems(rw, 2); err == nil {
				t.Error("expected error for out-of-range msg code, got nil")
			}
			if err := SendItems(rw, 1, "foo", "bar"); err != nil {
				t.Errorf("write error: %v", err)
			}
			return nil
		},
	}
	closer, rw, _, _ := testPeer([]Protocol{proto})
	defer closer()

	if err := ExpectMsg(rw, 17, []string{"foo", "bar"}); err != nil {
		t.Error(err)
	}
}

func TestPeerPing(t *testing.T) {
	closer, rw, _, _ := testPeer(nil)
	defer closer()
	if err := SendItems(rw, pingMsg); err != nil {
		t.Fatal(err)
	}
	if err := ExpectMsg(rw, pongMsg, nil); err != nil {
		t.Error(err)
	}
}

// This test checks that a disconnect message sent by a peer is returned
// as the error from Peer.run.
func TestPeerDisconnect(t *testing.T) {
	closer, rw, _, disc := testPeer(nil)
	defer closer()

	if err := SendItems(rw, discMsg, DiscQuitting); err != nil {
		t.Fatal(err)
	}
	select {
	case reason := <-disc:
		if reason != DiscQuitting {
			t.Errorf("run returned wrong reason: got %v, want %v", reason, DiscQuitting)
		}
	case <-time.After(500 * time.Millisecond):
		t.Error("peer did not return")
	}
}

// This test is supposed to verify that Peer can reliably handle
// multiple causes of disconnection occurring at the same time.
func TestPeerDisconnectRace(t *testing.T) {
	maybe := func() bool { return rand.Intn(2) == 1 }

	for i := 0; i < 1000; i++ {
		protoclose := make(chan error)
		protodisc := make(chan DiscReason)
		closer, rw, p, disc := testPeer([]Protocol{
			{
				Name:   "closereq",
				Run:    func(p *Peer, rw MsgReadWriter) error { return <-protoclose },
				Length: 1,
			},
			{
				Name:   "disconnect",
				Run:    func(p *Peer, rw MsgReadWriter) error { p.Disconnect(<-protodisc); return nil },
				Length: 1,
			},
		})

		// Simulate incoming messages.
		go SendItems(rw, baseProtocolLength+1)
		go SendItems(rw, baseProtocolLength+2)
		// Close the network connection.
		go closer()
		// Make protocol "closereq" return.
		protoclose <- errors.New("protocol closed")
		// Make protocol "disconnect" call peer.Disconnect
		protodisc <- DiscAlreadyConnected
		// In some cases, simulate something else calling peer.Disconnect.
		if maybe() {
			go p.Disconnect(DiscInvalidIdentity)
		}
		// In some cases, simulate remote requesting a disconnect.
		if maybe() {
			go SendItems(rw, discMsg, DiscQuitting)
		}

		select {
		case <-disc:
		case <-time.After(2 * time.Second):
			// Peer.run should return quickly. If it doesn't the Peer
			// goroutines are probably deadlocked. Call panic in order to
			// show the stacks.
			panic("Peer.run took to long to return.")
		}
	}
}

func TestNewPeer(t *testing.T) {
	name := "nodename"
	caps := []Cap{{"foo", 2}, {"bar", 3}}
	id := randomID()
	p := NewPeer(id, name, caps)
	if p.ID() != id {
		t.Errorf("ID mismatch: got %v, expected %v", p.ID(), id)
	}
	if p.Name() != name {
		t.Errorf("Name mismatch: got %v, expected %v", p.Name(), name)
	}
	if !reflect.DeepEqual(p.Caps(), caps) {
		t.Errorf("Caps mismatch: got %v, expected %v", p.Caps(), caps)
	}

	p.Disconnect(DiscAlreadyConnected) // Should not hang
}

func TestMatchProtocols(t *testing.T) {
	tests := []struct {
		Remote []Cap
		Local  []Protocol
		Match  map[string]protoRW
	}{
		{
			// No remote capabilities
			Local: []Protocol{{Name: "a"}},
		},
		{
			// No local protocols
			Remote: []Cap{{Name: "a"}},
		},
		{
			// No mutual protocols
			Remote: []Cap{{Name: "a"}},
			Local:  []Protocol{{Name: "b"}},
		},
		{
			// Some matches, some differences
			Remote: []Cap{{Name: "local"}, {Name: "match1"}, {Name: "match2"}},
			Local:  []Protocol{{Name: "match1"}, {Name: "match2"}, {Name: "remote"}},
			Match:  map[string]protoRW{"match1": {Protocol: Protocol{Name: "match1"}}, "match2": {Protocol: Protocol{Name: "match2"}}},
		},
		{
			// Various alphabetical ordering
			Remote: []Cap{{Name: "aa"}, {Name: "ab"}, {Name: "bb"}, {Name: "ba"}},
			Local:  []Protocol{{Name: "ba"}, {Name: "bb"}, {Name: "ab"}, {Name: "aa"}},
			Match:  map[string]protoRW{"aa": {Protocol: Protocol{Name: "aa"}}, "ab": {Protocol: Protocol{Name: "ab"}}, "ba": {Protocol: Protocol{Name: "ba"}}, "bb": {Protocol: Protocol{Name: "bb"}}},
		},
		{
			// No mutual versions
			Remote: []Cap{{Version: 1}},
			Local:  []Protocol{{Version: 2}},
		},
		{
			// Multiple versions, single common
			Remote: []Cap{{Version: 1}, {Version: 2}},
			Local:  []Protocol{{Version: 2}, {Version: 3}},
			Match:  map[string]protoRW{"": {Protocol: Protocol{Version: 2}}},
		},
		{
			// Multiple versions, multiple common
			Remote: []Cap{{Version: 1}, {Version: 2}, {Version: 3}, {Version: 4}},
			Local:  []Protocol{{Version: 2}, {Version: 3}},
			Match:  map[string]protoRW{"": {Protocol: Protocol{Version: 3}}},
		},
		{
			// Various version orderings
			Remote: []Cap{{Version: 4}, {Version: 1}, {Version: 3}, {Version: 2}},
			Local:  []Protocol{{Version: 2}, {Version: 3}, {Version: 1}},
			Match:  map[string]protoRW{"": {Protocol: Protocol{Version: 3}}},
		},
		{
			// Versions overriding sub-protocol lengths
			Remote: []Cap{{Version: 1}, {Version: 2}, {Version: 3}, {Name: "a"}},
			Local:  []Protocol{{Version: 1, Length: 1}, {Version: 2, Length: 2}, {Version: 3, Length: 3}, {Name: "a"}},
			Match:  map[string]protoRW{"": {Protocol: Protocol{Version: 3}}, "a": {Protocol: Protocol{Name: "a"}, offset: 3}},
		},
	}

	for i, tt := range tests {
		result := matchProtocols(tt.Local, tt.Remote, nil)
		if len(result) != len(tt.Match) {
			t.Errorf("test %d: negotiation mismatch: have %v, want %v", i, len(result), len(tt.Match))
			continue
		}
		// Make sure all negotiated protocols are needed and correct
		for name, proto := range result {
			match, ok := tt.Match[name]
			if !ok {
				t.Errorf("test %d, proto '%s': negotiated but shouldn't have", i, name)
				continue
			}
			if proto.Name != match.Name {
				t.Errorf("test %d, proto '%s': name mismatch: have %v, want %v", i, name, proto.Name, match.Name)
			}
			if proto.Version != match.Version {
				t.Errorf("test %d, proto '%s': version mismatch: have %v, want %v", i, name, proto.Version, match.Version)
			}
			if proto.offset-baseProtocolLength != match.offset {
				t.Errorf("test %d, proto '%s': offset mismatch: have %v, want %v", i, name, proto.offset-baseProtocolLength, match.offset)
			}
		}
		// Make sure no protocols missed negotiation
		for name := range tt.Match {
			if _, ok := result[name]; !ok {
				t.Errorf("test %d, proto '%s': not negotiated, should have", i, name)
				continue
			}
		}
	}
}
