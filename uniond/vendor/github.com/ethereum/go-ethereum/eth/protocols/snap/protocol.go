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

package snap

import (
	"errors"
	"fmt"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/rlp"
)

// Constants to match up protocol versions and messages
const (
	SNAP1 = 1
)

// ProtocolName is the official short name of the `snap` protocol used during
// devp2p capability negotiation.
const ProtocolName = "snap"

// ProtocolVersions are the supported versions of the `snap` protocol (first
// is primary).
var ProtocolVersions = []uint{SNAP1}

// protocolLengths are the number of implemented message corresponding to
// different protocol versions.
var protocolLengths = map[uint]uint64{SNAP1: 8}

// maxMessageSize is the maximum cap on the size of a protocol message.
const maxMessageSize = 10 * 1024 * 1024

const (
	GetAccountRangeMsg  = 0x00
	AccountRangeMsg     = 0x01
	GetStorageRangesMsg = 0x02
	StorageRangesMsg    = 0x03
	GetByteCodesMsg     = 0x04
	ByteCodesMsg        = 0x05
	GetTrieNodesMsg     = 0x06
	TrieNodesMsg        = 0x07
)

var (
	errMsgTooLarge    = errors.New("message too long")
	errDecode         = errors.New("invalid message")
	errInvalidMsgCode = errors.New("invalid message code")
	errBadRequest     = errors.New("bad request")
)

// Packet represents a p2p message in the `snap` protocol.
type Packet interface {
	Name() string // Name returns a string corresponding to the message type.
	Kind() byte   // Kind returns the message type.
}

// GetAccountRangePacket represents an account query.
type GetAccountRangePacket struct {
	ID     uint64      // Request ID to match up responses with
	Root   common.Hash // Root hash of the account trie to serve
	Origin common.Hash // Hash of the first account to retrieve
	Limit  common.Hash // Hash of the last account to retrieve
	Bytes  uint64      // Soft limit at which to stop returning data
}

// AccountRangePacket represents an account query response.
type AccountRangePacket struct {
	ID       uint64         // ID of the request this is a response for
	Accounts []*AccountData // List of consecutive accounts from the trie
	Proof    [][]byte       // List of trie nodes proving the account range
}

// AccountData represents a single account in a query response.
type AccountData struct {
	Hash common.Hash  // Hash of the account
	Body rlp.RawValue // Account body in slim format
}

// Unpack retrieves the accounts from the range packet and converts from slim
// wire representation to consensus format. The returned data is RLP encoded
// since it's expected to be serialized to disk without further interpretation.
//
// Note, this method does a round of RLP decoding and reencoding, so only use it
// once and cache the results if need be. Ideally discard the packet afterwards
// to not double the memory use.
func (p *AccountRangePacket) Unpack() ([]common.Hash, [][]byte, error) {
	var (
		hashes   = make([]common.Hash, len(p.Accounts))
		accounts = make([][]byte, len(p.Accounts))
	)
	for i, acc := range p.Accounts {
		val, err := types.FullAccountRLP(acc.Body)
		if err != nil {
			return nil, nil, fmt.Errorf("invalid account %x: %v", acc.Body, err)
		}
		hashes[i], accounts[i] = acc.Hash, val
	}
	return hashes, accounts, nil
}

// GetStorageRangesPacket represents an storage slot query.
type GetStorageRangesPacket struct {
	ID       uint64        // Request ID to match up responses with
	Root     common.Hash   // Root hash of the account trie to serve
	Accounts []common.Hash // Account hashes of the storage tries to serve
	Origin   []byte        // Hash of the first storage slot to retrieve (large contract mode)
	Limit    []byte        // Hash of the last storage slot to retrieve (large contract mode)
	Bytes    uint64        // Soft limit at which to stop returning data
}

