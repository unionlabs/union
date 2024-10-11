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
	"bytes"
	"container/list"
	"context"
	"crypto/ecdsa"
	crand "crypto/rand"
	"errors"
	"fmt"
	"io"
	"net"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/p2p/discover/v4wire"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/netutil"
)

// Errors
var (
	errExpired          = errors.New("expired")
	errUnsolicitedReply = errors.New("unsolicited reply")
	errUnknownNode      = errors.New("unknown node")
	errTimeout          = errors.New("RPC timeout")
	errClockWarp        = errors.New("reply deadline too far in the future")
	errClosed           = errors.New("socket closed")
	errLowPort          = errors.New("low port")
)

const (
	respTimeout    = 500 * time.Millisecond
	expiration     = 20 * time.Second
	bondExpiration = 24 * time.Hour

	maxFindnodeFailures = 5                // nodes exceeding this limit are dropped
	ntpFailureThreshold = 32               // Continuous timeouts after which to check NTP
	ntpWarningCooldown  = 10 * time.Minute // Minimum amount of time to pass before repeating NTP warning
	driftThreshold      = 10 * time.Second // Allowed clock drift before warning user

	// Discovery packets are defined to be no larger than 1280 bytes.
	// Packets larger than this size will be cut at the end and treated
	// as invalid because their hash won't match.
	maxPacketSize = 1280
)

// UDPv4 implements the v4 wire protocol.
type UDPv4 struct {
	conn        UDPConn
	log         log.Logger
	netrestrict *netutil.Netlist
	priv        *ecdsa.PrivateKey
	localNode   *enode.LocalNode
	db          *enode.DB
	tab         *Table
	closeOnce   sync.Once
	wg          sync.WaitGroup

	addReplyMatcher chan *replyMatcher
	gotreply        chan reply
	closeCtx        context.Context
	cancelCloseCtx  context.CancelFunc
}

// replyMatcher represents a pending reply.
//
// Some implementations of the protocol wish to send more than one
// reply packet to findnode. In general, any neighbors packet cannot
// be matched up with a specific findnode packet.
//
// Our implementation handles this by storing a callback function for
// each pending reply. Incoming packets from a node are dispatched
// to all callback functions for that node.
type replyMatcher struct {
	// these fields must match in the reply.
	from  enode.ID
	ip    net.IP
	ptype byte

	// time when the request must complete
	deadline time.Time

	// callback is called when a matching reply arrives. If it returns matched == true, the
	// reply was acceptable. The second return value indicates whether the callback should
	// be removed from the pending reply queue. If it returns false, the reply is considered
	// incomplete and the callback will be invoked again for the next matching reply.
	callback replyMatchFunc

	// errc receives nil when the callback indicates completion or an
	// error if no further reply is received within the timeout.
	errc chan error

	// reply contains the most recent reply. This field is safe for reading after errc has
	// received a value.
	reply v4wire.Packet
}

type replyMatchFunc func(v4wire.Packet) (matched bool, requestDone bool)

// reply is a reply packet from a certain node.
type reply struct {
	from enode.ID
	ip   net.IP
	data v4wire.Packet
	// loop indicates whether there was
	// a matching request by sending on this channel.
	matched chan<- bool
}

func ListenV4(c UDPConn, ln *enode.LocalNode, cfg Config) (*UDPv4, error) {
	cfg = cfg.withDefaults()
	closeCtx, cancel := context.WithCancel(context.Background())
	t := &UDPv4{
		conn:            newMeteredConn(c),
		priv:            cfg.PrivateKey,
		netrestrict:     cfg.NetRestrict,
		localNode:       ln,
		db:              ln.Database(),
		gotreply:        make(chan reply),
		addReplyMatcher: make(chan *replyMatcher),
		closeCtx:        closeCtx,
		cancelCloseCtx:  cancel,
		log:             cfg.Log,
	}

	tab, err := newMeteredTable(t, ln.Database(), cfg)
	if err != nil {
		return nil, err
	}
	t.tab = tab
	go tab.loop()

	t.wg.Add(2)
	go t.loop()
	go t.readLoop(cfg.Unhandled)
	return t, nil
}

