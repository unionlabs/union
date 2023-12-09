package blocksync

import (
	"fmt"

	"github.com/cosmos/gogoproto/proto"

	"github.com/cometbft/cometbft/p2p"
)

var _ p2p.Wrapper = &StatusRequest{}
var _ p2p.Wrapper = &StatusResponse{}
var _ p2p.Wrapper = &NoBlockResponse{}
var _ p2p.Wrapper = &BlockResponse{}
var _ p2p.Wrapper = &BlockRequest{}

const (
	BlockResponseMessagePrefixSize   = 4
	BlockResponseMessageFieldKeySize = 1
)

func (m *BlockRequest) Wrap() proto.Message {
	bm := &Message{}
	bm.Sum = &Message_BlockRequest{BlockRequest: m}
	return bm
}

func (m *BlockResponse) Wrap() proto.Message {
	bm := &Message{}
	bm.Sum = &Message_BlockResponse{BlockResponse: m}
	return bm
}

func (m *NoBlockResponse) Wrap() proto.Message {
	bm := &Message{}
	bm.Sum = &Message_NoBlockResponse{NoBlockResponse: m}
	return bm
}

func (m *StatusRequest) Wrap() proto.Message {
	bm := &Message{}
	bm.Sum = &Message_StatusRequest{StatusRequest: m}
	return bm
}

func (m *StatusResponse) Wrap() proto.Message {
	bm := &Message{}
	bm.Sum = &Message_StatusResponse{StatusResponse: m}
	return bm
}

// Unwrap implements the p2p Wrapper interface and unwraps a wrapped blockchain
// message.
func (m *Message) Unwrap() (proto.Message, error) {
	switch msg := m.Sum.(type) {
	case *Message_BlockRequest:
		return m.GetBlockRequest(), nil

	case *Message_BlockResponse:
		return m.GetBlockResponse(), nil

	case *Message_NoBlockResponse:
		return m.GetNoBlockResponse(), nil

	case *Message_StatusRequest:
		return m.GetStatusRequest(), nil

	case *Message_StatusResponse:
		return m.GetStatusResponse(), nil

	default:
		return nil, fmt.Errorf("unknown message: %T", msg)
	}
}
