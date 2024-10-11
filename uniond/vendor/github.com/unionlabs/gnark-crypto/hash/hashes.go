// Copyright 2020 ConsenSys Software Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Package hash provides MiMC hash function defined over curves implemented in gnark-crypto/ecc.
//
// Originally developed and used in a ZKP context.
package hash

import (
	"hash"

	bls377 "github.com/consensys/gnark-crypto/ecc/bls12-377/fr/mimc"
	bls378 "github.com/consensys/gnark-crypto/ecc/bls12-378/fr/mimc"
	bls381 "github.com/consensys/gnark-crypto/ecc/bls12-381/fr/mimc"
	bls315 "github.com/consensys/gnark-crypto/ecc/bls24-315/fr/mimc"
	bls317 "github.com/consensys/gnark-crypto/ecc/bls24-317/fr/mimc"
	bn254 "github.com/consensys/gnark-crypto/ecc/bn254/fr/mimc"
	bw633 "github.com/consensys/gnark-crypto/ecc/bw6-633/fr/mimc"
	bw756 "github.com/consensys/gnark-crypto/ecc/bw6-756/fr/mimc"
	bw761 "github.com/consensys/gnark-crypto/ecc/bw6-761/fr/mimc"
)

type Hash uint

const (
	MIMC_BN254 Hash = iota
	MIMC_BLS12_381
	MIMC_BLS12_377
	MIMC_BLS12_378
	MIMC_BW6_761
	MIMC_BLS24_315
	MIMC_BLS24_317
	MIMC_BW6_633
	MIMC_BW6_756
)

// size of digests in bytes
var digestSize = []uint8{
	MIMC_BN254:     32,
	MIMC_BLS12_381: 48,
	MIMC_BLS12_377: 48,
	MIMC_BLS12_378: 48,
	MIMC_BW6_761:   96,
	MIMC_BLS24_315: 48,
	MIMC_BLS24_317: 48,
	MIMC_BW6_633:   80,
	MIMC_BW6_756:   96,
}

// New creates the corresponding mimc hash function.
func (m Hash) New() hash.Hash {
	switch m {
	case MIMC_BN254:
		return bn254.NewMiMC()
	case MIMC_BLS12_381:
		return bls381.NewMiMC()
	case MIMC_BLS12_377:
		return bls377.NewMiMC()
	case MIMC_BLS12_378:
		return bls378.NewMiMC()
	case MIMC_BW6_761:
		return bw761.NewMiMC()
	case MIMC_BLS24_315:
		return bls315.NewMiMC()
	case MIMC_BLS24_317:
		return bls317.NewMiMC()
	case MIMC_BW6_633:
		return bw633.NewMiMC()
	case MIMC_BW6_756:
		return bw756.NewMiMC()
	default:
		panic("Unknown mimc ID")
	}
}

// String returns the mimc ID to string format.
func (m Hash) String() string {
	switch m {
	case MIMC_BN254:
		return "MIMC_BN254"
	case MIMC_BLS12_381:
		return "MIMC_BLS381"
	case MIMC_BLS12_377:
		return "MIMC_BLS377"
	case MIMC_BLS12_378:
		return "MIMC_BLS378"
	case MIMC_BW6_761:
		return "MIMC_BW761"
	case MIMC_BLS24_315:
		return "MIMC_BLS315"
	case MIMC_BLS24_317:
		return "MIMC_BLS317"
	case MIMC_BW6_633:
		return "MIMC_BW633"
	case MIMC_BW6_756:
		return "MIMC_BW756"
	default:
		panic("Unknown mimc ID")
	}
}

// Size returns the size of the digest of
// the corresponding hash function
func (m Hash) Size() int {
	return int(digestSize[m])
}
