package stateutil

import (
	"bytes"
	"encoding/binary"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/container/trie"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash/htr"
	"github.com/prysmaticlabs/prysm/v4/encoding/ssz"
	"github.com/prysmaticlabs/prysm/v4/math"
)

// ReturnTrieLayer returns the representation of a merkle trie when
// provided with the elements of a fixed sized trie and the corresponding depth of
// it.
func ReturnTrieLayer(elements [][32]byte, length uint64) ([][]*[32]byte, error) {
	leaves := elements

	if len(leaves) == 1 {
		return [][]*[32]byte{{&leaves[0]}}, nil
	}
	hashLayer := leaves
	layers := make([][][32]byte, ssz.Depth(length)+1)
	layers[0] = hashLayer
	var err error
	layers, _, err = MerkleizeTrieLeaves(layers, hashLayer)
	if err != nil {
		return nil, err
	}
	refLayers := make([][]*[32]byte, len(layers))
	for i, val := range layers {
		refLayers[i] = make([]*[32]byte, len(val))
		for j, innerVal := range val {
			newVal := innerVal
			refLayers[i][j] = &newVal
		}
	}
	return refLayers, nil
}

// ReturnTrieLayerVariable returns the representation of a merkle trie when
// provided with the elements of a variable sized trie and the corresponding depth of
// it.
func ReturnTrieLayerVariable(elements [][32]byte, length uint64) [][]*[32]byte {
	depth := ssz.Depth(length)
	layers := make([][]*[32]byte, depth+1)
	// Return zerohash at depth
	if len(elements) == 0 {
		zerohash := trie.ZeroHashes[depth]
		layers[len(layers)-1] = []*[32]byte{&zerohash}
		return layers
	}
	transformedLeaves := make([]*[32]byte, len(elements))
	for i := range elements {
		arr := elements[i]
		transformedLeaves[i] = &arr
	}
	layers[0] = transformedLeaves
	buffer := bytes.NewBuffer([]byte{})
	buffer.Grow(64)

	for i := uint8(0); i < depth; i++ {
		layerLen := len(layers[i])
		oddNodeLength := layerLen%2 == 1
		if oddNodeLength {
			zerohash := trie.ZeroHashes[i]
			elements = append(elements, zerohash)
			layerLen++
		}

		layers[i+1] = make([]*[32]byte, layerLen/2)
		elements = htr.VectorizedSha256(elements)
		for j := range elements {
			layers[i+1][j] = &elements[j]
		}
	}
	return layers
}

// RecomputeFromLayer recomputes specific branches of a fixed sized trie depending on the provided changed indexes.
func RecomputeFromLayer(changedLeaves [][32]byte, changedIdx []uint64, layer [][]*[32]byte) ([32]byte, [][]*[32]byte, error) {
	hasher := hash.CustomSHA256Hasher()
	for i, idx := range changedIdx {
		layer[0][idx] = &changedLeaves[i]
	}

	if len(changedIdx) == 0 {
		return *layer[0][0], layer, nil
	}

	leaves := layer[0]

	// We need to ensure we recompute indices of the Merkle tree which
	// changed in-between calls to this function. This check adds an offset
	// to the recomputed indices to ensure we do so evenly.
	maxChangedIndex := changedIdx[len(changedIdx)-1]
	if int(maxChangedIndex+2) == len(leaves) && maxChangedIndex%2 != 0 {
		changedIdx = append(changedIdx, maxChangedIndex+1)
	}

	root := *layer[0][0]

	for _, idx := range changedIdx {
		ii, err := math.Int(idx)
		if err != nil {
			return [32]byte{}, nil, err
		}
		root, layer, err = recomputeRootFromLayer(ii, layer, leaves, hasher)
		if err != nil {
			return [32]byte{}, nil, err
		}
	}
	return root, layer, nil
}

// RecomputeFromLayerVariable recomputes specific branches of a variable sized trie depending on the provided changed indexes.
func RecomputeFromLayerVariable(changedLeaves [][32]byte, changedIdx []uint64, layer [][]*[32]byte) ([32]byte, [][]*[32]byte, error) {
	hasher := hash.CustomSHA256Hasher()
	if len(changedIdx) == 0 {
		return *layer[0][0], layer, nil
	}
	root := *layer[len(layer)-1][0]

	for i, idx := range changedIdx {
		ii, err := math.Int(idx)
		if err != nil {
			return [32]byte{}, nil, err
		}
		root, layer, err = recomputeRootFromLayerVariable(ii, changedLeaves[i], layer, hasher)
		if err != nil {
			return [32]byte{}, nil, err
		}
	}
	return root, layer, nil
}

