/*
*
This implements the client side functions as specified in
https://github.com/cosmos/ics/tree/master/spec/ics-023-vector-commitments
In particular:

	// Assumes ExistenceProof
	type verifyMembership = (root: CommitmentRoot, proof: CommitmentProof, key: Key, value: Value) => boolean

	// Assumes NonExistenceProof
	type verifyNonMembership = (root: CommitmentRoot, proof: CommitmentProof, key: Key) => boolean

	// Assumes BatchProof - required ExistenceProofs may be a subset of all items proven
	type batchVerifyMembership = (root: CommitmentRoot, proof: CommitmentProof, items: Map<Key, Value>) => boolean

	// Assumes BatchProof - required NonExistenceProofs may be a subset of all items proven
	type batchVerifyNonMembership = (root: CommitmentRoot, proof: CommitmentProof, keys: Set<Key>) => boolean

We make an adjustment to accept a Spec to ensure the provided proof is in the format of the expected merkle store.
This can avoid an range of attacks on fake preimages, as we need to be careful on how to map key, value -> leaf
and determine neighbors
*/
package ics23

import (
	"bytes"
	"fmt"
)

// CommitmentRoot is a byte slice that represents the merkle root of a tree that can be used to validate proofs
type CommitmentRoot []byte

// VerifyMembership returns true iff
// proof is (contains) an ExistenceProof for the given key and value AND
// calculating the root for the ExistenceProof matches the provided CommitmentRoot
func VerifyMembership(spec *ProofSpec, root CommitmentRoot, proof *CommitmentProof, key []byte, value []byte) bool {
	// decompress it before running code (no-op if not compressed)
	proof = Decompress(proof)
	ep := getExistProofForKey(proof, key)
	if ep == nil {
		return false
	}
	err := ep.Verify(spec, root, key, value)
	return err == nil
}

// VerifyNonMembership returns true iff
// proof is (contains) a NonExistenceProof
// both left and right sub-proofs are valid existence proofs (see above) or nil
// left and right proofs are neighbors (or left/right most if one is nil)
// provided key is between the keys of the two proofs
func VerifyNonMembership(spec *ProofSpec, root CommitmentRoot, proof *CommitmentProof, key []byte) bool {
	// decompress it before running code (no-op if not compressed)
	proof = Decompress(proof)
	np := getNonExistProofForKey(proof, key)
	if np == nil {
		return false
	}
	err := np.Verify(spec, root, key)
	return err == nil
}

// BatchVerifyMembership will ensure all items are also proven by the CommitmentProof (which should be a BatchProof,
// unless there is one item, when a ExistenceProof may work)
func BatchVerifyMembership(spec *ProofSpec, root CommitmentRoot, proof *CommitmentProof, items map[string][]byte) bool {
	// decompress it before running code (no-op if not compressed) - once for batch
	proof = Decompress(proof)
	for k, v := range items {
		valid := VerifyMembership(spec, root, proof, []byte(k), v)
		if !valid {
			return false
		}
	}
	return true
}

// BatchVerifyNonMembership will ensure all items are also proven to not be in the Commitment by the CommitmentProof
// (which should be a BatchProof, unless there is one item, when a NonExistenceProof may work)
func BatchVerifyNonMembership(spec *ProofSpec, root CommitmentRoot, proof *CommitmentProof, keys [][]byte) bool {
	// decompress it before running code (no-op if not compressed) - once for batch
	proof = Decompress(proof)
	for _, k := range keys {
		valid := VerifyNonMembership(spec, root, proof, k)
		if !valid {
			return false
		}
	}
	return true
}

// CombineProofs takes a number of commitment proofs (simple or batch) and
// converts them into a batch and compresses them.
//
// This is designed for proof generation libraries to create efficient batches
func CombineProofs(proofs []*CommitmentProof) (*CommitmentProof, error) {
	var entries []*BatchEntry

	for _, proof := range proofs {
		if ex := proof.GetExist(); ex != nil {
			entry := &BatchEntry{
				Proof: &BatchEntry_Exist{
					Exist: ex,
				},
			}
			entries = append(entries, entry)
		} else if non := proof.GetNonexist(); non != nil {
			entry := &BatchEntry{
				Proof: &BatchEntry_Nonexist{
					Nonexist: non,
				},
			}
			entries = append(entries, entry)
		} else if batch := proof.GetBatch(); batch != nil {
			entries = append(entries, batch.Entries...)
		} else if comp := proof.GetCompressed(); comp != nil {
			decomp := Decompress(proof)
			entries = append(entries, decomp.GetBatch().Entries...)
		} else {
			return nil, fmt.Errorf("proof neither exist or nonexist: %#v", proof.GetProof())
		}
	}

	batch := &CommitmentProof{
		Proof: &CommitmentProof_Batch{
			Batch: &BatchProof{
				Entries: entries,
			},
		},
	}

	return Compress(batch), nil
}

func getExistProofForKey(proof *CommitmentProof, key []byte) *ExistenceProof {
	switch p := proof.Proof.(type) {
	case *CommitmentProof_Exist:
		ep := p.Exist
		if bytes.Equal(ep.Key, key) {
			return ep
		}
	case *CommitmentProof_Batch:
		for _, sub := range p.Batch.Entries {
			if ep := sub.GetExist(); ep != nil && bytes.Equal(ep.Key, key) {
				return ep
			}
		}
	}
	return nil
}

func getNonExistProofForKey(proof *CommitmentProof, key []byte) *NonExistenceProof {
	switch p := proof.Proof.(type) {
	case *CommitmentProof_Nonexist:
		np := p.Nonexist
		if isLeft(np.Left, key) && isRight(np.Right, key) {
			return np
		}
	case *CommitmentProof_Batch:
		for _, sub := range p.Batch.Entries {
			if np := sub.GetNonexist(); np != nil && isLeft(np.Left, key) && isRight(np.Right, key) {
				return np
			}
		}
	}
	return nil
}

func isLeft(left *ExistenceProof, key []byte) bool {
	return left == nil || bytes.Compare(left.Key, key) < 0
}

func isRight(right *ExistenceProof, key []byte) bool {
	return right == nil || bytes.Compare(right.Key, key) > 0
}
