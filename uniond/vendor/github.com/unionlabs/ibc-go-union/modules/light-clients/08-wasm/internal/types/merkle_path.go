package types

import (
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
)

// MerklePath is the path used to verify commitment proofs, which can be an
// arbitrary structured object (defined by a commitment type).
// MerklePath is represented from root-to-leaf
// NOTE(Forward compatibility): See https://github.com/cosmos/ibc-go/issues/6496
type MerklePath struct {
	KeyPath [][]byte `json:"key_path,omitempty"`
}

// ToMerklePathV2 converts a 23-commitment MerklePath to a v2 MerklePath using bytes for the key path.
// This provides the ability to prove values stored under keys which contain non-utf8 encoded symbols.
func ToMerklePathV2(merklePath commitmenttypes.MerklePath) MerklePath {
	var keyPath [][]byte
	for _, key := range merklePath.KeyPath {
		keyPath = append(keyPath, []byte(key))
	}

	return MerklePath{
		KeyPath: keyPath,
	}
}
