package trie_test

import (
	"testing"

	"github.com/golang/protobuf/proto"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/container/trie"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func FuzzSparseMerkleTrie_HashTreeRoot(f *testing.F) {
	h := hash.Hash([]byte("hi"))
	pb := &ethpb.SparseMerkleTrie{
		Layers: []*ethpb.TrieLayer{
			{
				Layer: [][]byte{h[:]},
			},
			{
				Layer: [][]byte{h[:]},
			},
			{
				Layer: [][]byte{h[:]},
			},
		},
		Depth: 2,
	}
	b, err := proto.Marshal(pb)
	require.NoError(f, err)
	f.Add(b)

	f.Fuzz(func(t *testing.T, b []byte) {
		pb := &ethpb.SparseMerkleTrie{}
		if err := proto.Unmarshal(b, pb); err != nil {
			return
		}
		smt, err := trie.CreateTrieFromProto(pb)
		if err != nil {
			return
		}
		if _, err := smt.HashTreeRoot(); err != nil {
			return
		}
	})
}

func FuzzSparseMerkleTrie_MerkleProof(f *testing.F) {
	h := hash.Hash([]byte("hi"))
	pb := &ethpb.SparseMerkleTrie{
		Layers: []*ethpb.TrieLayer{
			{
				Layer: [][]byte{h[:]},
			},
			{
				Layer: [][]byte{h[:]},
			},
			{
				Layer: [][]byte{h[:]},
			},
		},
		Depth: 2,
	}
	b, err := proto.Marshal(pb)
	require.NoError(f, err)
	f.Add(b, 0)

	f.Fuzz(func(t *testing.T, b []byte, i int) {
		pb := &ethpb.SparseMerkleTrie{}
		if err := proto.Unmarshal(b, pb); err != nil {
			return
		}
		smt, err := trie.CreateTrieFromProto(pb)
		if err != nil {
			return
		}
		if _, err := smt.MerkleProof(i); err != nil {
			return
		}
	})
}

func FuzzSparseMerkleTrie_Insert(f *testing.F) {
	h := hash.Hash([]byte("hi"))
	pb := &ethpb.SparseMerkleTrie{
		Layers: []*ethpb.TrieLayer{
			{
				Layer: [][]byte{h[:]},
			},
			{
				Layer: [][]byte{h[:]},
			},
			{
				Layer: [][]byte{h[:]},
			},
		},
		Depth: 2,
	}
	b, err := proto.Marshal(pb)
	require.NoError(f, err)
	f.Add(b, []byte{}, 0)

	f.Fuzz(func(t *testing.T, b, item []byte, i int) {
		pb := &ethpb.SparseMerkleTrie{}
		if err := proto.Unmarshal(b, pb); err != nil {
			return
		}
		smt, err := trie.CreateTrieFromProto(pb)
		if err != nil {
			return
		}
		if err := smt.Insert(item, i); err != nil {
			return
		}
	})
}

func FuzzSparseMerkleTrie_VerifyMerkleProofWithDepth(f *testing.F) {
	splitProofs := func(proofRaw []byte) [][]byte {
		var proofs [][]byte
		for i := 0; i < len(proofRaw); i += 32 {
			end := i + 32
			if end >= len(proofRaw) {
				end = len(proofRaw) - 1
			}
			proofs = append(proofs, proofRaw[i:end])
		}
		return proofs
	}

	items := [][]byte{
		[]byte("A"),
		[]byte("B"),
		[]byte("C"),
		[]byte("D"),
		[]byte("E"),
		[]byte("F"),
		[]byte("G"),
		[]byte("H"),
	}
	m, err := trie.GenerateTrieFromItems(items, params.BeaconConfig().DepositContractTreeDepth)
	require.NoError(f, err)
	proof, err := m.MerkleProof(0)
	require.NoError(f, err)
	require.Equal(f, int(params.BeaconConfig().DepositContractTreeDepth)+1, len(proof))
	root, err := m.HashTreeRoot()
	require.NoError(f, err)
	var proofRaw []byte
	for _, p := range proof {
		proofRaw = append(proofRaw, p...)
	}
	f.Add(root[:], items[0], uint64(0), proofRaw, params.BeaconConfig().DepositContractTreeDepth)

	f.Fuzz(func(t *testing.T, root, item []byte, merkleIndex uint64, proofRaw []byte, depth uint64) {
		trie.VerifyMerkleProofWithDepth(root, item, merkleIndex, splitProofs(proofRaw), depth)
	})
}
