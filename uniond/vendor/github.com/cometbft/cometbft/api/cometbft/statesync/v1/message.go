package v1

import (
	"fmt"

	"github.com/cosmos/gogoproto/proto"
)

func (m *SnapshotsResponse) Wrap() proto.Message {
	sm := &Message{}
	sm.Sum = &Message_SnapshotsResponse{SnapshotsResponse: m}
	return sm
}

func (m *SnapshotsRequest) Wrap() proto.Message {
	sm := &Message{}
	sm.Sum = &Message_SnapshotsRequest{SnapshotsRequest: m}
	return sm
}

func (m *ChunkResponse) Wrap() proto.Message {
	sm := &Message{}
	sm.Sum = &Message_ChunkResponse{ChunkResponse: m}
	return sm
}

func (m *ChunkRequest) Wrap() proto.Message {
	sm := &Message{}
	sm.Sum = &Message_ChunkRequest{ChunkRequest: m}
	return sm
}

// Unwrap implements the p2p Wrapper interface and unwraps a wrapped state sync
// proto message.
func (m *Message) Unwrap() (proto.Message, error) {
	switch msg := m.Sum.(type) {
	case *Message_ChunkRequest:
		return m.GetChunkRequest(), nil

	case *Message_ChunkResponse:
		return m.GetChunkResponse(), nil

	case *Message_SnapshotsRequest:
		return m.GetSnapshotsRequest(), nil

	case *Message_SnapshotsResponse:
		return m.GetSnapshotsResponse(), nil

	default:
		return nil, fmt.Errorf("unknown message: %T", msg)
	}
}
