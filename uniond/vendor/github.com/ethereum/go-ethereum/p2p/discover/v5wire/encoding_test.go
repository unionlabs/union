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

package v5wire

import (
	"bytes"
	"crypto/ecdsa"
	"encoding/hex"
	"errors"
	"flag"
	"fmt"
	"net"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/davecgh/go-spew/spew"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/p2p/enode"
)

// To regenerate discv5 test vectors, run
//
//	go test -run TestVectors -write-test-vectors
var writeTestVectorsFlag = flag.Bool("write-test-vectors", false, "Overwrite discv5 test vectors in testdata/")

var (
	testKeyA, _   = crypto.HexToECDSA("eef77acb6c6a6eebc5b363a475ac583ec7eccdb42b6481424c60f59aa326547f")
	testKeyB, _   = crypto.HexToECDSA("66fb62bfbd66b9177a138c1e5cddbe4f7c30c343e94e68df8769459cb1cde628")
	testEphKey, _ = crypto.HexToECDSA("0288ef00023598499cb6c940146d050d2b1fb914198c327f76aad590bead68b6")
	testIDnonce   = [16]byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16}
)

// This test checks that the minPacketSize and randomPacketMsgSize constants are well-defined.
func TestMinSizes(t *testing.T) {
	var (
		gcmTagSize = 16
		emptyMsg   = sizeofMessageAuthData + gcmTagSize
	)
	t.Log("static header size", sizeofStaticPacketData)
	t.Log("whoareyou size", sizeofStaticPacketData+sizeofWhoareyouAuthData)
	t.Log("empty msg size", sizeofStaticPacketData+emptyMsg)
	if want := emptyMsg; minMessageSize != want {
		t.Fatalf("wrong minMessageSize %d, want %d", minMessageSize, want)
	}
	if sizeofMessageAuthData+randomPacketMsgSize < minMessageSize {
		t.Fatalf("randomPacketMsgSize %d too small", randomPacketMsgSize)
	}
}

// This test checks the basic handshake flow where A talks to B and A has no secrets.
func TestHandshake(t *testing.T) {
	t.Parallel()
	net := newHandshakeTest()
	defer net.close()

	// A -> B   RANDOM PACKET
	packet, _ := net.nodeA.encode(t, net.nodeB, &Findnode{})
	resp := net.nodeB.expectDecode(t, UnknownPacket, packet)

	// A <- B   WHOAREYOU
	challenge := &Whoareyou{
		Nonce:     resp.(*Unknown).Nonce,
		IDNonce:   testIDnonce,
		RecordSeq: 0,
	}
	whoareyou, _ := net.nodeB.encode(t, net.nodeA, challenge)
	net.nodeA.expectDecode(t, WhoareyouPacket, whoareyou)

	// A -> B   FINDNODE (handshake packet)
	findnode, _ := net.nodeA.encodeWithChallenge(t, net.nodeB, challenge, &Findnode{})
	net.nodeB.expectDecode(t, FindnodeMsg, findnode)
	if len(net.nodeB.c.sc.handshakes) > 0 {
		t.Fatalf("node B didn't remove handshake from challenge map")
	}

	// A <- B   NODES
	nodes, _ := net.nodeB.encode(t, net.nodeA, &Nodes{RespCount: 1})
	net.nodeA.expectDecode(t, NodesMsg, nodes)
}

// This test checks that handshake attempts are removed within the timeout.
func TestHandshake_timeout(t *testing.T) {
	t.Parallel()
	net := newHandshakeTest()
	defer net.close()

	// A -> B   RANDOM PACKET
	packet, _ := net.nodeA.encode(t, net.nodeB, &Findnode{})
	resp := net.nodeB.expectDecode(t, UnknownPacket, packet)

	// A <- B   WHOAREYOU
	challenge := &Whoareyou{
		Nonce:     resp.(*Unknown).Nonce,
		IDNonce:   testIDnonce,
		RecordSeq: 0,
	}
	whoareyou, _ := net.nodeB.encode(t, net.nodeA, challenge)
	net.nodeA.expectDecode(t, WhoareyouPacket, whoareyou)

	// A -> B   FINDNODE (handshake packet) after timeout
	net.clock.Run(handshakeTimeout + 1)
	findnode, _ := net.nodeA.encodeWithChallenge(t, net.nodeB, challenge, &Findnode{})
	net.nodeB.expectDecodeErr(t, errUnexpectedHandshake, findnode)
}

