package pruning_test

import (
	"errors"
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"
	"go.uber.org/mock/gomock"

	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/log"
	"cosmossdk.io/store/mock"
	"cosmossdk.io/store/pruning"
	"cosmossdk.io/store/pruning/types"
)

const dbErr = "db error"

func TestNewManager(t *testing.T) {
	manager := pruning.NewManager(coretesting.NewMemDB(), log.NewNopLogger())
	require.NotNil(t, manager)
	require.Equal(t, types.PruningNothing, manager.GetOptions().GetPruningStrategy())
}

func TestStrategies(t *testing.T) {
	testcases := map[string]struct {
		strategy         types.PruningOptions
		snapshotInterval uint64
		strategyToAssert types.PruningStrategy
		isValid          bool
	}{
		"prune nothing - no snapshot": {
			strategy:         types.NewPruningOptions(types.PruningNothing),
			strategyToAssert: types.PruningNothing,
		},
		"prune nothing - snapshot": {
			strategy:         types.NewPruningOptions(types.PruningNothing),
			strategyToAssert: types.PruningNothing,
			snapshotInterval: 100,
		},
		"prune default - no snapshot": {
			strategy:         types.NewPruningOptions(types.PruningDefault),
			strategyToAssert: types.PruningDefault,
		},
		"prune default - snapshot": {
			strategy:         types.NewPruningOptions(types.PruningDefault),
			strategyToAssert: types.PruningDefault,
			snapshotInterval: 100,
		},
		"prune everything - no snapshot": {
			strategy:         types.NewPruningOptions(types.PruningEverything),
			strategyToAssert: types.PruningEverything,
		},
		"prune everything - snapshot": {
			strategy:         types.NewPruningOptions(types.PruningEverything),
			strategyToAssert: types.PruningEverything,
			snapshotInterval: 100,
		},
		"custom 100-10-15": {
			strategy:         types.NewCustomPruningOptions(100, 15),
			snapshotInterval: 10,
			strategyToAssert: types.PruningCustom,
		},
		"custom 10-10-15": {
			strategy:         types.NewCustomPruningOptions(10, 15),
			snapshotInterval: 10,
			strategyToAssert: types.PruningCustom,
		},
		"custom 100-0-15": {
			strategy:         types.NewCustomPruningOptions(100, 15),
			snapshotInterval: 0,
			strategyToAssert: types.PruningCustom,
		},
	}

	for name, tc := range testcases {
		tc := tc // Local copy to avoid shadowing.
		t.Run(name, func(t *testing.T) {
			t.Parallel()

			manager := pruning.NewManager(coretesting.NewMemDB(), log.NewNopLogger())
			require.NotNil(t, manager)

			curStrategy := tc.strategy
			manager.SetSnapshotInterval(tc.snapshotInterval)

			pruneStrategy := curStrategy.GetPruningStrategy()
			require.Equal(t, tc.strategyToAssert, pruneStrategy)

			// Validate strategy parameters
			switch pruneStrategy {
			case types.PruningDefault:
				require.Equal(t, uint64(362880), curStrategy.KeepRecent)
				require.Equal(t, uint64(10), curStrategy.Interval)
			case types.PruningNothing:
				require.Equal(t, uint64(0), curStrategy.KeepRecent)
				require.Equal(t, uint64(0), curStrategy.Interval)
			case types.PruningEverything:
				require.Equal(t, uint64(2), curStrategy.KeepRecent)
				require.Equal(t, uint64(10), curStrategy.Interval)
			default:
				//
			}

			manager.SetOptions(curStrategy)
			require.Equal(t, tc.strategy, manager.GetOptions())

			curKeepRecent := curStrategy.KeepRecent
			snHeight := int64(tc.snapshotInterval - 1)
			for curHeight := int64(0); curHeight < 110000; curHeight++ {
				if tc.snapshotInterval != 0 {
					if curHeight > int64(tc.snapshotInterval) && curHeight%int64(tc.snapshotInterval) == int64(tc.snapshotInterval)-1 {
						manager.HandleSnapshotHeight(curHeight - int64(tc.snapshotInterval) + 1)
						snHeight = curHeight
					}
				}

				pruningHeightActual := manager.GetPruningHeight(curHeight)
				curHeightStr := fmt.Sprintf("height: %d", curHeight)

				switch curStrategy.GetPruningStrategy() {
				case types.PruningNothing:
					require.Equal(t, int64(0), pruningHeightActual, curHeightStr)
				default:
					if curHeight > int64(curKeepRecent) && curHeight%int64(curStrategy.Interval) == 0 {
						pruningHeightExpected := curHeight - int64(curKeepRecent) - 1
						if tc.snapshotInterval > 0 && snHeight < pruningHeightExpected {
							pruningHeightExpected = snHeight
						}
						require.Equal(t, pruningHeightExpected, pruningHeightActual, curHeightStr)
					} else {
						require.Equal(t, int64(0), pruningHeightActual, curHeightStr)
					}
				}
			}
		})
	}
}

