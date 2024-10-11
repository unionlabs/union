package testutil

import (
	"context"
	"strconv"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/core"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
)

// MockBlocker is a fake implementation of lookup.Blocker.
type MockBlocker struct {
	BlockToReturn interfaces.ReadOnlySignedBeaconBlock
	ErrorToReturn error
	SlotBlockMap  map[primitives.Slot]interfaces.ReadOnlySignedBeaconBlock
	RootBlockMap  map[[32]byte]interfaces.ReadOnlySignedBeaconBlock
}

// Block --
func (m *MockBlocker) Block(_ context.Context, b []byte) (interfaces.ReadOnlySignedBeaconBlock, error) {
	if m.ErrorToReturn != nil {
		return nil, m.ErrorToReturn
	}
	if m.BlockToReturn != nil {
		return m.BlockToReturn, nil
	}
	slotNumber, parseErr := strconv.ParseUint(string(b), 10, 64)
	if parseErr != nil {
		//nolint:nilerr
		return m.RootBlockMap[bytesutil.ToBytes32(b)], nil
	}
	return m.SlotBlockMap[primitives.Slot(slotNumber)], nil
}

// Blobs --
func (m *MockBlocker) Blobs(_ context.Context, _ string, _ []uint64) ([]*blocks.VerifiedROBlob, *core.RpcError) {
	panic("implement me")
}
