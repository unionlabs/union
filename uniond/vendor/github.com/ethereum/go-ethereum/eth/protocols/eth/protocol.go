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

package eth

import (
	"errors"
	"fmt"
	"io"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/forkid"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/rlp"
)

// Constants to match up protocol versions and messages
const (
	ETH67 = 67
	ETH68 = 68
)

// ProtocolName is the official short name of the `eth` protocol used during
// devp2p capability negotiation.
const ProtocolName = "eth"

// ProtocolVersions are the supported versions of the `eth` protocol (first
// is primary).
var ProtocolVersions = []uint{ETH68, ETH67}

// protocolLengths are the number of implemented message corresponding to
// different protocol versions.
var protocolLengths = map[uint]uint64{ETH68: 17, ETH67: 17}

// maxMessageSize is the maximum cap on the size of a protocol message.
const maxMessageSize = 10 * 1024 * 1024

const (
	StatusMsg                     = 0x00
	NewBlockHashesMsg             = 0x01
	TransactionsMsg               = 0x02
	GetBlockHeadersMsg            = 0x03
	BlockHeadersMsg               = 0x04
	GetBlockBodiesMsg             = 0x05
	BlockBodiesMsg                = 0x06
	NewBlockMsg                   = 0x07
	NewPooledTransactionHashesMsg = 0x08
	GetPooledTransactionsMsg      = 0x09
	PooledTransactionsMsg         = 0x0a
	GetReceiptsMsg                = 0x0f
	ReceiptsMsg                   = 0x10
)

var (
	errNoStatusMsg             = errors.New("no status message")
	errMsgTooLarge             = errors.New("message too long")
	errDecode                  = errors.New("invalid message")
	errInvalidMsgCode          = errors.New("invalid message code")
	errProtocolVersionMismatch = errors.New("protocol version mismatch")
	errNetworkIDMismatch       = errors.New("network ID mismatch")
	errGenesisMismatch         = errors.New("genesis mismatch")
	errForkIDRejected          = errors.New("fork ID rejected")
)

// Packet represents a p2p message in the `eth` protocol.
type Packet interface {
	Name() string // Name returns a string corresponding to the message type.
	Kind() byte   // Kind returns the message type.
}

// StatusPacket is the network packet for the status message.
type StatusPacket struct {
	ProtocolVersion uint32
	NetworkID       uint64
	TD              *big.Int
	Head            common.Hash
	Genesis         common.Hash
	ForkID          forkid.ID
}

// NewBlockHashesPacket is the network packet for the block announcements.
type NewBlockHashesPacket []struct {
	Hash   common.Hash // Hash of one particular block being announced
	Number uint64      // Number of one particular block being announced
}

// Unpack retrieves the block hashes and numbers from the announcement packet
// and returns them in a split flat format that's more consistent with the
// internal data structures.
func (p *NewBlockHashesPacket) Unpack() ([]common.Hash, []uint64) {
	var (
		hashes  = make([]common.Hash, len(*p))
		numbers = make([]uint64, len(*p))
	)
	for i, body := range *p {
		hashes[i], numbers[i] = body.Hash, body.Number
	}
	return hashes, numbers
}

// TransactionsPacket is the network packet for broadcasting new transactions.
type TransactionsPacket []*types.Transaction

// GetBlockHeadersRequest represents a block header query.
type GetBlockHeadersRequest struct {
	Origin  HashOrNumber // Block from which to retrieve headers
	Amount  uint64       // Maximum number of headers to retrieve
	Skip    uint64       // Blocks to skip between consecutive headers
	Reverse bool         // Query direction (false = rising towards latest, true = falling towards genesis)
}

// GetBlockHeadersPacket represents a block header query with request ID wrapping.
type GetBlockHeadersPacket struct {
	RequestId uint64
	*GetBlockHeadersRequest
}

// HashOrNumber is a combined field for specifying an origin block.
type HashOrNumber struct {
	Hash   common.Hash // Block hash from which to retrieve headers (excludes Number)
	Number uint64      // Block hash from which to retrieve headers (excludes Hash)
}

// EncodeRLP is a specialized encoder for HashOrNumber to encode only one of the
// two contained union fields.
func (hn *HashOrNumber) EncodeRLP(w io.Writer) error {
	if hn.Hash == (common.Hash{}) {
		return rlp.Encode(w, hn.Number)
	}
	if hn.Number != 0 {
		return fmt.Errorf("both origin hash (%x) and number (%d) provided", hn.Hash, hn.Number)
	}
	return rlp.Encode(w, hn.Hash)
}

