// Copyright 2020 The go-ethereum Authors
// This file is part of go-ethereum.
//
// go-ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// go-ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with go-ethereum. If not, see <http://www.gnu.org/licenses/>.

package v4test

import (
	"bytes"
	"crypto/rand"
	"errors"
	"fmt"
	"net"
	"time"

	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/internal/utesting"
	"github.com/ethereum/go-ethereum/p2p/discover/v4wire"
)

const (
	expiration  = 20 * time.Second
	wrongPacket = 66
	macSize     = 256 / 8
)

var (
	// Remote node under test
	Remote string
	// Listen1 is the IP where the first tester is listening, port will be assigned
	Listen1 string = "127.0.0.1"
	// Listen2 is the IP where the second tester is listening, port will be assigned
	// Before running the test, you may have to `sudo ifconfig lo0 add 127.0.0.2` (on MacOS at least)
	Listen2 string = "127.0.0.2"
)

type pingWithJunk struct {
	Version    uint
	From, To   v4wire.Endpoint
	Expiration uint64
	JunkData1  uint
	JunkData2  []byte
}

func (req *pingWithJunk) Name() string { return "PING/v4" }
func (req *pingWithJunk) Kind() byte   { return v4wire.PingPacket }

type pingWrongType struct {
	Version    uint
	From, To   v4wire.Endpoint
	Expiration uint64
}

func (req *pingWrongType) Name() string { return "WRONG/v4" }
func (req *pingWrongType) Kind() byte   { return wrongPacket }

func futureExpiration() uint64 {
	return uint64(time.Now().Add(expiration).Unix())
}

// BasicPing just sends a PING packet and expects a response.
func BasicPing(t *utesting.T) {
	te := newTestEnv(Remote, Listen1, Listen2)
	defer te.close()

	pingHash := te.send(te.l1, &v4wire.Ping{
		Version:    4,
		From:       te.localEndpoint(te.l1),
		To:         te.remoteEndpoint(),
		Expiration: futureExpiration(),
	})
	if err := te.checkPingPong(pingHash); err != nil {
		t.Fatal(err)
	}
}

// checkPingPong verifies that the remote side sends both a PONG with the
// correct hash, and a PING.
// The two packets do not have to be in any particular order.
func (te *testenv) checkPingPong(pingHash []byte) error {
	var (
		pings int
		pongs int
	)
	for i := 0; i < 2; i++ {
		reply, _, err := te.read(te.l1)
		if err != nil {
			return err
		}
		switch reply.Kind() {
		case v4wire.PongPacket:
			if err := te.checkPong(reply, pingHash); err != nil {
				return err
			}
			pongs++
		case v4wire.PingPacket:
			pings++
		default:
			return fmt.Errorf("expected PING or PONG, got %v %v", reply.Name(), reply)
		}
	}
	if pongs == 1 && pings == 1 {
		return nil
	}
	return fmt.Errorf("expected 1 PING  (got %d) and 1 PONG (got %d)", pings, pongs)
}

// checkPong verifies that reply is a valid PONG matching the given ping hash,
// and a PING. The two packets do not have to be in any particular order.
func (te *testenv) checkPong(reply v4wire.Packet, pingHash []byte) error {
	if reply == nil {
		return errors.New("expected PONG reply, got nil")
	}
	if reply.Kind() != v4wire.PongPacket {
		return fmt.Errorf("expected PONG reply, got %v %v", reply.Name(), reply)
	}
	pong := reply.(*v4wire.Pong)
	if !bytes.Equal(pong.ReplyTok, pingHash) {
		return fmt.Errorf("PONG reply token mismatch: got %x, want %x", pong.ReplyTok, pingHash)
	}
	if want := te.localEndpoint(te.l1); !want.IP.Equal(pong.To.IP) || want.UDP != pong.To.UDP {
		return fmt.Errorf("PONG 'to' endpoint mismatch: got %+v, want %+v", pong.To, want)
	}
	if v4wire.Expired(pong.Expiration) {
		return fmt.Errorf("PONG is expired (%v)", pong.Expiration)
	}
	return nil
}

// PingWrongTo sends a PING packet with wrong 'to' field and expects a PONG response.
func PingWrongTo(t *utesting.T) {
	te := newTestEnv(Remote, Listen1, Listen2)
	defer te.close()

	wrongEndpoint := v4wire.Endpoint{IP: net.ParseIP("192.0.2.0")}
	pingHash := te.send(te.l1, &v4wire.Ping{
		Version:    4,
		From:       te.localEndpoint(te.l1),
		To:         wrongEndpoint,
		Expiration: futureExpiration(),
	})
	if err := te.checkPingPong(pingHash); err != nil {
		t.Fatal(err)
	}
}

