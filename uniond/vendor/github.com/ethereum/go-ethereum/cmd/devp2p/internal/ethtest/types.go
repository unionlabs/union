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

package ethtest

import (
	"crypto/ecdsa"
	"errors"
	"fmt"
	"time"

	"github.com/ethereum/go-ethereum/eth/protocols/eth"
	"github.com/ethereum/go-ethereum/p2p"
	"github.com/ethereum/go-ethereum/p2p/rlpx"
	"github.com/ethereum/go-ethereum/rlp"
)

type Message interface {
	Code() int
	ReqID() uint64
}

type Error struct {
	err error
}

func (e *Error) Unwrap() error  { return e.err }
func (e *Error) Error() string  { return e.err.Error() }
func (e *Error) String() string { return e.Error() }

func (e *Error) Code() int     { return -1 }
func (e *Error) ReqID() uint64 { return 0 }

func errorf(format string, args ...interface{}) *Error {
	return &Error{fmt.Errorf(format, args...)}
}

// Hello is the RLP structure of the protocol handshake.
type Hello struct {
	Version    uint64
	Name       string
	Caps       []p2p.Cap
	ListenPort uint64
	ID         []byte // secp256k1 public key

	// Ignore additional fields (for forward compatibility).
	Rest []rlp.RawValue `rlp:"tail"`
}

func (msg Hello) Code() int     { return 0x00 }
func (msg Hello) ReqID() uint64 { return 0 }

// Disconnect is the RLP structure for a disconnect message.
type Disconnect struct {
	Reason p2p.DiscReason
}

func (msg Disconnect) Code() int     { return 0x01 }
func (msg Disconnect) ReqID() uint64 { return 0 }

type Ping struct{}

func (msg Ping) Code() int     { return 0x02 }
func (msg Ping) ReqID() uint64 { return 0 }

type Pong struct{}

func (msg Pong) Code() int     { return 0x03 }
func (msg Pong) ReqID() uint64 { return 0 }

// Status is the network packet for the status message for eth/64 and later.
type Status eth.StatusPacket

func (msg Status) Code() int     { return 16 }
func (msg Status) ReqID() uint64 { return 0 }

// NewBlockHashes is the network packet for the block announcements.
type NewBlockHashes eth.NewBlockHashesPacket

func (msg NewBlockHashes) Code() int     { return 17 }
func (msg NewBlockHashes) ReqID() uint64 { return 0 }

type Transactions eth.TransactionsPacket

func (msg Transactions) Code() int     { return 18 }
func (msg Transactions) ReqID() uint64 { return 18 }

// GetBlockHeaders represents a block header query.
type GetBlockHeaders eth.GetBlockHeadersPacket

func (msg GetBlockHeaders) Code() int     { return 19 }
func (msg GetBlockHeaders) ReqID() uint64 { return msg.RequestId }

type BlockHeaders eth.BlockHeadersPacket

func (msg BlockHeaders) Code() int     { return 20 }
func (msg BlockHeaders) ReqID() uint64 { return msg.RequestId }

// GetBlockBodies represents a GetBlockBodies request
type GetBlockBodies eth.GetBlockBodiesPacket

func (msg GetBlockBodies) Code() int     { return 21 }
func (msg GetBlockBodies) ReqID() uint64 { return msg.RequestId }

// BlockBodies is the network packet for block content distribution.
type BlockBodies eth.BlockBodiesPacket

func (msg BlockBodies) Code() int     { return 22 }
func (msg BlockBodies) ReqID() uint64 { return msg.RequestId }

// NewBlock is the network packet for the block propagation message.
type NewBlock eth.NewBlockPacket

func (msg NewBlock) Code() int     { return 23 }
func (msg NewBlock) ReqID() uint64 { return 0 }

// NewPooledTransactionHashes66 is the network packet for the tx hash propagation message.
type NewPooledTransactionHashes66 eth.NewPooledTransactionHashesPacket67

func (msg NewPooledTransactionHashes66) Code() int     { return 24 }
func (msg NewPooledTransactionHashes66) ReqID() uint64 { return 0 }

// NewPooledTransactionHashes is the network packet for the tx hash propagation message.
type NewPooledTransactionHashes eth.NewPooledTransactionHashesPacket68

func (msg NewPooledTransactionHashes) Code() int     { return 24 }
func (msg NewPooledTransactionHashes) ReqID() uint64 { return 0 }

type GetPooledTransactions eth.GetPooledTransactionsPacket

func (msg GetPooledTransactions) Code() int     { return 25 }
func (msg GetPooledTransactions) ReqID() uint64 { return msg.RequestId }

type PooledTransactions eth.PooledTransactionsPacket

func (msg PooledTransactions) Code() int     { return 26 }
func (msg PooledTransactions) ReqID() uint64 { return msg.RequestId }

// Conn represents an individual connection with a peer
type Conn struct {
	*rlpx.Conn
	ourKey                     *ecdsa.PrivateKey
	negotiatedProtoVersion     uint
	negotiatedSnapProtoVersion uint
	ourHighestProtoVersion     uint
	ourHighestSnapProtoVersion uint
	caps                       []p2p.Cap
}