// DecodeRLP is a specialized decoder for HashOrNumber to decode the contents
// into either a block hash or a block number.
func (hn *HashOrNumber) DecodeRLP(s *rlp.Stream) error {
	_, size, err := s.Kind()
	switch {
	case err != nil:
		return err
	case size == 32:
		hn.Number = 0
		return s.Decode(&hn.Hash)
	case size <= 8:
		hn.Hash = common.Hash{}
		return s.Decode(&hn.Number)
	default:
		return fmt.Errorf("invalid input size %d for origin", size)
	}
}

// BlockHeadersRequest represents a block header response.
type BlockHeadersRequest []*types.Header

// BlockHeadersPacket represents a block header response over with request ID wrapping.
type BlockHeadersPacket struct {
	RequestId uint64
	BlockHeadersRequest
}

// BlockHeadersRLPResponse represents a block header response, to use when we already
// have the headers rlp encoded.
type BlockHeadersRLPResponse []rlp.RawValue

// BlockHeadersRLPPacket represents a block header response with request ID wrapping.
type BlockHeadersRLPPacket struct {
	RequestId uint64
	BlockHeadersRLPResponse
}

// NewBlockPacket is the network packet for the block propagation message.
type NewBlockPacket struct {
	Block *types.Block
	TD    *big.Int
}

// sanityCheck verifies that the values are reasonable, as a DoS protection
func (request *NewBlockPacket) sanityCheck() error {
	if err := request.Block.SanityCheck(); err != nil {
		return err
	}
	//TD at mainnet block #7753254 is 76 bits. If it becomes 100 million times
	// larger, it will still fit within 100 bits
	if tdlen := request.TD.BitLen(); tdlen > 100 {
		return fmt.Errorf("too large block TD: bitlen %d", tdlen)
	}
	return nil
}

// GetBlockBodiesRequest represents a block body query.
type GetBlockBodiesRequest []common.Hash

// GetBlockBodiesPacket represents a block body query with request ID wrapping.
type GetBlockBodiesPacket struct {
	RequestId uint64
	GetBlockBodiesRequest
}

// BlockBodiesResponse is the network packet for block content distribution.
type BlockBodiesResponse []*BlockBody

// BlockBodiesPacket is the network packet for block content distribution with
// request ID wrapping.
type BlockBodiesPacket struct {
	RequestId uint64
	BlockBodiesResponse
}

// BlockBodiesRLPResponse is used for replying to block body requests, in cases
// where we already have them RLP-encoded, and thus can avoid the decode-encode
// roundtrip.
type BlockBodiesRLPResponse []rlp.RawValue

// BlockBodiesRLPPacket is the BlockBodiesRLPResponse with request ID wrapping.
type BlockBodiesRLPPacket struct {
	RequestId uint64
	BlockBodiesRLPResponse
}

// BlockBody represents the data content of a single block.
type BlockBody struct {
	Transactions []*types.Transaction // Transactions contained within a block
	Uncles       []*types.Header      // Uncles contained within a block
	Withdrawals  []*types.Withdrawal  `rlp:"optional"` // Withdrawals contained within a block
}

// Unpack retrieves the transactions and uncles from the range packet and returns
// them in a split flat format that's more consistent with the internal data structures.
func (p *BlockBodiesResponse) Unpack() ([][]*types.Transaction, [][]*types.Header, [][]*types.Withdrawal) {
	// TODO(matt): add support for withdrawals to fetchers
	var (
		txset         = make([][]*types.Transaction, len(*p))
		uncleset      = make([][]*types.Header, len(*p))
		withdrawalset = make([][]*types.Withdrawal, len(*p))
	)
	for i, body := range *p {
		txset[i], uncleset[i], withdrawalset[i] = body.Transactions, body.Uncles, body.Withdrawals
	}
	return txset, uncleset, withdrawalset
}

// GetReceiptsRequest represents a block receipts query.
type GetReceiptsRequest []common.Hash

// GetReceiptsPacket represents a block receipts query with request ID wrapping.
type GetReceiptsPacket struct {
	RequestId uint64
	GetReceiptsRequest
}

// ReceiptsResponse is the network packet for block receipts distribution.
type ReceiptsResponse [][]*types.Receipt

