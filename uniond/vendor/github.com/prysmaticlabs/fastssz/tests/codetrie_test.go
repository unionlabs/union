package tests

import (
	"bytes"
	"encoding/hex"
	"math/rand"
	"testing"
	"time"

	ssz "github.com/prysmaticlabs/fastssz"

	"github.com/minio/sha256-simd"
)

func TestVerifyMetadataProof(t *testing.T) {
	testCases := []struct {
		root  string
		proof []string
		leaf  string
		index int
		valid bool
	}{
		{
			root: "2a23ef2b7a7221eaac2ffb3842a506a981c009ca6c2fcbf20adbc595e56f1a93",
			proof: []string{
				"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
				"f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b",
			},
			leaf:  "0100000000000000000000000000000000000000000000000000000000000000",
			index: 4,
			valid: true,
		},
		{
			root: "2a23ef2b7a7221eaac2ffb3842a506a981c009ca6c2fcbf20adbc595e56f1a93",
			proof: []string{
				"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
				"f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b",
			},
			leaf:  "0100000000000000000000000000000000000000000000000000000000000000",
			index: 6,
			valid: false,
		},
		{
			root: "2a23ef2b7a7221eaac2ffb3842a506a981c009ca6c2fcbf20adbc595e56f1a93",
			proof: []string{
				"0100000000000000000000000000000000000000000000000000000000000000",
				"f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b",
			},
			leaf:  "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
			index: 5,
			valid: true,
		},
	}

	for _, c := range testCases {
		// Decode values from string to []byte
		root, err := hex.DecodeString(c.root)
		if err != nil {
			t.Errorf("Failed to decode root: %s\n", c.root)
		}
		hashes := make([][]byte, len(c.proof))
		for i, p := range c.proof {
			b, err := hex.DecodeString(p)
			if err != nil {
				t.Errorf("Failed to decode proof element: %s\n", p)
			}
			hashes[i] = b
		}
		leaf, err := hex.DecodeString(c.leaf)
		if err != nil {
			t.Errorf("Failed to decode leaf: %s\n", c.leaf)
		}

		// Verify proof
		proof := &ssz.Proof{Hashes: hashes, Leaf: leaf, Index: c.index}
		ok, err := ssz.VerifyProof(root, proof)
		if err != nil {
			t.Errorf("Failed to verify proof: %v\n", err)
		}
		if ok != c.valid {
			t.Errorf("Incorrect proof verification: expected %v, got %v\n", c.valid, ok)
		}
	}
}

func TestVerifyCodeTrieProof(t *testing.T) {
	testCases := []struct {
		root  string
		proof []string
		leaf  string
		index int
		valid bool
	}{
		{
			root: "f1824b0084956084591ff4c91c11bcc94a40be82da280e5171932b967dd146e9",
			proof: []string{
				"35210d64853aee79d03f30cf0f29c1398706cbbcacaf05ab9524f00070aec91e",
				"f38a181470ef1eee90a29f0af0a9dba6b7e5d48af3c93c29b4f91fa11b777582",
			},
			leaf:  "0100000000000000000000000000000000000000000000000000000000000000",
			index: 7,
			valid: true,
		},
		{
			root: "f1824b0084956084591ff4c91c11bcc94a40be82da280e5171932b967dd146e9",
			proof: []string{
				"0000000000000000000000000000000000000000000000000000000000000000",
				"0000000000000000000000000000000000000000000000000000000000000000",
				"f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b",
				"0100000000000000000000000000000000000000000000000000000000000000",
				"f38a181470ef1eee90a29f0af0a9dba6b7e5d48af3c93c29b4f91fa11b777582",
			},
			leaf:  "6001000000000000000000000000000000000000000000000000000000000000",
			index: 49,
			valid: true,
		},
	}

	for _, c := range testCases {
		// Decode values from string to []byte
		root, err := hex.DecodeString(c.root)
		if err != nil {
			t.Errorf("Failed to decode root: %s\n", c.root)
		}
		hashes := make([][]byte, len(c.proof))
		for i, p := range c.proof {
			b, err := hex.DecodeString(p)
			if err != nil {
				t.Errorf("Failed to decode proof element: %s\n", p)
			}
			hashes[i] = b
		}
		leaf, err := hex.DecodeString(c.leaf)
		if err != nil {
			t.Errorf("Failed to decode leaf: %s\n", c.leaf)
		}

		// Verify proof
		proof := &ssz.Proof{Hashes: hashes, Leaf: leaf, Index: c.index}
		ok, err := ssz.VerifyProof(root, proof)
		if err != nil {
			t.Errorf("Failed to verify proof: %v\n", err)
		}
		if ok != c.valid {
			t.Errorf("Incorrect proof verification: expected %v, got %v\n", c.valid, ok)
		}
	}
}

