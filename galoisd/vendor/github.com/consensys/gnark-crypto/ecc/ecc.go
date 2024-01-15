/*
Copyright © 2020 ConsenSys

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

// Package ecc provides bls12-381, bls12-377, bls12-378, bn254, bw6-761, bls24-315, bls24-317, bw6-633, bls12-378, bw6-756, secp256k1 and stark-curve elliptic curves implementation (+pairing).
//
// Also
//
//   - Multi exponentiation
//   - FFT
//   - Polynomial commitment schemes
//   - MiMC
//   - twisted edwards "companion curves"
//   - EdDSA (on the "companion" twisted edwards curves)
package ecc

import (
	"errors"
	"math/big"
	"strings"

	"github.com/consensys/gnark-crypto/internal/generator/config"
)

// ID represent a unique ID for a curve
type ID uint16

// do not modify the order of this enum
const (
	UNKNOWN ID = iota
	BN254
	BLS12_377
	BLS12_378
	BLS12_381
	BLS24_315
	BLS24_317
	BW6_761
	BW6_633
	BW6_756
	STARK_CURVE
	SECP256K1
)

// Implemented return the list of curves fully implemented in gnark-crypto
func Implemented() []ID {
	return []ID{BN254, BLS12_377, BLS12_381, BW6_761, BLS24_315, BW6_633, BLS12_378, BW6_756, BLS24_317, STARK_CURVE, SECP256K1}
}

func IDFromString(s string) (ID, error) {
	for _, id := range Implemented() {
		if strings.ToLower(s) == id.String() {
			return id, nil
		}
	}
	return UNKNOWN, errors.New("unknown curve ID")
}

func (id ID) String() string {
	cfg := id.config()
	return strings.ToLower(cfg.EnumID)
}

// ScalarField returns the scalar field of the curve
func (id ID) ScalarField() *big.Int {
	cfg := id.config()
	return modulus(cfg, true)
}

// BaseField returns the base field of the curve
func (id ID) BaseField() *big.Int {
	cfg := id.config()
	return modulus(cfg, false)
}

func (id ID) config() *config.Curve {
	// note to avoid circular dependency these are hard coded
	// values are checked for non regression in code generation
	switch id {
	case BLS12_377:
		return &config.BLS12_377
	case BLS12_378:
		return &config.BLS12_378
	case BLS12_381:
		return &config.BLS12_381
	case BN254:
		return &config.BN254
	case BW6_761:
		return &config.BW6_761
	case BW6_633:
		return &config.BW6_633
	case BLS24_315:
		return &config.BLS24_315
	case BLS24_317:
		return &config.BLS24_317
	case BW6_756:
		return &config.BW6_756
	case STARK_CURVE:
		return &config.STARK_CURVE
	case SECP256K1:
		return &config.SECP256K1
	default:
		panic("unimplemented ecc ID")
	}
}

func modulus(c *config.Curve, scalarField bool) *big.Int {
	if scalarField {
		return new(big.Int).Set(c.FrInfo.Modulus())
	}

	return new(big.Int).Set(c.FpInfo.Modulus())
}

// MultiExpConfig enables to set optional configuration attribute to a call to MultiExp
type MultiExpConfig struct {
	NbTasks int // go routines to be used in the multiexp. can be larger than num cpus.
}
