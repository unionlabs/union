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
	"crypto/aes"
	"crypto/cipher"
	"crypto/ecdsa"
	crand "crypto/rand"
	"crypto/sha256"
	"encoding/binary"
	"errors"
	"fmt"
	"hash"

	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/enr"
	"github.com/ethereum/go-ethereum/rlp"
)

// TODO concurrent WHOAREYOU tie-breaker
// TODO rehandshake after X packets

// Header represents a packet header.
type Header struct {
	IV [sizeofMaskingIV]byte
	StaticHeader
	AuthData []byte

	src enode.ID // used by decoder
}

// StaticHeader contains the static fields of a packet header.
type StaticHeader struct {
	ProtocolID [6]byte
	Version    uint16
	Flag       byte
	Nonce      Nonce
	AuthSize   uint16
}

// Authdata layouts.
type (
	whoareyouAuthData struct {
		IDNonce   [16]byte // ID proof data
		RecordSeq uint64   // highest known ENR sequence of requester
	}

	handshakeAuthData struct {
		h struct {
			SrcID      enode.ID
			SigSize    byte // signature data
			PubkeySize byte // offset of
		}
		// Trailing variable-size data.
		signature, pubkey, record []byte
	}

	messageAuthData struct {
		SrcID enode.ID
	}
)

// Packet header flag values.
const (
	flagMessage = iota
	flagWhoareyou
	flagHandshake
)

// Protocol constants.
const (
	version         = 1
	minVersion      = 1
	sizeofMaskingIV = 16

	// The minimum size of any Discovery v5 packet is 63 bytes.
	// Should reject packets smaller than minPacketSize.
	minPacketSize = 63

	maxPacketSize = 1280

	minMessageSize      = 48 // this refers to data after static headers
	randomPacketMsgSize = 20
)

var DefaultProtocolID = [6]byte{'d', 'i', 's', 'c', 'v', '5'}

// Errors.
var (
	errTooShort            = errors.New("packet too short")
	errInvalidHeader       = errors.New("invalid packet header")
	errInvalidFlag         = errors.New("invalid flag value in header")
	errMinVersion          = errors.New("version of packet header below minimum")
	errMsgTooShort         = errors.New("message/handshake packet below minimum size")
	errAuthSize            = errors.New("declared auth size is beyond packet length")
	errUnexpectedHandshake = errors.New("unexpected auth response, not in handshake")
	errInvalidAuthKey      = errors.New("invalid ephemeral pubkey")
	errNoRecord            = errors.New("expected ENR in handshake but none sent")
	errInvalidNonceSig     = errors.New("invalid ID nonce signature")
	errMessageTooShort     = errors.New("message contains no data")
	errMessageDecrypt      = errors.New("cannot decrypt message")
)

// Public errors.
var (
	// ErrInvalidReqID represents error when the ID is invalid.
	ErrInvalidReqID = errors.New("request ID larger than 8 bytes")
)

// IsInvalidHeader reports whether 'err' is related to an invalid packet header. When it
// returns false, it is pretty certain that the packet causing the error does not belong
// to discv5.
func IsInvalidHeader(err error) bool {
	return err == errTooShort || err == errInvalidHeader || err == errMsgTooShort
}

// Packet sizes.
var (
	sizeofStaticHeader      = binary.Size(StaticHeader{})
	sizeofWhoareyouAuthData = binary.Size(whoareyouAuthData{})
	sizeofHandshakeAuthData = binary.Size(handshakeAuthData{}.h)
	sizeofMessageAuthData   = binary.Size(messageAuthData{})
	sizeofStaticPacketData  = sizeofMaskingIV + sizeofStaticHeader
)

// Codec encodes and decodes Discovery v5 packets.
// This type is not safe for concurrent use.
type Codec struct {
	sha256     hash.Hash
	localnode  *enode.LocalNode
	privkey    *ecdsa.PrivateKey
	sc         *SessionCache
	protocolID [6]byte

	// encoder buffers
	buf      bytes.Buffer // whole packet
	headbuf  bytes.Buffer // packet header
	msgbuf   bytes.Buffer // message RLP plaintext
	msgctbuf []byte       // message data ciphertext

	// decoder buffer
	decbuf []byte
	reader bytes.Reader
}