// This test checks handshake behavior when no record is sent in the auth response.
func TestHandshake_norecord(t *testing.T) {
	t.Parallel()
	net := newHandshakeTest()
	defer net.close()

	// A -> B   RANDOM PACKET
	packet, _ := net.nodeA.encode(t, net.nodeB, &Findnode{})
	resp := net.nodeB.expectDecode(t, UnknownPacket, packet)

	// A <- B   WHOAREYOU
	nodeA := net.nodeA.n()
	if nodeA.Seq() == 0 {
		t.Fatal("need non-zero sequence number")
	}
	challenge := &Whoareyou{
		Nonce:     resp.(*Unknown).Nonce,
		IDNonce:   testIDnonce,
		RecordSeq: nodeA.Seq(),
		Node:      nodeA,
	}
	whoareyou, _ := net.nodeB.encode(t, net.nodeA, challenge)
	net.nodeA.expectDecode(t, WhoareyouPacket, whoareyou)

	// A -> B   FINDNODE
	findnode, _ := net.nodeA.encodeWithChallenge(t, net.nodeB, challenge, &Findnode{})
	net.nodeB.expectDecode(t, FindnodeMsg, findnode)

	// A <- B   NODES
	nodes, _ := net.nodeB.encode(t, net.nodeA, &Nodes{RespCount: 1})
	net.nodeA.expectDecode(t, NodesMsg, nodes)
}

// In this test, A tries to send FINDNODE with existing secrets but B doesn't know
// anything about A.
func TestHandshake_rekey(t *testing.T) {
	t.Parallel()
	net := newHandshakeTest()
	defer net.close()

	session := &session{
		readKey:  []byte("BBBBBBBBBBBBBBBB"),
		writeKey: []byte("AAAAAAAAAAAAAAAA"),
	}
	net.nodeA.c.sc.storeNewSession(net.nodeB.id(), net.nodeB.addr(), session)

	// A -> B   FINDNODE (encrypted with zero keys)
	findnode, authTag := net.nodeA.encode(t, net.nodeB, &Findnode{})
	net.nodeB.expectDecode(t, UnknownPacket, findnode)

	// A <- B   WHOAREYOU
	challenge := &Whoareyou{Nonce: authTag, IDNonce: testIDnonce}
	whoareyou, _ := net.nodeB.encode(t, net.nodeA, challenge)
	net.nodeA.expectDecode(t, WhoareyouPacket, whoareyou)

	// Check that new keys haven't been stored yet.
	sa := net.nodeA.c.sc.session(net.nodeB.id(), net.nodeB.addr())
	if !bytes.Equal(sa.writeKey, session.writeKey) || !bytes.Equal(sa.readKey, session.readKey) {
		t.Fatal("node A stored keys too early")
	}
	if s := net.nodeB.c.sc.session(net.nodeA.id(), net.nodeA.addr()); s != nil {
		t.Fatal("node B stored keys too early")
	}

	// A -> B   FINDNODE encrypted with new keys
	findnode, _ = net.nodeA.encodeWithChallenge(t, net.nodeB, challenge, &Findnode{})
	net.nodeB.expectDecode(t, FindnodeMsg, findnode)

	// A <- B   NODES
	nodes, _ := net.nodeB.encode(t, net.nodeA, &Nodes{RespCount: 1})
	net.nodeA.expectDecode(t, NodesMsg, nodes)
}