func TestPruningHeight_Inputs(t *testing.T) {
	keepRecent := int64(types.NewPruningOptions(types.PruningEverything).KeepRecent)
	interval := int64(types.NewPruningOptions(types.PruningEverything).Interval)

	testcases := map[string]struct {
		height         int64
		expectedResult int64
		strategy       types.PruningStrategy
	}{
		"currentHeight is negative - prune everything - invalid currentHeight": {
			-1,
			0,
			types.PruningEverything,
		},
		"currentHeight is  zero - prune everything - invalid currentHeight": {
			0,
			0,
			types.PruningEverything,
		},
		"currentHeight is positive but within keep recent- prune everything - not kept": {
			keepRecent,
			0,
			types.PruningEverything,
		},
		"currentHeight is positive and equal to keep recent+1 - no kept": {
			keepRecent + 1,
			0,
			types.PruningEverything,
		},
		"currentHeight is positive and greater than keep recent+1 but not multiple of interval - no kept": {
			keepRecent + 2,
			0,
			types.PruningEverything,
		},
		"currentHeight is positive and greater than keep recent+1 and multiple of interval - kept": {
			interval,
			interval - keepRecent - 1,
			types.PruningEverything,
		},
		"pruning nothing, currentHeight is positive and greater than keep recent - not kept": {
			interval,
			0,
			types.PruningNothing,
		},
	}

	for name, tc := range testcases {
		t.Run(name, func(t *testing.T) {
			manager := pruning.NewManager(coretesting.NewMemDB(), log.NewNopLogger())
			require.NotNil(t, manager)
			manager.SetOptions(types.NewPruningOptions(tc.strategy))

			pruningHeightActual := manager.GetPruningHeight(tc.height)
			require.Equal(t, tc.expectedResult, pruningHeightActual)
		})
	}
}

func TestHandleSnapshotHeight_DbErr_Panic(t *testing.T) {
	ctrl := gomock.NewController(t)

	// Setup
	dbMock := mock.NewMockKVStoreWithBatch(ctrl)

	dbMock.EXPECT().Set(gomock.Any(), gomock.Any()).Return(errors.New(dbErr)).Times(1)

	manager := pruning.NewManager(dbMock, log.NewNopLogger())
	manager.SetOptions(types.NewPruningOptions(types.PruningEverything))
	require.NotNil(t, manager)

	defer func() {
		if r := recover(); r == nil {
			t.Fail()
		}
	}()

	manager.HandleSnapshotHeight(10)
}

func TestHandleSnapshotHeight_LoadFromDisk(t *testing.T) {
	snapshotInterval := uint64(10)

	// Setup
	db := coretesting.NewMemDB()
	manager := pruning.NewManager(db, log.NewNopLogger())
	require.NotNil(t, manager)

	manager.SetOptions(types.NewPruningOptions(types.PruningEverything))
	manager.SetSnapshotInterval(snapshotInterval)

	expected := 0
	for snapshotHeight := int64(-1); snapshotHeight < 100; snapshotHeight++ {
		snapshotHeightStr := fmt.Sprintf("snaphost height: %d", snapshotHeight)
		if snapshotHeight > int64(snapshotInterval) && snapshotHeight%int64(snapshotInterval) == 1 {
			// Test flush
			manager.HandleSnapshotHeight(snapshotHeight - 1)
			expected = 1
		}

		loadedSnapshotHeights, err := pruning.LoadPruningSnapshotHeights(db)
		require.NoError(t, err)
		require.Equal(t, expected, len(loadedSnapshotHeights), snapshotHeightStr)

		// Test load back
		err = manager.LoadSnapshotHeights(db)
		require.NoError(t, err)

		loadedSnapshotHeights, err = pruning.LoadPruningSnapshotHeights(db)
		require.NoError(t, err)
		require.Equal(t, expected, len(loadedSnapshotHeights), snapshotHeightStr)
	}
}

func TestLoadPruningSnapshotHeights(t *testing.T) {
	var (
		manager = pruning.NewManager(coretesting.NewMemDB(), log.NewNopLogger())
		err     error
	)
	require.NotNil(t, manager)

	// must not be PruningNothing
	manager.SetOptions(types.NewPruningOptions(types.PruningDefault))

	testcases := map[string]struct {
		getFlushedPruningSnapshotHeights func() []int64
		expectedResult                   error
	}{
		"negative snapshotPruningHeight - error": {
			getFlushedPruningSnapshotHeights: func() []int64 {
				return []int64{5, -2, 3}
			},
			expectedResult: &pruning.NegativeHeightsError{Height: -2},
		},
		"non-negative - success": {
			getFlushedPruningSnapshotHeights: func() []int64 {
				return []int64{5, 0, 3}
			},
		},
	}

	for name, tc := range testcases {
		t.Run(name, func(t *testing.T) {
			db := coretesting.NewMemDB()

			if tc.getFlushedPruningSnapshotHeights != nil {
				err = db.Set(pruning.PruneSnapshotHeightsKey, pruning.Int64SliceToBytes(tc.getFlushedPruningSnapshotHeights()))
				require.NoError(t, err)
			}

			err = manager.LoadSnapshotHeights(db)
			require.Equal(t, tc.expectedResult, err)
		})
	}
}

func TestLoadSnapshotHeights_PruneNothing(t *testing.T) {
	manager := pruning.NewManager(coretesting.NewMemDB(), log.NewNopLogger())
	require.NotNil(t, manager)

	manager.SetOptions(types.NewPruningOptions(types.PruningNothing))

	require.Nil(t, manager.LoadSnapshotHeights(coretesting.NewMemDB()))
}
