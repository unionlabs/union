package types

import (
	transfertypes "github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"

	"union/x/diferredack/types"
)

type DiferredAckMsg struct {
	/// Write an acknowledgement for a given packet
	WriteDiferredAck *WriteDiferredAck `json:"write_diferred_ack,omitempty"`
}

// CreateDenom creates a new factory denom, of denomination:
// factory/{creating contract address}/{Subdenom}
// Subdenom can be of length at most 44 characters, in [0-9a-zA-Z./]
// The (creating contract address, subdenom) pair must be unique.
// The created denom's admin is the creating contract address,
// but this admin can be changed using the ChangeAdmin binding.
type WriteDiferredAck struct {
	Packet             channeltypes.Packet
	Data               transfertypes.FungibleTokenPacketData
	DiferredPacketInfo types.DiferredPacketInfo `json:"diferred_packet_info"`
	Ack                channeltypes.Acknowledgement
}
