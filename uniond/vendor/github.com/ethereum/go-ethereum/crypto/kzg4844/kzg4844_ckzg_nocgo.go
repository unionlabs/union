// Copyright 2023 The go-ethereum Authors
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

//go:build !ckzg || nacl || js || !cgo || gofuzz

package kzg4844

import "sync"

// ckzgAvailable signals whether the library was compiled into Geth.
const ckzgAvailable = false

// ckzgIniter ensures that we initialize the KZG library once before using it.
var ckzgIniter sync.Once

// ckzgInit initializes the KZG library with the provided trusted setup.
func ckzgInit() {
	panic("unsupported platform")
}

// ckzgBlobToCommitment creates a small commitment out of a data blob.
func ckzgBlobToCommitment(blob Blob) (Commitment, error) {
	panic("unsupported platform")
}

// ckzgComputeProof computes the KZG proof at the given point for the polynomial
// represented by the blob.
func ckzgComputeProof(blob Blob, point Point) (Proof, Claim, error) {
	panic("unsupported platform")
}

// ckzgVerifyProof verifies the KZG proof that the polynomial represented by the blob
// evaluated at the given point is the claimed value.
func ckzgVerifyProof(commitment Commitment, point Point, claim Claim, proof Proof) error {
	panic("unsupported platform")
}

// ckzgComputeBlobProof returns the KZG proof that is used to verify the blob against
// the commitment.
//
// This method does not verify that the commitment is correct with respect to blob.
func ckzgComputeBlobProof(blob Blob, commitment Commitment) (Proof, error) {
	panic("unsupported platform")
}

// ckzgVerifyBlobProof verifies that the blob data corresponds to the provided commitment.
func ckzgVerifyBlobProof(blob Blob, commitment Commitment, proof Proof) error {
	panic("unsupported platform")
}