// NewCodec creates a wire codec.
func NewCodec(ln *enode.LocalNode, key *ecdsa.PrivateKey, clock mclock.Clock, protocolID *[6]byte) *Codec {
	c := &Codec{
		sha256:     sha256.New(),
		localnode:  ln,
		privkey:    key,
		sc:         NewSessionCache(1024, clock),
		protocolID: DefaultProtocolID,
		decbuf:     make([]byte, maxPacketSize),
	}
	if protocolID != nil {
		c.protocolID = *protocolID
	}
	return c
}

// Encode encodes a packet to a node. 'id' and 'addr' specify the destination node. The
// 'challenge' parameter should be the most recently received WHOAREYOU packet from that
// node.
func (c *Codec) Encode(id enode.ID, addr string, packet Packet, challenge *Whoareyou) ([]byte, Nonce, error) {
	// Create the packet header.
	var (
		head    Header
		session *session
		msgData []byte
		err     error
	)
	switch {
	case packet.Kind() == WhoareyouPacket:
		head, err = c.encodeWhoareyou(id, packet.(*Whoareyou))
	case challenge != nil:
		// We have an unanswered challenge, send handshake.
		head, session, err = c.encodeHandshakeHeader(id, addr, challenge)
	default:
		session = c.sc.session(id, addr)
		if session != nil {
			// There is a session, use it.
			head, err = c.encodeMessageHeader(id, session)
		} else {
			// No keys, send random data to kick off the handshake.
			head, msgData, err = c.encodeRandom(id)
		}
	}
	if err != nil {
		return nil, Nonce{}, err
	}

	// Generate masking IV.
	if err := c.sc.maskingIVGen(head.IV[:]); err != nil {
		return nil, Nonce{}, fmt.Errorf("can't generate masking IV: %v", err)
	}

	// Encode header data.
	c.writeHeaders(&head)

	// Store sent WHOAREYOU challenges.
	if challenge, ok := packet.(*Whoareyou); ok {
		challenge.ChallengeData = bytesCopy(&c.buf)
		c.sc.storeSentHandshake(id, addr, challenge)
	} else if msgData == nil {
		headerData := c.buf.Bytes()
		msgData, err = c.encryptMessage(session, packet, &head, headerData)
		if err != nil {
			return nil, Nonce{}, err
		}
	}

	enc, err := c.EncodeRaw(id, head, msgData)
	return enc, head.Nonce, err
}

// EncodeRaw encodes a packet with the given header.
func (c *Codec) EncodeRaw(id enode.ID, head Header, msgdata []byte) ([]byte, error) {
	c.writeHeaders(&head)

	// Apply masking.
	masked := c.buf.Bytes()[sizeofMaskingIV:]
	mask := head.mask(id)
	mask.XORKeyStream(masked[:], masked[:])

	// Write message data.
	c.buf.Write(msgdata)
	return c.buf.Bytes(), nil
}

func (c *Codec) writeHeaders(head *Header) {
	c.buf.Reset()
	c.buf.Write(head.IV[:])
	binary.Write(&c.buf, binary.BigEndian, &head.StaticHeader)
	c.buf.Write(head.AuthData)
}

// makeHeader creates a packet header.
func (c *Codec) makeHeader(toID enode.ID, flag byte, authsizeExtra int) Header {
	var authsize int
	switch flag {
	case flagMessage:
		authsize = sizeofMessageAuthData
	case flagWhoareyou:
		authsize = sizeofWhoareyouAuthData
	case flagHandshake:
		authsize = sizeofHandshakeAuthData
	default:
		panic(fmt.Errorf("BUG: invalid packet header flag %x", flag))
	}
	authsize += authsizeExtra
	if authsize > int(^uint16(0)) {
		panic(fmt.Errorf("BUG: auth size %d overflows uint16", authsize))
	}
	return Header{
		StaticHeader: StaticHeader{
			ProtocolID: c.protocolID,
			Version:    version,
			Flag:       flag,
			AuthSize:   uint16(authsize),
		},
	}
}