// StorageRangesPacket represents a storage slot query response.
type StorageRangesPacket struct {
	ID    uint64           // ID of the request this is a response for
	Slots [][]*StorageData // Lists of consecutive storage slots for the requested accounts
	Proof [][]byte         // Merkle proofs for the *last* slot range, if it's incomplete
}

// StorageData represents a single storage slot in a query response.
type StorageData struct {
	Hash common.Hash // Hash of the storage slot
	Body []byte      // Data content of the slot
}

// Unpack retrieves the storage slots from the range packet and returns them in
// a split flat format that's more consistent with the internal data structures.
func (p *StorageRangesPacket) Unpack() ([][]common.Hash, [][][]byte) {
	var (
		hashset = make([][]common.Hash, len(p.Slots))
		slotset = make([][][]byte, len(p.Slots))
	)
	for i, slots := range p.Slots {
		hashset[i] = make([]common.Hash, len(slots))
		slotset[i] = make([][]byte, len(slots))
		for j, slot := range slots {
			hashset[i][j] = slot.Hash
			slotset[i][j] = slot.Body
		}
	}
	return hashset, slotset
}

// GetByteCodesPacket represents a contract bytecode query.
type GetByteCodesPacket struct {
	ID     uint64        // Request ID to match up responses with
	Hashes []common.Hash // Code hashes to retrieve the code for
	Bytes  uint64        // Soft limit at which to stop returning data
}

// ByteCodesPacket represents a contract bytecode query response.
type ByteCodesPacket struct {
	ID    uint64   // ID of the request this is a response for
	Codes [][]byte // Requested contract bytecodes
}

// GetTrieNodesPacket represents a state trie node query.
type GetTrieNodesPacket struct {
	ID    uint64            // Request ID to match up responses with
	Root  common.Hash       // Root hash of the account trie to serve
	Paths []TrieNodePathSet // Trie node hashes to retrieve the nodes for
	Bytes uint64            // Soft limit at which to stop returning data
}

// TrieNodePathSet is a list of trie node paths to retrieve. A naive way to
// represent trie nodes would be a simple list of `account || storage` path
// segments concatenated, but that would be very wasteful on the network.
//
// Instead, this array special cases the first element as the path in the
// account trie and the remaining elements as paths in the storage trie. To
// address an account node, the slice should have a length of 1 consisting
// of only the account path. There's no need to be able to address both an
// account node and a storage node in the same request as it cannot happen
// that a slot is accessed before the account path is fully expanded.
type TrieNodePathSet [][]byte

// TrieNodesPacket represents a state trie node query response.
type TrieNodesPacket struct {
	ID    uint64   // ID of the request this is a response for
	Nodes [][]byte // Requested state trie nodes
}

func (*GetAccountRangePacket) Name() string { return "GetAccountRange" }
func (*GetAccountRangePacket) Kind() byte   { return GetAccountRangeMsg }

func (*AccountRangePacket) Name() string { return "AccountRange" }
func (*AccountRangePacket) Kind() byte   { return AccountRangeMsg }

func (*GetStorageRangesPacket) Name() string { return "GetStorageRanges" }
func (*GetStorageRangesPacket) Kind() byte   { return GetStorageRangesMsg }

func (*StorageRangesPacket) Name() string { return "StorageRanges" }
func (*StorageRangesPacket) Kind() byte   { return StorageRangesMsg }

func (*GetByteCodesPacket) Name() string { return "GetByteCodes" }
func (*GetByteCodesPacket) Kind() byte   { return GetByteCodesMsg }

func (*ByteCodesPacket) Name() string { return "ByteCodes" }
func (*ByteCodesPacket) Kind() byte   { return ByteCodesMsg }

func (*GetTrieNodesPacket) Name() string { return "GetTrieNodes" }
func (*GetTrieNodesPacket) Kind() byte   { return GetTrieNodesMsg }

func (*TrieNodesPacket) Name() string { return "TrieNodes" }
func (*TrieNodesPacket) Kind() byte   { return TrieNodesMsg }