// PingWrongFrom sends a PING packet with wrong 'from' field and expects a PONG response.
func PingWrongFrom(t *utesting.T) {
	te := newTestEnv(Remote, Listen1, Listen2)
	defer te.close()

	wrongEndpoint := v4wire.Endpoint{IP: net.ParseIP("192.0.2.0")}
	pingHash := te.send(te.l1, &v4wire.Ping{
		Version:    4,
		From:       wrongEndpoint,
		To:         te.remoteEndpoint(),
		Expiration: futureExpiration(),
	})

	if err := te.checkPingPong(pingHash); err != nil {
		t.Fatal(err)
	}
}

// PingExtraData This test sends a PING packet with additional data at the end and expects a PONG
// response. The remote node should respond because EIP-8 mandates ignoring additional
// trailing data.
func PingExtraData(t *utesting.T) {
	te := newTestEnv(Remote, Listen1, Listen2)
	defer te.close()

	pingHash := te.send(te.l1, &pingWithJunk{
		Version:    4,
		From:       te.localEndpoint(te.l1),
		To:         te.remoteEndpoint(),
		Expiration: futureExpiration(),
		JunkData1:  42,
		JunkData2:  []byte{9, 8, 7, 6, 5, 4, 3, 2, 1},
	})

	if err := te.checkPingPong(pingHash); err != nil {
		t.Fatal(err)
	}
}

// This test sends a PING packet with additional data and wrong 'from' field
// and expects a PONG response.
func PingExtraDataWrongFrom(t *utesting.T) {
	te := newTestEnv(Remote, Listen1, Listen2)
	defer te.close()

	wrongEndpoint := v4wire.Endpoint{IP: net.ParseIP("192.0.2.0")}
	req := pingWithJunk{
		Version:    4,
		From:       wrongEndpoint,
		To:         te.remoteEndpoint(),
		Expiration: futureExpiration(),
		JunkData1:  42,
		JunkData2:  []byte{9, 8, 7, 6, 5, 4, 3, 2, 1},
	}
	pingHash := te.send(te.l1, &req)
	if err := te.checkPingPong(pingHash); err != nil {
		t.Fatal(err)
	}
}

// This test sends a PING packet with an expiration in the past.
// The remote node should not respond.
func PingPastExpiration(t *utesting.T) {
	te := newTestEnv(Remote, Listen1, Listen2)
	defer te.close()

	te.send(te.l1, &v4wire.Ping{
		Version:    4,
		From:       te.localEndpoint(te.l1),
		To:         te.remoteEndpoint(),
		Expiration: -futureExpiration(),
	})

	reply, _, _ := te.read(te.l1)
	if reply != nil {
		t.Fatalf("Expected no reply, got %v %v", reply.Name(), reply)
	}
}

// This test sends an invalid packet. The remote node should not respond.
func WrongPacketType(t *utesting.T) {
	te := newTestEnv(Remote, Listen1, Listen2)
	defer te.close()

	te.send(te.l1, &pingWrongType{
		Version:    4,
		From:       te.localEndpoint(te.l1),
		To:         te.remoteEndpoint(),
		Expiration: futureExpiration(),
	})

	reply, _, _ := te.read(te.l1)
	if reply != nil {
		t.Fatalf("Expected no reply, got %v %v", reply.Name(), reply)
	}
}

// This test verifies that the default behaviour of ignoring 'from' fields is unaffected by
// the bonding process. After bonding, it pings the target with a different from endpoint.
func BondThenPingWithWrongFrom(t *utesting.T) {
	te := newTestEnv(Remote, Listen1, Listen2)
	defer te.close()

	bond(t, te)

	wrongEndpoint := v4wire.Endpoint{IP: net.ParseIP("192.0.2.0")}
	pingHash := te.send(te.l1, &v4wire.Ping{
		Version:    4,
		From:       wrongEndpoint,
		To:         te.remoteEndpoint(),
		Expiration: futureExpiration(),
	})

waitForPong:
	for {
		reply, _, err := te.read(te.l1)
		if err != nil {
			t.Fatal(err)
		}
		switch reply.Kind() {
		case v4wire.PongPacket:
			if err := te.checkPong(reply, pingHash); err != nil {
				t.Fatal(err)
			}
			break waitForPong
		case v4wire.FindnodePacket:
			// FINDNODE from the node is acceptable here since the endpoint
			// verification was performed earlier.
		default:
			t.Fatalf("Expected PONG, got %v %v", reply.Name(), reply)
		}
	}
}