// encodeRandom encodes a packet with random content.
func (c *Codec) encodeRandom(toID enode.ID) (Header, []byte, error) {
	head := c.makeHeader(toID, flagMessage, 0)

	// Encode auth data.
	auth := messageAuthData{SrcID: c.localnode.ID()}
	if _, err := crand.Read(head.Nonce[:]); err != nil {
		return head, nil, fmt.Errorf("can't get random data: %v", err)
	}
	c.headbuf.Reset()
	binary.Write(&c.headbuf, binary.BigEndian, auth)
	head.AuthData = c.headbuf.Bytes()

	// Fill message ciphertext buffer with random bytes.
	c.msgctbuf = append(c.msgctbuf[:0], make([]byte, randomPacketMsgSize)...)
	crand.Read(c.msgctbuf)
	return head, c.msgctbuf, nil
}

// encodeWhoareyou encodes a WHOAREYOU packet.
func (c *Codec) encodeWhoareyou(toID enode.ID, packet *Whoareyou) (Header, error) {
	// Sanity check node field to catch misbehaving callers.
	if packet.RecordSeq > 0 && packet.Node == nil {
		panic("BUG: missing node in whoareyou with non-zero seq")
	}

	// Create header.
	head := c.makeHeader(toID, flagWhoareyou, 0)
	head.AuthData = bytesCopy(&c.buf)
	head.Nonce = packet.Nonce

	// Encode auth data.
	auth := &whoareyouAuthData{
		IDNonce:   packet.IDNonce,
		RecordSeq: packet.RecordSeq,
	}
	c.headbuf.Reset()
	binary.Write(&c.headbuf, binary.BigEndian, auth)
	head.AuthData = c.headbuf.Bytes()
	return head, nil
}

// encodeHandshakeHeader encodes the handshake message packet header.
func (c *Codec) encodeHandshakeHeader(toID enode.ID, addr string, challenge *Whoareyou) (Header, *session, error) {
	// Ensure calling code sets challenge.node.
	if challenge.Node == nil {
		panic("BUG: missing challenge.Node in encode")
	}

	// Generate new secrets.
	auth, session, err := c.makeHandshakeAuth(toID, addr, challenge)
	if err != nil {
		return Header{}, nil, err
	}

	// Generate nonce for message.
	nonce, err := c.sc.nextNonce(session)
	if err != nil {
		return Header{}, nil, fmt.Errorf("can't generate nonce: %v", err)
	}

	// TODO: this should happen when the first authenticated message is received
	c.sc.storeNewSession(toID, addr, session)

	// Encode the auth header.
	var (
		authsizeExtra = len(auth.pubkey) + len(auth.signature) + len(auth.record)
		head          = c.makeHeader(toID, flagHandshake, authsizeExtra)
	)
	c.headbuf.Reset()
	binary.Write(&c.headbuf, binary.BigEndian, &auth.h)
	c.headbuf.Write(auth.signature)
	c.headbuf.Write(auth.pubkey)
	c.headbuf.Write(auth.record)
	head.AuthData = c.headbuf.Bytes()
	head.Nonce = nonce
	return head, session, err
}

