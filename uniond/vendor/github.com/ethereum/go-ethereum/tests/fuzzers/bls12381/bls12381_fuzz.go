// Copyright 2021 The go-ethereum Authors
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

package bls

import (
	"bytes"
	"crypto/rand"
	"fmt"
	"io"
	"math/big"

	"github.com/consensys/gnark-crypto/ecc"
	gnark "github.com/consensys/gnark-crypto/ecc/bls12-381"
	"github.com/consensys/gnark-crypto/ecc/bls12-381/fp"
	"github.com/consensys/gnark-crypto/ecc/bls12-381/fr"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto/bls12381"
	blst "github.com/supranational/blst/bindings/go"
)

func fuzzCrossPairing(data []byte) int {
	input := bytes.NewReader(data)

	// get random G1 points
	kpG1, cpG1, blG1, err := getG1Points(input)
	if err != nil {
		return 0
	}

	// get random G2 points
	kpG2, cpG2, blG2, err := getG2Points(input)
	if err != nil {
		return 0
	}

	// compute pairing using geth
	engine := bls12381.NewPairingEngine()
	engine.AddPair(kpG1, kpG2)
	kResult := engine.Result()

	// compute pairing using gnark
	cResult, err := gnark.Pair([]gnark.G1Affine{*cpG1}, []gnark.G2Affine{*cpG2})
	if err != nil {
		panic(fmt.Sprintf("gnark/bls12381 encountered error: %v", err))
	}

	// compare result
	if !(bytes.Equal(cResult.Marshal(), bls12381.NewGT().ToBytes(kResult))) {
		panic("pairing mismatch gnark / geth ")
	}

	// compute pairing using blst
	blstResult := blst.Fp12MillerLoop(blG2, blG1)
	blstResult.FinalExp()
	res := massageBLST(blstResult.ToBendian())
	if !(bytes.Equal(res, bls12381.NewGT().ToBytes(kResult))) {
		panic("pairing mismatch blst / geth")
	}

	return 1
}

func massageBLST(in []byte) []byte {
	out := make([]byte, len(in))
	len := 12 * 48
	// 1
	copy(out[0:], in[len-1*48:len])
	copy(out[1*48:], in[len-2*48:len-1*48])
	// 2
	copy(out[6*48:], in[len-3*48:len-2*48])
	copy(out[7*48:], in[len-4*48:len-3*48])
	// 3
	copy(out[2*48:], in[len-5*48:len-4*48])
	copy(out[3*48:], in[len-6*48:len-5*48])
	// 4
	copy(out[8*48:], in[len-7*48:len-6*48])
	copy(out[9*48:], in[len-8*48:len-7*48])
	// 5
	copy(out[4*48:], in[len-9*48:len-8*48])
	copy(out[5*48:], in[len-10*48:len-9*48])
	// 6
	copy(out[10*48:], in[len-11*48:len-10*48])
	copy(out[11*48:], in[len-12*48:len-11*48])
	return out
}

func fuzzCrossG1Add(data []byte) int {
	input := bytes.NewReader(data)

	// get random G1 points
	kp1, cp1, bl1, err := getG1Points(input)
	if err != nil {
		return 0
	}

	// get random G1 points
	kp2, cp2, bl2, err := getG1Points(input)
	if err != nil {
		return 0
	}

	// compute kp = kp1 + kp2
	g1 := bls12381.NewG1()
	kp := bls12381.PointG1{}
	g1.Add(&kp, kp1, kp2)

	// compute cp = cp1 + cp2
	_cp1 := new(gnark.G1Jac).FromAffine(cp1)
	_cp2 := new(gnark.G1Jac).FromAffine(cp2)
	cp := new(gnark.G1Affine).FromJacobian(_cp1.AddAssign(_cp2))

	// compare result
	if !(bytes.Equal(cp.Marshal(), g1.ToBytes(&kp))) {
		panic("G1 point addition mismatch gnark / geth ")
	}

	bl3 := blst.P1AffinesAdd([]*blst.P1Affine{bl1, bl2})
	if !(bytes.Equal(cp.Marshal(), bl3.Serialize())) {
		panic("G1 point addition mismatch blst / geth ")
	}

	return 1
}

func fuzzCrossG2Add(data []byte) int {
	input := bytes.NewReader(data)

	// get random G2 points
	kp1, cp1, bl1, err := getG2Points(input)
	if err != nil {
		return 0
	}

	// get random G2 points
	kp2, cp2, bl2, err := getG2Points(input)
	if err != nil {
		return 0
	}

	// compute kp = kp1 + kp2
	g2 := bls12381.NewG2()
	kp := bls12381.PointG2{}
	g2.Add(&kp, kp1, kp2)

	// compute cp = cp1 + cp2
	_cp1 := new(gnark.G2Jac).FromAffine(cp1)
	_cp2 := new(gnark.G2Jac).FromAffine(cp2)
	cp := new(gnark.G2Affine).FromJacobian(_cp1.AddAssign(_cp2))

	// compare result
	if !(bytes.Equal(cp.Marshal(), g2.ToBytes(&kp))) {
		panic("G2 point addition mismatch gnark / geth ")
	}

	bl3 := blst.P2AffinesAdd([]*blst.P2Affine{bl1, bl2})
	if !(bytes.Equal(cp.Marshal(), bl3.Serialize())) {
		panic("G1 point addition mismatch blst / geth ")
	}

	return 1
}