// Self returns the local node.
func (t *UDPv4) Self() *enode.Node {
	return t.localNode.Node()
}

// Close shuts down the socket and aborts any running queries.
func (t *UDPv4) Close() {
	t.closeOnce.Do(func() {
		t.cancelCloseCtx()
		t.conn.Close()
		t.wg.Wait()
		t.tab.close()
	})
}

// Resolve searches for a specific node with the given ID and tries to get the most recent
// version of the node record for it. It returns n if the node could not be resolved.
func (t *UDPv4) Resolve(n *enode.Node) *enode.Node {
	// Try asking directly. This works if the node is still responding on the endpoint we have.
	if rn, err := t.RequestENR(n); err == nil {
		return rn
	}
	// Check table for the ID, we might have a newer version there.
	if intable := t.tab.getNode(n.ID()); intable != nil && intable.Seq() > n.Seq() {
		n = intable
		if rn, err := t.RequestENR(n); err == nil {
			return rn
		}
	}
	// Otherwise perform a network lookup.
	var key enode.Secp256k1
	if n.Load(&key) != nil {
		return n // no secp256k1 key
	}
	result := t.LookupPubkey((*ecdsa.PublicKey)(&key))
	for _, rn := range result {
		if rn.ID() == n.ID() {
			if rn, err := t.RequestENR(rn); err == nil {
				return rn
			}
		}
	}
	return n
}

func (t *UDPv4) ourEndpoint() v4wire.Endpoint {
	n := t.Self()
	a := &net.UDPAddr{IP: n.IP(), Port: n.UDP()}
	return v4wire.NewEndpoint(a, uint16(n.TCP()))
}

// Ping sends a ping message to the given node.
func (t *UDPv4) Ping(n *enode.Node) error {
	_, err := t.ping(n)
	return err
}

// ping sends a ping message to the given node and waits for a reply.
func (t *UDPv4) ping(n *enode.Node) (seq uint64, err error) {
	rm := t.sendPing(n.ID(), &net.UDPAddr{IP: n.IP(), Port: n.UDP()}, nil)
	if err = <-rm.errc; err == nil {
		seq = rm.reply.(*v4wire.Pong).ENRSeq
	}
	return seq, err
}

// sendPing sends a ping message to the given node and invokes the callback
// when the reply arrives.
func (t *UDPv4) sendPing(toid enode.ID, toaddr *net.UDPAddr, callback func()) *replyMatcher {
	req := t.makePing(toaddr)
	packet, hash, err := v4wire.Encode(t.priv, req)
	if err != nil {
		errc := make(chan error, 1)
		errc <- err
		return &replyMatcher{errc: errc}
	}
	// Add a matcher for the reply to the pending reply queue. Pongs are matched if they
	// reference the ping we're about to send.
	rm := t.pending(toid, toaddr.IP, v4wire.PongPacket, func(p v4wire.Packet) (matched bool, requestDone bool) {
		matched = bytes.Equal(p.(*v4wire.Pong).ReplyTok, hash)
		if matched && callback != nil {
			callback()
		}
		return matched, matched
	})
	// Send the packet.
	t.localNode.UDPContact(toaddr)
	t.write(toaddr, toid, req.Name(), packet)
	return rm
}

func (t *UDPv4) makePing(toaddr *net.UDPAddr) *v4wire.Ping {
	return &v4wire.Ping{
		Version:    4,
		From:       t.ourEndpoint(),
		To:         v4wire.NewEndpoint(toaddr, 0),
		Expiration: uint64(time.Now().Add(expiration).Unix()),
		ENRSeq:     t.localNode.Node().Seq(),
	}
}

// LookupPubkey finds the closest nodes to the given public key.
func (t *UDPv4) LookupPubkey(key *ecdsa.PublicKey) []*enode.Node {
	if t.tab.len() == 0 {
		// All nodes were dropped, refresh. The very first query will hit this
		// case and run the bootstrapping logic.
		<-t.tab.refresh()
	}
	return t.newLookup(t.closeCtx, encodePubkey(key)).run()
}

