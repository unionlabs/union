package mock

import (
	"context"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
)

func NewReplayerBuilder(opt ...ReplayerBuilderOption) *ReplayerBuilder {
	b := &ReplayerBuilder{}
	for _, o := range opt {
		o(b)
	}
	return b
}

type ReplayerBuilderOption func(*ReplayerBuilder)

func WithMockState(s state.BeaconState) ReplayerBuilderOption {
	return func(b *ReplayerBuilder) {
		b.SetMockState(s)
	}
}

type ReplayerBuilder struct {
	forSlot map[primitives.Slot]*Replayer
}

func (b *ReplayerBuilder) SetMockState(s state.BeaconState) {
	if b.forSlot == nil {
		b.forSlot = make(map[primitives.Slot]*Replayer)
	}
	b.forSlot[s.Slot()] = &Replayer{State: s}
}

func (b *ReplayerBuilder) SetMockStateForSlot(s state.BeaconState, slot primitives.Slot) {
	if b.forSlot == nil {
		b.forSlot = make(map[primitives.Slot]*Replayer)
	}
	b.forSlot[slot] = &Replayer{State: s}
}

func (b *ReplayerBuilder) SetMockSlotError(s primitives.Slot, e error) {
	if b.forSlot == nil {
		b.forSlot = make(map[primitives.Slot]*Replayer)
	}
	b.forSlot[s] = &Replayer{Err: e}
}

func (b *ReplayerBuilder) ReplayerForSlot(target primitives.Slot) stategen.Replayer {
	return b.forSlot[target]
}

var _ stategen.ReplayerBuilder = &ReplayerBuilder{}

type Replayer struct {
	State state.BeaconState
	Err   error
}

func (m *Replayer) ReplayBlocks(_ context.Context) (state.BeaconState, error) {
	return m.State, m.Err
}

func (m *Replayer) ReplayToSlot(_ context.Context, _ primitives.Slot) (state.BeaconState, error) {
	return m.State, m.Err
}

var _ stategen.Replayer = &Replayer{}

type CanonicalChecker struct {
	Is  bool
	Err error
}

func (m *CanonicalChecker) IsCanonical(_ context.Context, _ [32]byte) (bool, error) {
	return m.Is, m.Err
}

var _ stategen.CanonicalChecker = &CanonicalChecker{}

type CurrentSlotter struct {
	Slot primitives.Slot
}

func (c *CurrentSlotter) CurrentSlot() primitives.Slot {
	return c.Slot
}

var _ stategen.CurrentSlotter = &CurrentSlotter{}