// this method assumes that the provided trie already has all its elements included
// in the base depth.
func recomputeRootFromLayer(idx int, layers [][]*[32]byte, chunks []*[32]byte,
	hasher func([]byte) [32]byte) ([32]byte, [][]*[32]byte, error) {
	root := *chunks[idx]
	layers[0] = chunks
	// The merkle tree structure looks as follows:
	// [[r1, r2, r3, r4], [parent1, parent2], [root]]
	// Using information about the index which changed, idx, we recompute
	// only its branch up the tree.
	currentIndex := idx
	// Allocate only once.
	var combinedChunks [64]byte
	for i := 0; i < len(layers)-1; i++ {
		isLeft := currentIndex%2 == 0
		neighborIdx := currentIndex ^ 1

		var neighbor [32]byte
		if layers[i] != nil && len(layers[i]) != 0 && neighborIdx < len(layers[i]) {
			neighbor = *layers[i][neighborIdx]
		}
		if isLeft {
			copy(combinedChunks[:32], root[:])
			copy(combinedChunks[32:], neighbor[:])
		} else {
			copy(combinedChunks[:32], neighbor[:])
			copy(combinedChunks[32:], root[:])
		}

		parentHash := hasher(combinedChunks[:])
		root = parentHash

		parentIdx := currentIndex / 2
		// Update the cached layers at the parent index.
		rootVal := root
		if len(layers[i+1]) == 0 {
			layers[i+1] = append(layers[i+1], &rootVal)
		} else {
			layers[i+1][parentIdx] = &rootVal
		}
		currentIndex = parentIdx
	}
	// If there is only a single leaf, we return it (the identity element).
	if len(layers[0]) == 1 {
		return *layers[0][0], layers, nil
	}
	return root, layers, nil
}

// this method assumes that the base branch does not consist of all leaves of the
// trie. Instead missing leaves are assumed to be zerohashes, following the structure
// of a sparse merkle trie.
func recomputeRootFromLayerVariable(idx int, item [32]byte, layers [][]*[32]byte,
	hasher func([]byte) [32]byte) ([32]byte, [][]*[32]byte, error) {
	for idx >= len(layers[0]) {
		zerohash := trie.ZeroHashes[0]
		layers[0] = append(layers[0], &zerohash)
	}
	layers[0][idx] = &item

	currentIndex := idx
	root := item
	// Allocate only once.
	var neighbor [32]byte
	var combinedChunks [64]byte

	for i := 0; i < len(layers)-1; i++ {
		isLeft := currentIndex%2 == 0
		neighborIdx := currentIndex ^ 1

		if neighborIdx >= len(layers[i]) {
			neighbor = trie.ZeroHashes[i]
		} else {
			neighbor = *layers[i][neighborIdx]
		}
		if isLeft {
			copy(combinedChunks[:32], root[:])
			copy(combinedChunks[32:], neighbor[:])
		} else {
			copy(combinedChunks[:32], neighbor[:])
			copy(combinedChunks[32:], root[:])
		}

		parentHash := hasher(combinedChunks[:])
		root = parentHash

		parentIdx := currentIndex / 2
		if len(layers[i+1]) == 0 || parentIdx >= len(layers[i+1]) {
			newItem := root
			layers[i+1] = append(layers[i+1], &newItem)
		} else {
			newItem := root
			layers[i+1][parentIdx] = &newItem
		}
		currentIndex = parentIdx
	}
	return root, layers, nil
}

// AddInMixin describes a method from which a length mixin is added to the
// provided root.
func AddInMixin(root [32]byte, length uint64) ([32]byte, error) {
	rootBuf := new(bytes.Buffer)
	if err := binary.Write(rootBuf, binary.LittleEndian, length); err != nil {
		return [32]byte{}, errors.Wrap(err, "could not marshal eth1data votes length")
	}
	// We need to mix in the length of the slice.
	rootBufRoot := make([]byte, 32)
	copy(rootBufRoot, rootBuf.Bytes())
	return ssz.MixInLength(root, rootBufRoot), nil
}

// Merkleize 32-byte leaves into a Merkle trie for its adequate depth, returning
// the resulting layers of the trie based on the appropriate depth. This function
// pads the leaves to a length of 32.
func Merkleize(leaves [][]byte) [][][]byte {
	hashFunc := hash.CustomSHA256Hasher()
	layers := make([][][]byte, ssz.Depth(uint64(len(leaves)))+1)
	for len(leaves) != 32 {
		leaves = append(leaves, make([]byte, 32))
	}
	currentLayer := leaves
	layers[0] = currentLayer

	// We keep track of the hash layers of a Merkle trie until we reach
	// the top layer of length 1, which contains the single root element.
	//        [Root]      -> Top layer has length 1.
	//    [E]       [F]   -> This layer has length 2.
	// [A]  [B]  [C]  [D] -> The bottom layer has length 4 (needs to be a power of two).
	i := 1
	for len(currentLayer) > 1 && i < len(layers) {
		layer := make([][]byte, 0)
		for i := 0; i < len(currentLayer); i += 2 {
			hashedChunk := hashFunc(append(currentLayer[i], currentLayer[i+1]...))
			layer = append(layer, hashedChunk[:])
		}
		currentLayer = layer
		layers[i] = currentLayer
		i++
	}
	return layers
}

// MerkleizeTrieLeaves merkleize the trie leaves.
func MerkleizeTrieLeaves(layers [][][32]byte, hashLayer [][32]byte) ([][][32]byte, [][32]byte, error) {
	// We keep track of the hash layers of a Merkle trie until we reach
	// the top layer of length 1, which contains the single root element.
	//        [Root]      -> Top layer has length 1.
	//    [E]       [F]   -> This layer has length 2.
	// [A]  [B]  [C]  [D] -> The bottom layer has length 4 (needs to be a power of two).
	i := 1
	chunkBuffer := bytes.NewBuffer([]byte{})
	chunkBuffer.Grow(64)
	for len(hashLayer) > 1 && i < len(layers) {
		if !math.IsPowerOf2(uint64(len(hashLayer))) {
			return nil, nil, errors.Errorf("hash layer is a non power of 2: %d", len(hashLayer))
		}
		hashLayer = htr.VectorizedSha256(hashLayer)
		layers[i] = hashLayer
		i++
	}
	return layers, hashLayer, nil
}
