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
	"compress/gzip"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"math/big"
	"os"
	"strings"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/forkid"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/rlp"
)

type Chain struct {
	genesis     core.Genesis
	blocks      []*types.Block
	chainConfig *params.ChainConfig
}

// Len returns the length of the chain.
func (c *Chain) Len() int {
	return len(c.blocks)
}

// TD calculates the total difficulty of the chain at the
// chain head.
func (c *Chain) TD() *big.Int {
	sum := new(big.Int)
	for _, block := range c.blocks[:c.Len()] {
		sum.Add(sum, block.Difficulty())
	}
	return sum
}

// TotalDifficultyAt calculates the total difficulty of the chain
// at the given block height.
func (c *Chain) TotalDifficultyAt(height int) *big.Int {
	sum := new(big.Int)
	if height >= c.Len() {
		return sum
	}
	for _, block := range c.blocks[:height+1] {
		sum.Add(sum, block.Difficulty())
	}
	return sum
}

func (c *Chain) RootAt(height int) common.Hash {
	if height < c.Len() {
		return c.blocks[height].Root()
	}
	return common.Hash{}
}

// ForkID gets the fork id of the chain.
func (c *Chain) ForkID() forkid.ID {
	return forkid.NewID(c.chainConfig, c.blocks[0], uint64(c.Len()), c.blocks[0].Time())
}

// Shorten returns a copy chain of a desired height from the imported
func (c *Chain) Shorten(height int) *Chain {
	blocks := make([]*types.Block, height)
	copy(blocks, c.blocks[:height])

	config := *c.chainConfig
	return &Chain{
		blocks:      blocks,
		chainConfig: &config,
	}
}

// Head returns the chain head.
func (c *Chain) Head() *types.Block {
	return c.blocks[c.Len()-1]
}

func (c *Chain) GetHeaders(req *GetBlockHeaders) ([]*types.Header, error) {
	if req.Amount < 1 {
		return nil, errors.New("no block headers requested")
	}

	headers := make([]*types.Header, req.Amount)
	var blockNumber uint64

	// range over blocks to check if our chain has the requested header
	for _, block := range c.blocks {
		if block.Hash() == req.Origin.Hash || block.Number().Uint64() == req.Origin.Number {
			headers[0] = block.Header()
			blockNumber = block.Number().Uint64()
		}
	}
	if headers[0] == nil {
		return nil, fmt.Errorf("no headers found for given origin number %v, hash %v", req.Origin.Number, req.Origin.Hash)
	}

	if req.Reverse {
		for i := 1; i < int(req.Amount); i++ {
			blockNumber -= (1 - req.Skip)
			headers[i] = c.blocks[blockNumber].Header()
		}

		return headers, nil
	}

	for i := 1; i < int(req.Amount); i++ {
		blockNumber += (1 + req.Skip)
		headers[i] = c.blocks[blockNumber].Header()
	}

	return headers, nil
}

// loadChain takes the given chain.rlp file, and decodes and returns
// the blocks from the file.
func loadChain(chainfile string, genesis string) (*Chain, error) {
	gen, err := loadGenesis(genesis)
	if err != nil {
		return nil, err
	}
	gblock := gen.ToBlock()

	blocks, err := blocksFromFile(chainfile, gblock)
	if err != nil {
		return nil, err
	}

	c := &Chain{genesis: gen, blocks: blocks, chainConfig: gen.Config}
	return c, nil
}

func loadGenesis(genesisFile string) (core.Genesis, error) {
	chainConfig, err := os.ReadFile(genesisFile)
	if err != nil {
		return core.Genesis{}, err
	}
	var gen core.Genesis
	if err := json.Unmarshal(chainConfig, &gen); err != nil {
		return core.Genesis{}, err
	}
	return gen, nil
}

func blocksFromFile(chainfile string, gblock *types.Block) ([]*types.Block, error) {
	// Load chain.rlp.
	fh, err := os.Open(chainfile)
	if err != nil {
		return nil, err
	}
	defer fh.Close()
	var reader io.Reader = fh
	if strings.HasSuffix(chainfile, ".gz") {
		if reader, err = gzip.NewReader(reader); err != nil {
			return nil, err
		}
	}
	stream := rlp.NewStream(reader, 0)
	var blocks = make([]*types.Block, 1)
	blocks[0] = gblock
	for i := 0; ; i++ {
		var b types.Block
		if err := stream.Decode(&b); err == io.EOF {
			break
		} else if err != nil {
			return nil, fmt.Errorf("at block index %d: %v", i, err)
		}
		if b.NumberU64() != uint64(i+1) {
			return nil, fmt.Errorf("block at index %d has wrong number %d", i, b.NumberU64())
		}
		blocks = append(blocks, &b)
	}
	return blocks, nil
}
