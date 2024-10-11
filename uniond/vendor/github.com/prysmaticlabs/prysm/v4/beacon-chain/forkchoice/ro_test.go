package forkchoice

import (
	"io"
	"testing"

	forkchoicetypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/types"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

type mockCall int

const (
	lockCalled mockCall = iota
	unlockCalled
	rlockCalled
	runlockCalled
	hasNodeCalled
	proposerBoostCalled
	isCanonicalCalled
	finalizedCheckpointCalled
	isViableForCheckpointCalled
	finalizedPayloadBlockHashCalled
	justifiedCheckpointCalled
	previousJustifiedCheckpointCalled
	justifiedPayloadBlockHashCalled
	unrealizedJustifiedPayloadBlockHashCalled
	nodeCountCalled
	highestReceivedBlockSlotCalled
	highestReceivedBlockDelayCalled
	receivedBlocksLastEpochCalled
	weightCalled
	isOptimisticCalled
	shouldOverrideFCUCalled
	slotCalled
	lastRootCalled
	targetRootForEpochCalled
)

func _discard(t *testing.T, e error) {
	if e != nil {
		_, err := io.Discard.Write([]byte(e.Error()))
		require.NoError(t, err)
	}
}

// ensures that the correct func was called with the correct lock pattern
// for each method in the interface.
func TestROLocking(t *testing.T) {
	cases := []struct {
		name string
		call mockCall
		cb   func(FastGetter)
	}{
		{
			name: "hasNodeCalled",
			call: hasNodeCalled,
			cb:   func(g FastGetter) { g.HasNode([32]byte{}) },
		},
		{
			name: "proposerBoostCalled",
			call: proposerBoostCalled,
			cb:   func(g FastGetter) { g.ProposerBoost() },
		},
		{
			name: "isCanonicalCalled",
			call: isCanonicalCalled,
			cb:   func(g FastGetter) { g.IsCanonical([32]byte{}) },
		},
		{
			name: "finalizedCheckpointCalled",
			call: finalizedCheckpointCalled,
			cb:   func(g FastGetter) { g.FinalizedCheckpoint() },
		},
		{
			name: "isViableForCheckpointCalled",
			call: isViableForCheckpointCalled,
			cb:   func(g FastGetter) { _, err := g.IsViableForCheckpoint(nil); _discard(t, err) },
		},
		{
			name: "finalizedPayloadBlockHashCalled",
			call: finalizedPayloadBlockHashCalled,
			cb:   func(g FastGetter) { g.FinalizedPayloadBlockHash() },
		},
		{
			name: "justifiedCheckpointCalled",
			call: justifiedCheckpointCalled,
			cb:   func(g FastGetter) { g.JustifiedCheckpoint() },
		},
		{
			name: "previousJustifiedCheckpointCalled",
			call: previousJustifiedCheckpointCalled,
			cb:   func(g FastGetter) { g.PreviousJustifiedCheckpoint() },
		},
		{
			name: "justifiedPayloadBlockHashCalled",
			call: justifiedPayloadBlockHashCalled,
			cb:   func(g FastGetter) { g.JustifiedPayloadBlockHash() },
		},
		{
			name: "unrealizedJustifiedPayloadBlockHashCalled",
			call: unrealizedJustifiedPayloadBlockHashCalled,
			cb:   func(g FastGetter) { g.UnrealizedJustifiedPayloadBlockHash() },
		},
		{
			name: "nodeCountCalled",
			call: nodeCountCalled,
			cb:   func(g FastGetter) { g.NodeCount() },
		},
		{
			name: "highestReceivedBlockSlotCalled",
			call: highestReceivedBlockSlotCalled,
			cb:   func(g FastGetter) { g.HighestReceivedBlockSlot() },
		},
		{
			name: "highestReceivedBlockDelayCalled",
			call: highestReceivedBlockDelayCalled,
			cb:   func(g FastGetter) { g.HighestReceivedBlockDelay() },
		},
		{
			name: "receivedBlocksLastEpochCalled",
			call: receivedBlocksLastEpochCalled,
			cb:   func(g FastGetter) { _, err := g.ReceivedBlocksLastEpoch(); _discard(t, err) },
		},
		{
			name: "weightCalled",
			call: weightCalled,
			cb:   func(g FastGetter) { _, err := g.Weight([32]byte{}); _discard(t, err) },
		},
		{
			name: "isOptimisticCalled",
			call: isOptimisticCalled,
			cb:   func(g FastGetter) { _, err := g.IsOptimistic([32]byte{}); _discard(t, err) },
		},
		{
			name: "shouldOverrideFCUCalled",
			call: shouldOverrideFCUCalled,
			cb:   func(g FastGetter) { g.ShouldOverrideFCU() },
		},
		{
			name: "slotCalled",
			call: slotCalled,
			cb:   func(g FastGetter) { _, err := g.Slot([32]byte{}); _discard(t, err) },
		},
		{
			name: "lastRootCalled",
			call: lastRootCalled,
			cb:   func(g FastGetter) { g.LastRoot(0) },
		},
		{
			name: "targetRootForEpochCalled",
			call: targetRootForEpochCalled,
			cb:   func(g FastGetter) { _, err := g.TargetRootForEpoch([32]byte{}, 0); _discard(t, err) },
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			m := &mockROForkchoice{}
			ro := NewROForkChoice(m)
			c.cb(ro)
			require.Equal(t, rlockCalled, m.calls[0])
			require.Equal(t, c.call, m.calls[1])
			require.Equal(t, runlockCalled, m.calls[2])
		})
	}
}

