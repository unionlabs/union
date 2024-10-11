package trie_test

import (
	"strconv"
	"testing"

	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/container/trie"
	contracts "github.com/prysmaticlabs/prysm/v4/contracts/deposit/mock"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestCreateTrieFromProto_Validation(t *testing.T) {
	h := hash.Hash([]byte("hi"))
	genValidLayers := func(num int) []*ethpb.TrieLayer {
		l := make([]*ethpb.TrieLayer, num)
		for i := 0; i < num; i++ {
			l[i] = &ethpb.TrieLayer{
				Layer: [][]byte{h[:]},
			}
		}
		return l
	}
	tests := []struct {
		trie      *ethpb.SparseMerkleTrie
		errString string
	}{
		{
			trie: &ethpb.SparseMerkleTrie{
				Layers: []*ethpb.TrieLayer{},
				Depth:  0,
			},
			errString: "no branches",
		},
		{
			trie: &ethpb.SparseMerkleTrie{
				Layers: []*ethpb.TrieLayer{
					{
						Layer: [][]byte{h[:]},
					},
					{
						Layer: [][]byte{h[:]},
					},
					{
						Layer: [][]byte{},
					},
				},
				Depth: 2,
			},
			errString: "invalid branches provided",
		},
		{
			trie: &ethpb.SparseMerkleTrie{
				Layers: genValidLayers(3),
				Depth:  12,
			},
			errString: "depth is greater than or equal to number of branches",
		},
		{
			trie: &ethpb.SparseMerkleTrie{
				Layers: genValidLayers(66),
				Depth:  63,
			},
			errString: "supported merkle trie depth exceeded (max uint64 depth is 63, " +
				"theoretical max sparse merkle trie depth is 64)",
		},
	}
	for _, tt := range tests {
		t.Run(tt.errString, func(t *testing.T) {
			_, err := trie.CreateTrieFromProto(tt.trie)
			require.ErrorContains(t, tt.errString, err)
		})
	}
}

func TestMarshalDepositWithProof(t *testing.T) {
	items := [][]byte{
		[]byte("A"),
		[]byte("BB"),
		[]byte("CCC"),
		[]byte("DDDD"),
		[]byte("EEEEE"),
		[]byte("FFFFFF"),
		[]byte("GGGGGGG"),
	}
	m, err := trie.GenerateTrieFromItems(items, params.BeaconConfig().DepositContractTreeDepth)
	require.NoError(t, err)
	proof, err := m.MerkleProof(2)
	require.NoError(t, err)
	require.Equal(t, len(proof), int(params.BeaconConfig().DepositContractTreeDepth)+1)
	someRoot := [32]byte{1, 2, 3, 4}
	someSig := [96]byte{1, 2, 3, 4}
	someKey := [fieldparams.BLSPubkeyLength]byte{1, 2, 3, 4}
	dep := &ethpb.Deposit{
		Proof: proof,
		Data: &ethpb.Deposit_Data{
			PublicKey:             someKey[:],
			WithdrawalCredentials: someRoot[:],
			Amount:                32,
			Signature:             someSig[:],
		},
	}
	enc, err := dep.MarshalSSZ()
	require.NoError(t, err)
	dec := &ethpb.Deposit{}
	require.NoError(t, dec.UnmarshalSSZ(enc))
	require.DeepEqual(t, dec, dep)
}

func TestMerkleTrie_MerkleProofOutOfRange(t *testing.T) {
	h := hash.Hash([]byte("hi"))
	m, err := trie.CreateTrieFromProto(&ethpb.SparseMerkleTrie{
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
	})
	require.NoError(t, err)
	if _, err := m.MerkleProof(6); err == nil {
		t.Error("Expected out of range failure, received nil", err)
	}
}

func TestMerkleTrieRoot_EmptyTrie(t *testing.T) {
	newTrie, err := trie.NewTrie(params.BeaconConfig().DepositContractTreeDepth)
	require.NoError(t, err)
	testAccount, err := contracts.Setup()
	require.NoError(t, err)

	depRoot, err := testAccount.Contract.GetDepositRoot(&bind.CallOpts{})
	require.NoError(t, err)
	root, err := newTrie.HashTreeRoot()
	require.NoError(t, err)
	require.DeepEqual(t, depRoot, root)
}

func TestGenerateTrieFromItems_NoItemsProvided(t *testing.T) {
	if _, err := trie.GenerateTrieFromItems(nil, params.BeaconConfig().DepositContractTreeDepth); err == nil {
		t.Error("Expected error when providing nil items received nil")
	}
}

func TestGenerateTrieFromItems_DepthSupport(t *testing.T) {
	items := [][]byte{
		[]byte("A"),
		[]byte("BB"),
		[]byte("CCC"),
		[]byte("DDDD"),
		[]byte("EEEEE"),
		[]byte("FFFFFF"),
		[]byte("GGGGGGG"),
	}
	// max supported depth is 62 (uint64 will overflow above this)
	// max theoretical depth is 64
	var max_supported_trie_depth uint64 = 62
	// Supported depth
	m1, err := trie.GenerateTrieFromItems(items, max_supported_trie_depth)
	require.NoError(t, err)
	proof, err := m1.MerkleProof(2)
	require.NoError(t, err)
	require.Equal(t, len(proof), int(max_supported_trie_depth)+1)
	// Unsupported depth
	_, err = trie.GenerateTrieFromItems(items, max_supported_trie_depth+1)
	errString := "supported merkle trie depth exceeded (max uint64 depth is 63, " +
		"theoretical max sparse merkle trie depth is 64)"
	require.ErrorContains(t, errString, err)
}

func TestMerkleTrie_VerifyMerkleProofWithDepth(t *testing.T) {
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
	require.NoError(t, err)
	proof, err := m.MerkleProof(0)
	require.NoError(t, err)
	require.Equal(t, int(params.BeaconConfig().DepositContractTreeDepth)+1, len(proof))
	root, err := m.HashTreeRoot()
	require.NoError(t, err)
	if ok := trie.VerifyMerkleProofWithDepth(root[:], items[0], 0, proof, params.BeaconConfig().DepositContractTreeDepth); !ok {
		t.Error("First Merkle proof did not verify")
	}
	proof, err = m.MerkleProof(3)
	require.NoError(t, err)
	require.Equal(t, true, trie.VerifyMerkleProofWithDepth(root[:], items[3], 3, proof, params.BeaconConfig().DepositContractTreeDepth))
	require.Equal(t, false, trie.VerifyMerkleProofWithDepth(root[:], []byte("buzz"), 3, proof, params.BeaconConfig().DepositContractTreeDepth))
}

func TestMerkleTrie_VerifyMerkleProof(t *testing.T) {
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
	require.NoError(t, err)
	proof, err := m.MerkleProof(0)
	require.NoError(t, err)
	require.Equal(t, int(params.BeaconConfig().DepositContractTreeDepth)+1, len(proof))
	root, err := m.HashTreeRoot()
	require.NoError(t, err)
	if ok := trie.VerifyMerkleProof(root[:], items[0], 0, proof); !ok {
		t.Error("First Merkle proof did not verify")
	}
	proof, err = m.MerkleProof(3)
	require.NoError(t, err)
	require.Equal(t, true, trie.VerifyMerkleProof(root[:], items[3], 3, proof))
	require.Equal(t, false, trie.VerifyMerkleProof(root[:], []byte("buzz"), 3, proof))
}

func TestMerkleTrie_NegativeIndexes(t *testing.T) {
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
	require.NoError(t, err)
	_, err = m.MerkleProof(-1)
	require.ErrorContains(t, "merkle index is negative", err)
	require.ErrorContains(t, "negative index provided", m.Insert([]byte{'J'}, -1))
}

func TestMerkleTrie_VerifyMerkleProof_TrieUpdated(t *testing.T) {
	items := [][]byte{
		{1},
		{2},
		{3},
		{4},
	}
	depth := params.BeaconConfig().DepositContractTreeDepth + 1
	m, err := trie.GenerateTrieFromItems(items, depth)
	require.NoError(t, err)
	proof, err := m.MerkleProof(0)
	require.NoError(t, err)
	root, err := m.HashTreeRoot()
	require.NoError(t, err)
	require.Equal(t, true, trie.VerifyMerkleProofWithDepth(root[:], items[0], 0, proof, depth))

	// Now we update the trie.
	assert.NoError(t, m.Insert([]byte{5}, 3))
	proof, err = m.MerkleProof(3)
	require.NoError(t, err)
	root, err = m.HashTreeRoot()
	require.NoError(t, err)
	if ok := trie.VerifyMerkleProofWithDepth(root[:], []byte{5}, 3, proof, depth); !ok {
		t.Error("Second Merkle proof did not verify")
	}
	if ok := trie.VerifyMerkleProofWithDepth(root[:], []byte{4}, 3, proof, depth); ok {
		t.Error("Old item should not verify")
	}

	// Now we update the trie at an index larger than the number of items.
	assert.NoError(t, m.Insert([]byte{6}, 15))
}

func TestRoundtripProto_OK(t *testing.T) {
	items := [][]byte{
		{1},
		{2},
		{3},
		{4},
	}
	m, err := trie.GenerateTrieFromItems(items, params.BeaconConfig().DepositContractTreeDepth+1)
	require.NoError(t, err)

	protoTrie := m.ToProto()
	depositRoot, err := m.HashTreeRoot()
	require.NoError(t, err)

	newTrie, err := trie.CreateTrieFromProto(protoTrie)
	require.NoError(t, err)
	root, err := newTrie.HashTreeRoot()
	require.NoError(t, err)
	require.DeepEqual(t, depositRoot, root)

}

func TestCopy_OK(t *testing.T) {
	items := [][]byte{
		{1},
		{2},
		{3},
		{4},
	}
	source, err := trie.GenerateTrieFromItems(items, params.BeaconConfig().DepositContractTreeDepth+1)
	require.NoError(t, err)
	copiedTrie := source.Copy()

	if copiedTrie == source {
		t.Errorf("Original trie returned.")
	}
	a, err := copiedTrie.HashTreeRoot()
	require.NoError(t, err)
	b, err := source.HashTreeRoot()
	require.NoError(t, err)
	require.DeepEqual(t, a, b)
}

func BenchmarkGenerateTrieFromItems(b *testing.B) {
	items := [][]byte{
		[]byte("A"),
		[]byte("BB"),
		[]byte("CCC"),
		[]byte("DDDD"),
		[]byte("EEEEE"),
		[]byte("FFFFFF"),
		[]byte("GGGGGGG"),
	}
	for i := 0; i < b.N; i++ {
		_, err := trie.GenerateTrieFromItems(items, params.BeaconConfig().DepositContractTreeDepth)
		require.NoError(b, err, "Could not generate Merkle trie from items")
	}
}

func BenchmarkInsertTrie_Optimized(b *testing.B) {
	b.StopTimer()
	numDeposits := 16000
	items := make([][]byte, numDeposits)
	for i := 0; i < numDeposits; i++ {
		someRoot := bytesutil.ToBytes32([]byte(strconv.Itoa(i)))
		items[i] = someRoot[:]
	}
	tr, err := trie.GenerateTrieFromItems(items, params.BeaconConfig().DepositContractTreeDepth)
	require.NoError(b, err)

	someItem := bytesutil.ToBytes32([]byte("hello-world"))
	b.StartTimer()
	for i := 0; i < b.N; i++ {
		require.NoError(b, tr.Insert(someItem[:], i%numDeposits))
	}
}

func BenchmarkGenerateProof(b *testing.B) {
	b.StopTimer()
	items := [][]byte{
		[]byte("A"),
		[]byte("BB"),
		[]byte("CCC"),
		[]byte("DDDD"),
		[]byte("EEEEE"),
		[]byte("FFFFFF"),
		[]byte("GGGGGGG"),
	}
	normalTrie, err := trie.GenerateTrieFromItems(items, params.BeaconConfig().DepositContractTreeDepth)
	require.NoError(b, err)

	b.StartTimer()
	for i := 0; i < b.N; i++ {
		_, err := normalTrie.MerkleProof(3)
		require.NoError(b, err)
	}
}

func BenchmarkVerifyMerkleProofWithDepth(b *testing.B) {
	b.StopTimer()
	items := [][]byte{
		[]byte("A"),
		[]byte("BB"),
		[]byte("CCC"),
		[]byte("DDDD"),
		[]byte("EEEEE"),
		[]byte("FFFFFF"),
		[]byte("GGGGGGG"),
	}
	m, err := trie.GenerateTrieFromItems(items, params.BeaconConfig().DepositContractTreeDepth)
	require.NoError(b, err)
	proof, err := m.MerkleProof(2)
	require.NoError(b, err)

	root, err := m.HashTreeRoot()
	require.NoError(b, err)
	b.StartTimer()
	for i := 0; i < b.N; i++ {
		if ok := trie.VerifyMerkleProofWithDepth(root[:], items[2], 2, proof, params.BeaconConfig().DepositContractTreeDepth); !ok {
			b.Error("Merkle proof did not verify")
		}
	}
}