// In this test A and B have different keys before the handshake.
func TestHandshake_rekey2(t *testing.T) {
	t.Parallel()
	net := newHandshakeTest()
	defer net.close()

	initKeysA := &session{
		readKey:  []byte("BBBBBBBBBBBBBBBB"),
		writeKey: []byte("AAAAAAAAAAAAAAAA"),
	}
	initKeysB := &session{
		readKey:  []byte("CCCCCCCCCCCCCCCC"),
		writeKey: []byte("DDDDDDDDDDDDDDDD"),
	}
	net.nodeA.c.sc.storeNewSession(net.nodeB.id(), net.nodeB.addr(), initKeysA)
	net.nodeB.c.sc.storeNewSession(net.nodeA.id(), net.nodeA.addr(), initKeysB)

	// A -> B   FINDNODE encrypted with initKeysA
	findnode, authTag := net.nodeA.encode(t, net.nodeB, &Findnode{Distances: []uint{3}})
	net.nodeB.expectDecode(t, UnknownPacket, findnode)

	// A <- B   WHOAREYOU
	challenge := &Whoareyou{Nonce: authTag, IDNonce: testIDnonce}
	whoareyou, _ := net.nodeB.encode(t, net.nodeA, challenge)
	net.nodeA.expectDecode(t, WhoareyouPacket, whoareyou)

	// A -> B   FINDNODE (handshake packet)
	findnode, _ = net.nodeA.encodeWithChallenge(t, net.nodeB, challenge, &Findnode{})
	net.nodeB.expectDecode(t, FindnodeMsg, findnode)

	// A <- B   NODES
	nodes, _ := net.nodeB.encode(t, net.nodeA, &Nodes{RespCount: 1})
	net.nodeA.expectDecode(t, NodesMsg, nodes)
}

func TestHandshake_BadHandshakeAttack(t *testing.T) {
	t.Parallel()
	net := newHandshakeTest()
	defer net.close()

	// A -> B   RANDOM PACKET
	packet, _ := net.nodeA.encode(t, net.nodeB, &Findnode{})
	resp := net.nodeB.expectDecode(t, UnknownPacket, packet)

	// A <- B   WHOAREYOU
	challenge := &Whoareyou{
		Nonce:     resp.(*Unknown).Nonce,
		IDNonce:   testIDnonce,
		RecordSeq: 0,
	}
	whoareyou, _ := net.nodeB.encode(t, net.nodeA, challenge)
	net.nodeA.expectDecode(t, WhoareyouPacket, whoareyou)

	// A -> B   FINDNODE
	incorrect_challenge := &Whoareyou{
		IDNonce:   [16]byte{5, 6, 7, 8, 9, 6, 11, 12},
		RecordSeq: challenge.RecordSeq,
		Node:      challenge.Node,
		sent:      challenge.sent,
	}
	incorrect_findnode, _ := net.nodeA.encodeWithChallenge(t, net.nodeB, incorrect_challenge, &Findnode{})
	incorrect_findnode2 := make([]byte, len(incorrect_findnode))
	copy(incorrect_findnode2, incorrect_findnode)

	net.nodeB.expectDecodeErr(t, errInvalidNonceSig, incorrect_findnode)

	// Reject new findnode as previous handshake is now deleted.
	net.nodeB.expectDecodeErr(t, errUnexpectedHandshake, incorrect_findnode2)

	// The findnode packet is again rejected even with a valid challenge this time.
	findnode, _ := net.nodeA.encodeWithChallenge(t, net.nodeB, challenge, &Findnode{})
	net.nodeB.expectDecodeErr(t, errUnexpectedHandshake, findnode)
}

// This test checks some malformed packets.
func TestDecodeErrorsV5(t *testing.T) {
	t.Parallel()
	net := newHandshakeTest()
	defer net.close()

	b := make([]byte, 0)
	net.nodeA.expectDecodeErr(t, errTooShort, b)

	b = make([]byte, 62)
	net.nodeA.expectDecodeErr(t, errTooShort, b)

	b = make([]byte, 63)
	net.nodeA.expectDecodeErr(t, errInvalidHeader, b)

	// TODO some more tests would be nice :)
	// - check invalid authdata sizes
	// - check invalid handshake data sizes
}

