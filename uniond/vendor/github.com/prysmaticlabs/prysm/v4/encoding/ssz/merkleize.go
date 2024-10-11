package ssz

import (
	"encoding/binary"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/gohashtree"
	"github.com/prysmaticlabs/prysm/v4/container/trie"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash/htr"
)

var errInvalidNilSlice = errors.New("invalid empty slice")

const (
	mask0 = ^uint64((1 << (1 << iota)) - 1)
	mask1
	mask2
	mask3
	mask4
	mask5
)

const (
	bit0 = uint8(1 << iota)
	bit1
	bit2
	bit3
	bit4
	bit5
)

// Depth retrieves the appropriate depth for the provided trie size.
func Depth(v uint64) (out uint8) {
	// bitmagic: binary search through a uint32, offset down by 1 to not round powers of 2 up.
	// Then adding 1 to it to not get the index of the first bit, but the length of the bits (depth of tree)
	// Zero is a special case, it has a 0 depth.
	// Example:
	//  (in out): (0 0), (1 0), (2 1), (3 2), (4 2), (5 3), (6 3), (7 3), (8 3), (9 4)
	if v <= 1 {
		return 0
	}
	v--
	if v&mask5 != 0 {
		v >>= bit5
		out |= bit5
	}
	if v&mask4 != 0 {
		v >>= bit4
		out |= bit4
	}
	if v&mask3 != 0 {
		v >>= bit3
		out |= bit3
	}
	if v&mask2 != 0 {
		v >>= bit2
		out |= bit2
	}
	if v&mask1 != 0 {
		v >>= bit1
		out |= bit1
	}
	if v&mask0 != 0 {
		out |= bit0
	}
	out++
	return
}

// Merkleize with log(N) space allocation
func Merkleize(hasher Hasher, count, limit uint64, leaf func(i uint64) []byte) (out [32]byte) {
	if count > limit {
		panic("merkleizing list that is too large, over limit")
	}
	if limit == 0 {
		return
	}
	if limit == 1 {
		if count == 1 {
			copy(out[:], leaf(0))
		}
		return
	}
	depth := Depth(count)
	limitDepth := Depth(limit)
	tmp := make([][32]byte, limitDepth+1)

	j := uint8(0)
	var hArr [32]byte
	h := hArr[:]

	merge := func(i uint64) {
		// merge back up from bottom to top, as far as we can
		for j = 0; ; j++ {
			// stop merging when we are in the left side of the next combi
			if i&(uint64(1)<<j) == 0 {
				// if we are at the count, we want to merge in zero-hashes for padding
				if i == count && j < depth {
					v := hasher.Combi(hArr, trie.ZeroHashes[j])
					copy(h, v[:])
				} else {
					break
				}
			} else {
				// keep merging up if we are the right side
				v := hasher.Combi(tmp[j], hArr)
				copy(h, v[:])
			}
		}
		// store the merge result (may be no merge, i.e. bottom leaf node)
		copy(tmp[j][:], h)
	}

	// merge in leaf by leaf.
	for i := uint64(0); i < count; i++ {
		copy(h, leaf(i))
		merge(i)
	}

	// complement with 0 if empty, or if not the right power of 2
	if (uint64(1) << depth) != count {
		copy(h, trie.ZeroHashes[0][:])
		merge(count)
	}

	// the next power of two may be smaller than the ultimate virtual size,
	// complement with zero-hashes at each depth.
	for j := depth; j < limitDepth; j++ {
		tmp[j+1] = hasher.Combi(tmp[j], trie.ZeroHashes[j])
	}

	return tmp[limitDepth]
}

// MerkleizeVector uses our optimized routine to hash a list of 32-byte
// elements.
func MerkleizeVector(elements [][32]byte, length uint64) [32]byte {
	depth := Depth(length)
	// Return zerohash at depth
	if len(elements) == 0 {
		return trie.ZeroHashes[depth]
	}
	for i := uint8(0); i < depth; i++ {
		layerLen := len(elements)
		oddNodeLength := layerLen%2 == 1
		if oddNodeLength {
			zerohash := trie.ZeroHashes[i]
			elements = append(elements, zerohash)
		}
		elements = htr.VectorizedSha256(elements)
	}
	return elements[0]
}

// Hashable is an interface representing objects that implement HashTreeRoot()
type Hashable interface {
	HashTreeRoot() ([32]byte, error)
}

// MerkleizeVectorSSZ hashes each element in the list and then returns the HTR
// of the corresponding list of roots
func MerkleizeVectorSSZ[T Hashable](elements []T, length uint64) ([32]byte, error) {
	roots := make([][32]byte, len(elements))
	var err error
	for i, el := range elements {
		roots[i], err = el.HashTreeRoot()
		if err != nil {
			return [32]byte{}, err
		}
	}
	return MerkleizeVector(roots, length), nil
}

// MerkleizeListSSZ hashes each element in the list and then returns the HTR of
// the list of corresponding roots, with the length mixed in.
func MerkleizeListSSZ[T Hashable](elements []T, limit uint64) ([32]byte, error) {
	body, err := MerkleizeVectorSSZ(elements, limit)
	if err != nil {
		return [32]byte{}, err
	}
	chunks := make([][32]byte, 2)
	chunks[0] = body
	binary.LittleEndian.PutUint64(chunks[1][:], uint64(len(elements)))
	if err := gohashtree.Hash(chunks, chunks); err != nil {
		return [32]byte{}, err
	}
	return chunks[0], err
}

// MerkleizeByteSliceSSZ hashes a byteslice by chunkifying it and returning the
// corresponding HTR as if it were a fixed vector of bytes of the given length.
func MerkleizeByteSliceSSZ(input []byte) ([32]byte, error) {
	numChunks := (len(input) + 31) / 32
	if numChunks == 0 {
		return [32]byte{}, errInvalidNilSlice
	}
	chunks := make([][32]byte, numChunks)
	for i := range chunks {
		copy(chunks[i][:], input[32*i:])
	}
	return MerkleizeVector(chunks, uint64(numChunks)), nil
}
