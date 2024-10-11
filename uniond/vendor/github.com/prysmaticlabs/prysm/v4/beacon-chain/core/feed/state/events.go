// Package state contains types for state operation-specific events fired
// during the runtime of a beacon node such state initialization, state updates,
// and chain start.
package state

import (
	"time"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
)

const (
	// BlockProcessed is sent after a block has been processed and updated the state database.
	BlockProcessed = iota + 1
	// ChainStarted is sent when enough validators are active to start proposing blocks.
	ChainStarted
	// deprecated: Initialized is sent when the internal beacon node's state is ready to be accessed.
	_
	// deprecated: Synced is sent when the beacon node has completed syncing and is ready to participate in the network.
	_
	// Reorg is an event sent when the new head is not a descendant of the previous head.
	Reorg
	// FinalizedCheckpoint event.
	FinalizedCheckpoint
	// NewHead of the chain event.
	NewHead
	// MissedSlot is sent when we need to notify users that a slot was missed.
	MissedSlot
	// LightClientFinalityUpdate event
	LightClientFinalityUpdate
	// LightClientOptimisticUpdate event
	LightClientOptimisticUpdate
)

// BlockProcessedData is the data sent with BlockProcessed events.
type BlockProcessedData struct {
	// Slot is the slot of the processed block.
	Slot primitives.Slot
	// BlockRoot of the processed block.
	BlockRoot [32]byte
	// SignedBlock is the physical processed block.
	SignedBlock interfaces.ReadOnlySignedBeaconBlock
	// Verified is true if the block's BLS contents have been verified.
	Verified bool
	// Optimistic is true if the block is optimistic.
	Optimistic bool
}

// ChainStartedData is the data sent with ChainStarted events.
type ChainStartedData struct {
	// StartTime is the time at which the chain started.
	StartTime time.Time
}

// SyncedData is the data sent with Synced events.
type SyncedData struct {
	// StartTime is the time at which the chain started.
	StartTime time.Time
}

// InitializedData is the data sent with Initialized events.
type InitializedData struct {
	// StartTime is the time at which the chain started.
	StartTime time.Time
	// GenesisValidatorsRoot represents state.validators.HashTreeRoot().
	GenesisValidatorsRoot []byte
}