// Read reads an eth66 packet from the connection.
func (c *Conn) Read() Message {
	code, rawData, _, err := c.Conn.Read()
	if err != nil {
		return errorf("could not read from connection: %v", err)
	}

	var msg Message
	switch int(code) {
	case (Hello{}).Code():
		msg = new(Hello)
	case (Ping{}).Code():
		msg = new(Ping)
	case (Pong{}).Code():
		msg = new(Pong)
	case (Disconnect{}).Code():
		msg = new(Disconnect)
	case (Status{}).Code():
		msg = new(Status)
	case (GetBlockHeaders{}).Code():
		ethMsg := new(eth.GetBlockHeadersPacket)
		if err := rlp.DecodeBytes(rawData, ethMsg); err != nil {
			return errorf("could not rlp decode message: %v", err)
		}
		return (*GetBlockHeaders)(ethMsg)
	case (BlockHeaders{}).Code():
		ethMsg := new(eth.BlockHeadersPacket)
		if err := rlp.DecodeBytes(rawData, ethMsg); err != nil {
			return errorf("could not rlp decode message: %v", err)
		}
		return (*BlockHeaders)(ethMsg)
	case (GetBlockBodies{}).Code():
		ethMsg := new(eth.GetBlockBodiesPacket)
		if err := rlp.DecodeBytes(rawData, ethMsg); err != nil {
			return errorf("could not rlp decode message: %v", err)
		}
		return (*GetBlockBodies)(ethMsg)
	case (BlockBodies{}).Code():
		ethMsg := new(eth.BlockBodiesPacket)
		if err := rlp.DecodeBytes(rawData, ethMsg); err != nil {
			return errorf("could not rlp decode message: %v", err)
		}
		return (*BlockBodies)(ethMsg)
	case (NewBlock{}).Code():
		msg = new(NewBlock)
	case (NewBlockHashes{}).Code():
		msg = new(NewBlockHashes)
	case (Transactions{}).Code():
		msg = new(Transactions)
	case (NewPooledTransactionHashes66{}).Code():
		// Try decoding to eth68
		ethMsg := new(NewPooledTransactionHashes)
		if err := rlp.DecodeBytes(rawData, ethMsg); err == nil {
			return ethMsg
		}
		msg = new(NewPooledTransactionHashes66)
	case (GetPooledTransactions{}.Code()):
		ethMsg := new(eth.GetPooledTransactionsPacket)
		if err := rlp.DecodeBytes(rawData, ethMsg); err != nil {
			return errorf("could not rlp decode message: %v", err)
		}
		return (*GetPooledTransactions)(ethMsg)
	case (PooledTransactions{}.Code()):
		ethMsg := new(eth.PooledTransactionsPacket)
		if err := rlp.DecodeBytes(rawData, ethMsg); err != nil {
			return errorf("could not rlp decode message: %v", err)
		}
		return (*PooledTransactions)(ethMsg)
	default:
		msg = errorf("invalid message code: %d", code)
	}

	if msg != nil {
		if err := rlp.DecodeBytes(rawData, msg); err != nil {
			return errorf("could not rlp decode message: %v", err)
		}
		return msg
	}
	return errorf("invalid message: %s", string(rawData))
}

// Write writes a eth packet to the connection.
func (c *Conn) Write(msg Message) error {
	payload, err := rlp.EncodeToBytes(msg)
	if err != nil {
		return err
	}
	_, err = c.Conn.Write(uint64(msg.Code()), payload)
	return err
}

// ReadSnap reads a snap/1 response with the given id from the connection.
func (c *Conn) ReadSnap(id uint64) (Message, error) {
	respId := id + 1
	start := time.Now()
	for respId != id && time.Since(start) < timeout {
		code, rawData, _, err := c.Conn.Read()
		if err != nil {
			return nil, fmt.Errorf("could not read from connection: %v", err)
		}
		var snpMsg interface{}
		switch int(code) {
		case (GetAccountRange{}).Code():
			snpMsg = new(GetAccountRange)
		case (AccountRange{}).Code():
			snpMsg = new(AccountRange)
		case (GetStorageRanges{}).Code():
			snpMsg = new(GetStorageRanges)
		case (StorageRanges{}).Code():
			snpMsg = new(StorageRanges)
		case (GetByteCodes{}).Code():
			snpMsg = new(GetByteCodes)
		case (ByteCodes{}).Code():
			snpMsg = new(ByteCodes)
		case (GetTrieNodes{}).Code():
			snpMsg = new(GetTrieNodes)
		case (TrieNodes{}).Code():
			snpMsg = new(TrieNodes)
		default:
			//return nil, fmt.Errorf("invalid message code: %d", code)
			continue
		}
		if err := rlp.DecodeBytes(rawData, snpMsg); err != nil {
			return nil, fmt.Errorf("could not rlp decode message: %v", err)
		}
		return snpMsg.(Message), nil
	}
	return nil, errors.New("request timed out")
}