// RandomNodes is an iterator yielding nodes from a random walk of the DHT.
func (t *UDPv4) RandomNodes() enode.Iterator {
	return newLookupIterator(t.closeCtx, t.newRandomLookup)
}

// lookupRandom implements transport.
func (t *UDPv4) lookupRandom() []*enode.Node {
	return t.newRandomLookup(t.closeCtx).run()
}

// lookupSelf implements transport.
func (t *UDPv4) lookupSelf() []*enode.Node {
	return t.newLookup(t.closeCtx, encodePubkey(&t.priv.PublicKey)).run()
}

func (t *UDPv4) newRandomLookup(ctx context.Context) *lookup {
	var target encPubkey
	crand.Read(target[:])
	return t.newLookup(ctx, target)
}

func (t *UDPv4) newLookup(ctx context.Context, targetKey encPubkey) *lookup {
	target := enode.ID(crypto.Keccak256Hash(targetKey[:]))
	ekey := v4wire.Pubkey(targetKey)
	it := newLookup(ctx, t.tab, target, func(n *node) ([]*node, error) {
		return t.findnode(n.ID(), n.addr(), ekey)
	})
	return it
}

// findnode sends a findnode request to the given node and waits until
// the node has sent up to k neighbors.
func (t *UDPv4) findnode(toid enode.ID, toaddr *net.UDPAddr, target v4wire.Pubkey) ([]*node, error) {
	t.ensureBond(toid, toaddr)

	// Add a matcher for 'neighbours' replies to the pending reply queue. The matcher is
	// active until enough nodes have been received.
	nodes := make([]*node, 0, bucketSize)
	nreceived := 0
	rm := t.pending(toid, toaddr.IP, v4wire.NeighborsPacket, func(r v4wire.Packet) (matched bool, requestDone bool) {
		reply := r.(*v4wire.Neighbors)
		for _, rn := range reply.Nodes {
			nreceived++
			n, err := t.nodeFromRPC(toaddr, rn)
			if err != nil {
				t.log.Trace("Invalid neighbor node received", "ip", rn.IP, "addr", toaddr, "err", err)
				continue
			}
			nodes = append(nodes, n)
		}
		return true, nreceived >= bucketSize
	})
	t.send(toaddr, toid, &v4wire.Findnode{
		Target:     target,
		Expiration: uint64(time.Now().Add(expiration).Unix()),
	})
	// Ensure that callers don't see a timeout if the node actually responded. Since
	// findnode can receive more than one neighbors response, the reply matcher will be
	// active until the remote node sends enough nodes. If the remote end doesn't have
	// enough nodes the reply matcher will time out waiting for the second reply, but
	// there's no need for an error in that case.
	err := <-rm.errc
	if errors.Is(err, errTimeout) && rm.reply != nil {
		err = nil
	}
	return nodes, err
}

// RequestENR sends ENRRequest to the given node and waits for a response.
func (t *UDPv4) RequestENR(n *enode.Node) (*enode.Node, error) {
	addr := &net.UDPAddr{IP: n.IP(), Port: n.UDP()}
	t.ensureBond(n.ID(), addr)

	req := &v4wire.ENRRequest{
		Expiration: uint64(time.Now().Add(expiration).Unix()),
	}
	packet, hash, err := v4wire.Encode(t.priv, req)
	if err != nil {
		return nil, err
	}

	// Add a matcher for the reply to the pending reply queue. Responses are matched if
	// they reference the request we're about to send.
	rm := t.pending(n.ID(), addr.IP, v4wire.ENRResponsePacket, func(r v4wire.Packet) (matched bool, requestDone bool) {
		matched = bytes.Equal(r.(*v4wire.ENRResponse).ReplyTok, hash)
		return matched, matched
	})
	// Send the packet and wait for the reply.
	t.write(addr, n.ID(), req.Name(), packet)
	if err := <-rm.errc; err != nil {
		return nil, err
	}
	// Verify the response record.
	respN, err := enode.New(enode.ValidSchemes, &rm.reply.(*v4wire.ENRResponse).Record)
	if err != nil {
		return nil, err
	}
	if respN.ID() != n.ID() {
		return nil, fmt.Errorf("invalid ID in response record")
	}
	if respN.Seq() < n.Seq() {
		return n, nil // response record is older
	}
	if err := netutil.CheckRelayIP(addr.IP, respN.IP()); err != nil {
		return nil, fmt.Errorf("invalid IP in response record: %v", err)
	}
	return respN, nil
}