// This test checks that all test vectors can be decoded.
func TestTestVectorsV5(t *testing.T) {
	var (
		idA     = enode.PubkeyToIDV4(&testKeyA.PublicKey)
		idB     = enode.PubkeyToIDV4(&testKeyB.PublicKey)
		addr    = "127.0.0.1"
		session = &session{
			writeKey: hexutil.MustDecode("0x00000000000000000000000000000000"),
			readKey:  hexutil.MustDecode("0x01010101010101010101010101010101"),
		}
		challenge0A, challenge1A, challenge0B Whoareyou
	)

	// Create challenge packets.
	c := Whoareyou{
		Nonce:   Nonce{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12},
		IDNonce: testIDnonce,
	}
	challenge0A, challenge1A, challenge0B = c, c, c
	challenge1A.RecordSeq = 1
	net := newHandshakeTest()
	challenge0A.Node = net.nodeA.n()
	challenge0B.Node = net.nodeB.n()
	challenge1A.Node = net.nodeA.n()
	net.close()

	type testVectorTest struct {
		name      string               // test vector name
		packet    Packet               // the packet to be encoded
		challenge *Whoareyou           // handshake challenge passed to encoder
		prep      func(*handshakeTest) // called before encode/decode
	}
	tests := []testVectorTest{
		{
			name:   "v5.1-whoareyou",
			packet: &challenge0B,
		},
		{
			name: "v5.1-ping-message",
			packet: &Ping{
				ReqID:  []byte{0, 0, 0, 1},
				ENRSeq: 2,
			},
			prep: func(net *handshakeTest) {
				net.nodeA.c.sc.storeNewSession(idB, addr, session)
				net.nodeB.c.sc.storeNewSession(idA, addr, session.keysFlipped())
			},
		},
		{
			name: "v5.1-ping-handshake-enr",
			packet: &Ping{
				ReqID:  []byte{0, 0, 0, 1},
				ENRSeq: 1,
			},
			challenge: &challenge0A,
			prep: func(net *handshakeTest) {
				// Update challenge.Header.AuthData.
				net.nodeA.c.Encode(idB, "", &challenge0A, nil)
				net.nodeB.c.sc.storeSentHandshake(idA, addr, &challenge0A)
			},
		},
		{
			name: "v5.1-ping-handshake",
			packet: &Ping{
				ReqID:  []byte{0, 0, 0, 1},
				ENRSeq: 1,
			},
			challenge: &challenge1A,
			prep: func(net *handshakeTest) {
				// Update challenge data.
				net.nodeA.c.Encode(idB, "", &challenge1A, nil)
				net.nodeB.c.sc.storeSentHandshake(idA, addr, &challenge1A)
			},
		},
	}

	for _, test := range tests {
		test := test
		t.Run(test.name, func(t *testing.T) {
			net := newHandshakeTest()
			defer net.close()

			// Override all random inputs.
			net.nodeA.c.sc.nonceGen = func(counter uint32) (Nonce, error) {
				return Nonce{0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF}, nil
			}
			net.nodeA.c.sc.maskingIVGen = func(buf []byte) error {
				return nil // all zero
			}
			net.nodeA.c.sc.ephemeralKeyGen = func() (*ecdsa.PrivateKey, error) {
				return testEphKey, nil
			}

			// Prime the codec for encoding/decoding.
			if test.prep != nil {
				test.prep(net)
			}

			file := filepath.Join("testdata", test.name+".txt")
			if *writeTestVectorsFlag {
				// Encode the packet.
				d, nonce := net.nodeA.encodeWithChallenge(t, net.nodeB, test.challenge, test.packet)
				comment := testVectorComment(net, test.packet, test.challenge, nonce)
				writeTestVector(file, comment, d)
			}
			enc := hexFile(file)
			net.nodeB.expectDecode(t, test.packet.Kind(), enc)
		})
	}
}

// testVectorComment creates the commentary for discv5 test vector files.
func testVectorComment(net *handshakeTest, p Packet, challenge *Whoareyou, nonce Nonce) string {
	o := new(strings.Builder)
	printWhoareyou := func(p *Whoareyou) {
		fmt.Fprintf(o, "whoareyou.challenge-data = %#x\n", p.ChallengeData)
		fmt.Fprintf(o, "whoareyou.request-nonce = %#x\n", p.Nonce[:])
		fmt.Fprintf(o, "whoareyou.id-nonce = %#x\n", p.IDNonce[:])
		fmt.Fprintf(o, "whoareyou.enr-seq = %d\n", p.RecordSeq)
	}

	fmt.Fprintf(o, "src-node-id = %#x\n", net.nodeA.id().Bytes())
	fmt.Fprintf(o, "dest-node-id = %#x\n", net.nodeB.id().Bytes())
	switch p := p.(type) {
	case *Whoareyou:
		// WHOAREYOU packet.
		printWhoareyou(p)
	case *Ping:
		fmt.Fprintf(o, "nonce = %#x\n", nonce[:])
		fmt.Fprintf(o, "read-key = %#x\n", net.nodeA.c.sc.session(net.nodeB.id(), net.nodeB.addr()).writeKey)
		fmt.Fprintf(o, "ping.req-id = %#x\n", p.ReqID)
		fmt.Fprintf(o, "ping.enr-seq = %d\n", p.ENRSeq)
		if challenge != nil {
			// Handshake message packet.
			fmt.Fprint(o, "\nhandshake inputs:\n\n")
			printWhoareyou(challenge)
			fmt.Fprintf(o, "ephemeral-key = %#x\n", testEphKey.D.Bytes())
			fmt.Fprintf(o, "ephemeral-pubkey = %#x\n", crypto.CompressPubkey(&testEphKey.PublicKey))
		}
	default:
		panic(fmt.Errorf("unhandled packet type %T", p))
	}
	return o.String()
}

