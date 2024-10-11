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
	"crypto/ecdsa"
	crand "crypto/rand"
	"encoding/binary"
	"time"

	"github.com/ethereum/go-ethereum/common/lru"
	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/p2p/enode"
)

const handshakeTimeout = time.Second

// The SessionCache keeps negotiated encryption keys and
// state for in-progress handshakes in the Discovery v5 wire protocol.
type SessionCache struct {
	sessions   lru.BasicLRU[sessionID, *session]
	handshakes map[sessionID]*Whoareyou
	clock      mclock.Clock

	// hooks for overriding randomness.
	nonceGen        func(uint32) (Nonce, error)
	maskingIVGen    func([]byte) error
	ephemeralKeyGen func() (*ecdsa.PrivateKey, error)
}

// sessionID identifies a session or handshake.
type sessionID struct {
	id   enode.ID
	addr string
}

// session contains session information
type session struct {
	writeKey     []byte
	readKey      []byte
	nonceCounter uint32
}

// keysFlipped returns a copy of s with the read and write keys flipped.
func (s *session) keysFlipped() *session {
	return &session{s.readKey, s.writeKey, s.nonceCounter}
}

func NewSessionCache(maxItems int, clock mclock.Clock) *SessionCache {
	return &SessionCache{
		sessions:        lru.NewBasicLRU[sessionID, *session](maxItems),
		handshakes:      make(map[sessionID]*Whoareyou),
		clock:           clock,
		nonceGen:        generateNonce,
		maskingIVGen:    generateMaskingIV,
		ephemeralKeyGen: crypto.GenerateKey,
	}
}

func generateNonce(counter uint32) (n Nonce, err error) {
	binary.BigEndian.PutUint32(n[:4], counter)
	_, err = crand.Read(n[4:])
	return n, err
}

func generateMaskingIV(buf []byte) error {
	_, err := crand.Read(buf)
	return err
}

// nextNonce creates a nonce for encrypting a message to the given session.
func (sc *SessionCache) nextNonce(s *session) (Nonce, error) {
	s.nonceCounter++
	return sc.nonceGen(s.nonceCounter)
}

// session returns the current session for the given node, if any.
func (sc *SessionCache) session(id enode.ID, addr string) *session {
	item, _ := sc.sessions.Get(sessionID{id, addr})
	return item
}

// readKey returns the current read key for the given node.
func (sc *SessionCache) readKey(id enode.ID, addr string) []byte {
	if s := sc.session(id, addr); s != nil {
		return s.readKey
	}
	return nil
}

// storeNewSession stores new encryption keys in the cache.
func (sc *SessionCache) storeNewSession(id enode.ID, addr string, s *session) {
	sc.sessions.Add(sessionID{id, addr}, s)
}

// getHandshake gets the handshake challenge we previously sent to the given remote node.
func (sc *SessionCache) getHandshake(id enode.ID, addr string) *Whoareyou {
	return sc.handshakes[sessionID{id, addr}]
}

// storeSentHandshake stores the handshake challenge sent to the given remote node.
func (sc *SessionCache) storeSentHandshake(id enode.ID, addr string, challenge *Whoareyou) {
	challenge.sent = sc.clock.Now()
	sc.handshakes[sessionID{id, addr}] = challenge
}

// deleteHandshake deletes handshake data for the given node.
func (sc *SessionCache) deleteHandshake(id enode.ID, addr string) {
	delete(sc.handshakes, sessionID{id, addr})
}

// handshakeGC deletes timed-out handshakes.
func (sc *SessionCache) handshakeGC() {
	deadline := sc.clock.Now().Add(-handshakeTimeout)
	for key, challenge := range sc.handshakes {
		if challenge.sent < deadline {
			delete(sc.handshakes, key)
		}
	}
}