// pending adds a reply matcher to the pending reply queue.
// see the documentation of type replyMatcher for a detailed explanation.
func (t *UDPv4) pending(id enode.ID, ip net.IP, ptype byte, callback replyMatchFunc) *replyMatcher {
	ch := make(chan error, 1)
	p := &replyMatcher{from: id, ip: ip, ptype: ptype, callback: callback, errc: ch}
	select {
	case t.addReplyMatcher <- p:
		// loop will handle it
	case <-t.closeCtx.Done():
		ch <- errClosed
	}
	return p
}

// handleReply dispatches a reply packet, invoking reply matchers. It returns
// whether any matcher considered the packet acceptable.
func (t *UDPv4) handleReply(from enode.ID, fromIP net.IP, req v4wire.Packet) bool {
	matched := make(chan bool, 1)
	select {
	case t.gotreply <- reply{from, fromIP, req, matched}:
		// loop will handle it
		return <-matched
	case <-t.closeCtx.Done():
		return false
	}
}

// loop runs in its own goroutine. it keeps track of
// the refresh timer and the pending reply queue.
func (t *UDPv4) loop() {
	defer t.wg.Done()

	var (
		plist        = list.New()
		timeout      = time.NewTimer(0)
		nextTimeout  *replyMatcher // head of plist when timeout was last reset
		contTimeouts = 0           // number of continuous timeouts to do NTP checks
		ntpWarnTime  = time.Unix(0, 0)
	)
	<-timeout.C // ignore first timeout
	defer timeout.Stop()

	resetTimeout := func() {
		if plist.Front() == nil || nextTimeout == plist.Front().Value {
			return
		}
		// Start the timer so it fires when the next pending reply has expired.
		now := time.Now()
		for el := plist.Front(); el != nil; el = el.Next() {
			nextTimeout = el.Value.(*replyMatcher)
			if dist := nextTimeout.deadline.Sub(now); dist < 2*respTimeout {
				timeout.Reset(dist)
				return
			}
			// Remove pending replies whose deadline is too far in the
			// future. These can occur if the system clock jumped
			// backwards after the deadline was assigned.
			nextTimeout.errc <- errClockWarp
			plist.Remove(el)
		}
		nextTimeout = nil
		timeout.Stop()
	}

	for {
		resetTimeout()

		select {
		case <-t.closeCtx.Done():
			for el := plist.Front(); el != nil; el = el.Next() {
				el.Value.(*replyMatcher).errc <- errClosed
			}
			return

		case p := <-t.addReplyMatcher:
			p.deadline = time.Now().Add(respTimeout)
			plist.PushBack(p)

		case r := <-t.gotreply:
			var matched bool // whether any replyMatcher considered the reply acceptable.
			for el := plist.Front(); el != nil; el = el.Next() {
				p := el.Value.(*replyMatcher)
				if p.from == r.from && p.ptype == r.data.Kind() && p.ip.Equal(r.ip) {
					ok, requestDone := p.callback(r.data)
					matched = matched || ok
					p.reply = r.data
					// Remove the matcher if callback indicates that all replies have been received.
					if requestDone {
						p.errc <- nil
						plist.Remove(el)
					}
					// Reset the continuous timeout counter (time drift detection)
					contTimeouts = 0
				}
			}
			r.matched <- matched

		case now := <-timeout.C:
			nextTimeout = nil

			// Notify and remove callbacks whose deadline is in the past.
			for el := plist.Front(); el != nil; el = el.Next() {
				p := el.Value.(*replyMatcher)
				if now.After(p.deadline) || now.Equal(p.deadline) {
					p.errc <- errTimeout
					plist.Remove(el)
					contTimeouts++
				}
			}
			// If we've accumulated too many timeouts, do an NTP time sync check
			if contTimeouts > ntpFailureThreshold {
				if time.Since(ntpWarnTime) >= ntpWarningCooldown {
					ntpWarnTime = time.Now()
					go checkClockDrift()
				}
				contTimeouts = 0
			}
		}
	}
}

