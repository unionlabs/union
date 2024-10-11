// Package block contains types for block-specific events fired
// during the runtime of a beacon node.
package block

import "github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"

const (
	// ReceivedBlock is sent after a block has been received by the beacon node via p2p or RPC.
	ReceivedBlock = iota + 1
)

// ReceivedBlockData is the data sent with ReceivedBlock events.
type ReceivedBlockData struct {
	SignedBlock  interfaces.ReadOnlySignedBeaconBlock
	IsOptimistic bool
}
