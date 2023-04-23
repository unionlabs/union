package statesync

import (
	"errors"
	"fmt"

	"github.com/cosmos/gogoproto/proto"

	ssproto "github.com/cometbft/cometbft/proto/tendermint/statesync"
)

const (
	// snapshotMsgSize is the maximum size of a snapshotResponseMessage
	snapshotMsgSize = int(4e6)
	// chunkMsgSize is the maximum size of a chunkResponseMessage
	chunkMsgSize = int(16e6)
)

// validateMsg validates a message.
func validateMsg(pb proto.Message) error {
	if pb == nil {
		return errors.New("message cannot be nil")
	}
	switch msg := pb.(type) {
	case *ssproto.ChunkRequest:
		if msg.Height == 0 {
			return errors.New("height cannot be 0")
		}
	case *ssproto.ChunkResponse:
		if msg.Height == 0 {
			return errors.New("height cannot be 0")
		}
		if msg.Missing && len(msg.Chunk) > 0 {
			return errors.New("missing chunk cannot have contents")
		}
		if !msg.Missing && msg.Chunk == nil {
			return errors.New("chunk cannot be nil")
		}
	case *ssproto.SnapshotsRequest:
	case *ssproto.SnapshotsResponse:
		if msg.Height == 0 {
			return errors.New("height cannot be 0")
		}
		if len(msg.Hash) == 0 {
			return errors.New("snapshot has no hash")
		}
		if msg.Chunks == 0 {
			return errors.New("snapshot has no chunks")
		}
	default:
		return fmt.Errorf("unknown message type %T", msg)
	}
	return nil
}
