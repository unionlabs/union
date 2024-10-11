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

package v5test

import (
	"bytes"
	"crypto/ecdsa"
	"encoding/binary"
	"fmt"
	"net"
	"time"

	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/p2p/discover/v5wire"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/enr"
)

// readError represents an error during packet reading.
// This exists to facilitate type-switching on the result of conn.read.
type readError struct {
	err error
}

func (p *readError) Kind() byte          { return 99 }
func (p *readError) Name() string        { return fmt.Sprintf("error: %v", p.err) }
func (p *readError) Error() string       { return p.err.Error() }
func (p *readError) Unwrap() error       { return p.err }
func (p *readError) RequestID() []byte   { return nil }
func (p *readError) SetRequestID([]byte) {}

func (p *readError) AppendLogInfo(ctx []interface{}) []interface{} { return ctx }

// readErrorf creates a readError with the given text.
func readErrorf(format string, args ...interface{}) *readError {
	return &readError{fmt.Errorf(format, args...)}
}

// This is the response timeout used in tests.
const waitTime = 300 * time.Millisecond

// conn is a connection to the node under test.
type conn struct {
	localNode  *enode.LocalNode
	localKey   *ecdsa.PrivateKey
	remote     *enode.Node
	remoteAddr *net.UDPAddr
	listeners  []net.PacketConn

	log       logger
	codec     *v5wire.Codec
	idCounter uint32
}

type logger interface {
	Logf(string, ...interface{})
}

// newConn sets up a connection to the given node.
func newConn(dest *enode.Node, log logger) *conn {
	key, err := crypto.GenerateKey()
	if err != nil {
		panic(err)
	}
	db, err := enode.OpenDB("")
	if err != nil {
		panic(err)
	}
	ln := enode.NewLocalNode(db, key)

	return &conn{
		localKey:   key,
		localNode:  ln,
		remote:     dest,
		remoteAddr: &net.UDPAddr{IP: dest.IP(), Port: dest.UDP()},
		codec:      v5wire.NewCodec(ln, key, mclock.System{}, nil),
		log:        log,
	}
}

func (tc *conn) setEndpoint(c net.PacketConn) {
	tc.localNode.SetStaticIP(laddr(c).IP)
	tc.localNode.SetFallbackUDP(laddr(c).Port)
}

func (tc *conn) listen(ip string) net.PacketConn {
	l, err := net.ListenPacket("udp", fmt.Sprintf("%v:0", ip))
	if err != nil {
		panic(err)
	}
	tc.listeners = append(tc.listeners, l)
	return l
}

// close shuts down all listeners and the local node.
func (tc *conn) close() {
	for _, l := range tc.listeners {
		l.Close()
	}
	tc.localNode.Database().Close()
}

// nextReqID creates a request id.
func (tc *conn) nextReqID() []byte {
	id := make([]byte, 4)
	tc.idCounter++
	binary.BigEndian.PutUint32(id, tc.idCounter)
	return id
}

// reqresp performs a request/response interaction on the given connection.
// The request is retried if a handshake is requested.
func (tc *conn) reqresp(c net.PacketConn, req v5wire.Packet) v5wire.Packet {
	reqnonce := tc.write(c, req, nil)
	switch resp := tc.read(c).(type) {
	case *v5wire.Whoareyou:
		if resp.Nonce != reqnonce {
			return readErrorf("wrong nonce %x in WHOAREYOU (want %x)", resp.Nonce[:], reqnonce[:])
		}
		resp.Node = tc.remote
		tc.write(c, req, resp)
		return tc.read(c)
	default:
		return resp
	}
}