// This test just sends FINDNODE. The remote node should not reply
// because the endpoint proof has not completed.
func FindnodeWithoutEndpointProof(t *utesting.T) {
	te := newTestEnv(Remote, Listen1, Listen2)
	defer te.close()

	req := v4wire.Findnode{Expiration: futureExpiration()}
	rand.Read(req.Target[:])
	te.send(te.l1, &req)

	for {
		reply, _, _ := te.read(te.l1)
		if reply == nil {
			// No response, all good
			break
		}
		if reply.Kind() == v4wire.PingPacket {
			continue // A ping is ok, just ignore it
		}
		t.Fatalf("Expected no reply, got %v %v", reply.Name(), reply)
	}
}

// BasicFindnode sends a FINDNODE request after performing the endpoint
// proof. The remote node should respond.
func BasicFindnode(t *utesting.T) {
	te := newTestEnv(Remote, Listen1, Listen2)
	defer te.close()
	bond(t, te)

	findnode := v4wire.Findnode{Expiration: futureExpiration()}
	rand.Read(findnode.Target[:])
	te.send(te.l1, &findnode)

	reply, _, err := te.read(te.l1)
	if err != nil {
		t.Fatal("read find nodes", err)
	}
	if reply.Kind() != v4wire.NeighborsPacket {
		t.Fatalf("Expected neighbors, got %v %v", reply.Name(), reply)
	}
}

// This test sends an unsolicited NEIGHBORS packet after the endpoint proof, then sends
// FINDNODE to read the remote table. The remote node should not return the node contained
// in the unsolicited NEIGHBORS packet.
func UnsolicitedNeighbors(t *utesting.T) {
	te := newTestEnv(Remote, Listen1, Listen2)
	defer te.close()
	bond(t, te)

	// Send unsolicited NEIGHBORS response.
	fakeKey, _ := crypto.GenerateKey()
	encFakeKey := v4wire.EncodePubkey(&fakeKey.PublicKey)
	neighbors := v4wire.Neighbors{
		Expiration: futureExpiration(),
		Nodes: []v4wire.Node{{
			ID:  encFakeKey,
			IP:  net.IP{1, 2, 3, 4},
			UDP: 30303,
			TCP: 30303,
		}},
	}
	te.send(te.l1, &neighbors)

	// Check if the remote node included the fake node.
	te.send(te.l1, &v4wire.Findnode{
		Expiration: futureExpiration(),
		Target:     encFakeKey,
	})

	reply, _, err := te.read(te.l1)
	if err != nil {
		t.Fatal("read find nodes", err)
	}
	if reply.Kind() != v4wire.NeighborsPacket {
		t.Fatalf("Expected neighbors, got %v %v", reply.Name(), reply)
	}
	nodes := reply.(*v4wire.Neighbors).Nodes
	if contains(nodes, encFakeKey) {
		t.Fatal("neighbors response contains node from earlier unsolicited neighbors response")
	}
}

// This test sends FINDNODE with an expiration timestamp in the past.
// The remote node should not respond.
func FindnodePastExpiration(t *utesting.T) {
	te := newTestEnv(Remote, Listen1, Listen2)
	defer te.close()
	bond(t, te)

	findnode := v4wire.Findnode{Expiration: -futureExpiration()}
	rand.Read(findnode.Target[:])
	te.send(te.l1, &findnode)

	for {
		reply, _, _ := te.read(te.l1)
		if reply == nil {
			return
		} else if reply.Kind() == v4wire.NeighborsPacket {
			t.Fatal("Unexpected NEIGHBORS response for expired FINDNODE request")
		}
	}
}

// bond performs the endpoint proof with the remote node.
func bond(t *utesting.T, te *testenv) {
	pingHash := te.send(te.l1, &v4wire.Ping{
		Version:    4,
		From:       te.localEndpoint(te.l1),
		To:         te.remoteEndpoint(),
		Expiration: futureExpiration(),
	})

	var gotPing, gotPong bool
	for !gotPing || !gotPong {
		req, hash, err := te.read(te.l1)
		if err != nil {
			t.Fatal(err)
		}
		switch req.(type) {
		case *v4wire.Ping:
			te.send(te.l1, &v4wire.Pong{
				To:         te.remoteEndpoint(),
				ReplyTok:   hash,
				Expiration: futureExpiration(),
			})
			gotPing = true
		case *v4wire.Pong:
			if err := te.checkPong(req, pingHash); err != nil {
				t.Fatal(err)
			}
			gotPong = true
		}
	}
}