func TestVerifyCodeTrieMultiProof(t *testing.T) {
	testCases := []struct {
		root    string
		proof   []string
		leaves  []string
		indices []int
		valid   bool
	}{
		{
			root: "f1824b0084956084591ff4c91c11bcc94a40be82da280e5171932b967dd146e9",
			proof: []string{
				"0000000000000000000000000000000000000000000000000000000000000000",
				"0000000000000000000000000000000000000000000000000000000000000000",
				"f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b",
				"0000000000000000000000000000000000000000000000000000000000000000",
				"0100000000000000000000000000000000000000000000000000000000000000",
				"f58f76419d9235451a8290a88ba380d852350a1843f8f26b8257a421633042b4",
			},
			leaves: []string{
				"0200000000000000000000000000000000000000000000000000000000000000",
				"6001000000000000000000000000000000000000000000000000000000000000",
			},
			indices: []int{10, 49},
			valid:   true,
		},
	}

	for _, c := range testCases {
		// Decode values from string to []byte
		root, err := hex.DecodeString(c.root)
		if err != nil {
			t.Errorf("Failed to decode root: %s\n", c.root)
		}
		proof := make([][]byte, len(c.proof))
		for i, p := range c.proof {
			b, err := hex.DecodeString(p)
			if err != nil {
				t.Errorf("Failed to decode proof element: %s\n", p)
			}
			proof[i] = b
		}
		leaves := make([][]byte, len(c.leaves))
		for i, l := range c.leaves {
			b, err := hex.DecodeString(l)
			if err != nil {
				t.Errorf("Failed to decode leaf: %s\n", l)
			}
			leaves[i] = b
		}

		// Verify proof
		ok, err := ssz.VerifyMultiproof(root, proof, leaves, c.indices)
		if err != nil {
			t.Errorf("Failed to verify proof: %v\n", err)
		}
		if ok != c.valid {
			t.Errorf("Incorrect proof verification: expected %v, got %v\n", c.valid, ok)
		}
	}
}

func TestMetadataTree(t *testing.T) {
	code := []byte{0x60, 0x01}
	codeHash := sha256.Sum256(code)

	codePadded := make([]byte, 32)
	copy(codePadded[:2], code[:])

	md := &Metadata{Version: 1, CodeLength: uint16(len(code)), CodeHash: codeHash[:]}
	mdRoot, err := md.HashTreeRoot()
	if err != nil {
		t.Errorf("failed to hash metadata tree root: %v\n", err)
	}

	mdTree, err := md.GetTree()
	if err != nil {
		t.Errorf("Failed to construct tree for metadata: %v\n", err)
	}

	r := mdTree.Hash()
	if !bytes.Equal(r, mdRoot[:]) {
		t.Errorf("Computed incorrect root. Expected %s, got %s\n", hex.EncodeToString(mdRoot[:]), hex.EncodeToString(r))
	}
}

func TestChunkTree(t *testing.T) {
	code := []byte{0x60, 0x01}
	codePadded := make([]byte, 32)
	copy(codePadded[:2], code[:])
	chunk := &Chunk{FIO: 0, Code: codePadded[:]}
	chunkRoot, err := chunk.HashTreeRoot()
	if err != nil {
		t.Errorf("Failed to hash chunk to root: %v\n", err)
	}

	tree, err := chunk.GetTree()
	if err != nil {
		t.Errorf("Failed to construct tree for chunk: %v\n", err)
	}

	r := tree.Hash()
	if !bytes.Equal(r, chunkRoot[:]) {
		t.Errorf("Computed incorrect root. Expected %s, got %s\n", hex.EncodeToString(chunkRoot[:]), hex.EncodeToString(r))
	}
}