func (t *UDPv4) send(toaddr *net.UDPAddr, toid enode.ID, req v4wire.Packet) ([]byte, error) {
	packet, hash, err := v4wire.Encode(t.priv, req)
	if err != nil {
		return hash, err
	}
	return hash, t.write(toaddr, toid, req.Name(), packet)
}

func (t *UDPv4) write(toaddr *net.UDPAddr, toid enode.ID, what string, packet []byte) error {
	_, err := t.conn.WriteToUDP(packet, toaddr)
	t.log.Trace(">> "+what, "id", toid, "addr", toaddr, "err", err)
	return err
}

// readLoop runs in its own goroutine. it handles incoming UDP packets.
func (t *UDPv4) readLoop(unhandled chan<- ReadPacket) {
	defer t.wg.Done()
	if unhandled != nil {
		defer close(unhandled)
	}

	buf := make([]byte, maxPacketSize)
	for {
		nbytes, from, err := t.conn.ReadFromUDP(buf)
		if netutil.IsTemporaryError(err) {
			// Ignore temporary read errors.
			t.log.Debug("Temporary UDP read error", "err", err)
			continue
		} else if err != nil {
			// Shut down the loop for permanent errors.
			if !errors.Is(err, io.EOF) {
				t.log.Debug("UDP read error", "err", err)
			}
			return
		}
		if t.handlePacket(from, buf[:nbytes]) != nil && unhandled != nil {
			select {
			case unhandled <- ReadPacket{buf[:nbytes], from}:
			default:
			}
		}
	}
}

func (t *UDPv4) handlePacket(from *net.UDPAddr, buf []byte) error {
	rawpacket, fromKey, hash, err := v4wire.Decode(buf)
	if err != nil {
		t.log.Debug("Bad discv4 packet", "addr", from, "err", err)
		return err
	}
	packet := t.wrapPacket(rawpacket)
	fromID := fromKey.ID()
	if err == nil && packet.preverify != nil {
		err = packet.preverify(packet, from, fromID, fromKey)
	}
	t.log.Trace("<< "+packet.Name(), "id", fromID, "addr", from, "err", err)
	if err == nil && packet.handle != nil {
		packet.handle(packet, from, fromID, hash)
	}
	return err
}

// checkBond checks if the given node has a recent enough endpoint proof.
func (t *UDPv4) checkBond(id enode.ID, ip net.IP) bool {
	return time.Since(t.db.LastPongReceived(id, ip)) < bondExpiration
}

// ensureBond solicits a ping from a node if we haven't seen a ping from it for a while.
// This ensures there is a valid endpoint proof on the remote end.
func (t *UDPv4) ensureBond(toid enode.ID, toaddr *net.UDPAddr) {
	tooOld := time.Since(t.db.LastPingReceived(toid, toaddr.IP)) > bondExpiration
	if tooOld || t.db.FindFails(toid, toaddr.IP) > maxFindnodeFailures {
		rm := t.sendPing(toid, toaddr, nil)
		<-rm.errc
		// Wait for them to ping back and process our pong.
		time.Sleep(respTimeout)
	}
}

