// Copyright 2017 The go-ethereum Authors
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

// Package ethash implements the ethash proof-of-work consensus engine.
package ethash

import (
	"time"

	"github.com/ethereum/go-ethereum/consensus"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/rpc"
)

// Ethash is a consensus engine based on proof-of-work implementing the ethash
// algorithm.
type Ethash struct {
	fakeFail  *uint64        // Block number which fails PoW check even in fake mode
	fakeDelay *time.Duration // Time delay to sleep for before returning from verify
	fakeFull  bool           // Accepts everything as valid
}

// NewFaker creates an ethash consensus engine with a fake PoW scheme that accepts
// all blocks' seal as valid, though they still have to conform to the Ethereum
// consensus rules.
func NewFaker() *Ethash {
	return new(Ethash)
}

// NewFakeFailer creates a ethash consensus engine with a fake PoW scheme that
// accepts all blocks as valid apart from the single one specified, though they
// still have to conform to the Ethereum consensus rules.
func NewFakeFailer(fail uint64) *Ethash {
	return &Ethash{
		fakeFail: &fail,
	}
}

// NewFakeDelayer creates a ethash consensus engine with a fake PoW scheme that
// accepts all blocks as valid, but delays verifications by some time, though
// they still have to conform to the Ethereum consensus rules.
func NewFakeDelayer(delay time.Duration) *Ethash {
	return &Ethash{
		fakeDelay: &delay,
	}
}

// NewFullFaker creates an ethash consensus engine with a full fake scheme that
// accepts all blocks as valid, without checking any consensus rules whatsoever.
func NewFullFaker() *Ethash {
	return &Ethash{
		fakeFull: true,
	}
}

// Close closes the exit channel to notify all backend threads exiting.
func (ethash *Ethash) Close() error {
	return nil
}

// APIs implements consensus.Engine, returning no APIs as ethash is an empty
// shell in the post-merge world.
func (ethash *Ethash) APIs(chain consensus.ChainHeaderReader) []rpc.API {
	return []rpc.API{}
}

// Seal generates a new sealing request for the given input block and pushes
// the result into the given channel. For the ethash engine, this method will
// just panic as sealing is not supported anymore.
func (ethash *Ethash) Seal(chain consensus.ChainHeaderReader, block *types.Block, results chan<- *types.Block, stop <-chan struct{}) error {
	panic("ethash (pow) sealing not supported any more")
}