// This benchmark checks performance of handshake packet decoding.
func BenchmarkV5_DecodeHandshakePingSecp256k1(b *testing.B) {
	net := newHandshakeTest()
	defer net.close()

	var (
		idA       = net.nodeA.id()
		challenge = &Whoareyou{Node: net.nodeB.n()}
		message   = &Ping{ReqID: []byte("reqid")}
	)
	enc, _, err := net.nodeA.c.Encode(net.nodeB.id(), "", message, challenge)
	if err != nil {
		b.Fatal("can't encode handshake packet")
	}
	challenge.Node = nil // force ENR signature verification in decoder
	b.ResetTimer()

	input := make([]byte, len(enc))
	for i := 0; i < b.N; i++ {
		copy(input, enc)
		net.nodeB.c.sc.storeSentHandshake(idA, "", challenge)
		_, _, _, err := net.nodeB.c.Decode(input, "")
		if err != nil {
			b.Fatal(err)
		}
	}
}

// This benchmark checks how long it takes to decode an encrypted ping packet.
func BenchmarkV5_DecodePing(b *testing.B) {
	net := newHandshakeTest()
	defer net.close()

	session := &session{
		readKey:  []byte{233, 203, 93, 195, 86, 47, 177, 186, 227, 43, 2, 141, 244, 230, 120, 17},
		writeKey: []byte{79, 145, 252, 171, 167, 216, 252, 161, 208, 190, 176, 106, 214, 39, 178, 134},
	}
	net.nodeA.c.sc.storeNewSession(net.nodeB.id(), net.nodeB.addr(), session)
	net.nodeB.c.sc.storeNewSession(net.nodeA.id(), net.nodeA.addr(), session.keysFlipped())
	addrB := net.nodeA.addr()
	ping := &Ping{ReqID: []byte("reqid"), ENRSeq: 5}
	enc, _, err := net.nodeA.c.Encode(net.nodeB.id(), addrB, ping, nil)
	if err != nil {
		b.Fatalf("can't encode: %v", err)
	}
	b.ResetTimer()

	input := make([]byte, len(enc))
	for i := 0; i < b.N; i++ {
		copy(input, enc)
		_, _, packet, _ := net.nodeB.c.Decode(input, addrB)
		if _, ok := packet.(*Ping); !ok {
			b.Fatalf("wrong packet type %T", packet)
		}
	}
}

var pp = spew.NewDefaultConfig()

type handshakeTest struct {
	nodeA, nodeB handshakeTestNode
	clock        mclock.Simulated
}

type handshakeTestNode struct {
	ln *enode.LocalNode
	c  *Codec
}

func newHandshakeTest() *handshakeTest {
	t := new(handshakeTest)
	t.nodeA.init(testKeyA, net.IP{127, 0, 0, 1}, &t.clock, DefaultProtocolID)
	t.nodeB.init(testKeyB, net.IP{127, 0, 0, 1}, &t.clock, DefaultProtocolID)
	return t
}

func (t *handshakeTest) close() {
	t.nodeA.ln.Database().Close()
	t.nodeB.ln.Database().Close()
}