type mockROForkchoice struct {
	calls []mockCall
}

var _ FastGetter = &mockROForkchoice{}

var _ RLocker = &mockROForkchoice{}

func (ro *mockROForkchoice) Lock() {
	ro.calls = append(ro.calls, lockCalled)
}

func (ro *mockROForkchoice) RLock() {
	ro.calls = append(ro.calls, rlockCalled)
}

func (ro *mockROForkchoice) Unlock() {
	ro.calls = append(ro.calls, unlockCalled)
}

func (ro *mockROForkchoice) RUnlock() {
	ro.calls = append(ro.calls, runlockCalled)
}

func (ro *mockROForkchoice) HasNode(_ [32]byte) bool {
	ro.calls = append(ro.calls, hasNodeCalled)
	return false
}

func (ro *mockROForkchoice) ProposerBoost() [fieldparams.RootLength]byte {
	ro.calls = append(ro.calls, proposerBoostCalled)
	return [fieldparams.RootLength]byte{}
}

func (ro *mockROForkchoice) IsCanonical(_ [32]byte) bool {
	ro.calls = append(ro.calls, isCanonicalCalled)
	return false
}

func (ro *mockROForkchoice) FinalizedCheckpoint() *forkchoicetypes.Checkpoint {
	ro.calls = append(ro.calls, finalizedCheckpointCalled)
	return nil
}

func (ro *mockROForkchoice) IsViableForCheckpoint(_ *forkchoicetypes.Checkpoint) (bool, error) {
	ro.calls = append(ro.calls, isViableForCheckpointCalled)
	return false, nil
}

func (ro *mockROForkchoice) FinalizedPayloadBlockHash() [32]byte {
	ro.calls = append(ro.calls, finalizedPayloadBlockHashCalled)
	return [32]byte{}
}

func (ro *mockROForkchoice) JustifiedCheckpoint() *forkchoicetypes.Checkpoint {
	ro.calls = append(ro.calls, justifiedCheckpointCalled)
	return nil
}

func (ro *mockROForkchoice) PreviousJustifiedCheckpoint() *forkchoicetypes.Checkpoint {
	ro.calls = append(ro.calls, previousJustifiedCheckpointCalled)
	return nil
}

func (ro *mockROForkchoice) JustifiedPayloadBlockHash() [32]byte {
	ro.calls = append(ro.calls, justifiedPayloadBlockHashCalled)
	return [32]byte{}
}

func (ro *mockROForkchoice) UnrealizedJustifiedPayloadBlockHash() [32]byte {
	ro.calls = append(ro.calls, unrealizedJustifiedPayloadBlockHashCalled)
	return [32]byte{}
}

func (ro *mockROForkchoice) NodeCount() int {
	ro.calls = append(ro.calls, nodeCountCalled)
	return 0
}

func (ro *mockROForkchoice) HighestReceivedBlockSlot() primitives.Slot {
	ro.calls = append(ro.calls, highestReceivedBlockSlotCalled)
	return 0
}

func (ro *mockROForkchoice) HighestReceivedBlockDelay() primitives.Slot {
	ro.calls = append(ro.calls, highestReceivedBlockDelayCalled)
	return 0
}

func (ro *mockROForkchoice) ReceivedBlocksLastEpoch() (uint64, error) {
	ro.calls = append(ro.calls, receivedBlocksLastEpochCalled)
	return 0, nil
}

func (ro *mockROForkchoice) Weight(_ [32]byte) (uint64, error) {
	ro.calls = append(ro.calls, weightCalled)
	return 0, nil
}

func (ro *mockROForkchoice) IsOptimistic(_ [32]byte) (bool, error) {
	ro.calls = append(ro.calls, isOptimisticCalled)
	return false, nil
}

func (ro *mockROForkchoice) ShouldOverrideFCU() bool {
	ro.calls = append(ro.calls, shouldOverrideFCUCalled)
	return false
}

func (ro *mockROForkchoice) Slot(_ [32]byte) (primitives.Slot, error) {
	ro.calls = append(ro.calls, slotCalled)
	return 0, nil
}

func (ro *mockROForkchoice) LastRoot(_ primitives.Epoch) [32]byte {
	ro.calls = append(ro.calls, lastRootCalled)
	return [32]byte{}
}

// TargetRootForEpoch implements FastGetter.
func (ro *mockROForkchoice) TargetRootForEpoch(_ [32]byte, _ primitives.Epoch) ([32]byte, error) {
	ro.calls = append(ro.calls, targetRootForEpochCalled)
	return [32]byte{}, nil
}
