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

package les

import (
	"fmt"
	"math/big"
	"sync"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/eth/ethconfig"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/light"
	"github.com/ethereum/go-ethereum/p2p"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/params"
)

func errResp(code errCode, format string, v ...interface{}) error {
	return fmt.Errorf("%v - %v", code, fmt.Sprintf(format, v...))
}

type chainReader interface {
	CurrentHeader() *types.Header
}

// lesCommons contains fields needed by both server and client.
type lesCommons struct {
	genesis                      common.Hash
	config                       *ethconfig.Config
	chainConfig                  *params.ChainConfig
	iConfig                      *light.IndexerConfig
	chainDb, lesDb               ethdb.Database
	chainReader                  chainReader
	chtIndexer, bloomTrieIndexer *core.ChainIndexer

	closeCh chan struct{}
	wg      sync.WaitGroup
}

// NodeInfo represents a short summary of the Ethereum sub-protocol metadata
// known about the host peer.
type NodeInfo struct {
	Network    uint64              `json:"network"`    // Ethereum network ID (1=Mainnet, Goerli=5)
	Difficulty *big.Int            `json:"difficulty"` // Total difficulty of the host's blockchain
	Genesis    common.Hash         `json:"genesis"`    // SHA3 hash of the host's genesis block
	Config     *params.ChainConfig `json:"config"`     // Chain configuration for the fork rules
	Head       common.Hash         `json:"head"`       // SHA3 hash of the host's best owned block
}

// makeProtocols creates protocol descriptors for the given LES versions.
func (c *lesCommons) makeProtocols(versions []uint, runPeer func(version uint, p *p2p.Peer, rw p2p.MsgReadWriter) error, peerInfo func(id enode.ID) interface{}, dialCandidates enode.Iterator) []p2p.Protocol {
	protos := make([]p2p.Protocol, len(versions))
	for i, version := range versions {
		version := version
		protos[i] = p2p.Protocol{
			Name:     "les",
			Version:  version,
			Length:   ProtocolLengths[version],
			NodeInfo: c.nodeInfo,
			Run: func(peer *p2p.Peer, rw p2p.MsgReadWriter) error {
				return runPeer(version, peer, rw)
			},
			PeerInfo:       peerInfo,
			DialCandidates: dialCandidates,
		}
	}
	return protos
}

// nodeInfo retrieves some protocol metadata about the running host node.
func (c *lesCommons) nodeInfo() interface{} {
	head := c.chainReader.CurrentHeader()
	hash := head.Hash()
	return &NodeInfo{
		Network:    c.config.NetworkId,
		Difficulty: rawdb.ReadTd(c.chainDb, hash, head.Number.Uint64()),
		Genesis:    c.genesis,
		Config:     c.chainConfig,
		Head:       hash,
	}
}