func TestSmallCodeTrieTree(t *testing.T) {
	code := []byte{0x60, 0x01}
	codeHash := sha256.Sum256(code)

	codePadded := make([]byte, 32)
	copy(codePadded[:2], code[:])

	md := &Metadata{Version: 1, CodeLength: uint16(len(code)), CodeHash: codeHash[:]}
	chunks := []*Chunk{
		{FIO: 0, Code: codePadded[:]},
	}
	codeTrie := &CodeTrieSmall{Metadata: md, Chunks: chunks}
	codeRoot, err := codeTrie.HashTreeRoot()
	if err != nil {
		t.Errorf("failed to hash tree root: %v\n", err)
	}

	tree, err := codeTrie.GetTree()
	if err != nil {
		t.Errorf("Failed to construct tree for codeTrie: %v\n", err)
	}

	r := tree.Hash()
	if !bytes.Equal(r, codeRoot[:]) {
		t.Errorf("Computed incorrect root. Expected %s, got %s\n", hex.EncodeToString(codeRoot[:]), hex.EncodeToString(r))
	}
}

func TestProveSmallCodeTrie(t *testing.T) {
	expectedProofHex := []string{
		"0000000000000000000000000000000000000000000000000000000000000000",
		"0000000000000000000000000000000000000000000000000000000000000000",
		"f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b",
		"0100000000000000000000000000000000000000000000000000000000000000",
		"f38a181470ef1eee90a29f0af0a9dba6b7e5d48af3c93c29b4f91fa11b777582",
	}
	expectedProof, err := parseStringSlice(expectedProofHex)
	if err != nil {
		t.Errorf("Failed to decode expected proof: %v\n", err)
	}

	code := []byte{0x60, 0x01}
	codeHash := sha256.Sum256(code)

	codePadded := make([]byte, 32)
	copy(codePadded[:2], code[:])

	md := &Metadata{Version: 1, CodeLength: uint16(len(code)), CodeHash: codeHash[:]}
	chunks := []*Chunk{
		{FIO: 0, Code: codePadded[:]},
	}
	codeTrie := &CodeTrieSmall{Metadata: md, Chunks: chunks}

	tree, err := codeTrie.GetTree()
	if err != nil {
		t.Errorf("Failed to construct tree for codeTrie: %v\n", err)
	}

	proof, err := tree.Prove(49)
	if err != nil {
		t.Errorf("Failed to generate proof for codeTrie: %v\n", err)
	}

	if proof.Index != 49 {
		t.Errorf("Proof has invalid index\n")
	}
	if !bytes.Equal(proof.Leaf, codePadded) {
		t.Errorf("Proof has invalid leaf\n")
	}
	if len(proof.Hashes) != len(expectedProof) {
		t.Errorf("Generated proof has invalid length\n")
	}

	for i, p := range proof.Hashes {
		if !bytes.Equal(p, expectedProof[i]) {
			t.Errorf("Proof element mismatch. Expected %s, got %s\n", hex.EncodeToString(expectedProof[i]), hex.EncodeToString(p))
		}
	}

	root := tree.Hash()
	ok, err := ssz.VerifyProof(root, proof)
	if err != nil {
		t.Error(err)
	}
	if !ok {
		t.Errorf("Could not verify generated proof")
	}
}

