package backfill

import (
	"bytes"
	"context"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	blocktest "github.com/prysmaticlabs/prysm/v4/consensus-types/blocks/testing"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/proto/dbval"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

var errEmptyMockDBMethod = errors.New("uninitialized mock db method called")

type mockBackfillDB struct {
	saveBackfillBlockRoot     func(ctx context.Context, blockRoot [32]byte) error
	originCheckpointBlockRoot func(ctx context.Context) ([32]byte, error)
	block                     func(ctx context.Context, blockRoot [32]byte) (interfaces.ReadOnlySignedBeaconBlock, error)
	saveBackfillStatus        func(ctx context.Context, status *dbval.BackfillStatus) error
	backfillStatus            func(context.Context) (*dbval.BackfillStatus, error)
	status                    *dbval.BackfillStatus
	err                       error
	states                    map[[32]byte]state.BeaconState
	blocks                    map[[32]byte]blocks.ROBlock
}

var _ BeaconDB = &mockBackfillDB{}

func (d *mockBackfillDB) StateOrError(_ context.Context, blockRoot [32]byte) (state.BeaconState, error) {
	st, ok := d.states[blockRoot]
	if !ok {
		return nil, db.ErrNotFoundState
	}
	return st, nil
}

func (d *mockBackfillDB) SaveBackfillStatus(ctx context.Context, status *dbval.BackfillStatus) error {
	if d.saveBackfillStatus != nil {
		return d.saveBackfillStatus(ctx, status)
	}
	d.status = status
	return nil
}

func (d *mockBackfillDB) BackfillStatus(ctx context.Context) (*dbval.BackfillStatus, error) {
	if d.backfillStatus != nil {
		return d.backfillStatus(ctx)
	}
	return d.status, nil
}

func (d *mockBackfillDB) OriginCheckpointBlockRoot(ctx context.Context) ([32]byte, error) {
	if d.originCheckpointBlockRoot != nil {
		return d.originCheckpointBlockRoot(ctx)
	}
	return [32]byte{}, errEmptyMockDBMethod
}

func (d *mockBackfillDB) Block(ctx context.Context, blockRoot [32]byte) (interfaces.ReadOnlySignedBeaconBlock, error) {
	if d.block != nil {
		return d.block(ctx, blockRoot)
	}
	b, ok := d.blocks[blockRoot]
	if !ok {
		return nil, db.ErrNotFound
	}
	return b, nil
}

func (d *mockBackfillDB) SaveROBlocks(ctx context.Context, blks []blocks.ROBlock, cache bool) error {
	if d.blocks == nil {
		d.blocks = make(map[[32]byte]blocks.ROBlock)
	}
	for i := range blks {
		d.blocks[blks[i].Root()] = blks[i]
	}
	return nil
}

func (d *mockBackfillDB) BackfillFinalizedIndex(ctx context.Context, blocks []blocks.ROBlock, finalizedChildRoot [32]byte) error {
	return nil
}

func TestSlotCovered(t *testing.T) {
	cases := []struct {
		name   string
		slot   primitives.Slot
		status *Store
		result bool
	}{
		{
			name:   "genesis true",
			status: &Store{bs: &dbval.BackfillStatus{LowSlot: 10}},
			slot:   0,
			result: true,
		},
		{
			name:   "above end true",
			status: &Store{bs: &dbval.BackfillStatus{LowSlot: 1}},
			slot:   2,
			result: true,
		},
		{
			name:   "equal end true",
			status: &Store{bs: &dbval.BackfillStatus{LowSlot: 1}},
			slot:   1,
			result: true,
		},
		{
			name:   "genesisSync always true",
			status: &Store{genesisSync: true},
			slot:   100,
			result: true,
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			result := c.status.AvailableBlock(c.slot)
			require.Equal(t, c.result, result)
		})
	}
}

func TestStatusUpdater_FillBack(t *testing.T) {
	ctx := context.Background()
	mdb := &mockBackfillDB{}
	b, err := setupTestBlock(90)
	require.NoError(t, err)
	rob, err := blocks.NewROBlock(b)
	require.NoError(t, err)
	s := &Store{bs: &dbval.BackfillStatus{LowSlot: 100, LowParentRoot: rob.RootSlice()}, store: mdb}
	require.Equal(t, false, s.AvailableBlock(95))
	_, err = s.fillBack(ctx, []blocks.ROBlock{rob})
	require.NoError(t, err)
	require.Equal(t, true, s.AvailableBlock(95))
}