func (t *UDPv4) nodeFromRPC(sender *net.UDPAddr, rn v4wire.Node) (*node, error) {
	if rn.UDP <= 1024 {
		return nil, errLowPort
	}
	if err := netutil.CheckRelayIP(sender.IP, rn.IP); err != nil {
		return nil, err
	}
	if t.netrestrict != nil && !t.netrestrict.Contains(rn.IP) {
		return nil, errors.New("not contained in netrestrict list")
	}
	key, err := v4wire.DecodePubkey(crypto.S256(), rn.ID)
	if err != nil {
		return nil, err
	}
	n := wrapNode(enode.NewV4(key, rn.IP, int(rn.TCP), int(rn.UDP)))
	err = n.ValidateComplete()
	return n, err
}

func nodeToRPC(n *node) v4wire.Node {
	var key ecdsa.PublicKey
	var ekey v4wire.Pubkey
	if err := n.Load((*enode.Secp256k1)(&key)); err == nil {
		ekey = v4wire.EncodePubkey(&key)
	}
	return v4wire.Node{ID: ekey, IP: n.IP(), UDP: uint16(n.UDP()), TCP: uint16(n.TCP())}
}

// wrapPacket returns the handler functions applicable to a packet.
func (t *UDPv4) wrapPacket(p v4wire.Packet) *packetHandlerV4 {
	var h packetHandlerV4
	h.Packet = p
	switch p.(type) {
	case *v4wire.Ping:
		h.preverify = t.verifyPing
		h.handle = t.handlePing
	case *v4wire.Pong:
		h.preverify = t.verifyPong
	case *v4wire.Findnode:
		h.preverify = t.verifyFindnode
		h.handle = t.handleFindnode
	case *v4wire.Neighbors:
		h.preverify = t.verifyNeighbors
	case *v4wire.ENRRequest:
		h.preverify = t.verifyENRRequest
		h.handle = t.handleENRRequest
	case *v4wire.ENRResponse:
		h.preverify = t.verifyENRResponse
	}
	return &h
}

// packetHandlerV4 wraps a packet with handler functions.
type packetHandlerV4 struct {
	v4wire.Packet
	senderKey *ecdsa.PublicKey // used for ping

	// preverify checks whether the packet is valid and should be handled at all.
	preverify func(p *packetHandlerV4, from *net.UDPAddr, fromID enode.ID, fromKey v4wire.Pubkey) error
	// handle handles the packet.
	handle func(req *packetHandlerV4, from *net.UDPAddr, fromID enode.ID, mac []byte)
}

// PING/v4

func (t *UDPv4) verifyPing(h *packetHandlerV4, from *net.UDPAddr, fromID enode.ID, fromKey v4wire.Pubkey) error {
	req := h.Packet.(*v4wire.Ping)

	if v4wire.Expired(req.Expiration) {
		return errExpired
	}
	senderKey, err := v4wire.DecodePubkey(crypto.S256(), fromKey)
	if err != nil {
		return err
	}
	h.senderKey = senderKey
	return nil
}

func (t *UDPv4) handlePing(h *packetHandlerV4, from *net.UDPAddr, fromID enode.ID, mac []byte) {
	req := h.Packet.(*v4wire.Ping)

	// Reply.
	t.send(from, fromID, &v4wire.Pong{
		To:         v4wire.NewEndpoint(from, req.From.TCP),
		ReplyTok:   mac,
		Expiration: uint64(time.Now().Add(expiration).Unix()),
		ENRSeq:     t.localNode.Node().Seq(),
	})

	// Ping back if our last pong on file is too far in the past.
	n := wrapNode(enode.NewV4(h.senderKey, from.IP, int(req.From.TCP), from.Port))
	if time.Since(t.db.LastPongReceived(n.ID(), from.IP)) > bondExpiration {
		t.sendPing(fromID, from, func() {
			t.tab.addVerifiedNode(n)
		})
	} else {
		t.tab.addVerifiedNode(n)
	}

	// Update node database and endpoint predictor.
	t.db.UpdateLastPingReceived(n.ID(), from.IP, time.Now())
	t.localNode.UDPEndpointStatement(from, &net.UDPAddr{IP: req.To.IP, Port: int(req.To.UDP)})
}

// PONG/v4

