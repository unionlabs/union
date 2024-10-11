// Copyright 2015 The go-ethereum Authors
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

package rpc

import (
	"context"
	"encoding/json"
	"fmt"
	"math"
	"strings"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
)

// API describes the set of methods offered over the RPC interface
type API struct {
	Namespace     string      // namespace under which the rpc methods of Service are exposed
	Version       string      // deprecated - this field is no longer used, but retained for compatibility
	Service       interface{} // receiver instance which holds the methods
	Public        bool        // deprecated - this field is no longer used, but retained for compatibility
	Authenticated bool        // whether the api should only be available behind authentication.
}

// ServerCodec implements reading, parsing and writing RPC messages for the server side of
// a RPC session. Implementations must be go-routine safe since the codec can be called in
// multiple go-routines concurrently.
type ServerCodec interface {
	peerInfo() PeerInfo
	readBatch() (msgs []*jsonrpcMessage, isBatch bool, err error)
	close()

	jsonWriter
}

// jsonWriter can write JSON messages to its underlying connection.
// Implementations must be safe for concurrent use.
type jsonWriter interface {
	// writeJSON writes a message to the connection.
	writeJSON(ctx context.Context, msg interface{}, isError bool) error

	// Closed returns a channel which is closed when the connection is closed.
	closed() <-chan interface{}
	// RemoteAddr returns the peer address of the connection.
	remoteAddr() string
}

type BlockNumber int64

const (
	SafeBlockNumber      = BlockNumber(-4)
	FinalizedBlockNumber = BlockNumber(-3)
	LatestBlockNumber    = BlockNumber(-2)
	PendingBlockNumber   = BlockNumber(-1)
	EarliestBlockNumber  = BlockNumber(0)
)

// UnmarshalJSON parses the given JSON fragment into a BlockNumber. It supports:
// - "safe", "finalized", "latest", "earliest" or "pending" as string arguments
// - the block number
// Returned errors:
// - an invalid block number error when the given argument isn't a known strings
// - an out of range error when the given block number is either too little or too large
func (bn *BlockNumber) UnmarshalJSON(data []byte) error {
	input := strings.TrimSpace(string(data))
	if len(input) >= 2 && input[0] == '"' && input[len(input)-1] == '"' {
		input = input[1 : len(input)-1]
	}

	switch input {
	case "earliest":
		*bn = EarliestBlockNumber
		return nil
	case "latest":
		*bn = LatestBlockNumber
		return nil
	case "pending":
		*bn = PendingBlockNumber
		return nil
	case "finalized":
		*bn = FinalizedBlockNumber
		return nil
	case "safe":
		*bn = SafeBlockNumber
		return nil
	}

	blckNum, err := hexutil.DecodeUint64(input)
	if err != nil {
		return err
	}
	if blckNum > math.MaxInt64 {
		return fmt.Errorf("block number larger than int64")
	}
	*bn = BlockNumber(blckNum)
	return nil
}

// Int64 returns the block number as int64.
func (bn BlockNumber) Int64() int64 {
	return (int64)(bn)
}

// MarshalText implements encoding.TextMarshaler. It marshals:
// - "safe", "finalized", "latest", "earliest" or "pending" as strings
// - other numbers as hex
func (bn BlockNumber) MarshalText() ([]byte, error) {
	return []byte(bn.String()), nil
}

func (bn BlockNumber) String() string {
	switch bn {
	case EarliestBlockNumber:
		return "earliest"
	case LatestBlockNumber:
		return "latest"
	case PendingBlockNumber:
		return "pending"
	case FinalizedBlockNumber:
		return "finalized"
	case SafeBlockNumber:
		return "safe"
	default:
		if bn < 0 {
			return fmt.Sprintf("<invalid %d>", bn)
		}
		return hexutil.Uint64(bn).String()
	}
}

type BlockNumberOrHash struct {
	BlockNumber      *BlockNumber `json:"blockNumber,omitempty"`
	BlockHash        *common.Hash `json:"blockHash,omitempty"`
	RequireCanonical bool         `json:"requireCanonical,omitempty"`
}

func (bnh *BlockNumberOrHash) UnmarshalJSON(data []byte) error {
	type erased BlockNumberOrHash
	e := erased{}
	err := json.Unmarshal(data, &e)
	if err == nil {
		if e.BlockNumber != nil && e.BlockHash != nil {
			return fmt.Errorf("cannot specify both BlockHash and BlockNumber, choose one or the other")
		}
		bnh.BlockNumber = e.BlockNumber
		bnh.BlockHash = e.BlockHash
		bnh.RequireCanonical = e.RequireCanonical
		return nil
	}
	var input string
	err = json.Unmarshal(data, &input)
	if err != nil {
		return err
	}
	switch input {
	case "earliest":
		bn := EarliestBlockNumber
		bnh.BlockNumber = &bn
		return nil
	case "latest":
		bn := LatestBlockNumber
		bnh.BlockNumber = &bn
		return nil
	case "pending":
		bn := PendingBlockNumber
		bnh.BlockNumber = &bn
		return nil
	case "finalized":
		bn := FinalizedBlockNumber
		bnh.BlockNumber = &bn
		return nil
	case "safe":
		bn := SafeBlockNumber
		bnh.BlockNumber = &bn
		return nil
	default:
		if len(input) == 66 {
			hash := common.Hash{}
			err := hash.UnmarshalText([]byte(input))
			if err != nil {
				return err
			}
			bnh.BlockHash = &hash
			return nil
		} else {
			blckNum, err := hexutil.DecodeUint64(input)
			if err != nil {
				return err
			}
			if blckNum > math.MaxInt64 {
				return fmt.Errorf("blocknumber too high")
			}
			bn := BlockNumber(blckNum)
			bnh.BlockNumber = &bn
			return nil
		}
	}
}

func (bnh *BlockNumberOrHash) Number() (BlockNumber, bool) {
	if bnh.BlockNumber != nil {
		return *bnh.BlockNumber, true
	}
	return BlockNumber(0), false
}

func (bnh *BlockNumberOrHash) String() string {
	if bnh.BlockNumber != nil {
		return bnh.BlockNumber.String()
	}
	if bnh.BlockHash != nil {
		return bnh.BlockHash.String()
	}
	return "nil"
}

func (bnh *BlockNumberOrHash) Hash() (common.Hash, bool) {
	if bnh.BlockHash != nil {
		return *bnh.BlockHash, true
	}
	return common.Hash{}, false
}

func BlockNumberOrHashWithNumber(blockNr BlockNumber) BlockNumberOrHash {
	return BlockNumberOrHash{
		BlockNumber:      &blockNr,
		BlockHash:        nil,
		RequireCanonical: false,
	}
}

func BlockNumberOrHashWithHash(hash common.Hash, canonical bool) BlockNumberOrHash {
	return BlockNumberOrHash{
		BlockNumber:      nil,
		BlockHash:        &hash,
		RequireCanonical: canonical,
	}
}