func (n *handshakeTestNode) init(key *ecdsa.PrivateKey, ip net.IP, clock mclock.Clock, protocolID [6]byte) {
	db, _ := enode.OpenDB("")
	n.ln = enode.NewLocalNode(db, key)
	n.ln.SetStaticIP(ip)
	n.c = NewCodec(n.ln, key, clock, nil)
}

func (n *handshakeTestNode) encode(t testing.TB, to handshakeTestNode, p Packet) ([]byte, Nonce) {
	t.Helper()
	return n.encodeWithChallenge(t, to, nil, p)
}

func (n *handshakeTestNode) encodeWithChallenge(t testing.TB, to handshakeTestNode, c *Whoareyou, p Packet) ([]byte, Nonce) {
	t.Helper()

	// Copy challenge and add destination node. This avoids sharing 'c' among the two codecs.
	var challenge *Whoareyou
	if c != nil {
		challengeCopy := *c
		challenge = &challengeCopy
		challenge.Node = to.n()
	}
	// Encode to destination.
	enc, nonce, err := n.c.Encode(to.id(), to.addr(), p, challenge)
	if err != nil {
		t.Fatal(fmt.Errorf("(%s) %v", n.ln.ID().TerminalString(), err))
	}
	t.Logf("(%s) -> (%s)   %s\n%s", n.ln.ID().TerminalString(), to.id().TerminalString(), p.Name(), hex.Dump(enc))
	return enc, nonce
}

func (n *handshakeTestNode) expectDecode(t *testing.T, ptype byte, p []byte) Packet {
	t.Helper()

	dec, err := n.decode(p)
	if err != nil {
		t.Fatal(fmt.Errorf("(%s) %v", n.ln.ID().TerminalString(), err))
	}
	t.Logf("(%s) %#v", n.ln.ID().TerminalString(), pp.NewFormatter(dec))
	if dec.Kind() != ptype {
		t.Fatalf("expected packet type %d, got %d", ptype, dec.Kind())
	}
	return dec
}

func (n *handshakeTestNode) expectDecodeErr(t *testing.T, wantErr error, p []byte) {
	t.Helper()
	if _, err := n.decode(p); !errors.Is(err, wantErr) {
		t.Fatal(fmt.Errorf("(%s) got err %q, want %q", n.ln.ID().TerminalString(), err, wantErr))
	}
}

func (n *handshakeTestNode) decode(input []byte) (Packet, error) {
	_, _, p, err := n.c.Decode(input, "127.0.0.1")
	return p, err
}

func (n *handshakeTestNode) n() *enode.Node {
	return n.ln.Node()
}

func (n *handshakeTestNode) addr() string {
	return n.ln.Node().IP().String()
}

func (n *handshakeTestNode) id() enode.ID {
	return n.ln.ID()
}

// hexFile reads the given file and decodes the hex data contained in it.
// Whitespace and any lines beginning with the # character are ignored.
func hexFile(file string) []byte {
	fileContent, err := os.ReadFile(file)
	if err != nil {
		panic(err)
	}

	// Gather hex data, ignore comments.
	var text []byte
	for _, line := range bytes.Split(fileContent, []byte("\n")) {
		line = bytes.TrimSpace(line)
		if len(line) > 0 && line[0] == '#' {
			continue
		}
		text = append(text, line...)
	}

	// Parse the hex.
	if bytes.HasPrefix(text, []byte("0x")) {
		text = text[2:]
	}
	data := make([]byte, hex.DecodedLen(len(text)))
	if _, err := hex.Decode(data, text); err != nil {
		panic("invalid hex in " + file)
	}
	return data
}

// writeTestVector writes a test vector file with the given commentary and binary data.
func writeTestVector(file, comment string, data []byte) {
	fd, err := os.OpenFile(file, os.O_WRONLY|os.O_CREATE|os.O_TRUNC, 0644)
	if err != nil {
		panic(err)
	}
	defer fd.Close()

	if len(comment) > 0 {
		for _, line := range strings.Split(strings.TrimSpace(comment), "\n") {
			fmt.Fprintf(fd, "# %s\n", line)
		}
		fmt.Fprintln(fd)
	}
	for len(data) > 0 {
		var chunk []byte
		if len(data) < 32 {
			chunk = data
		} else {
			chunk = data[:32]
		}
		data = data[len(chunk):]
		fmt.Fprintf(fd, "%x\n", chunk)
	}
}