// findnode sends a FINDNODE request and waits for its responses.
func (tc *conn) findnode(c net.PacketConn, dists []uint) ([]*enode.Node, error) {
	var (
		findnode = &v5wire.Findnode{ReqID: tc.nextReqID(), Distances: dists}
		reqnonce = tc.write(c, findnode, nil)
		first    = true
		total    uint8
		results  []*enode.Node
	)
	for n := 1; n > 0; {
		switch resp := tc.read(c).(type) {
		case *v5wire.Whoareyou:
			// Handle handshake.
			if resp.Nonce == reqnonce {
				resp.Node = tc.remote
				tc.write(c, findnode, resp)
			} else {
				return nil, fmt.Errorf("unexpected WHOAREYOU (nonce %x), waiting for NODES", resp.Nonce[:])
			}
		case *v5wire.Ping:
			// Handle ping from remote.
			tc.write(c, &v5wire.Pong{
				ReqID:  resp.ReqID,
				ENRSeq: tc.localNode.Seq(),
			}, nil)
		case *v5wire.Nodes:
			// Got NODES! Check request ID.
			if !bytes.Equal(resp.ReqID, findnode.ReqID) {
				return nil, fmt.Errorf("NODES response has wrong request id %x", resp.ReqID)
			}
			// Check total count. It should be greater than one
			// and needs to be the same across all responses.
			if first {
				if resp.RespCount == 0 || resp.RespCount > 6 {
					return nil, fmt.Errorf("invalid NODES response count %d (not in (0,7))", resp.RespCount)
				}
				total = resp.RespCount
				n = int(total) - 1
				first = false
			} else {
				n--
				if resp.RespCount != total {
					return nil, fmt.Errorf("invalid NODES response count %d (!= %d)", resp.RespCount, total)
				}
			}
			// Check nodes.
			nodes, err := checkRecords(resp.Nodes)
			if err != nil {
				return nil, fmt.Errorf("invalid node in NODES response: %v", err)
			}
			results = append(results, nodes...)
		default:
			return nil, fmt.Errorf("expected NODES, got %v", resp)
		}
	}
	return results, nil
}

// write sends a packet on the given connection.
func (tc *conn) write(c net.PacketConn, p v5wire.Packet, challenge *v5wire.Whoareyou) v5wire.Nonce {
	packet, nonce, err := tc.codec.Encode(tc.remote.ID(), tc.remoteAddr.String(), p, challenge)
	if err != nil {
		panic(fmt.Errorf("can't encode %v packet: %v", p.Name(), err))
	}
	if _, err := c.WriteTo(packet, tc.remoteAddr); err != nil {
		tc.logf("Can't send %s: %v", p.Name(), err)
	} else {
		tc.logf(">> %s", p.Name())
	}
	return nonce
}

// read waits for an incoming packet on the given connection.
func (tc *conn) read(c net.PacketConn) v5wire.Packet {
	buf := make([]byte, 1280)
	if err := c.SetReadDeadline(time.Now().Add(waitTime)); err != nil {
		return &readError{err}
	}
	n, fromAddr, err := c.ReadFrom(buf)
	if err != nil {
		return &readError{err}
	}
	_, _, p, err := tc.codec.Decode(buf[:n], fromAddr.String())
	if err != nil {
		return &readError{err}
	}
	tc.logf("<< %s", p.Name())
	return p
}

// logf prints to the test log.
func (tc *conn) logf(format string, args ...interface{}) {
	if tc.log != nil {
		tc.log.Logf("(%s) %s", tc.localNode.ID().TerminalString(), fmt.Sprintf(format, args...))
	}
}

func laddr(c net.PacketConn) *net.UDPAddr {
	return c.LocalAddr().(*net.UDPAddr)
}

func checkRecords(records []*enr.Record) ([]*enode.Node, error) {
	nodes := make([]*enode.Node, len(records))
	for i := range records {
		n, err := enode.New(enode.ValidSchemes, records[i])
		if err != nil {
			return nil, err
		}
		nodes[i] = n
	}
	return nodes, nil
}

func containsUint(ints []uint, x uint) bool {
	for i := range ints {
		if ints[i] == x {
			return true
		}
	}
	return false
}
