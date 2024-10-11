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

package kzg4844

import (
	"crypto/rand"
	"testing"

	"github.com/consensys/gnark-crypto/ecc/bls12-381/fr"
	gokzg4844 "github.com/crate-crypto/go-kzg-4844"
)

func randFieldElement() [32]byte {
	bytes := make([]byte, 32)
	_, err := rand.Read(bytes)
	if err != nil {
		panic("failed to get random field element")
	}
	var r fr.Element
	r.SetBytes(bytes)

	return gokzg4844.SerializeScalar(r)
}

func randBlob() Blob {
	var blob Blob
	for i := 0; i < len(blob); i += gokzg4844.SerializedScalarSize {
		fieldElementBytes := randFieldElement()
		copy(blob[i:i+gokzg4844.SerializedScalarSize], fieldElementBytes[:])
	}
	return blob
}

func TestCKZGWithPoint(t *testing.T)  { testKZGWithPoint(t, true) }
func TestGoKZGWithPoint(t *testing.T) { testKZGWithPoint(t, false) }
func testKZGWithPoint(t *testing.T, ckzg bool) {
	if ckzg && !ckzgAvailable {
		t.Skip("CKZG unavailable in this test build")
	}
	defer func(old bool) { useCKZG.Store(old) }(useCKZG.Load())
	useCKZG.Store(ckzg)

	blob := randBlob()

	commitment, err := BlobToCommitment(blob)
	if err != nil {
		t.Fatalf("failed to create KZG commitment from blob: %v", err)
	}
	point := randFieldElement()
	proof, claim, err := ComputeProof(blob, point)
	if err != nil {
		t.Fatalf("failed to create KZG proof at point: %v", err)
	}
	if err := VerifyProof(commitment, point, claim, proof); err != nil {
		t.Fatalf("failed to verify KZG proof at point: %v", err)
	}
}

func TestCKZGWithBlob(t *testing.T)  { testKZGWithBlob(t, true) }
func TestGoKZGWithBlob(t *testing.T) { testKZGWithBlob(t, false) }
func testKZGWithBlob(t *testing.T, ckzg bool) {
	if ckzg && !ckzgAvailable {
		t.Skip("CKZG unavailable in this test build")
	}
	defer func(old bool) { useCKZG.Store(old) }(useCKZG.Load())
	useCKZG.Store(ckzg)

	blob := randBlob()

	commitment, err := BlobToCommitment(blob)
	if err != nil {
		t.Fatalf("failed to create KZG commitment from blob: %v", err)
	}
	proof, err := ComputeBlobProof(blob, commitment)
	if err != nil {
		t.Fatalf("failed to create KZG proof for blob: %v", err)
	}
	if err := VerifyBlobProof(blob, commitment, proof); err != nil {
		t.Fatalf("failed to verify KZG proof for blob: %v", err)
	}
}

func BenchmarkCKZGBlobToCommitment(b *testing.B)  { benchmarkBlobToCommitment(b, true) }
func BenchmarkGoKZGBlobToCommitment(b *testing.B) { benchmarkBlobToCommitment(b, false) }
func benchmarkBlobToCommitment(b *testing.B, ckzg bool) {
	if ckzg && !ckzgAvailable {
		b.Skip("CKZG unavailable in this test build")
	}
	defer func(old bool) { useCKZG.Store(old) }(useCKZG.Load())
	useCKZG.Store(ckzg)

	blob := randBlob()

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		BlobToCommitment(blob)
	}
}

func BenchmarkCKZGComputeProof(b *testing.B)  { benchmarkComputeProof(b, true) }
func BenchmarkGoKZGComputeProof(b *testing.B) { benchmarkComputeProof(b, false) }
func benchmarkComputeProof(b *testing.B, ckzg bool) {
	if ckzg && !ckzgAvailable {
		b.Skip("CKZG unavailable in this test build")
	}
	defer func(old bool) { useCKZG.Store(old) }(useCKZG.Load())
	useCKZG.Store(ckzg)

	var (
		blob  = randBlob()
		point = randFieldElement()
	)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		ComputeProof(blob, point)
	}
}

func BenchmarkCKZGVerifyProof(b *testing.B)  { benchmarkVerifyProof(b, true) }
func BenchmarkGoKZGVerifyProof(b *testing.B) { benchmarkVerifyProof(b, false) }
func benchmarkVerifyProof(b *testing.B, ckzg bool) {
	if ckzg && !ckzgAvailable {
		b.Skip("CKZG unavailable in this test build")
	}
	defer func(old bool) { useCKZG.Store(old) }(useCKZG.Load())
	useCKZG.Store(ckzg)

	var (
		blob            = randBlob()
		point           = randFieldElement()
		commitment, _   = BlobToCommitment(blob)
		proof, claim, _ = ComputeProof(blob, point)
	)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		VerifyProof(commitment, point, claim, proof)
	}
}

func BenchmarkCKZGComputeBlobProof(b *testing.B)  { benchmarkComputeBlobProof(b, true) }
func BenchmarkGoKZGComputeBlobProof(b *testing.B) { benchmarkComputeBlobProof(b, false) }
func benchmarkComputeBlobProof(b *testing.B, ckzg bool) {
	if ckzg && !ckzgAvailable {
		b.Skip("CKZG unavailable in this test build")
	}
	defer func(old bool) { useCKZG.Store(old) }(useCKZG.Load())
	useCKZG.Store(ckzg)

	var (
		blob          = randBlob()
		commitment, _ = BlobToCommitment(blob)
	)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		ComputeBlobProof(blob, commitment)
	}
}

func BenchmarkCKZGVerifyBlobProof(b *testing.B)  { benchmarkVerifyBlobProof(b, true) }
func BenchmarkGoKZGVerifyBlobProof(b *testing.B) { benchmarkVerifyBlobProof(b, false) }
func benchmarkVerifyBlobProof(b *testing.B, ckzg bool) {
	if ckzg && !ckzgAvailable {
		b.Skip("CKZG unavailable in this test build")
	}
	defer func(old bool) { useCKZG.Store(old) }(useCKZG.Load())
	useCKZG.Store(ckzg)

	var (
		blob          = randBlob()
		commitment, _ = BlobToCommitment(blob)
		proof, _      = ComputeBlobProof(blob, commitment)
	)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		VerifyBlobProof(blob, commitment, proof)
	}
}
