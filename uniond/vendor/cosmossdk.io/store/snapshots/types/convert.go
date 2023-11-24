package types

import (
	abci "github.com/cometbft/cometbft/abci/types"
	proto "github.com/cosmos/gogoproto/proto"

	"cosmossdk.io/errors"
)

// Converts an ABCI snapshot to a snapshot. Mainly to decode the SDK metadata.
func SnapshotFromABCI(in *abci.Snapshot) (Snapshot, error) {
	snapshot := Snapshot{
		Height: in.Height,
		Format: in.Format,
		Chunks: in.Chunks,
		Hash:   in.Hash,
	}
	err := proto.Unmarshal(in.Metadata, &snapshot.Metadata)
	if err != nil {
		return Snapshot{}, errors.Wrap(err, "failed to unmarshal snapshot metadata")
	}
	return snapshot, nil
}

// Converts a Snapshot to its ABCI representation. Mainly to encode the SDK metadata.
func (s Snapshot) ToABCI() (abci.Snapshot, error) {
	out := abci.Snapshot{
		Height: s.Height,
		Format: s.Format,
		Chunks: s.Chunks,
		Hash:   s.Hash,
	}
	var err error
	out.Metadata, err = proto.Marshal(&s.Metadata)
	if err != nil {
		return abci.Snapshot{}, errors.Wrap(err, "failed to marshal snapshot metadata")
	}
	return out, nil
}
