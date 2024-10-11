// Copyright 2022 The go-ethereum Authors
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

package types

import (
	"crypto/sha256"
	"fmt"
	"os"
	"sort"
	"strconv"
	"strings"

	"github.com/ethereum/go-ethereum/beacon/merkle"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"gopkg.in/yaml.v3"
)

// syncCommitteeDomain specifies the signatures specific use to avoid clashes
// across signing different data structures.
const syncCommitteeDomain = 7

// Fork describes a single beacon chain fork and also stores the calculated
// signature domain used after this fork.
type Fork struct {
	// Name of the fork in the chain config (config.yaml) file{
	Name string

	// Epoch when given fork version is activated
	Epoch uint64

	// Fork version, see https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#custom-types
	Version []byte

	// calculated by computeDomain, based on fork version and genesis validators root
	domain merkle.Value
}

// computeDomain returns the signature domain based on the given fork version
// and genesis validator set root.
func (f *Fork) computeDomain(genesisValidatorsRoot common.Hash) {
	var (
		hasher        = sha256.New()
		forkVersion32 merkle.Value
		forkDataRoot  merkle.Value
	)
	copy(forkVersion32[:], f.Version)
	hasher.Write(forkVersion32[:])
	hasher.Write(genesisValidatorsRoot[:])
	hasher.Sum(forkDataRoot[:0])

	f.domain[0] = syncCommitteeDomain
	copy(f.domain[4:], forkDataRoot[:28])
}

// Forks is the list of all beacon chain forks in the chain configuration.
type Forks []*Fork

// domain returns the signature domain for the given epoch (assumes that domains
// have already been calculated).
func (f Forks) domain(epoch uint64) (merkle.Value, error) {
	for i := len(f) - 1; i >= 0; i-- {
		if epoch >= f[i].Epoch {
			return f[i].domain, nil
		}
	}
	return merkle.Value{}, fmt.Errorf("unknown fork for epoch %d", epoch)
}

// SigningRoot calculates the signing root of the given header.
func (f Forks) SigningRoot(header Header) (common.Hash, error) {
	domain, err := f.domain(header.Epoch())
	if err != nil {
		return common.Hash{}, err
	}
	var (
		signingRoot common.Hash
		headerHash  = header.Hash()
		hasher      = sha256.New()
	)
	hasher.Write(headerHash[:])
	hasher.Write(domain[:])
	hasher.Sum(signingRoot[:0])

	return signingRoot, nil
}

func (f Forks) Len() int           { return len(f) }
func (f Forks) Swap(i, j int)      { f[i], f[j] = f[j], f[i] }
func (f Forks) Less(i, j int) bool { return f[i].Epoch < f[j].Epoch }

// ChainConfig contains the beacon chain configuration.
type ChainConfig struct {
	GenesisTime           uint64      // Unix timestamp of slot 0
	GenesisValidatorsRoot common.Hash // Root hash of the genesis validator set, used for signature domain calculation
	Forks                 Forks
}

// AddFork adds a new item to the list of forks.
func (c *ChainConfig) AddFork(name string, epoch uint64, version []byte) *ChainConfig {
	fork := &Fork{
		Name:    name,
		Epoch:   epoch,
		Version: version,
	}
	fork.computeDomain(c.GenesisValidatorsRoot)

	c.Forks = append(c.Forks, fork)
	sort.Sort(c.Forks)

	return c
}

// LoadForks parses the beacon chain configuration file (config.yaml) and extracts
// the list of forks.
func (c *ChainConfig) LoadForks(path string) error {
	file, err := os.ReadFile(path)
	if err != nil {
		return fmt.Errorf("failed to read beacon chain config file: %v", err)
	}
	config := make(map[string]string)
	if err := yaml.Unmarshal(file, &config); err != nil {
		return fmt.Errorf("failed to parse beacon chain config file: %v", err)
	}
	var (
		versions = make(map[string][]byte)
		epochs   = make(map[string]uint64)
	)
	epochs["GENESIS"] = 0

	for key, value := range config {
		if strings.HasSuffix(key, "_FORK_VERSION") {
			name := key[:len(key)-len("_FORK_VERSION")]
			if v, err := hexutil.Decode(value); err == nil {
				versions[name] = v
			} else {
				return fmt.Errorf("failed to decode hex fork id %q in beacon chain config file: %v", value, err)
			}
		}
		if strings.HasSuffix(key, "_FORK_EPOCH") {
			name := key[:len(key)-len("_FORK_EPOCH")]
			if v, err := strconv.ParseUint(value, 10, 64); err == nil {
				epochs[name] = v
			} else {
				return fmt.Errorf("failed to parse epoch number %q in beacon chain config file: %v", value, err)
			}
		}
	}
	for name, epoch := range epochs {
		if version, ok := versions[name]; ok {
			delete(versions, name)
			c.AddFork(name, epoch, version)
		} else {
			return fmt.Errorf("fork id missing for %q in beacon chain config file", name)
		}
	}
	for name := range versions {
		return fmt.Errorf("epoch number missing for fork %q in beacon chain config file", name)
	}
	sort.Sort(c.Forks)
	return nil
}
