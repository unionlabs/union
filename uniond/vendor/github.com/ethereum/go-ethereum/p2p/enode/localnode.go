// Copyright 2018 The go-ethereum Authors
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

package enode

import (
	"crypto/ecdsa"
	"fmt"
	"net"
	"reflect"
	"strconv"
	"sync"
	"sync/atomic"
	"time"

	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/p2p/enr"
	"github.com/ethereum/go-ethereum/p2p/netutil"
)

const (
	// IP tracker configuration
	iptrackMinStatements = 10
	iptrackWindow        = 5 * time.Minute
	iptrackContactWindow = 10 * time.Minute

	// time needed to wait between two updates to the local ENR
	recordUpdateThrottle = time.Millisecond
)

// LocalNode produces the signed node record of a local node, i.e. a node run in the
// current process. Setting ENR entries via the Set method updates the record. A new version
// of the record is signed on demand when the Node method is called.
type LocalNode struct {
	cur atomic.Value // holds a non-nil node pointer while the record is up-to-date

	id  ID
	key *ecdsa.PrivateKey
	db  *DB

	// everything below is protected by a lock
	mu        sync.RWMutex
	seq       uint64
	update    time.Time // timestamp when the record was last updated
	entries   map[string]enr.Entry
	endpoint4 lnEndpoint
	endpoint6 lnEndpoint
}

type lnEndpoint struct {
	track                *netutil.IPTracker
	staticIP, fallbackIP net.IP
	fallbackUDP          uint16 // port
}

// NewLocalNode creates a local node.
func NewLocalNode(db *DB, key *ecdsa.PrivateKey) *LocalNode {
	ln := &LocalNode{
		id:      PubkeyToIDV4(&key.PublicKey),
		db:      db,
		key:     key,
		entries: make(map[string]enr.Entry),
		endpoint4: lnEndpoint{
			track: netutil.NewIPTracker(iptrackWindow, iptrackContactWindow, iptrackMinStatements),
		},
		endpoint6: lnEndpoint{
			track: netutil.NewIPTracker(iptrackWindow, iptrackContactWindow, iptrackMinStatements),
		},
	}
	ln.seq = db.localSeq(ln.id)
	ln.update = time.Now()
	ln.cur.Store((*Node)(nil))
	return ln
}

// Database returns the node database associated with the local node.
func (ln *LocalNode) Database() *DB {
	return ln.db
}

// Node returns the current version of the local node record.
func (ln *LocalNode) Node() *Node {
	// If we have a valid record, return that
	n := ln.cur.Load().(*Node)
	if n != nil {
		return n
	}

	// Record was invalidated, sign a new copy.
	ln.mu.Lock()
	defer ln.mu.Unlock()

	// Double check the current record, since multiple goroutines might be waiting
	// on the write mutex.
	if n = ln.cur.Load().(*Node); n != nil {
		return n
	}

	// The initial sequence number is the current timestamp in milliseconds. To ensure
	// that the initial sequence number will always be higher than any previous sequence
	// number (assuming the clock is correct), we want to avoid updating the record faster
	// than once per ms. So we need to sleep here until the next possible update time has
	// arrived.
	lastChange := time.Since(ln.update)
	if lastChange < recordUpdateThrottle {
		time.Sleep(recordUpdateThrottle - lastChange)
	}

	ln.sign()
	ln.update = time.Now()
	return ln.cur.Load().(*Node)
}

// Seq returns the current sequence number of the local node record.
func (ln *LocalNode) Seq() uint64 {
	ln.mu.Lock()
	defer ln.mu.Unlock()

	return ln.seq
}

// ID returns the local node ID.
func (ln *LocalNode) ID() ID {
	return ln.id
}

// Set puts the given entry into the local record, overwriting any existing value.
// Use Set*IP and SetFallbackUDP to set IP addresses and UDP port, otherwise they'll
// be overwritten by the endpoint predictor.
//
// Since node record updates are throttled to one per second, Set is asynchronous.
// Any update will be queued up and published when at least one second passes from
// the last change.
func (ln *LocalNode) Set(e enr.Entry) {
	ln.mu.Lock()
	defer ln.mu.Unlock()

	ln.set(e)
}

func (ln *LocalNode) set(e enr.Entry) {
	val, exists := ln.entries[e.ENRKey()]
	if !exists || !reflect.DeepEqual(val, e) {
		ln.entries[e.ENRKey()] = e
		ln.invalidate()
	}
}

// Delete removes the given entry from the local record.
func (ln *LocalNode) Delete(e enr.Entry) {
	ln.mu.Lock()
	defer ln.mu.Unlock()

	ln.delete(e)
}

func (ln *LocalNode) delete(e enr.Entry) {
	_, exists := ln.entries[e.ENRKey()]
	if exists {
		delete(ln.entries, e.ENRKey())
		ln.invalidate()
	}
}

func (ln *LocalNode) endpointForIP(ip net.IP) *lnEndpoint {
	if ip.To4() != nil {
		return &ln.endpoint4
	}
	return &ln.endpoint6
}

