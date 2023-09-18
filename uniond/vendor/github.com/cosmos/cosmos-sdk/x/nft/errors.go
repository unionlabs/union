package nft

import (
	"cosmossdk.io/errors"
)

// x/nft module sentinel errors
var (
	ErrClassExists    = errors.Register(ModuleName, 3, "nft class already exists")
	ErrClassNotExists = errors.Register(ModuleName, 4, "nft class does not exist")
	ErrNFTExists      = errors.Register(ModuleName, 5, "nft already exists")
	ErrNFTNotExists   = errors.Register(ModuleName, 6, "nft does not exist")
	ErrEmptyClassID   = errors.Register(ModuleName, 7, "empty class id")
	ErrEmptyNFTID     = errors.Register(ModuleName, 8, "empty nft id")
)
