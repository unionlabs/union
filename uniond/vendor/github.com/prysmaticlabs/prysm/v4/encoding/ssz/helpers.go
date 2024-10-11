// Package ssz defines HashTreeRoot utility functions.
package ssz

import (
	"bytes"
	"encoding/binary"

	"github.com/minio/sha256-simd"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
)

const bytesPerChunk = 32

// BitlistRoot returns the mix in length of a bitwise Merkleized bitfield.
func BitlistRoot(bfield bitfield.Bitfield, maxCapacity uint64) ([32]byte, error) {
	limit := (maxCapacity + 255) / 256
	if bfield == nil || bfield.Len() == 0 {
		length := make([]byte, 32)
		root, err := BitwiseMerkleize([][32]byte{}, 0, limit)
		if err != nil {
			return [32]byte{}, err
		}
		return MixInLength(root, length), nil
	}
	chunks, err := PackByChunk([][]byte{bfield.Bytes()})
	if err != nil {
		return [32]byte{}, err
	}
	buf := new(bytes.Buffer)
	if err := binary.Write(buf, binary.LittleEndian, bfield.Len()); err != nil {
		return [32]byte{}, err
	}
	output := make([]byte, 32)
	copy(output, buf.Bytes())
	root, err := BitwiseMerkleize(chunks, uint64(len(chunks)), limit)
	if err != nil {
		return [32]byte{}, err
	}
	return MixInLength(root, output), nil
}

// BitwiseMerkleize - given ordered BYTES_PER_CHUNK-byte chunks, if necessary utilize
// zero chunks so that the number of chunks is a power of two, Merkleize the chunks,
// and return the root.
// Note that merkleize on a single chunk is simply that chunk, i.e. the identity
// when the number of chunks is one.
func BitwiseMerkleize(chunks [][32]byte, count, limit uint64) ([32]byte, error) {
	if count > limit {
		return [32]byte{}, errors.New("merkleizing list that is too large, over limit")
	}
	return MerkleizeVector(chunks, limit), nil
}

// PackByChunk a given byte array's final chunk with zeroes if needed.
func PackByChunk(serializedItems [][]byte) ([][bytesPerChunk]byte, error) {
	var emptyChunk [bytesPerChunk]byte
	// If there are no items, we return an empty chunk.
	if len(serializedItems) == 0 {
		return [][bytesPerChunk]byte{emptyChunk}, nil
	} else if len(serializedItems[0]) == bytesPerChunk {
		// If each item has exactly BYTES_PER_CHUNK length, we return the list of serialized items.
		chunks := make([][bytesPerChunk]byte, 0, len(serializedItems))
		for _, c := range serializedItems {
			chunks = append(chunks, bytesutil.ToBytes32(c))
		}
		return chunks, nil
	}
	// We flatten the list in order to pack its items into byte chunks correctly.
	orderedItems := make([]byte, 0, len(serializedItems)*len(serializedItems[0]))
	for _, item := range serializedItems {
		orderedItems = append(orderedItems, item...)
	}
	// If all our serialized item slices are length zero, we
	// exit early.
	if len(orderedItems) == 0 {
		return [][bytesPerChunk]byte{emptyChunk}, nil
	}
	numItems := len(orderedItems)
	var chunks [][bytesPerChunk]byte
	for i := 0; i < numItems; i += bytesPerChunk {
		j := i + bytesPerChunk
		// We create our upper bound index of the chunk, if it is greater than numItems,
		// we set it as numItems itself.
		if j > numItems {
			j = numItems
		}
		// We create chunks from the list of items based on the
		// indices determined above.
		// Right-pad the last chunk with zero bytes if it does not
		// have length bytesPerChunk from the helper.
		// The ToBytes32 helper allocates a 32-byte array, before
		// copying the ordered items in. This ensures that even if
		// the last chunk is != 32 in length, we will right-pad it with
		// zero bytes.
		chunks = append(chunks, bytesutil.ToBytes32(orderedItems[i:j]))
	}

	return chunks, nil
}

// MixInLength appends hash length to root
func MixInLength(root [32]byte, length []byte) [32]byte {
	var hash [32]byte
	h := sha256.New()
	h.Write(root[:])
	h.Write(length)
	// The hash interface never returns an error, for that reason
	// we are not handling the error below. For reference, it is
	// stated here https://golang.org/pkg/hash/#Hash
	// #nosec G104
	h.Sum(hash[:0])
	return hash
}
