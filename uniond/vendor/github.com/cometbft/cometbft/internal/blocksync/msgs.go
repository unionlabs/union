package blocksync

import (
	"fmt"

	"github.com/cosmos/gogoproto/proto"

	bcproto "github.com/cometbft/cometbft/api/cometbft/blocksync/v1"
	"github.com/cometbft/cometbft/types"
)

const (
	// NOTE: keep up to date with bcproto.BlockResponse.
	BlockResponseMessagePrefixSize   = 4
	BlockResponseMessageFieldKeySize = 1
	MaxMsgSize                       = types.MaxBlockSizeBytes +
		BlockResponseMessagePrefixSize +
		BlockResponseMessageFieldKeySize
)

// ValidateMsg validates a message.
func ValidateMsg(pb proto.Message) error {
	if pb == nil {
		return ErrNilMessage
	}

	switch msg := pb.(type) {
	case *bcproto.BlockRequest:
		if msg.Height < 0 {
			return ErrInvalidHeight{Height: msg.Height, Reason: "negative height"}
		}
	case *bcproto.BlockResponse:
		// Avoid double-calling `types.BlockFromProto` for performance reasons.
		// See https://github.com/cometbft/cometbft/issues/1964
		return nil
	case *bcproto.NoBlockResponse:
		if msg.Height < 0 {
			return ErrInvalidHeight{Height: msg.Height, Reason: "negative height"}
		}
	case *bcproto.StatusResponse:
		if msg.Base < 0 {
			return ErrInvalidBase{Base: msg.Base, Reason: "negative base"}
		}
		if msg.Height < 0 {
			return ErrInvalidHeight{Height: msg.Height, Reason: "negative height"}
		}
		if msg.Base > msg.Height {
			return ErrInvalidHeight{Height: msg.Height, Reason: fmt.Sprintf("base %v cannot be greater than height", msg.Base)}
		}
	case *bcproto.StatusRequest:
		return nil
	default:
		return ErrUnknownMessageType{Msg: msg}
	}
	return nil
}