func (t *UDPv4) verifyPong(h *packetHandlerV4, from *net.UDPAddr, fromID enode.ID, fromKey v4wire.Pubkey) error {
	req := h.Packet.(*v4wire.Pong)

	if v4wire.Expired(req.Expiration) {
		return errExpired
	}
	if !t.handleReply(fromID, from.IP, req) {
		return errUnsolicitedReply
	}
	t.localNode.UDPEndpointStatement(from, &net.UDPAddr{IP: req.To.IP, Port: int(req.To.UDP)})
	t.db.UpdateLastPongReceived(fromID, from.IP, time.Now())
	return nil
}

// FINDNODE/v4

func (t *UDPv4) verifyFindnode(h *packetHandlerV4, from *net.UDPAddr, fromID enode.ID, fromKey v4wire.Pubkey) error {
	req := h.Packet.(*v4wire.Findnode)

	if v4wire.Expired(req.Expiration) {
		return errExpired
	}
	if !t.checkBond(fromID, from.IP) {
		// No endpoint proof pong exists, we don't process the packet. This prevents an
		// attack vector where the discovery protocol could be used to amplify traffic in a
		// DDOS attack. A malicious actor would send a findnode request with the IP address
		// and UDP port of the target as the source address. The recipient of the findnode
		// packet would then send a neighbors packet (which is a much bigger packet than
		// findnode) to the victim.
		return errUnknownNode
	}
	return nil
}

func (t *UDPv4) handleFindnode(h *packetHandlerV4, from *net.UDPAddr, fromID enode.ID, mac []byte) {
	req := h.Packet.(*v4wire.Findnode)

	// Determine closest nodes.
	target := enode.ID(crypto.Keccak256Hash(req.Target[:]))
	closest := t.tab.findnodeByID(target, bucketSize, true).entries

	// Send neighbors in chunks with at most maxNeighbors per packet
	// to stay below the packet size limit.
	p := v4wire.Neighbors{Expiration: uint64(time.Now().Add(expiration).Unix())}
	var sent bool
	for _, n := range closest {
		if netutil.CheckRelayIP(from.IP, n.IP()) == nil {
			p.Nodes = append(p.Nodes, nodeToRPC(n))
		}
		if len(p.Nodes) == v4wire.MaxNeighbors {
			t.send(from, fromID, &p)
			p.Nodes = p.Nodes[:0]
			sent = true
		}
	}
	if len(p.Nodes) > 0 || !sent {
		t.send(from, fromID, &p)
	}
}

// NEIGHBORS/v4

func (t *UDPv4) verifyNeighbors(h *packetHandlerV4, from *net.UDPAddr, fromID enode.ID, fromKey v4wire.Pubkey) error {
	req := h.Packet.(*v4wire.Neighbors)

	if v4wire.Expired(req.Expiration) {
		return errExpired
	}
	if !t.handleReply(fromID, from.IP, h.Packet) {
		return errUnsolicitedReply
	}
	return nil
}

// ENRREQUEST/v4

func (t *UDPv4) verifyENRRequest(h *packetHandlerV4, from *net.UDPAddr, fromID enode.ID, fromKey v4wire.Pubkey) error {
	req := h.Packet.(*v4wire.ENRRequest)

	if v4wire.Expired(req.Expiration) {
		return errExpired
	}
	if !t.checkBond(fromID, from.IP) {
		return errUnknownNode
	}
	return nil
}

func (t *UDPv4) handleENRRequest(h *packetHandlerV4, from *net.UDPAddr, fromID enode.ID, mac []byte) {
	t.send(from, fromID, &v4wire.ENRResponse{
		ReplyTok: mac,
		Record:   *t.localNode.Node().Record(),
	})
}

// ENRRESPONSE/v4

func (t *UDPv4) verifyENRResponse(h *packetHandlerV4, from *net.UDPAddr, fromID enode.ID, fromKey v4wire.Pubkey) error {
	if !t.handleReply(fromID, from.IP, h.Packet) {
		return errUnsolicitedReply
	}
	return nil
}
