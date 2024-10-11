package sha256

import (
	"crypto/sha256"
	"hash"
)

const (
	Size      = sha256.Size
	BlockSize = sha256.BlockSize
)

// New returns a new hash.Hash.
func New() hash.Hash {
	return sha256.New()
}

// Sum returns the SHA256 of the bz.
func Sum(bz []byte) []byte {
	h := sha256.Sum256(bz)
	return h[:]
}

// SumMany takes at least 1 byteslice along with a variadic
// number of other byteslices and produces the SHA256 sum from
// hashing them as if they were 1 joined slice.
func SumMany(data []byte, rest ...[]byte) []byte {
	h := sha256.New()
	h.Write(data)
	for _, data := range rest {
		h.Write(data)
	}
	return h.Sum(nil)
}

// -------------------------------------------------------------

const (
	TruncatedSize = 20
)

type sha256trunc struct {
	sha256 hash.Hash
}

func (h sha256trunc) Write(p []byte) (n int, err error) {
	return h.sha256.Write(p)
}

func (h sha256trunc) Sum(b []byte) []byte {
	shasum := h.sha256.Sum(b)
	return shasum[:TruncatedSize]
}

func (h sha256trunc) Reset() {
	h.sha256.Reset()
}

func (sha256trunc) Size() int {
	return TruncatedSize
}

func (h sha256trunc) BlockSize() int {
	return h.sha256.BlockSize()
}

// NewTruncated returns a new hash.Hash.
func NewTruncated() hash.Hash {
	return sha256trunc{
		sha256: sha256.New(),
	}
}

// SumTruncated returns the first 20 bytes of SHA256 of the bz.
func SumTruncated(bz []byte) []byte {
	h := sha256.Sum256(bz)
	return h[:TruncatedSize]
}
