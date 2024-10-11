package blocks

import (
	"bytes"
	"sort"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
)

// ROBlock is a value that embeds a ReadOnlySignedBeaconBlock along with its block root ([32]byte).
// This allows the block root to be cached within a value that satisfies the ReadOnlySignedBeaconBlock interface.
// Since the root and slot for each ROBlock is known, slices can be efficiently sorted using ROBlockSlice.
type ROBlock struct {
	interfaces.ReadOnlySignedBeaconBlock
	root [32]byte
}

// Root returns the block hash_tree_root for the embedded ReadOnlySignedBeaconBlock.Block().
func (b ROBlock) Root() [32]byte {
	return b.root
}

// RootSlice returns a slice of the value returned by Root(). This is convenient because slicing the result of a func
// is not allowed, so only offering a fixed-length array version results in boilerplate code to
func (b ROBlock) RootSlice() []byte {
	r := make([]byte, 32)
	copy(r, b.root[:])
	return r
}

// NewROBlockWithRoot creates an ROBlock embedding the given block with its root. It accepts the root as parameter rather than
// computing it internally, because in some cases a block is retrieved by its root and recomputing it is a waste.
func NewROBlockWithRoot(b interfaces.ReadOnlySignedBeaconBlock, root [32]byte) (ROBlock, error) {
	if err := BeaconBlockIsNil(b); err != nil {
		return ROBlock{}, err
	}
	return ROBlock{ReadOnlySignedBeaconBlock: b, root: root}, nil
}

// NewROBlock creates a ROBlock from a ReadOnlySignedBeaconBlock. It uses the HashTreeRoot method of the given
// ReadOnlySignedBeaconBlock.Block to compute the cached root.
func NewROBlock(b interfaces.ReadOnlySignedBeaconBlock) (ROBlock, error) {
	if err := BeaconBlockIsNil(b); err != nil {
		return ROBlock{}, err
	}
	root, err := b.Block().HashTreeRoot()
	if err != nil {
		return ROBlock{}, err
	}
	return ROBlock{ReadOnlySignedBeaconBlock: b, root: root}, nil
}

// NewROBlockSlice is a helper method for converting a slice of the ReadOnlySignedBeaconBlock interface
// to a slice of ROBlock.
func NewROBlockSlice(blks []interfaces.ReadOnlySignedBeaconBlock) ([]ROBlock, error) {
	robs := make([]ROBlock, len(blks))
	var err error
	for i := range blks {
		robs[i], err = NewROBlock(blks[i])
		if err != nil {
			return nil, err
		}
	}
	return robs, nil
}

// ROBlockSlice implements sort.Interface so that slices of ROBlocks can be easily sorted.
// A slice of ROBlock is sorted first by slot, with ties broken by cached block roots.
type ROBlockSlice []ROBlock

var _ sort.Interface = ROBlockSlice{}

// Less reports whether the element with index i must sort before the element with index j.
// ROBlocks are ordered first by their slot,
// with a lexicographic sort of roots breaking ties for slots with duplicate blocks.
func (s ROBlockSlice) Less(i, j int) bool {
	si, sj := s[i].Block().Slot(), s[j].Block().Slot()

	// lower slot wins
	if si != sj {
		return si < sj
	}

	// break slot tie lexicographically comparing roots byte for byte
	ri, rj := s[i].Root(), s[j].Root()
	return bytes.Compare(ri[:], rj[:]) < 0
}

// Swap swaps the elements with indexes i and j.
func (s ROBlockSlice) Swap(i, j int) {
	s[i], s[j] = s[j], s[i]
}

// Len is the number of elements in the collection.
func (s ROBlockSlice) Len() int {
	return len(s)
}

// BlockWithROBlobs is a wrapper that collects the block and blob values together.
// This is helpful because these values are collated from separate RPC requests.
type BlockWithROBlobs struct {
	Block ROBlock
	Blobs []ROBlob
}

// BlockWithROBlobsSlice gives convenient access to getting a slice of just the ROBlocks,
// and defines sorting helpers.
type BlockWithROBlobsSlice []BlockWithROBlobs

func (s BlockWithROBlobsSlice) ROBlocks() []ROBlock {
	r := make([]ROBlock, len(s))
	for i := range s {
		r[i] = s[i].Block
	}
	return r
}

// Less reports whether the element with index i must sort before the element with index j.
// ROBlocks are ordered first by their slot,
// with a lexicographic sort of roots breaking ties for slots with duplicate blocks.
func (s BlockWithROBlobsSlice) Less(i, j int) bool {
	si, sj := s[i].Block.Block().Slot(), s[j].Block.Block().Slot()

	// lower slot wins
	if si != sj {
		return si < sj
	}

	// break slot tie lexicographically comparing roots byte for byte
	ri, rj := s[i].Block.Root(), s[j].Block.Root()
	return bytes.Compare(ri[:], rj[:]) < 0
}

// Swap swaps the elements with indexes i and j.
func (s BlockWithROBlobsSlice) Swap(i, j int) {
	s[i], s[j] = s[j], s[i]
}

// Len is the number of elements in the collection.
func (s BlockWithROBlobsSlice) Len() int {
	return len(s)
}
