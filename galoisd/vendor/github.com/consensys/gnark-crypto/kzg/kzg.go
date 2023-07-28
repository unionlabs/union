// Package kzg provides constructor for curved-typed KZG SRS
//
// For more details, see ecc/XXX/fr/kzg package
package kzg

import (
	"io"

	"github.com/consensys/gnark-crypto/ecc"

	kzg_bls12377 "github.com/consensys/gnark-crypto/ecc/bls12-377/fr/kzg"
	kzg_bls12378 "github.com/consensys/gnark-crypto/ecc/bls12-378/fr/kzg"
	kzg_bls12381 "github.com/consensys/gnark-crypto/ecc/bls12-381/fr/kzg"
	kzg_bls24315 "github.com/consensys/gnark-crypto/ecc/bls24-315/fr/kzg"
	kzg_bls24317 "github.com/consensys/gnark-crypto/ecc/bls24-317/fr/kzg"
	kzg_bn254 "github.com/consensys/gnark-crypto/ecc/bn254/fr/kzg"
	kzg_bw6633 "github.com/consensys/gnark-crypto/ecc/bw6-633/fr/kzg"
	kzg_bw6756 "github.com/consensys/gnark-crypto/ecc/bw6-756/fr/kzg"
	kzg_bw6761 "github.com/consensys/gnark-crypto/ecc/bw6-761/fr/kzg"
)

// SRS ...
type SRS interface {
	io.ReaderFrom
	io.WriterTo
}

// NewSRS returns an empty curved-typed SRS object
// that implements io.ReaderFrom and io.WriterTo interfaces
func NewSRS(curveID ecc.ID) SRS {
	switch curveID {
	case ecc.BN254:
		return &kzg_bn254.SRS{}
	case ecc.BLS12_377:
		return &kzg_bls12377.SRS{}
	case ecc.BLS12_378:
		return &kzg_bls12378.SRS{}
	case ecc.BLS12_381:
		return &kzg_bls12381.SRS{}
	case ecc.BLS24_315:
		return &kzg_bls24315.SRS{}
	case ecc.BLS24_317:
		return &kzg_bls24317.SRS{}
	case ecc.BW6_761:
		return &kzg_bw6761.SRS{}
	case ecc.BW6_633:
		return &kzg_bw6633.SRS{}
	case ecc.BW6_756:
		return &kzg_bw6756.SRS{}
	default:
		panic("not implemented")
	}
}