func goodBlockRoot(root [32]byte) func(ctx context.Context) ([32]byte, error) {
	return func(ctx context.Context) ([32]byte, error) {
		return root, nil
	}
}

func setupTestBlock(slot primitives.Slot) (interfaces.ReadOnlySignedBeaconBlock, error) {
	bRaw := util.NewBeaconBlock()
	b, err := blocks.NewSignedBeaconBlock(bRaw)
	if err != nil {
		return nil, err
	}
	return blocktest.SetBlockSlot(b, slot)
}

func TestNewUpdater(t *testing.T) {
	ctx := context.Background()

	originSlot := primitives.Slot(100)
	var originRoot [32]byte
	copy(originRoot[:], []byte{0x01})
	originBlock, err := setupTestBlock(originSlot)
	require.NoError(t, err)

	backfillSlot := primitives.Slot(50)
	var backfillRoot [32]byte
	copy(backfillRoot[:], []byte{0x02})
	backfillBlock, err := setupTestBlock(backfillSlot)
	require.NoError(t, err)
	var parentRoot [32]byte
	copy(parentRoot[:], []byte{0x03})
	var rootSlice = func(r [32]byte) []byte { return r[:] }
	typicalBackfillStatus := &dbval.BackfillStatus{
		LowSlot:       23,
		LowRoot:       backfillRoot[:],
		LowParentRoot: parentRoot[:],
		OriginSlot:    1123,
		OriginRoot:    originRoot[:],
	}
	cases := []struct {
		name     string
		db       BeaconDB
		err      error
		expected *Store
	}{
		{
			name: "origin not found, implying genesis sync ",
			db: &mockBackfillDB{
				backfillStatus: func(context.Context) (*dbval.BackfillStatus, error) {
					return nil, db.ErrNotFound
				},
				originCheckpointBlockRoot: func(ctx context.Context) ([32]byte, error) {
					return [32]byte{}, db.ErrNotFoundOriginBlockRoot
				}},
			expected: &Store{genesisSync: true},
		},
		{
			name: "legacy recovery",
			db: &mockBackfillDB{
				originCheckpointBlockRoot: goodBlockRoot(originRoot),
				block: func(ctx context.Context, root [32]byte) (interfaces.ReadOnlySignedBeaconBlock, error) {
					switch root {
					case originRoot:
						return originBlock, nil
					case backfillRoot:
						return backfillBlock, nil
					}
					return nil, errors.New("not derp")
				},
				backfillStatus: func(context.Context) (*dbval.BackfillStatus, error) { return nil, db.ErrNotFound },
			},
			expected: &Store{bs: &dbval.BackfillStatus{
				LowSlot: uint64(originSlot), OriginSlot: uint64(originSlot),
				LowRoot: originRoot[:], OriginRoot: originRoot[:], LowParentRoot: rootSlice(originBlock.Block().ParentRoot()),
			}},
		},
		{
			name: "backfill found",
			db: &mockBackfillDB{backfillStatus: func(ctx context.Context) (*dbval.BackfillStatus, error) {
				return typicalBackfillStatus, nil
			}},
			expected: &Store{bs: typicalBackfillStatus},
		},
	}

	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			s, err := NewUpdater(ctx, c.db)
			if c.err != nil {
				require.ErrorIs(t, err, c.err)
				return
			}
			require.NoError(t, err)
			if c.expected == nil {
				return
			}
			require.Equal(t, c.expected.genesisSync, s.genesisSync)
			if c.expected.genesisSync {
				require.IsNil(t, s.bs)
				return
			}
			require.Equal(t, c.expected.bs.LowSlot, s.bs.LowSlot)
			require.Equal(t, c.expected.bs.OriginSlot, s.bs.OriginSlot)
			require.Equal(t, true, bytes.Equal(c.expected.bs.OriginRoot, s.bs.OriginRoot))
			require.Equal(t, true, bytes.Equal(c.expected.bs.LowRoot, s.bs.LowRoot))
			require.Equal(t, true, bytes.Equal(c.expected.bs.LowParentRoot, s.bs.LowParentRoot))
		})
	}

}