// makeHandshakeAuth creates the auth header on a request packet following WHOAREYOU.
func (c *Codec) makeHandshakeAuth(toID enode.ID, addr string, challenge *Whoareyou) (*handshakeAuthData, *session, error) {
	auth := new(handshakeAuthData)
	auth.h.SrcID = c.localnode.ID()

	// Create the ephemeral key. This needs to be first because the
	// key is part of the ID nonce signature.
	var remotePubkey = new(ecdsa.PublicKey)
	if err := challenge.Node.Load((*enode.Secp256k1)(remotePubkey)); err != nil {
		return nil, nil, fmt.Errorf("can't find secp256k1 key for recipient")
	}
	ephkey, err := c.sc.ephemeralKeyGen()
	if err != nil {
		return nil, nil, fmt.Errorf("can't generate ephemeral key")
	}
	ephpubkey := EncodePubkey(&ephkey.PublicKey)
	auth.pubkey = ephpubkey[:]
	auth.h.PubkeySize = byte(len(auth.pubkey))

	// Add ID nonce signature to response.
	cdata := challenge.ChallengeData
	idsig, err := makeIDSignature(c.sha256, c.privkey, cdata, ephpubkey[:], toID)
	if err != nil {
		return nil, nil, fmt.Errorf("can't sign: %v", err)
	}
	auth.signature = idsig
	auth.h.SigSize = byte(len(auth.signature))

	// Add our record to response if it's newer than what remote side has.
	ln := c.localnode.Node()
	if challenge.RecordSeq < ln.Seq() {
		auth.record, _ = rlp.EncodeToBytes(ln.Record())
	}

	// Create session keys.
	sec := deriveKeys(sha256.New, ephkey, remotePubkey, c.localnode.ID(), challenge.Node.ID(), cdata)
	if sec == nil {
		return nil, nil, fmt.Errorf("key derivation failed")
	}
	return auth, sec, err
}

// encodeMessageHeader encodes an encrypted message packet.
func (c *Codec) encodeMessageHeader(toID enode.ID, s *session) (Header, error) {
	head := c.makeHeader(toID, flagMessage, 0)

	// Create the header.
	nonce, err := c.sc.nextNonce(s)
	if err != nil {
		return Header{}, fmt.Errorf("can't generate nonce: %v", err)
	}
	auth := messageAuthData{SrcID: c.localnode.ID()}
	c.buf.Reset()
	binary.Write(&c.buf, binary.BigEndian, &auth)
	head.AuthData = bytesCopy(&c.buf)
	head.Nonce = nonce
	return head, err
}

func (c *Codec) encryptMessage(s *session, p Packet, head *Header, headerData []byte) ([]byte, error) {
	// Encode message plaintext.
	c.msgbuf.Reset()
	c.msgbuf.WriteByte(p.Kind())
	if err := rlp.Encode(&c.msgbuf, p); err != nil {
		return nil, err
	}
	messagePT := c.msgbuf.Bytes()

	// Encrypt into message ciphertext buffer.
	messageCT, err := encryptGCM(c.msgctbuf[:0], s.writeKey, head.Nonce[:], messagePT, headerData)
	if err == nil {
		c.msgctbuf = messageCT
	}
	return messageCT, err
}

// Decode decodes a discovery packet.
func (c *Codec) Decode(inputData []byte, addr string) (src enode.ID, n *enode.Node, p Packet, err error) {
	if len(inputData) < minPacketSize {
		return enode.ID{}, nil, nil, errTooShort
	}
	// Copy the packet to a tmp buffer to avoid modifying it.
	c.decbuf = append(c.decbuf[:0], inputData...)
	input := c.decbuf
	// Unmask the static header.
	var head Header
	copy(head.IV[:], input[:sizeofMaskingIV])
	mask := head.mask(c.localnode.ID())
	staticHeader := input[sizeofMaskingIV:sizeofStaticPacketData]
	mask.XORKeyStream(staticHeader, staticHeader)

	// Decode and verify the static header.
	c.reader.Reset(staticHeader)
	binary.Read(&c.reader, binary.BigEndian, &head.StaticHeader)
	remainingInput := len(input) - sizeofStaticPacketData
	if err := head.checkValid(remainingInput, c.protocolID); err != nil {
		return enode.ID{}, nil, nil, err
	}

	// Unmask auth data.
	authDataEnd := sizeofStaticPacketData + int(head.AuthSize)
	authData := input[sizeofStaticPacketData:authDataEnd]
	mask.XORKeyStream(authData, authData)
	head.AuthData = authData

	// Delete timed-out handshakes. This must happen before decoding to avoid
	// processing the same handshake twice.
	c.sc.handshakeGC()

	// Decode auth part and message.
	headerData := input[:authDataEnd]
	msgData := input[authDataEnd:]
	switch head.Flag {
	case flagWhoareyou:
		p, err = c.decodeWhoareyou(&head, headerData)
	case flagHandshake:
		n, p, err = c.decodeHandshakeMessage(addr, &head, headerData, msgData)
	case flagMessage:
		p, err = c.decodeMessage(addr, &head, headerData, msgData)
	default:
		err = errInvalidFlag
	}
	return head.src, n, p, err
}