// ReceiptsPacket is the network packet for block receipts distribution with
// request ID wrapping.
type ReceiptsPacket struct {
	RequestId uint64
	ReceiptsResponse
}

// ReceiptsRLPResponse is used for receipts, when we already have it encoded
type ReceiptsRLPResponse []rlp.RawValue

// ReceiptsRLPPacket is ReceiptsRLPResponse with request ID wrapping.
type ReceiptsRLPPacket struct {
	RequestId uint64
	ReceiptsRLPResponse
}

// NewPooledTransactionHashesPacket67 represents a transaction announcement packet on eth/67.
type NewPooledTransactionHashesPacket67 []common.Hash

// NewPooledTransactionHashesPacket68 represents a transaction announcement packet on eth/68 and newer.
type NewPooledTransactionHashesPacket68 struct {
	Types  []byte
	Sizes  []uint32
	Hashes []common.Hash
}

// GetPooledTransactionsRequest represents a transaction query.
type GetPooledTransactionsRequest []common.Hash

// GetPooledTransactionsPacket represents a transaction query with request ID wrapping.
type GetPooledTransactionsPacket struct {
	RequestId uint64
	GetPooledTransactionsRequest
}

// PooledTransactionsResponse is the network packet for transaction distribution.
type PooledTransactionsResponse []*types.Transaction

// PooledTransactionsPacket is the network packet for transaction distribution
// with request ID wrapping.
type PooledTransactionsPacket struct {
	RequestId uint64
	PooledTransactionsResponse
}

// PooledTransactionsRLPResponse is the network packet for transaction distribution, used
// in the cases we already have them in rlp-encoded form
type PooledTransactionsRLPResponse []rlp.RawValue

// PooledTransactionsRLPPacket is PooledTransactionsRLPResponse with request ID wrapping.
type PooledTransactionsRLPPacket struct {
	RequestId uint64
	PooledTransactionsRLPResponse
}

func (*StatusPacket) Name() string { return "Status" }
func (*StatusPacket) Kind() byte   { return StatusMsg }

func (*NewBlockHashesPacket) Name() string { return "NewBlockHashes" }
func (*NewBlockHashesPacket) Kind() byte   { return NewBlockHashesMsg }

func (*TransactionsPacket) Name() string { return "Transactions" }
func (*TransactionsPacket) Kind() byte   { return TransactionsMsg }

func (*GetBlockHeadersRequest) Name() string { return "GetBlockHeaders" }
func (*GetBlockHeadersRequest) Kind() byte   { return GetBlockHeadersMsg }

func (*BlockHeadersRequest) Name() string { return "BlockHeaders" }
func (*BlockHeadersRequest) Kind() byte   { return BlockHeadersMsg }

func (*GetBlockBodiesRequest) Name() string { return "GetBlockBodies" }
func (*GetBlockBodiesRequest) Kind() byte   { return GetBlockBodiesMsg }

func (*BlockBodiesResponse) Name() string { return "BlockBodies" }
func (*BlockBodiesResponse) Kind() byte   { return BlockBodiesMsg }

func (*NewBlockPacket) Name() string { return "NewBlock" }
func (*NewBlockPacket) Kind() byte   { return NewBlockMsg }

func (*NewPooledTransactionHashesPacket67) Name() string { return "NewPooledTransactionHashes" }
func (*NewPooledTransactionHashesPacket67) Kind() byte   { return NewPooledTransactionHashesMsg }
func (*NewPooledTransactionHashesPacket68) Name() string { return "NewPooledTransactionHashes" }
func (*NewPooledTransactionHashesPacket68) Kind() byte   { return NewPooledTransactionHashesMsg }

func (*GetPooledTransactionsRequest) Name() string { return "GetPooledTransactions" }
func (*GetPooledTransactionsRequest) Kind() byte   { return GetPooledTransactionsMsg }

func (*PooledTransactionsResponse) Name() string { return "PooledTransactions" }
func (*PooledTransactionsResponse) Kind() byte   { return PooledTransactionsMsg }

func (*GetReceiptsRequest) Name() string { return "GetReceipts" }
func (*GetReceiptsRequest) Kind() byte   { return GetReceiptsMsg }

func (*ReceiptsResponse) Name() string { return "Receipts" }
func (*ReceiptsResponse) Kind() byte   { return ReceiptsMsg }
