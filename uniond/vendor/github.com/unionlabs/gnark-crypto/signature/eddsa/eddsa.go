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

package eddsa

import (
	"io"

	eddsa_bls12377 "github.com/consensys/gnark-crypto/ecc/bls12-377/twistededwards/eddsa"
	eddsa_bls12378 "github.com/consensys/gnark-crypto/ecc/bls12-378/twistededwards/eddsa"
	eddsa_bls12381_bandersnatch "github.com/consensys/gnark-crypto/ecc/bls12-381/bandersnatch/eddsa"
	eddsa_bls12381 "github.com/consensys/gnark-crypto/ecc/bls12-381/twistededwards/eddsa"
	eddsa_bls24315 "github.com/consensys/gnark-crypto/ecc/bls24-315/twistededwards/eddsa"
	eddsa_bls24317 "github.com/consensys/gnark-crypto/ecc/bls24-317/twistededwards/eddsa"
	eddsa_bn254 "github.com/consensys/gnark-crypto/ecc/bn254/twistededwards/eddsa"
	eddsa_bw6633 "github.com/consensys/gnark-crypto/ecc/bw6-633/twistededwards/eddsa"
	eddsa_bw6756 "github.com/consensys/gnark-crypto/ecc/bw6-756/twistededwards/eddsa"
	eddsa_bw6761 "github.com/consensys/gnark-crypto/ecc/bw6-761/twistededwards/eddsa"
	"github.com/consensys/gnark-crypto/ecc/twistededwards"
	"github.com/consensys/gnark-crypto/signature"
)

// New takes a source of randomness and returns a new key pair
func New(ss twistededwards.ID, r io.Reader) (signature.Signer, error) {
	switch ss {
	case twistededwards.BN254:
		return eddsa_bn254.GenerateKey(r)
	case twistededwards.BLS12_381:
		return eddsa_bls12381.GenerateKey(r)
	case twistededwards.BLS12_381_BANDERSNATCH:
		return eddsa_bls12381_bandersnatch.GenerateKey(r)
	case twistededwards.BLS12_377:
		return eddsa_bls12377.GenerateKey(r)
	case twistededwards.BLS12_378:
		return eddsa_bls12378.GenerateKey(r)
	case twistededwards.BW6_761:
		return eddsa_bw6761.GenerateKey(r)
	case twistededwards.BW6_756:
		return eddsa_bw6756.GenerateKey(r)
	case twistededwards.BLS24_315:
		return eddsa_bls24315.GenerateKey(r)
	case twistededwards.BLS24_317:
		return eddsa_bls24317.GenerateKey(r)
	case twistededwards.BW6_633:
		return eddsa_bw6633.GenerateKey(r)
	default:
		panic("not implemented")
	}
}
