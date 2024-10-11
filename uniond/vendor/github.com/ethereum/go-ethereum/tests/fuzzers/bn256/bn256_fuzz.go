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

package bn256

import (
	"bytes"
	"fmt"
	"io"
	"math/big"

	"github.com/consensys/gnark-crypto/ecc/bn254"
	cloudflare "github.com/ethereum/go-ethereum/crypto/bn256/cloudflare"
	google "github.com/ethereum/go-ethereum/crypto/bn256/google"
)

func getG1Points(input io.Reader) (*cloudflare.G1, *google.G1, *bn254.G1Affine) {
	_, xc, err := cloudflare.RandomG1(input)
	if err != nil {
		// insufficient input
		return nil, nil, nil
	}
	xg := new(google.G1)
	if _, err := xg.Unmarshal(xc.Marshal()); err != nil {
		panic(fmt.Sprintf("Could not marshal cloudflare -> google: %v", err))
	}
	xs := new(bn254.G1Affine)
	if err := xs.Unmarshal(xc.Marshal()); err != nil {
		panic(fmt.Sprintf("Could not marshal cloudflare -> gnark: %v", err))
	}
	return xc, xg, xs
}

func getG2Points(input io.Reader) (*cloudflare.G2, *google.G2, *bn254.G2Affine) {
	_, xc, err := cloudflare.RandomG2(input)
	if err != nil {
		// insufficient input
		return nil, nil, nil
	}
	xg := new(google.G2)
	if _, err := xg.Unmarshal(xc.Marshal()); err != nil {
		panic(fmt.Sprintf("Could not marshal cloudflare -> google: %v", err))
	}
	xs := new(bn254.G2Affine)
	if err := xs.Unmarshal(xc.Marshal()); err != nil {
		panic(fmt.Sprintf("Could not marshal cloudflare -> gnark: %v", err))
	}
	return xc, xg, xs
}

// fuzzAdd fuzzez bn256 addition between the Google and Cloudflare libraries.
func fuzzAdd(data []byte) int {
	input := bytes.NewReader(data)
	xc, xg, xs := getG1Points(input)
	if xc == nil {
		return 0
	}
	yc, yg, ys := getG1Points(input)
	if yc == nil {
		return 0
	}
	// Ensure both libs can parse the second curve point
	// Add the two points and ensure they result in the same output
	rc := new(cloudflare.G1)
	rc.Add(xc, yc)

	rg := new(google.G1)
	rg.Add(xg, yg)

	tmpX := new(bn254.G1Jac).FromAffine(xs)
	tmpY := new(bn254.G1Jac).FromAffine(ys)
	rs := new(bn254.G1Affine).FromJacobian(tmpX.AddAssign(tmpY))

	if !bytes.Equal(rc.Marshal(), rg.Marshal()) {
		panic("add mismatch: cloudflare/google")
	}

	if !bytes.Equal(rc.Marshal(), rs.Marshal()) {
		panic("add mismatch: cloudflare/gnark")
	}
	return 1
}

// fuzzMul fuzzez bn256 scalar multiplication between the Google and Cloudflare
// libraries.
func fuzzMul(data []byte) int {
	input := bytes.NewReader(data)
	pc, pg, ps := getG1Points(input)
	if pc == nil {
		return 0
	}
	// Add the two points and ensure they result in the same output
	remaining := input.Len()
	if remaining == 0 {
		return 0
	}
	if remaining > 128 {
		// The evm only ever uses 32 byte integers, we need to cap this otherwise
		// we run into slow exec. A 236Kb byte integer cause oss-fuzz to report it as slow.
		// 128 bytes should be fine though
		return 0
	}
	buf := make([]byte, remaining)
	input.Read(buf)

	rc := new(cloudflare.G1)
	rc.ScalarMult(pc, new(big.Int).SetBytes(buf))

	rg := new(google.G1)
	rg.ScalarMult(pg, new(big.Int).SetBytes(buf))

	rs := new(bn254.G1Jac)
	psJac := new(bn254.G1Jac).FromAffine(ps)
	rs.ScalarMultiplication(psJac, new(big.Int).SetBytes(buf))
	rsAffine := new(bn254.G1Affine).FromJacobian(rs)

	if !bytes.Equal(rc.Marshal(), rg.Marshal()) {
		panic("scalar mul mismatch: cloudflare/google")
	}
	if !bytes.Equal(rc.Marshal(), rsAffine.Marshal()) {
		panic("scalar mul mismatch: cloudflare/gnark")
	}
	return 1
}

func fuzzPair(data []byte) int {
	input := bytes.NewReader(data)
	pc, pg, ps := getG1Points(input)
	if pc == nil {
		return 0
	}
	tc, tg, ts := getG2Points(input)
	if tc == nil {
		return 0
	}

	// Pair the two points and ensure they result in the same output
	clPair := cloudflare.Pair(pc, tc).Marshal()
	gPair := google.Pair(pg, tg).Marshal()
	if !bytes.Equal(clPair, gPair) {
		panic("pairing mismatch: cloudflare/google")
	}
	cPair, err := bn254.Pair([]bn254.G1Affine{*ps}, []bn254.G2Affine{*ts})
	if err != nil {
		panic(fmt.Sprintf("gnark/bn254 encountered error: %v", err))
	}

	// gnark uses a different pairing algorithm which might produce
	// different but also correct outputs, we need to scale the output by s

	u, _ := new(big.Int).SetString("0x44e992b44a6909f1", 0)
	u_exp2 := new(big.Int).Exp(u, big.NewInt(2), nil)   // u^2
	u_6_exp2 := new(big.Int).Mul(big.NewInt(6), u_exp2) // 6*u^2
	u_3 := new(big.Int).Mul(big.NewInt(3), u)           // 3*u
	inner := u_6_exp2.Add(u_6_exp2, u_3)                // 6*u^2 + 3*u
	inner.Add(inner, big.NewInt(1))                     // 6*u^2 + 3*u + 1
	u_2 := new(big.Int).Mul(big.NewInt(2), u)           // 2*u
	s := u_2.Mul(u_2, inner)                            // 2*u(6*u^2 + 3*u + 1)

	gRes := new(bn254.GT)
	if err := gRes.SetBytes(clPair); err != nil {
		panic(err)
	}
	gRes = gRes.Exp(*gRes, s)
	if !bytes.Equal(cPair.Marshal(), gRes.Marshal()) {
		panic("pairing mismatch: cloudflare/gnark")
	}

	return 1
}
