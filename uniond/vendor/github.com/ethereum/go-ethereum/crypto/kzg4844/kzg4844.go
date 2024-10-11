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

// Package kzg4844 implements the KZG crypto for EIP-4844.
package kzg4844

import (
	"embed"
	"errors"
	"sync/atomic"
)

//go:embed trusted_setup.json
var content embed.FS

// Blob represents a 4844 data blob.
type Blob [131072]byte

// Commitment is a serialized commitment to a polynomial.
type Commitment [48]byte

// Proof is a serialized commitment to the quotient polynomial.
type Proof [48]byte

// Point is a BLS field element.
type Point [32]byte

// Claim is a claimed evaluation value in a specific point.
type Claim [32]byte

// useCKZG controls whether the cryptography should use the Go or C backend.
var useCKZG atomic.Bool

// UseCKZG can be called to switch the default Go implementation of KZG to the C
// library if fo some reason the user wishes to do so (e.g. consensus bug in one
// or the other).
func UseCKZG(use bool) error {
	if use && !ckzgAvailable {
		return errors.New("CKZG unavailable on your platform")
	}
	useCKZG.Store(use)

	// Initializing the library can take 2-4 seconds - and can potentially crash
	// on CKZG and non-ADX CPUs - so might as well do it now and don't wait until
	// a crypto operation is actually needed live.
	if use {
		ckzgIniter.Do(ckzgInit)
	} else {
		gokzgIniter.Do(gokzgInit)
	}
	return nil
}

// BlobToCommitment creates a small commitment out of a data blob.
func BlobToCommitment(blob Blob) (Commitment, error) {
	if useCKZG.Load() {
		return ckzgBlobToCommitment(blob)
	}
	return gokzgBlobToCommitment(blob)
}

// ComputeProof computes the KZG proof at the given point for the polynomial
// represented by the blob.
func ComputeProof(blob Blob, point Point) (Proof, Claim, error) {
	if useCKZG.Load() {
		return ckzgComputeProof(blob, point)
	}
	return gokzgComputeProof(blob, point)
}

// VerifyProof verifies the KZG proof that the polynomial represented by the blob
// evaluated at the given point is the claimed value.
func VerifyProof(commitment Commitment, point Point, claim Claim, proof Proof) error {
	if useCKZG.Load() {
		return ckzgVerifyProof(commitment, point, claim, proof)
	}
	return gokzgVerifyProof(commitment, point, claim, proof)
}

// ComputeBlobProof returns the KZG proof that is used to verify the blob against
// the commitment.
//
// This method does not verify that the commitment is correct with respect to blob.
func ComputeBlobProof(blob Blob, commitment Commitment) (Proof, error) {
	if useCKZG.Load() {
		return ckzgComputeBlobProof(blob, commitment)
	}
	return gokzgComputeBlobProof(blob, commitment)
}

// VerifyBlobProof verifies that the blob data corresponds to the provided commitment.
func VerifyBlobProof(blob Blob, commitment Commitment, proof Proof) error {
	if useCKZG.Load() {
		return ckzgVerifyBlobProof(blob, commitment, proof)
	}
	return gokzgVerifyBlobProof(blob, commitment, proof)
}
