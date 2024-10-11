package store

import "github.com/cometbft/cometbft/types"

// Store is anything that can persistently store headers.
type Store interface {
	// SaveSignedHeaderAndValidatorSet saves a SignedHeader (h: sh.Height) and a
	// ValidatorSet (h: sh.Height).
	//
	// height must be > 0.
	SaveLightBlock(lb *types.LightBlock) error

	// DeleteSignedHeaderAndValidatorSet deletes SignedHeader (h: height) and
	// ValidatorSet (h: height).
	//
	// height must be > 0.
	DeleteLightBlock(height int64) error

	// LightBlock returns the LightBlock that corresponds to the given
	// height.
	//
	// height must be > 0.
	//
	// If LightBlock is not found, ErrLightBlockNotFound is returned.
	LightBlock(height int64) (*types.LightBlock, error)

	// LastLightBlockHeight returns the last (newest) LightBlock height.
	//
	// If the store is empty, -1 and nil error are returned.
	LastLightBlockHeight() (int64, error)

	// FirstLightBlockHeight returns the first (oldest) LightBlock height.
	//
	// If the store is empty, -1 and nil error are returned.
	FirstLightBlockHeight() (int64, error)

	// LightBlockBefore returns the LightBlock before a certain height.
	//
	// height must be > 0 && <= LastLightBlockHeight.
	LightBlockBefore(height int64) (*types.LightBlock, error)

	// Prune removes headers & the associated validator sets when Store reaches a
	// defined size (number of header & validator set pairs).
	Prune(size uint16) error

	// Size returns a number of currently existing header & validator set pairs.
	Size() uint16
}
