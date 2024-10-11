package stategen

import (
	"context"
	"fmt"
	"sort"
	"testing"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/blocks"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	consensusblocks "github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	blocktest "github.com/prysmaticlabs/prysm/v4/consensus-types/blocks/testing"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestMockHistoryStates(t *testing.T) {
	ctx := context.Background()
	var begin, middle, end primitives.Slot = 100, 150, 155
	specs := []mockHistorySpec{
		{slot: begin},
		{slot: middle, savedState: true},
		{slot: end, canonicalBlock: true},
	}
	hist := newMockHistory(t, specs, end+1)
	// we should have 2 "saved" states, genesis and "middle" (savedState == true)
	require.Equal(t, 2, len(hist.states))
	genesisRoot := hist.slotMap[0]
	st, err := hist.StateOrError(ctx, genesisRoot)
	require.NoError(t, err)
	expectedHTR, err := hist.states[genesisRoot].HashTreeRoot(ctx)
	require.NoError(t, err)
	actualHTR, err := st.HashTreeRoot(ctx)
	require.NoError(t, err)
	require.DeepEqual(t, expectedHTR, actualHTR)
	require.Equal(t, primitives.Slot(0), st.Slot())

	shouldExist, err := hist.StateOrError(ctx, hist.slotMap[middle])
	require.NoError(t, err)
	expectedHTR, err = hist.states[hist.slotMap[middle]].HashTreeRoot(ctx)
	require.NoError(t, err)
	actualHTR, err = shouldExist.HashTreeRoot(ctx)
	require.NoError(t, err)
	require.DeepEqual(t, expectedHTR, actualHTR)
	require.Equal(t, middle, shouldExist.Slot())

	cantExist, err := hist.StateOrError(ctx, hist.slotMap[end])
	require.ErrorIs(t, err, db.ErrNotFoundState)
	require.Equal(t, nil, cantExist)

	cantExist, err = hist.StateOrError(ctx, hist.slotMap[begin])
	require.ErrorIs(t, err, db.ErrNotFoundState)
	require.Equal(t, nil, cantExist)
}

func TestMockHistoryParentRoot(t *testing.T) {
	ctx := context.Background()
	var begin, middle, end primitives.Slot = 100, 150, 155
	specs := []mockHistorySpec{
		{slot: begin},
		{slot: middle, savedState: true},
		{slot: end, canonicalBlock: true},
	}
	hist := newMockHistory(t, specs, end+1)
	endRoot := hist.slotMap[end]
	endBlock, err := hist.Block(ctx, endRoot)
	require.NoError(t, err)
	// middle should be the parent of end, compare the middle root to endBlock's parent root
	require.Equal(t, hist.slotMap[middle], endBlock.Block().ParentRoot())
}

type mockHistorySpec struct {
	slot           primitives.Slot
	savedState     bool
	canonicalBlock bool
}

type mockHistory struct {
	blocks                         map[[32]byte]interfaces.ReadOnlySignedBeaconBlock
	slotMap                        map[primitives.Slot][32]byte
	slotIndex                      slotList
	canonical                      map[[32]byte]bool
	states                         map[[32]byte]state.BeaconState
	hiddenStates                   map[[32]byte]state.BeaconState
	current                        primitives.Slot
	overrideHighestSlotBlocksBelow func(context.Context, primitives.Slot) (primitives.Slot, [][32]byte, error)
}

type slotList []primitives.Slot

func (m slotList) Len() int {
	return len(m)
}

func (m slotList) Less(i, j int) bool {
	return m[i] < m[j]
}

func (m slotList) Swap(i, j int) {
	m[i], m[j] = m[j], m[i]
}

var errFallThroughOverride = errors.New("override yielding control back to real HighestRootsBelowSlot")

func (m *mockHistory) HighestRootsBelowSlot(_ context.Context, slot primitives.Slot) (primitives.Slot, [][32]byte, error) {
	if m.overrideHighestSlotBlocksBelow != nil {
		s, r, err := m.overrideHighestSlotBlocksBelow(context.Background(), slot)
		if !errors.Is(err, errFallThroughOverride) {
			return s, r, err
		}
	}
	if len(m.slotIndex) == 0 && len(m.slotMap) > 0 {
		for k := range m.slotMap {
			m.slotIndex = append(m.slotIndex, k)
		}
		sort.Sort(sort.Reverse(m.slotIndex))
	}
	for _, s := range m.slotIndex {
		if s < slot {
			return s, [][32]byte{m.slotMap[s]}, nil
		}
	}
	return 0, [][32]byte{}, nil
}

var errGenesisBlockNotFound = errors.New("canonical genesis block not found in db")

func (m *mockHistory) GenesisBlockRoot(_ context.Context) ([32]byte, error) {
	genesisRoot, ok := m.slotMap[0]
	if !ok {
		return [32]byte{}, errGenesisBlockNotFound
	}
	return genesisRoot, nil
}

func (m *mockHistory) Block(_ context.Context, blockRoot [32]byte) (interfaces.ReadOnlySignedBeaconBlock, error) {
	if b, ok := m.blocks[blockRoot]; ok {
		return b, nil
	}
	return nil, nil
}

func (m *mockHistory) StateOrError(_ context.Context, blockRoot [32]byte) (state.BeaconState, error) {
	if s, ok := m.states[blockRoot]; ok {
		return s.Copy(), nil
	}
	return nil, db.ErrNotFoundState
}

func (m *mockHistory) IsCanonical(_ context.Context, blockRoot [32]byte) (bool, error) {
	canon, ok := m.canonical[blockRoot]
	return ok && canon, nil
}

func (m *mockHistory) CurrentSlot() primitives.Slot {
	return m.current
}

func (h *mockHistory) addBlock(root [32]byte, b interfaces.ReadOnlySignedBeaconBlock, canon bool) {
	h.blocks[root] = b
	h.slotMap[b.Block().Slot()] = root
	h.canonical[root] = canon
}

func (h *mockHistory) addState(root [32]byte, s state.BeaconState) {
	h.states[root] = s
}

func (h *mockHistory) hideState(root [32]byte, s state.BeaconState) {
	h.hiddenStates[root] = s
}

func (h *mockHistory) validateRoots() error {
	uniqParentRoots := make(map[[32]byte]primitives.Slot)
	for s, root := range h.slotMap {
		b := h.blocks[root]
		htr, err := b.Block().HashTreeRoot()
		if err != nil {
			return errors.Wrap(err, fmt.Sprintf("error computing htr for block at slot %d", s))
		}
		if htr != root {
			return fmt.Errorf("htr mismatch, expected=%#x, actual=%#x", root, htr)
		}
		if ps, ok := uniqParentRoots[htr]; ok {
			return fmt.Errorf("duplicate parent_root %#x seen at slots %d, %d", htr, ps, s)
		}
		uniqParentRoots[htr] = s
	}
	return nil
}

func newMockHistory(t *testing.T, hist []mockHistorySpec, current primitives.Slot) *mockHistory {
	ctx := context.Background()
	mh := &mockHistory{
		blocks:       map[[32]byte]interfaces.ReadOnlySignedBeaconBlock{},
		canonical:    map[[32]byte]bool{},
		states:       map[[32]byte]state.BeaconState{},
		hiddenStates: map[[32]byte]state.BeaconState{},
		slotMap:      map[primitives.Slot][32]byte{},
		slotIndex:    slotList{},
		current:      current,
	}

	// genesis state for history
	gs, _ := util.DeterministicGenesisState(t, 32)
	gsr, err := gs.HashTreeRoot(ctx)
	require.NoError(t, err)

	// generate new genesis block using the root of the deterministic state
	gb, err := consensusblocks.NewSignedBeaconBlock(blocks.NewGenesisBlock(gsr[:]))
	require.NoError(t, err)
	pr, err := gb.Block().HashTreeRoot()
	require.NoError(t, err)

	// add genesis block as canonical
	mh.addBlock(pr, gb, true)
	// add genesis state, indexed by unapplied genesis block - genesis block is never really processed...
	mh.addState(pr, gs.Copy())

	ps := gs.Copy()
	for _, spec := range hist {
		// call process_slots and process_block separately, because process_slots updates values used in randao mix
		// which influences proposer_index.
		s, err := ReplayProcessSlots(ctx, ps, spec.slot)
		require.NoError(t, err)

		// create proposer block, setting values in the order seen in the validator.md spec
		b, err := consensusblocks.NewSignedBeaconBlock(util.NewBeaconBlock())
		require.NoError(t, err)

		// set slot to mock history spec value
		b, err = blocktest.SetBlockSlot(b, spec.slot)
		require.NoError(t, err)

		// set the correct proposer_index in the "proposal" block
		// so that it will pass validation in process_block. important that we do this
		// after process_slots!
		idx, err := helpers.BeaconProposerIndex(ctx, s)
		require.NoError(t, err)
		b, err = blocktest.SetProposerIndex(b, idx)
		require.NoError(t, err)

		// set parent root
		b, err = blocktest.SetBlockParentRoot(b, pr)
		require.NoError(t, err)

		// now do process_block only if block slot is greater than latest header slot
		if b.Block().Slot() > s.LatestBlockHeader().Slot {
			s, err = transition.ProcessBlockForStateRoot(ctx, s, b)
			require.NoError(t, err)
		}

		sr, err := s.HashTreeRoot(ctx)
		require.NoError(t, err)
		b, err = blocktest.SetBlockStateRoot(b, sr)
		require.NoError(t, err)

		pr, err = b.Block().HashTreeRoot()
		require.NoError(t, err)
		if spec.savedState {
			mh.addState(pr, s)
		} else {
			mh.hideState(pr, s)
		}
		mh.addBlock(pr, b, spec.canonicalBlock)
		ps = s.Copy()
	}

	require.NoError(t, mh.validateRoots())
	return mh
}

var _ HistoryAccessor = &mockHistory{}
var _ CanonicalChecker = &mockHistory{}
var _ CurrentSlotter = &mockHistory{}

type mockCachedGetter struct {
	cache map[[32]byte]state.BeaconState
}

func (m mockCachedGetter) ByBlockRoot(root [32]byte) (state.BeaconState, error) {
	st, ok := m.cache[root]
	if !ok {
		return nil, ErrNotInCache
	}
	return st, nil
}

var _ CachedGetter = &mockCachedGetter{}