// SetStaticIP sets the local IP to the given one unconditionally.
// This disables endpoint prediction.
func (ln *LocalNode) SetStaticIP(ip net.IP) {
	ln.mu.Lock()
	defer ln.mu.Unlock()

	ln.endpointForIP(ip).staticIP = ip
	ln.updateEndpoints()
}

// SetFallbackIP sets the last-resort IP address. This address is used
// if no endpoint prediction can be made and no static IP is set.
func (ln *LocalNode) SetFallbackIP(ip net.IP) {
	ln.mu.Lock()
	defer ln.mu.Unlock()

	ln.endpointForIP(ip).fallbackIP = ip
	ln.updateEndpoints()
}

// SetFallbackUDP sets the last-resort UDP-on-IPv4 port. This port is used
// if no endpoint prediction can be made.
func (ln *LocalNode) SetFallbackUDP(port int) {
	ln.mu.Lock()
	defer ln.mu.Unlock()

	ln.endpoint4.fallbackUDP = uint16(port)
	ln.endpoint6.fallbackUDP = uint16(port)
	ln.updateEndpoints()
}

// UDPEndpointStatement should be called whenever a statement about the local node's
// UDP endpoint is received. It feeds the local endpoint predictor.
func (ln *LocalNode) UDPEndpointStatement(fromaddr, endpoint *net.UDPAddr) {
	ln.mu.Lock()
	defer ln.mu.Unlock()

	ln.endpointForIP(endpoint.IP).track.AddStatement(fromaddr.String(), endpoint.String())
	ln.updateEndpoints()
}

// UDPContact should be called whenever the local node has announced itself to another node
// via UDP. It feeds the local endpoint predictor.
func (ln *LocalNode) UDPContact(toaddr *net.UDPAddr) {
	ln.mu.Lock()
	defer ln.mu.Unlock()

	ln.endpointForIP(toaddr.IP).track.AddContact(toaddr.String())
	ln.updateEndpoints()
}

// updateEndpoints updates the record with predicted endpoints.
func (ln *LocalNode) updateEndpoints() {
	ip4, udp4 := ln.endpoint4.get()
	ip6, udp6 := ln.endpoint6.get()

	if ip4 != nil && !ip4.IsUnspecified() {
		ln.set(enr.IPv4(ip4))
	} else {
		ln.delete(enr.IPv4{})
	}
	if ip6 != nil && !ip6.IsUnspecified() {
		ln.set(enr.IPv6(ip6))
	} else {
		ln.delete(enr.IPv6{})
	}
	if udp4 != 0 {
		ln.set(enr.UDP(udp4))
	} else {
		ln.delete(enr.UDP(0))
	}
	if udp6 != 0 && udp6 != udp4 {
		ln.set(enr.UDP6(udp6))
	} else {
		ln.delete(enr.UDP6(0))
	}
}

// get returns the endpoint with highest precedence.
func (e *lnEndpoint) get() (newIP net.IP, newPort uint16) {
	newPort = e.fallbackUDP
	if e.fallbackIP != nil {
		newIP = e.fallbackIP
	}
	if e.staticIP != nil {
		newIP = e.staticIP
	} else if ip, port := predictAddr(e.track); ip != nil {
		newIP = ip
		newPort = port
	}
	return newIP, newPort
}

// predictAddr wraps IPTracker.PredictEndpoint, converting from its string-based
// endpoint representation to IP and port types.
func predictAddr(t *netutil.IPTracker) (net.IP, uint16) {
	ep := t.PredictEndpoint()
	if ep == "" {
		return nil, 0
	}
	ipString, portString, _ := net.SplitHostPort(ep)
	ip := net.ParseIP(ipString)
	port, err := strconv.ParseUint(portString, 10, 16)
	if err != nil {
		return nil, 0
	}
	return ip, uint16(port)
}

func (ln *LocalNode) invalidate() {
	ln.cur.Store((*Node)(nil))
}

func (ln *LocalNode) sign() {
	if n := ln.cur.Load().(*Node); n != nil {
		return // no changes
	}

	var r enr.Record
	for _, e := range ln.entries {
		r.Set(e)
	}
	ln.bumpSeq()
	r.SetSeq(ln.seq)
	if err := SignV4(&r, ln.key); err != nil {
		panic(fmt.Errorf("enode: can't sign record: %v", err))
	}
	n, err := New(ValidSchemes, &r)
	if err != nil {
		panic(fmt.Errorf("enode: can't verify local record: %v", err))
	}
	ln.cur.Store(n)
	log.Info("New local node record", "seq", ln.seq, "id", n.ID(), "ip", n.IP(), "udp", n.UDP(), "tcp", n.TCP())
}

func (ln *LocalNode) bumpSeq() {
	ln.seq++
	ln.db.storeLocalSeq(ln.id, ln.seq)
}

// nowMilliseconds gives the current timestamp at millisecond precision.
func nowMilliseconds() uint64 {
	ns := time.Now().UnixNano()
	if ns < 0 {
		return 0
	}
	return uint64(ns / 1000 / 1000)
}
