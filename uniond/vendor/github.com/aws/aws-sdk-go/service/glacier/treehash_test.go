package glacier_test

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"io"
	"testing"

	"github.com/aws/aws-sdk-go/service/glacier"
)

func ExampleComputeHashes() {
	r := testCreateReader()

	h := glacier.ComputeHashes(r)
	n, _ := r.Seek(0, 1) // Check position after checksumming

	fmt.Printf("linear: %x\n", h.LinearHash)
	fmt.Printf("tree: %x\n", h.TreeHash)
	fmt.Printf("pos: %d\n", n)

	// Output:
	// linear: 68aff0c5a91aa0491752bfb96e3fef33eb74953804f6a2f7b708d5bcefa8ff6b
	// tree: 154e26c78fd74d0c2c9b3cc4644191619dc4f2cd539ae2a74d5fd07957a3ee6a
	// pos: 0
}

func testCreateReader() io.ReadSeeker {
	buf := make([]byte, 5767168) // 5.5MB buffer
	for i := range buf {
		buf[i] = '0' // Fill with zero characters
	}

	return bytes.NewReader(buf)
}

func ExampleComputeTreeHash() {
	r := testCreateReader()

	const chunkSize = 1024 * 1024 // 1MB
	buf := make([]byte, chunkSize)
	hashes := [][]byte{}

	for {
		// Reach 1MB chunks from reader to generate hashes from
		n, err := io.ReadAtLeast(r, buf, chunkSize)
		if n == 0 {
			break
		}

		tmpHash := sha256.Sum256(buf[:n])
		hashes = append(hashes, tmpHash[:])
		if err != nil {
			break // last chunk
		}
	}

	treeHash := glacier.ComputeTreeHash(hashes)
	fmt.Printf("TreeHash: %x\n", treeHash)

	// Output:
	// TreeHash: 154e26c78fd74d0c2c9b3cc4644191619dc4f2cd539ae2a74d5fd07957a3ee6a
}

func TestComputeHashes(t *testing.T) {

	t.Run("no hash", func(t *testing.T) {
		var hashes [][]byte
		treeHash := glacier.ComputeTreeHash(hashes)
		if treeHash != nil {
			t.Fatalf("expected []byte(nil), got %v", treeHash)
		}
	})

	t.Run("one hash", func(t *testing.T) {
		hash := sha256.Sum256([]byte("hash"))
		treeHash := glacier.ComputeTreeHash([][]byte{hash[:]})

		expected, actual := hex.EncodeToString(hash[:]), hex.EncodeToString(treeHash)
		if expected != actual {
			t.Fatalf("expected %v, got %v", expected, actual)
		}
	})

	t.Run("even hashes", func(t *testing.T) {
		h1 := sha256.Sum256([]byte("h1"))
		h2 := sha256.Sum256([]byte("h2"))

		hash := sha256.Sum256(append(h1[:], h2[:]...))
		expected := hex.EncodeToString(hash[:])

		treeHash := glacier.ComputeTreeHash([][]byte{h1[:], h2[:]})
		actual := hex.EncodeToString(treeHash)

		if expected != actual {
			t.Fatalf("expected %v, got %v", expected, actual)
		}
	})

	t.Run("odd hashes", func(t *testing.T) {
		h1 := sha256.Sum256([]byte("h1"))
		h2 := sha256.Sum256([]byte("h2"))
		h3 := sha256.Sum256([]byte("h3"))

		h12 := sha256.Sum256(append(h1[:], h2[:]...))
		hash := sha256.Sum256(append(h12[:], h3[:]...))
		expected := hex.EncodeToString(hash[:])

		treeHash := glacier.ComputeTreeHash([][]byte{h1[:], h2[:], h3[:]})
		actual := hex.EncodeToString(treeHash)

		if expected != actual {
			t.Fatalf("expected %v, got %v", expected, actual)
		}
	})

}