// decodeWhoareyou reads packet data after the header as a WHOAREYOU packet.
func (c *Codec) decodeWhoareyou(head *Header, headerData []byte) (Packet, error) {
	if len(head.AuthData) != sizeofWhoareyouAuthData {
		return nil, fmt.Errorf("invalid auth size %d for WHOAREYOU", len(head.AuthData))
	}
	var auth whoareyouAuthData
	c.reader.Reset(head.AuthData)
	binary.Read(&c.reader, binary.BigEndian, &auth)
	p := &Whoareyou{
		Nonce:         head.Nonce,
		IDNonce:       auth.IDNonce,
		RecordSeq:     auth.RecordSeq,
		ChallengeData: make([]byte, len(headerData)),
	}
	copy(p.ChallengeData, headerData)
	return p, nil
}

func (c *Codec) decodeHandshakeMessage(fromAddr string, head *Header, headerData, msgData []byte) (n *enode.Node, p Packet, err error) {
	node, auth, session, err := c.decodeHandshake(fromAddr, head)
	if err != nil {
		c.sc.deleteHandshake(auth.h.SrcID, fromAddr)
		return nil, nil, err
	}

	// Decrypt the message using the new session keys.
	msg, err := c.decryptMessage(msgData, head.Nonce[:], headerData, session.readKey)
	if err != nil {
		c.sc.deleteHandshake(auth.h.SrcID, fromAddr)
		return node, msg, err
	}

	// Handshake OK, drop the challenge and store the new session keys.
	c.sc.storeNewSession(auth.h.SrcID, fromAddr, session)
	c.sc.deleteHandshake(auth.h.SrcID, fromAddr)
	return node, msg, nil
}

func (c *Codec) decodeHandshake(fromAddr string, head *Header) (n *enode.Node, auth handshakeAuthData, s *session, err error) {
	if auth, err = c.decodeHandshakeAuthData(head); err != nil {
		return nil, auth, nil, err
	}

	// Verify against our last WHOAREYOU.
	challenge := c.sc.getHandshake(auth.h.SrcID, fromAddr)
	if challenge == nil {
		return nil, auth, nil, errUnexpectedHandshake
	}
	// Get node record.
	n, err = c.decodeHandshakeRecord(challenge.Node, auth.h.SrcID, auth.record)
	if err != nil {
		return nil, auth, nil, err
	}
	// Verify ID nonce signature.
	sig := auth.signature
	cdata := challenge.ChallengeData
	err = verifyIDSignature(c.sha256, sig, n, cdata, auth.pubkey, c.localnode.ID())
	if err != nil {
		return nil, auth, nil, err
	}
	// Verify ephemeral key is on curve.
	ephkey, err := DecodePubkey(c.privkey.Curve, auth.pubkey)
	if err != nil {
		return nil, auth, nil, errInvalidAuthKey
	}
	// Derive session keys.
	session := deriveKeys(sha256.New, c.privkey, ephkey, auth.h.SrcID, c.localnode.ID(), cdata)
	session = session.keysFlipped()
	return n, auth, session, nil
}