func fuzzCrossG1MultiExp(data []byte) int {
	var (
		input        = bytes.NewReader(data)
		gethScalars  []*big.Int
		gnarkScalars []fr.Element
		gethPoints   []*bls12381.PointG1
		gnarkPoints  []gnark.G1Affine
	)
	// n random scalars (max 17)
	for i := 0; i < 17; i++ {
		// note that geth/crypto/bls12381 works only with scalars <= 32bytes
		s, err := randomScalar(input, fr.Modulus())
		if err != nil {
			break
		}
		// get a random G1 point as basis
		kp1, cp1, _, err := getG1Points(input)
		if err != nil {
			break
		}
		gethScalars = append(gethScalars, s)
		var gnarkScalar = &fr.Element{}
		gnarkScalar = gnarkScalar.SetBigInt(s)
		gnarkScalars = append(gnarkScalars, *gnarkScalar)

		gethPoints = append(gethPoints, new(bls12381.PointG1).Set(kp1))
		gnarkPoints = append(gnarkPoints, *cp1)
	}
	if len(gethScalars) == 0 {
		return 0
	}
	// compute multi exponentiation
	g1 := bls12381.NewG1()
	kp := bls12381.PointG1{}
	if _, err := g1.MultiExp(&kp, gethPoints, gethScalars); err != nil {
		panic(fmt.Sprintf("G1 multi exponentiation errored (geth): %v", err))
	}
	// note that geth/crypto/bls12381.MultiExp mutates the scalars slice (and sets all the scalars to zero)

	// gnark multi exp
	cp := new(gnark.G1Affine)
	cp.MultiExp(gnarkPoints, gnarkScalars, ecc.MultiExpConfig{})

	// compare result
	if !(bytes.Equal(cp.Marshal(), g1.ToBytes(&kp))) {
		panic("G1 multi exponentiation mismatch gnark / geth ")
	}

	return 1
}

func getG1Points(input io.Reader) (*bls12381.PointG1, *gnark.G1Affine, *blst.P1Affine, error) {
	// sample a random scalar
	s, err := randomScalar(input, fp.Modulus())
	if err != nil {
		return nil, nil, nil, err
	}

	// compute a random point
	cp := new(gnark.G1Affine)
	_, _, g1Gen, _ := gnark.Generators()
	cp.ScalarMultiplication(&g1Gen, s)
	cpBytes := cp.Marshal()

	// marshal gnark point -> geth point
	g1 := bls12381.NewG1()
	kp, err := g1.FromBytes(cpBytes)
	if err != nil {
		panic(fmt.Sprintf("Could not marshal gnark.G1 -> geth.G1: %v", err))
	}
	if !bytes.Equal(g1.ToBytes(kp), cpBytes) {
		panic("bytes(gnark.G1) != bytes(geth.G1)")
	}

	// marshal gnark point -> blst point
	scalar := new(blst.Scalar).FromBEndian(common.LeftPadBytes(s.Bytes(), 32))
	p1 := new(blst.P1Affine).From(scalar)
	if !bytes.Equal(p1.Serialize(), cpBytes) {
		panic("bytes(blst.G1) != bytes(geth.G1)")
	}

	return kp, cp, p1, nil
}

func getG2Points(input io.Reader) (*bls12381.PointG2, *gnark.G2Affine, *blst.P2Affine, error) {
	// sample a random scalar
	s, err := randomScalar(input, fp.Modulus())
	if err != nil {
		return nil, nil, nil, err
	}

	// compute a random point
	cp := new(gnark.G2Affine)
	_, _, _, g2Gen := gnark.Generators()
	cp.ScalarMultiplication(&g2Gen, s)
	cpBytes := cp.Marshal()

	// marshal gnark point -> geth point
	g2 := bls12381.NewG2()
	kp, err := g2.FromBytes(cpBytes)
	if err != nil {
		panic(fmt.Sprintf("Could not marshal gnark.G2 -> geth.G2: %v", err))
	}
	if !bytes.Equal(g2.ToBytes(kp), cpBytes) {
		panic("bytes(gnark.G2) != bytes(geth.G2)")
	}

	// marshal gnark point -> blst point
	// Left pad the scalar to 32 bytes
	scalar := new(blst.Scalar).FromBEndian(common.LeftPadBytes(s.Bytes(), 32))
	p2 := new(blst.P2Affine).From(scalar)
	if !bytes.Equal(p2.Serialize(), cpBytes) {
		panic("bytes(blst.G2) != bytes(geth.G2)")
	}

	return kp, cp, p2, nil
}

func randomScalar(r io.Reader, max *big.Int) (k *big.Int, err error) {
	for {
		k, err = rand.Int(r, max)
		if err != nil || k.Sign() > 0 {
			return
		}
	}
}