// This test attempts to perform a traffic amplification attack against a
// 'victim' endpoint using FINDNODE. In this attack scenario, the attacker
// attempts to complete the endpoint proof non-interactively by sending a PONG
// with mismatching reply token from the 'victim' endpoint. The attack works if
// the remote node does not verify the PONG reply token field correctly. The
// attacker could then perform traffic amplification by sending many FINDNODE
// requests to the discovery node, which would reply to the 'victim' address.
func FindnodeAmplificationInvalidPongHash(t *utesting.T) {
	te := newTestEnv(Remote, Listen1, Listen2)
	defer te.close()

	// Send PING to start endpoint verification.
	te.send(te.l1, &v4wire.Ping{
		Version:    4,
		From:       te.localEndpoint(te.l1),
		To:         te.remoteEndpoint(),
		Expiration: futureExpiration(),
	})

	var gotPing, gotPong bool
	for !gotPing || !gotPong {
		req, _, err := te.read(te.l1)
		if err != nil {
			t.Fatal(err)
		}
		switch req.(type) {
		case *v4wire.Ping:
			// Send PONG from this node ID, but with invalid ReplyTok.
			te.send(te.l1, &v4wire.Pong{
				To:         te.remoteEndpoint(),
				ReplyTok:   make([]byte, macSize),
				Expiration: futureExpiration(),
			})
			gotPing = true
		case *v4wire.Pong:
			gotPong = true
		}
	}

	// Now send FINDNODE. The remote node should not respond because our
	// PONG did not reference the PING hash.
	findnode := v4wire.Findnode{Expiration: futureExpiration()}
	rand.Read(findnode.Target[:])
	te.send(te.l1, &findnode)

	// If we receive a NEIGHBORS response, the attack worked and the test fails.
	reply, _, _ := te.read(te.l1)
	if reply != nil && reply.Kind() == v4wire.NeighborsPacket {
		t.Error("Got neighbors")
	}
}

// This test attempts to perform a traffic amplification attack using FINDNODE.
// The attack works if the remote node does not verify the IP address of FINDNODE
// against the endpoint verification proof done by PING/PONG.
func FindnodeAmplificationWrongIP(t *utesting.T) {
	te := newTestEnv(Remote, Listen1, Listen2)
	defer te.close()

	// Do the endpoint proof from the l1 IP.
	bond(t, te)

	// Now send FINDNODE from the same node ID, but different IP address.
	// The remote node should not respond.
	findnode := v4wire.Findnode{Expiration: futureExpiration()}
	rand.Read(findnode.Target[:])
	te.send(te.l2, &findnode)

	// If we receive a NEIGHBORS response, the attack worked and the test fails.
	reply, _, _ := te.read(te.l2)
	if reply != nil {
		t.Error("Got NEIGHORS response for FINDNODE from wrong IP")
	}
}

var AllTests = []utesting.Test{
	{Name: "Ping/Basic", Fn: BasicPing},
	{Name: "Ping/WrongTo", Fn: PingWrongTo},
	{Name: "Ping/WrongFrom", Fn: PingWrongFrom},
	{Name: "Ping/ExtraData", Fn: PingExtraData},
	{Name: "Ping/ExtraDataWrongFrom", Fn: PingExtraDataWrongFrom},
	{Name: "Ping/PastExpiration", Fn: PingPastExpiration},
	{Name: "Ping/WrongPacketType", Fn: WrongPacketType},
	{Name: "Ping/BondThenPingWithWrongFrom", Fn: BondThenPingWithWrongFrom},
	{Name: "Findnode/WithoutEndpointProof", Fn: FindnodeWithoutEndpointProof},
	{Name: "Findnode/BasicFindnode", Fn: BasicFindnode},
	{Name: "Findnode/UnsolicitedNeighbors", Fn: UnsolicitedNeighbors},
	{Name: "Findnode/PastExpiration", Fn: FindnodePastExpiration},
	{Name: "Amplification/InvalidPongHash", Fn: FindnodeAmplificationInvalidPongHash},
	{Name: "Amplification/WrongIP", Fn: FindnodeAmplificationWrongIP},
}