// decodeHandshakeAuthData reads the authdata section of a handshake packet.
func (c *Codec) decodeHandshakeAuthData(head *Header) (auth handshakeAuthData, err error) {
	// Decode fixed size part.
	if len(head.AuthData) < sizeofHandshakeAuthData {
		return auth, fmt.Errorf("header authsize %d too low for handshake", head.AuthSize)
	}
	c.reader.Reset(head.AuthData)
	binary.Read(&c.reader, binary.BigEndian, &auth.h)
	head.src = auth.h.SrcID

	// Decode variable-size part.
	var (
		vardata       = head.AuthData[sizeofHandshakeAuthData:]
		sigAndKeySize = int(auth.h.SigSize) + int(auth.h.PubkeySize)
		keyOffset     = int(auth.h.SigSize)
		recOffset     = keyOffset + int(auth.h.PubkeySize)
	)
	if len(vardata) < sigAndKeySize {
		return auth, errTooShort
	}
	auth.signature = vardata[:keyOffset]
	auth.pubkey = vardata[keyOffset:recOffset]
	auth.record = vardata[recOffset:]
	return auth, nil
}

// decodeHandshakeRecord verifies the node record contained in a handshake packet. The
// remote node should include the record if we don't have one or if ours is older than the
// latest sequence number.
func (c *Codec) decodeHandshakeRecord(local *enode.Node, wantID enode.ID, remote []byte) (*enode.Node, error) {
	node := local
	if len(remote) > 0 {
		var record enr.Record
		if err := rlp.DecodeBytes(remote, &record); err != nil {
			return nil, err
		}
		if local == nil || local.Seq() < record.Seq() {
			n, err := enode.New(enode.ValidSchemes, &record)
			if err != nil {
				return nil, fmt.Errorf("invalid node record: %v", err)
			}
			if n.ID() != wantID {
				return nil, fmt.Errorf("record in handshake has wrong ID: %v", n.ID())
			}
			node = n
		}
	}
	if node == nil {
		return nil, errNoRecord
	}
	return node, nil
}

// decodeMessage reads packet data following the header as an ordinary message packet.
func (c *Codec) decodeMessage(fromAddr string, head *Header, headerData, msgData []byte) (Packet, error) {
	if len(head.AuthData) != sizeofMessageAuthData {
		return nil, fmt.Errorf("invalid auth size %d for message packet", len(head.AuthData))
	}
	var auth messageAuthData
	c.reader.Reset(head.AuthData)
	binary.Read(&c.reader, binary.BigEndian, &auth)
	head.src = auth.SrcID

	// Try decrypting the message.
	key := c.sc.readKey(auth.SrcID, fromAddr)
	msg, err := c.decryptMessage(msgData, head.Nonce[:], headerData, key)
	if errors.Is(err, errMessageDecrypt) {
		// It didn't work. Start the handshake since this is an ordinary message packet.
		return &Unknown{Nonce: head.Nonce}, nil
	}
	return msg, err
}

func (c *Codec) decryptMessage(input, nonce, headerData, readKey []byte) (Packet, error) {
	msgdata, err := decryptGCM(readKey, nonce, input, headerData)
	if err != nil {
		return nil, errMessageDecrypt
	}
	if len(msgdata) == 0 {
		return nil, errMessageTooShort
	}
	return DecodeMessage(msgdata[0], msgdata[1:])
}

// checkValid performs some basic validity checks on the header.
// The packetLen here is the length remaining after the static header.
func (h *StaticHeader) checkValid(packetLen int, protocolID [6]byte) error {
	if h.ProtocolID != protocolID {
		return errInvalidHeader
	}
	if h.Version < minVersion {
		return errMinVersion
	}
	if h.Flag != flagWhoareyou && packetLen < minMessageSize {
		return errMsgTooShort
	}
	if int(h.AuthSize) > packetLen {
		return errAuthSize
	}
	return nil
}

// mask returns a cipher for 'masking' / 'unmasking' packet headers.
func (h *Header) mask(destID enode.ID) cipher.Stream {
	block, err := aes.NewCipher(destID[:16])
	if err != nil {
		panic("can't create cipher")
	}
	return cipher.NewCTR(block, h.IV[:])
}

func bytesCopy(r *bytes.Buffer) []byte {
	b := make([]byte, r.Len())
	copy(b, r.Bytes())
	return b
}
