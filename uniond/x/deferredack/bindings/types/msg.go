package types

import (
	"union/x/deferredack/types"
)

type DeferredAckMsg struct {
	/// Write an acknowledgement for a given packet
	WriteDeferredAck *WriteDeferredAck `json:"write_deferred_ack,omitempty"`
}

// CreateDenom creates a new factory denom, of denomination:
// factory/{creating contract address}/{Subdenom}
// Subdenom can be of length at most 44 characters, in [0-9a-zA-Z./]
// The (creating contract address, subdenom) pair must be unique.
// The created denom's admin is the creating contract address,
// but this admin can be changed using the ChangeAdmin binding.
type WriteDeferredAck struct {
	DeferredPacketInfo types.DeferredPacketInfo `json:"deferred_packet_info"`
	Ack                types.Acknowledgement
}