func TestProveMultiSmallCodeTrie(t *testing.T) {
	expectedProofHex := []string{
		"0000000000000000000000000000000000000000000000000000000000000000",
		"0000000000000000000000000000000000000000000000000000000000000000",
		"f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b",
		"0000000000000000000000000000000000000000000000000000000000000000",
		"0100000000000000000000000000000000000000000000000000000000000000",
		"f58f76419d9235451a8290a88ba380d852350a1843f8f26b8257a421633042b4",
	}
	expectedCProofHex := []string{
		"",
		"",
		"",
		"",
		"0100000000000000000000000000000000000000000000000000000000000000",
		"f58f76419d9235451a8290a88ba380d852350a1843f8f26b8257a421633042b4",
	}
	expectedProof, err := parseStringSlice(expectedProofHex)
	if err != nil {
		t.Errorf("Failed to decode expected proof: %v\n", err)
	}
	expectedCProof, err := parseStringSlice(expectedCProofHex)
	if err != nil {
		t.Errorf("Failed to decode expected compressed proof: %v\n", err)
	}

	code := []byte{0x60, 0x01}
	codeHash := sha256.Sum256(code)

	codePadded := make([]byte, 32)
	copy(codePadded[:2], code[:])

	md := &Metadata{Version: 1, CodeLength: uint16(len(code)), CodeHash: codeHash[:]}
	chunks := []*Chunk{
		{FIO: 0, Code: codePadded[:]},
	}
	codeTrie := &CodeTrieSmall{Metadata: md, Chunks: chunks}

	tree, err := codeTrie.GetTree()
	if err != nil {
		t.Errorf("Failed to construct tree for codeTrie: %v\n", err)
	}

	proof, err := tree.ProveMulti([]int{10, 49})
	if err != nil {
		t.Errorf("Failed to generate proof for codeTrie: %v\n", err)
	}

	if len(proof.Hashes) != len(expectedProof) {
		t.Errorf("Generated proof has invalid length\n")
	}

	for i, p := range proof.Hashes {
		if !bytes.Equal(p, expectedProof[i]) {
			t.Errorf("Proof element mismatch. Expected %s, got %s\n", hex.EncodeToString(expectedProof[i]), hex.EncodeToString(p))
		}
	}

	cproof := proof.Compress()
	if len(cproof.Hashes) != len(expectedCProof) {
		t.Errorf("Generated compressed proof has invalid length\n")
	}

	for i, p := range cproof.Hashes {
		e := expectedCProof[i]
		if (p == nil && e != nil) || (p != nil && e == nil) {
			t.Errorf("Proof element at pos %d was unexpectedly empty\n", i)
		}
		if !bytes.Equal(p, e) {
			t.Errorf("Proof element mismatch. Expected %s, got %s\n", hex.EncodeToString(e), hex.EncodeToString(p))
		}
	}

	// Test uncompression
	uncompressed := cproof.Decompress()
	if len(uncompressed.Hashes) != len(expectedProof) {
		t.Errorf("Uncompressed proof has invalid length. Expected %d, got %d\n", len(expectedProof), len(uncompressed.Hashes))
	}

	for i, p := range uncompressed.Hashes {
		e := expectedProof[i]
		if !bytes.Equal(p, e) {
			t.Errorf("Uncompressed proof element mismatch. Expected %s, got %s\n", hex.EncodeToString(e), hex.EncodeToString(p))
		}
	}
}

func BenchmarkHashTreeRootVsNode(b *testing.B) {
	rand.Seed(time.Now().UnixNano())
	codeSize := 24 * 1024
	code := make([]byte, codeSize) // 24Kb
	rand.Read(code)
	codeHash := sha256.Sum256(code)

	md := &Metadata{Version: 1, CodeLength: uint16(codeSize), CodeHash: codeHash[:]}
	chunks := make([]*Chunk, codeSize/32)
	for i := 0; i < len(chunks); i++ {
		chunks[i] = &Chunk{FIO: uint8(i % 256), Code: code[i*32 : (i+1)*32]}
	}

	codeTrie := &CodeTrieBig{Metadata: md, Chunks: chunks}

	b.Run("HashTreeRoot", func(b *testing.B) {
		b.ResetTimer()
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			codeTrie.HashTreeRoot()
		}
	})
	b.Run("NodeHash", func(b *testing.B) {
		b.ResetTimer()
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			tree, err := codeTrie.GetTree()
			if err != nil {
				b.Errorf("Failed to construct tree for codeTrie: %v\n", err)
			}

			tree.Hash()
		}
	})
}

func parseStringSlice(slice []string) ([][]byte, error) {
	res := make([][]byte, len(slice))
	for i, s := range slice {
		if len(s) == 0 {
			res[i] = nil
			continue
		}

		b, err := hex.DecodeString(s)
		if err != nil {
			return nil, err
		}
		res[i] = b
	}
	return res, nil
}
