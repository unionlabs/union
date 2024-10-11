package blockchain

import "github.com/pkg/errors"

var (
	// ErrInvalidPayload is returned when the payload is invalid
	ErrInvalidPayload = invalidBlock{error: errors.New("received an INVALID payload from execution engine")}
	// ErrInvalidBlockHashPayloadStatus is returned when the payload has invalid block hash.
	ErrInvalidBlockHashPayloadStatus = invalidBlock{error: errors.New("received an INVALID_BLOCK_HASH payload from execution engine")}
	// ErrUndefinedExecutionEngineError is returned when the execution engine returns an error that is not defined
	ErrUndefinedExecutionEngineError = errors.New("received an undefined execution engine error")
	// errNilFinalizedInStore is returned when a nil finalized checkpt is returned from store.
	errNilFinalizedInStore = errors.New("nil finalized checkpoint returned from store")
	// errNilFinalizedCheckpoint is returned when a nil finalized checkpt is returned from a state.
	errNilFinalizedCheckpoint = errors.New("nil finalized checkpoint returned from state")
	// errNilJustifiedCheckpoint is returned when a nil justified checkpt is returned from a state.
	errNilJustifiedCheckpoint = errors.New("nil justified checkpoint returned from state")
	// errBlockDoesNotExist is returned when a block does not exist for a particular state summary.
	errBlockDoesNotExist = errors.New("could not find block in DB")
	// errBlockNotFoundInCacheOrDB is returned when a block is not found in the cache or DB.
	errBlockNotFoundInCacheOrDB = errors.New("block not found in cache or db")
	// errWSBlockNotFound is returned when a block is not found in the WS cache or DB.
	errWSBlockNotFound = errors.New("weak subjectivity root not found in db")
	// errWSBlockNotFoundInEpoch is returned when a block is not found in the WS cache or DB within epoch.
	errWSBlockNotFoundInEpoch = errors.New("weak subjectivity root not found in db within epoch")
	// ErrNotDescendantOfFinalized is returned when a block is not a descendant of the finalized checkpoint
	ErrNotDescendantOfFinalized = invalidBlock{error: errors.New("not descendant of finalized checkpoint")}
	// ErrNotCheckpoint is returned when a given checkpoint is not a
	// checkpoint in any chain known to forkchoice
	ErrNotCheckpoint = errors.New("not a checkpoint in forkchoice")
	// ErrNilHead is returned when no head is present in the blockchain service.
	ErrNilHead = errors.New("nil head")
)

var errMaxBlobsExceeded = errors.New("Expected commitments in block exceeds MAX_BLOBS_PER_BLOCK")

// An invalid block is the block that fails state transition based on the core protocol rules.
// The beacon node shall not be accepting nor building blocks that branch off from an invalid block.
// Some examples of invalid blocks are:
// The block violates state transition rules.
// The block is deemed invalid according to execution layer client.
// The block violates certain fork choice rules (before finalized slot, not finalized ancestor)
type invalidBlock struct {
	invalidAncestorRoots [][32]byte
	error
	root          [32]byte
	lastValidHash [32]byte
}

type invalidBlockError interface {
	Error() string
	InvalidAncestorRoots() [][32]byte
	BlockRoot() [32]byte
	LastValidHash() [32]byte
}

// BlockRoot returns the invalid block root.
func (e invalidBlock) BlockRoot() [32]byte {
	return e.root
}

// LastValidHash returns the last valid hash root.
func (e invalidBlock) LastValidHash() [32]byte {
	return e.lastValidHash
}

// InvalidAncestorRoots returns an optional list of invalid roots of the invalid block which leads up last valid root.
func (e invalidBlock) InvalidAncestorRoots() [][32]byte {
	return e.invalidAncestorRoots
}

// IsInvalidBlock returns true if the error has `invalidBlock`.
func IsInvalidBlock(e error) bool {
	if e == nil {
		return false
	}
	var d invalidBlockError
	return errors.As(e, &d)
}

// InvalidBlockLVH returns the invalid block last valid hash root. If the error
// doesn't have a last valid hash, [32]byte{} is returned.
func InvalidBlockLVH(e error) [32]byte {
	if e == nil {
		return [32]byte{}
	}
	var d invalidBlockError
	ok := errors.As(e, &d)
	if !ok {
		return [32]byte{}
	}
	return d.LastValidHash()
}

// InvalidBlockRoot returns the invalid block root. If the error
// doesn't have an invalid blockroot. [32]byte{} is returned.
func InvalidBlockRoot(e error) [32]byte {
	if e == nil {
		return [32]byte{}
	}
	var d invalidBlockError
	ok := errors.As(e, &d)
	if !ok {
		return [32]byte{}
	}
	return d.BlockRoot()
}

// InvalidAncestorRoots returns a list of invalid roots up to last valid root.
func InvalidAncestorRoots(e error) [][32]byte {
	if e == nil {
		return [][32]byte{}
	}
	var d invalidBlockError
	ok := errors.As(e, &d)
	if !ok {
		return [][32]byte{}
	}
	return d.InvalidAncestorRoots()
}
