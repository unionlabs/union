package kv

import (
	"context"
	"encoding/binary"
	"math/big"
	"math/rand"
	"strconv"
	"testing"
	"time"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	bolt "go.etcd.io/bbolt"
)

func TestStateNil(t *testing.T) {
	db := setupDB(t)
	_, err := db.StateOrError(context.Background(), [32]byte{})
	require.ErrorIs(t, err, ErrNotFoundState)
}

func TestState_CanSaveRetrieve(t *testing.T) {
	type testCase struct {
		name     string
		s        func() state.BeaconState
		rootSeed byte
	}

	cases := []testCase{
		{
			name: "phase0",
			s: func() state.BeaconState {
				st, err := util.NewBeaconState()
				require.NoError(t, err)
				require.NoError(t, st.SetSlot(100))
				return st
			},
			rootSeed: '0',
		},
		{
			name: "altair",
			s: func() state.BeaconState {
				st, err := util.NewBeaconStateAltair()
				require.NoError(t, err)
				require.NoError(t, st.SetSlot(100))
				return st
			},
			rootSeed: 'A',
		},
		{
			name: "bellatrix",
			s: func() state.BeaconState {
				st, err := util.NewBeaconStateBellatrix()
				require.NoError(t, err)
				require.NoError(t, st.SetSlot(100))
				p, err := blocks.WrappedExecutionPayloadHeader(&enginev1.ExecutionPayloadHeader{
					ParentHash:       make([]byte, 32),
					FeeRecipient:     make([]byte, 20),
					StateRoot:        make([]byte, 32),
					ReceiptsRoot:     make([]byte, 32),
					LogsBloom:        make([]byte, 256),
					PrevRandao:       make([]byte, 32),
					ExtraData:        []byte("foo"),
					BaseFeePerGas:    make([]byte, 32),
					BlockHash:        make([]byte, 32),
					TransactionsRoot: make([]byte, 32),
				})
				require.NoError(t, err)
				require.NoError(t, st.SetLatestExecutionPayloadHeader(p))
				return st
			},
			rootSeed: 'B',
		},
		{
			name: "capella",
			s: func() state.BeaconState {
				st, err := util.NewBeaconStateCapella()
				require.NoError(t, err)
				require.NoError(t, st.SetSlot(100))
				p, err := blocks.WrappedExecutionPayloadHeaderCapella(&enginev1.ExecutionPayloadHeaderCapella{
					ParentHash:       make([]byte, 32),
					FeeRecipient:     make([]byte, 20),
					StateRoot:        make([]byte, 32),
					ReceiptsRoot:     make([]byte, 32),
					LogsBloom:        make([]byte, 256),
					PrevRandao:       make([]byte, 32),
					ExtraData:        []byte("foo"),
					BaseFeePerGas:    make([]byte, 32),
					BlockHash:        make([]byte, 32),
					TransactionsRoot: make([]byte, 32),
					WithdrawalsRoot:  make([]byte, 32),
				}, big.NewInt(0))
				require.NoError(t, err)
				require.NoError(t, st.SetLatestExecutionPayloadHeader(p))
				return st
			},
			rootSeed: 'C',
		},
		{
			name: "deneb",
			s: func() state.BeaconState {
				st, err := util.NewBeaconStateDeneb()
				require.NoError(t, err)
				require.NoError(t, st.SetSlot(100))
				p, err := blocks.WrappedExecutionPayloadHeaderDeneb(&enginev1.ExecutionPayloadHeaderDeneb{
					ParentHash:       make([]byte, 32),
					FeeRecipient:     make([]byte, 20),
					StateRoot:        make([]byte, 32),
					ReceiptsRoot:     make([]byte, 32),
					LogsBloom:        make([]byte, 256),
					PrevRandao:       make([]byte, 32),
					ExtraData:        []byte("foo"),
					BaseFeePerGas:    make([]byte, 32),
					BlockHash:        make([]byte, 32),
					TransactionsRoot: make([]byte, 32),
					WithdrawalsRoot:  make([]byte, 32),
				}, big.NewInt(0))
				require.NoError(t, err)
				require.NoError(t, st.SetLatestExecutionPayloadHeader(p))
				return st
			},
			rootSeed: 'D',
		},
	}

	db := setupDB(t)

	for _, enableFlag := range []bool{true, false} {
		reset := features.InitWithReset(&features.Flags{EnableHistoricalSpaceRepresentation: enableFlag})

		for _, tc := range cases {
			t.Run(tc.name+" - EnableHistoricalSpaceRepresentation is "+strconv.FormatBool(enableFlag), func(t *testing.T) {
				rootNonce := byte('0')
				if enableFlag {
					rootNonce = '1'
				}
				root := bytesutil.ToBytes32([]byte{tc.rootSeed, rootNonce})
				require.Equal(t, false, db.HasState(context.Background(), root))
				st := tc.s()

				require.NoError(t, db.SaveState(context.Background(), st, root))
				assert.Equal(t, true, db.HasState(context.Background(), root))

				savedSt, err := db.State(context.Background(), root)
				require.NoError(t, err)

				assert.DeepSSZEqual(t, st.ToProtoUnsafe(), savedSt.ToProtoUnsafe())
			})
		}

		reset()
	}
}

func TestState_CanSaveRetrieveValidatorEntries(t *testing.T) {
	db := setupDB(t)

	// enable historical state representation flag to test this
	resetCfg := features.InitWithReset(&features.Flags{
		EnableHistoricalSpaceRepresentation: true,
	})
	defer resetCfg()

	r := [32]byte{'A'}

	require.Equal(t, false, db.HasState(context.Background(), r))

	stateValidators := validators(10)
	st, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, st.SetSlot(100))
	require.NoError(t, st.SetValidators(stateValidators))

	ctx := context.Background()
	require.NoError(t, db.SaveState(ctx, st, r))
	assert.Equal(t, true, db.HasState(context.Background(), r))

	savedS, err := db.State(context.Background(), r)
	require.NoError(t, err)

	require.DeepSSZEqual(t, st.ToProtoUnsafe(), savedS.ToProtoUnsafe(), "saved state with validators and retrieved state are not matching")

	// check if the index of the second state is still present.
	err = db.db.Update(func(tx *bolt.Tx) error {
		idxBkt := tx.Bucket(blockRootValidatorHashesBucket)
		data := idxBkt.Get(r[:])
		require.NotEqual(t, 0, len(data))
		return nil
	})
	require.NoError(t, err)

	// check if all the validator entries are still intact in the validator entry bucket.
	err = db.db.Update(func(tx *bolt.Tx) error {
		valBkt := tx.Bucket(stateValidatorsBucket)
		// if any of the original validator entry is not present, then fail the test.
		for _, val := range stateValidators {
			hash, hashErr := val.HashTreeRoot()
			assert.NoError(t, hashErr)
			data := valBkt.Get(hash[:])
			require.NotNil(t, data)
			require.NotEqual(t, 0, len(data))
		}
		return nil
	})
	require.NoError(t, err)
}

func TestStateAltair_CanSaveRetrieveValidatorEntries(t *testing.T) {
	db := setupDB(t)

	// enable historical state representation flag to test this
	resetCfg := features.InitWithReset(&features.Flags{
		EnableHistoricalSpaceRepresentation: true,
	})
	defer resetCfg()

	r := [32]byte{'A'}

	require.Equal(t, false, db.HasState(context.Background(), r))

	stateValidators := validators(10)
	st, _ := util.DeterministicGenesisStateAltair(t, 20)
	require.NoError(t, st.SetSlot(100))
	require.NoError(t, st.SetValidators(stateValidators))

	ctx := context.Background()
	require.NoError(t, db.SaveState(ctx, st, r))
	assert.Equal(t, true, db.HasState(context.Background(), r))

	savedS, err := db.State(context.Background(), r)
	require.NoError(t, err)

	require.DeepSSZEqual(t, st.ToProtoUnsafe(), savedS.ToProtoUnsafe(), "saved state with validators and retrieved state are not matching")

	// check if the index of the second state is still present.
	err = db.db.Update(func(tx *bolt.Tx) error {
		idxBkt := tx.Bucket(blockRootValidatorHashesBucket)
		data := idxBkt.Get(r[:])
		require.NotEqual(t, 0, len(data))
		return nil
	})
	require.NoError(t, err)

	// check if all the validator entries are still intact in the validator entry bucket.
	err = db.db.Update(func(tx *bolt.Tx) error {
		valBkt := tx.Bucket(stateValidatorsBucket)
		// if any of the original validator entry is not present, then fail the test.
		for _, val := range stateValidators {
			hash, hashErr := val.HashTreeRoot()
			assert.NoError(t, hashErr)
			data := valBkt.Get(hash[:])
			require.NotNil(t, data)
			require.NotEqual(t, 0, len(data))
		}
		return nil
	})
	require.NoError(t, err)
}

func TestState_CanSaveRetrieveValidatorEntriesFromCache(t *testing.T) {
	db := setupDB(t)

	// enable historical state representation flag to test this
	resetCfg := features.InitWithReset(&features.Flags{
		EnableHistoricalSpaceRepresentation: true,
	})
	defer resetCfg()

	r := [32]byte{'A'}

	require.Equal(t, false, db.HasState(context.Background(), r))

	stateValidators := validators(10)
	st, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, st.SetSlot(100))
	require.NoError(t, st.SetValidators(stateValidators))

	ctx := context.Background()
	require.NoError(t, db.SaveState(ctx, st, r))
	assert.Equal(t, true, db.HasState(context.Background(), r))

	// check if the state is in cache
	for i := 0; i < len(stateValidators); i++ {
		hash, hashErr := stateValidators[i].HashTreeRoot()
		assert.NoError(t, hashErr)

		data, ok := db.validatorEntryCache.Get(string(hash[:]))
		assert.Equal(t, true, ok)
		require.NotNil(t, data)

		valEntry, vType := data.(*ethpb.Validator)
		assert.Equal(t, true, vType)
		require.NotNil(t, valEntry)

		require.DeepSSZEqual(t, stateValidators[i], valEntry, "validator entry is not matching")
	}

	// check if all the validator entries are still intact in the validator entry bucket.
	err = db.db.Update(func(tx *bolt.Tx) error {
		valBkt := tx.Bucket(stateValidatorsBucket)
		// if any of the original validator entry is not present, then fail the test.
		for _, val := range stateValidators {
			hash, hashErr := val.HashTreeRoot()
			assert.NoError(t, hashErr)
			data := valBkt.Get(hash[:])
			require.NotNil(t, data)
			require.NotEqual(t, 0, len(data))
		}
		return nil
	})
	require.NoError(t, err)

}

func TestState_CanSaveRetrieveValidatorEntriesWithoutCache(t *testing.T) {
	db := setupDB(t)

	// enable historical state representation flag to test this
	resetCfg := features.InitWithReset(&features.Flags{
		EnableHistoricalSpaceRepresentation: true,
	})
	defer resetCfg()

	r := [32]byte{'A'}

	require.Equal(t, false, db.HasState(context.Background(), r))

	stateValidators := validators(10)
	st, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, st.SetSlot(100))
	require.NoError(t, st.SetValidators(stateValidators))

	ctx := context.Background()
	require.NoError(t, db.SaveState(ctx, st, r))
	assert.Equal(t, true, db.HasState(context.Background(), r))
	db.validatorEntryCache.Clear()

	savedS, err := db.State(context.Background(), r)
	require.NoError(t, err)

	require.DeepSSZEqual(t, st.ToProtoUnsafe(), savedS.ToProtoUnsafe(), "saved state with validators and retrieved state are not matching")

	// check if the index of the second state is still present.
	err = db.db.Update(func(tx *bolt.Tx) error {
		idxBkt := tx.Bucket(blockRootValidatorHashesBucket)
		data := idxBkt.Get(r[:])
		require.NotEqual(t, 0, len(data))
		return nil
	})
	require.NoError(t, err)

	// check if all the validator entries are still intact in the validator entry bucket.
	err = db.db.Update(func(tx *bolt.Tx) error {
		valBkt := tx.Bucket(stateValidatorsBucket)
		// if any of the original validator entry is not present, then fail the test.
		for _, val := range stateValidators {
			hash, hashErr := val.HashTreeRoot()
			assert.NoError(t, hashErr)
			data := valBkt.Get(hash[:])
			require.NotNil(t, data)
			require.NotEqual(t, 0, len(data))
		}
		return nil
	})
	require.NoError(t, err)

}

func TestState_DeleteState(t *testing.T) {
	db := setupDB(t)

	// enable historical state representation flag to test this
	resetCfg := features.InitWithReset(&features.Flags{
		EnableHistoricalSpaceRepresentation: true,
	})
	defer resetCfg()

	r1 := [32]byte{'A'}
	r2 := [32]byte{'B'}

	require.Equal(t, false, db.HasState(context.Background(), r1))
	require.Equal(t, false, db.HasState(context.Background(), r2))

	// create two states with the same set of validators.
	stateValidators := validators(10)
	st1, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, st1.SetSlot(100))
	require.NoError(t, st1.SetValidators(stateValidators))

	st2, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, st2.SetSlot(101))
	require.NoError(t, st2.SetValidators(stateValidators))

	// save both the states.
	ctx := context.Background()
	require.NoError(t, db.SaveState(ctx, st1, r1))
	require.NoError(t, db.SaveState(ctx, st2, r2))

	// delete the first state.
	var deleteBlockRoots [][32]byte
	deleteBlockRoots = append(deleteBlockRoots, r1)
	require.NoError(t, db.DeleteStates(ctx, deleteBlockRoots))

	// check if the validator entries of this state is removed from cache.
	for _, val := range stateValidators {
		hash, hashErr := val.HashTreeRoot()
		assert.NoError(t, hashErr)
		v, found := db.validatorEntryCache.Get(hash[:])
		require.Equal(t, false, found)
		require.Equal(t, nil, v)
	}

	// check if the index of the first state is deleted.
	err = db.db.Update(func(tx *bolt.Tx) error {
		idxBkt := tx.Bucket(blockRootValidatorHashesBucket)
		data := idxBkt.Get(r1[:])
		require.Equal(t, 0, len(data))
		return nil
	})
	require.NoError(t, err)

	// check if the index of the second state is still present.
	err = db.db.Update(func(tx *bolt.Tx) error {
		idxBkt := tx.Bucket(blockRootValidatorHashesBucket)
		data := idxBkt.Get(r2[:])
		require.NotEqual(t, 0, len(data))
		return nil
	})
	require.NoError(t, err)

	// check if all the validator entries are still intact in the validator entry bucket.
	err = db.db.Update(func(tx *bolt.Tx) error {
		valBkt := tx.Bucket(stateValidatorsBucket)
		// if any of the original validator entry is not present, then fail the test.
		for _, val := range stateValidators {
			hash, hashErr := val.HashTreeRoot()
			assert.NoError(t, hashErr)
			data := valBkt.Get(hash[:])
			require.NotNil(t, data)
			require.NotEqual(t, 0, len(data))
		}
		return nil
	})
	require.NoError(t, err)
}

func TestGenesisState_CanSaveRetrieve(t *testing.T) {
	db := setupDB(t)

	headRoot := [32]byte{'B'}

	st, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, st.SetSlot(1))
	require.NoError(t, db.SaveGenesisBlockRoot(context.Background(), headRoot))
	require.NoError(t, db.SaveState(context.Background(), st, headRoot))

	savedGenesisS, err := db.GenesisState(context.Background())
	require.NoError(t, err)
	assert.DeepSSZEqual(t, st.ToProtoUnsafe(), savedGenesisS.ToProtoUnsafe(), "Did not retrieve saved state")
	require.NoError(t, db.SaveGenesisBlockRoot(context.Background(), [32]byte{'C'}))
}

func TestStore_StatesBatchDelete(t *testing.T) {
	db := setupDB(t)
	ctx := context.Background()
	numBlocks := 100
	totalBlocks := make([]interfaces.ReadOnlySignedBeaconBlock, numBlocks)
	blockRoots := make([][32]byte, 0)
	evenBlockRoots := make([][32]byte, 0)
	for i := 0; i < len(totalBlocks); i++ {
		b := util.NewBeaconBlock()
		b.Block.Slot = primitives.Slot(i)
		var err error
		totalBlocks[i], err = blocks.NewSignedBeaconBlock(b)
		require.NoError(t, err)
		r, err := totalBlocks[i].Block().HashTreeRoot()
		require.NoError(t, err)
		st, err := util.NewBeaconState()
		require.NoError(t, err)
		require.NoError(t, st.SetSlot(primitives.Slot(i)))
		require.NoError(t, db.SaveState(context.Background(), st, r))
		blockRoots = append(blockRoots, r)
		if i%2 == 0 {
			evenBlockRoots = append(evenBlockRoots, r)
		}
	}
	require.NoError(t, db.SaveBlocks(ctx, totalBlocks))
	// We delete all even indexed states.
	require.NoError(t, db.DeleteStates(ctx, evenBlockRoots))
	// When we retrieve the data, only the odd indexed state should remain.
	for _, r := range blockRoots {
		s, err := db.State(context.Background(), r)
		require.NoError(t, err)
		if s == nil {
			continue
		}
		assert.Equal(t, primitives.Slot(1), s.Slot()%2, "State with slot %d should have been deleted", s.Slot())
	}
}

func TestStore_DeleteGenesisState(t *testing.T) {
	db := setupDB(t)
	ctx := context.Background()

	genesisBlockRoot := [32]byte{'A'}
	require.NoError(t, db.SaveGenesisBlockRoot(ctx, genesisBlockRoot))
	st, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, st.SetSlot(100))
	require.NoError(t, db.SaveState(ctx, st, genesisBlockRoot))
	wantedErr := "cannot delete finalized block or state"
	assert.ErrorContains(t, wantedErr, db.DeleteState(ctx, genesisBlockRoot))
}

func TestStore_DeleteFinalizedState(t *testing.T) {
	db := setupDB(t)
	ctx := context.Background()

	genesis := bytesutil.ToBytes32([]byte{'G', 'E', 'N', 'E', 'S', 'I', 'S'})
	require.NoError(t, db.SaveGenesisBlockRoot(ctx, genesis))

	blk := util.NewBeaconBlock()
	blk.Block.ParentRoot = genesis[:]
	blk.Block.Slot = 100

	wsb, err := blocks.NewSignedBeaconBlock(blk)
	require.NoError(t, err)
	require.NoError(t, db.SaveBlock(ctx, wsb))

	finalizedBlockRoot, err := blk.Block.HashTreeRoot()
	require.NoError(t, err)

	finalizedState, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, finalizedState.SetSlot(100))
	require.NoError(t, db.SaveState(ctx, finalizedState, finalizedBlockRoot))
	finalizedCheckpoint := &ethpb.Checkpoint{Root: finalizedBlockRoot[:]}
	require.NoError(t, db.SaveFinalizedCheckpoint(ctx, finalizedCheckpoint))
	wantedErr := "cannot delete finalized block or state"
	assert.ErrorContains(t, wantedErr, db.DeleteState(ctx, finalizedBlockRoot))
}

func TestStore_DeleteHeadState(t *testing.T) {
	db := setupDB(t)
	ctx := context.Background()

	genesis := bytesutil.ToBytes32([]byte{'G', 'E', 'N', 'E', 'S', 'I', 'S'})
	require.NoError(t, db.SaveGenesisBlockRoot(ctx, genesis))

	blk := util.NewBeaconBlock()
	blk.Block.ParentRoot = genesis[:]
	blk.Block.Slot = 100
	wsb, err := blocks.NewSignedBeaconBlock(blk)
	require.NoError(t, err)
	require.NoError(t, db.SaveBlock(ctx, wsb))

	headBlockRoot, err := blk.Block.HashTreeRoot()
	require.NoError(t, err)
	st, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, st.SetSlot(100))
	require.NoError(t, db.SaveState(ctx, st, headBlockRoot))
	require.NoError(t, db.SaveHeadBlockRoot(ctx, headBlockRoot))
	require.NoError(t, db.DeleteState(ctx, headBlockRoot)) // Ok to delete head state if it's optimistic.
}

func TestStore_SaveDeleteState_CanGetHighestBelow(t *testing.T) {
	db := setupDB(t)

	b := util.NewBeaconBlock()
	b.Block.Slot = 1
	r, err := b.Block.HashTreeRoot()
	require.NoError(t, err)
	wsb, err := blocks.NewSignedBeaconBlock(b)
	require.NoError(t, err)
	require.NoError(t, db.SaveBlock(context.Background(), wsb))
	st, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, st.SetSlot(1))
	s0 := st.ToProtoUnsafe()
	require.NoError(t, db.SaveState(context.Background(), st, r))

	b.Block.Slot = 100
	r1, err := b.Block.HashTreeRoot()
	require.NoError(t, err)
	wsb, err = blocks.NewSignedBeaconBlock(b)
	require.NoError(t, err)
	require.NoError(t, db.SaveBlock(context.Background(), wsb))
	st, err = util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, st.SetSlot(100))
	s1 := st.ToProtoUnsafe()
	require.NoError(t, db.SaveState(context.Background(), st, r1))

	b.Block.Slot = 1000
	r2, err := b.Block.HashTreeRoot()
	require.NoError(t, err)
	wsb, err = blocks.NewSignedBeaconBlock(b)
	require.NoError(t, err)
	require.NoError(t, db.SaveBlock(context.Background(), wsb))
	st, err = util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, st.SetSlot(1000))
	s2 := st.ToProtoUnsafe()

	require.NoError(t, db.SaveState(context.Background(), st, r2))

	highest, err := db.HighestSlotStatesBelow(context.Background(), 2)
	require.NoError(t, err)
	assert.DeepSSZEqual(t, highest[0].ToProtoUnsafe(), s0)

	highest, err = db.HighestSlotStatesBelow(context.Background(), 101)
	require.NoError(t, err)
	assert.DeepSSZEqual(t, highest[0].ToProtoUnsafe(), s1)

	highest, err = db.HighestSlotStatesBelow(context.Background(), 1001)
	require.NoError(t, err)
	assert.DeepSSZEqual(t, highest[0].ToProtoUnsafe(), s2)
}

func TestStore_GenesisState_CanGetHighestBelow(t *testing.T) {
	db := setupDB(t)

	genesisState, err := util.NewBeaconState()
	require.NoError(t, err)
	genesisRoot := [32]byte{'a'}
	require.NoError(t, db.SaveGenesisBlockRoot(context.Background(), genesisRoot))
	require.NoError(t, db.SaveState(context.Background(), genesisState, genesisRoot))

	b := util.NewBeaconBlock()
	b.Block.Slot = 1
	r, err := b.Block.HashTreeRoot()
	require.NoError(t, err)
	wsb, err := blocks.NewSignedBeaconBlock(b)
	require.NoError(t, err)
	require.NoError(t, db.SaveBlock(context.Background(), wsb))

	st, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, st.SetSlot(1))
	require.NoError(t, db.SaveState(context.Background(), st, r))

	highest, err := db.HighestSlotStatesBelow(context.Background(), 2)
	require.NoError(t, err)
	assert.DeepSSZEqual(t, highest[0].ToProtoUnsafe(), st.ToProtoUnsafe())

	highest, err = db.HighestSlotStatesBelow(context.Background(), 1)
	require.NoError(t, err)
	assert.DeepSSZEqual(t, highest[0].ToProtoUnsafe(), genesisState.ToProtoUnsafe())
	highest, err = db.HighestSlotStatesBelow(context.Background(), 0)
	require.NoError(t, err)
	assert.DeepSSZEqual(t, highest[0].ToProtoUnsafe(), genesisState.ToProtoUnsafe())
}

func TestStore_CleanUpDirtyStates_AboveThreshold(t *testing.T) {
	db := setupDB(t)

	genesisState, err := util.NewBeaconState()
	require.NoError(t, err)
	genesisRoot := [32]byte{'a'}
	require.NoError(t, db.SaveGenesisBlockRoot(context.Background(), genesisRoot))
	require.NoError(t, db.SaveState(context.Background(), genesisState, genesisRoot))
	require.NoError(t, db.SaveOriginCheckpointBlockRoot(context.Background(), [32]byte{'a'}))

	bRoots := make([][32]byte, 0)
	slotsPerArchivedPoint := primitives.Slot(128)
	prevRoot := genesisRoot
	for i := primitives.Slot(1); i <= slotsPerArchivedPoint; i++ {
		b := util.NewBeaconBlock()
		b.Block.Slot = i
		b.Block.ParentRoot = prevRoot[:]
		r, err := b.Block.HashTreeRoot()
		require.NoError(t, err)
		wsb, err := blocks.NewSignedBeaconBlock(b)
		require.NoError(t, err)
		require.NoError(t, db.SaveBlock(context.Background(), wsb))
		bRoots = append(bRoots, r)
		prevRoot = r

		st, err := util.NewBeaconState()
		require.NoError(t, err)
		require.NoError(t, st.SetSlot(i))
		require.NoError(t, db.SaveState(context.Background(), st, r))
	}

	require.NoError(t, db.SaveFinalizedCheckpoint(context.Background(), &ethpb.Checkpoint{
		Root:  bRoots[len(bRoots)-1][:],
		Epoch: primitives.Epoch(slotsPerArchivedPoint / params.BeaconConfig().SlotsPerEpoch),
	}))
	require.NoError(t, db.CleanUpDirtyStates(context.Background(), slotsPerArchivedPoint))

	for i, root := range bRoots {
		if primitives.Slot(i) >= slotsPerArchivedPoint.SubSlot(slotsPerArchivedPoint.Div(3)) {
			require.Equal(t, true, db.HasState(context.Background(), root))
		} else {
			require.Equal(t, false, db.HasState(context.Background(), root))
		}
	}
}

func TestStore_CleanUpDirtyStates_Finalized(t *testing.T) {
	db := setupDB(t)

	genesisState, err := util.NewBeaconState()
	require.NoError(t, err)
	genesisRoot := [32]byte{'a'}
	require.NoError(t, db.SaveGenesisBlockRoot(context.Background(), genesisRoot))
	require.NoError(t, db.SaveState(context.Background(), genesisState, genesisRoot))
	require.NoError(t, db.SaveOriginCheckpointBlockRoot(context.Background(), [32]byte{'a'}))

	for i := primitives.Slot(1); i <= params.BeaconConfig().SlotsPerEpoch; i++ {
		b := util.NewBeaconBlock()
		b.Block.Slot = i
		r, err := b.Block.HashTreeRoot()
		require.NoError(t, err)
		wsb, err := blocks.NewSignedBeaconBlock(b)
		require.NoError(t, err)
		require.NoError(t, db.SaveBlock(context.Background(), wsb))

		st, err := util.NewBeaconState()
		require.NoError(t, err)
		require.NoError(t, st.SetSlot(i))
		require.NoError(t, db.SaveState(context.Background(), st, r))
	}

	require.NoError(t, db.SaveFinalizedCheckpoint(context.Background(), &ethpb.Checkpoint{Root: genesisRoot[:]}))
	require.NoError(t, db.CleanUpDirtyStates(context.Background(), params.BeaconConfig().SlotsPerEpoch))
	require.Equal(t, true, db.HasState(context.Background(), genesisRoot))
}

func TestStore_CleanUpDirtyStates_OriginRoot(t *testing.T) {
	db := setupDB(t)

	genesisState, err := util.NewBeaconState()
	require.NoError(t, err)
	r := [32]byte{'a'}
	require.NoError(t, db.SaveGenesisBlockRoot(context.Background(), r))
	require.NoError(t, db.SaveState(context.Background(), genesisState, r))

	for i := primitives.Slot(1); i <= params.BeaconConfig().SlotsPerEpoch; i++ {
		b := util.NewBeaconBlock()
		b.Block.Slot = i
		r, err := b.Block.HashTreeRoot()
		require.NoError(t, err)
		wsb, err := blocks.NewSignedBeaconBlock(b)
		require.NoError(t, err)
		require.NoError(t, db.SaveBlock(context.Background(), wsb))

		st, err := util.NewBeaconState()
		require.NoError(t, err)
		require.NoError(t, st.SetSlot(i))
		require.NoError(t, db.SaveState(context.Background(), st, r))
	}

	require.NoError(t, db.SaveOriginCheckpointBlockRoot(context.Background(), r))
	require.NoError(t, db.CleanUpDirtyStates(context.Background(), params.BeaconConfig().SlotsPerEpoch))
	require.Equal(t, true, db.HasState(context.Background(), r))
}

func TestStore_CleanUpDirtyStates_DontDeleteNonFinalized(t *testing.T) {
	db := setupDB(t)

	genesisState, err := util.NewBeaconState()
	require.NoError(t, err)
	genesisRoot := [32]byte{'a'}
	require.NoError(t, db.SaveGenesisBlockRoot(context.Background(), genesisRoot))
	require.NoError(t, db.SaveState(context.Background(), genesisState, genesisRoot))
	require.NoError(t, db.SaveOriginCheckpointBlockRoot(context.Background(), [32]byte{'a'}))

	var unfinalizedRoots [][32]byte
	for i := primitives.Slot(1); i <= params.BeaconConfig().SlotsPerEpoch; i++ {
		b := util.NewBeaconBlock()
		b.Block.Slot = i
		r, err := b.Block.HashTreeRoot()
		require.NoError(t, err)
		wsb, err := blocks.NewSignedBeaconBlock(b)
		require.NoError(t, err)
		require.NoError(t, db.SaveBlock(context.Background(), wsb))
		unfinalizedRoots = append(unfinalizedRoots, r)

		st, err := util.NewBeaconState()
		require.NoError(t, err)
		require.NoError(t, st.SetSlot(i))
		require.NoError(t, db.SaveState(context.Background(), st, r))
	}

	require.NoError(t, db.SaveFinalizedCheckpoint(context.Background(), &ethpb.Checkpoint{Root: genesisRoot[:]}))
	require.NoError(t, db.CleanUpDirtyStates(context.Background(), params.BeaconConfig().SlotsPerEpoch))

	for _, rt := range unfinalizedRoots {
		require.Equal(t, true, db.HasState(context.Background(), rt))
	}
}

func TestAltairState_CanSaveRetrieve(t *testing.T) {
	db := setupDB(t)

	r := [32]byte{'A'}

	require.Equal(t, false, db.HasState(context.Background(), r))

	st, _ := util.DeterministicGenesisStateAltair(t, 1)
	require.NoError(t, st.SetSlot(100))

	require.NoError(t, db.SaveState(context.Background(), st, r))
	require.Equal(t, true, db.HasState(context.Background(), r))

	savedS, err := db.State(context.Background(), r)
	require.NoError(t, err)

	require.DeepSSZEqual(t, st.ToProtoUnsafe(), savedS.ToProtoUnsafe())

	savedS, err = db.State(context.Background(), [32]byte{'B'})
	require.NoError(t, err)
	require.Equal(t, state.ReadOnlyBeaconState(nil), savedS, "Unsaved state should've been nil")
}

func TestAltairState_CanDelete(t *testing.T) {
	db := setupDB(t)

	r := [32]byte{'A'}

	require.Equal(t, false, db.HasState(context.Background(), r))

	st, _ := util.DeterministicGenesisStateAltair(t, 1)
	require.NoError(t, st.SetSlot(100))

	require.NoError(t, db.SaveState(context.Background(), st, r))
	require.Equal(t, true, db.HasState(context.Background(), r))

	require.NoError(t, db.DeleteState(context.Background(), r))
	savedS, err := db.State(context.Background(), r)
	require.NoError(t, err)
	require.Equal(t, state.ReadOnlyBeaconState(nil), savedS, "Unsaved state should've been nil")
}

func validators(limit int) []*ethpb.Validator {
	var vals []*ethpb.Validator
	for i := 0; i < limit; i++ {
		pubKey := make([]byte, params.BeaconConfig().BLSPubkeyLength)
		binary.LittleEndian.PutUint64(pubKey, rand.Uint64())
		val := &ethpb.Validator{
			PublicKey:                  pubKey,
			WithdrawalCredentials:      bytesutil.ToBytes(rand.Uint64(), 32),
			EffectiveBalance:           rand.Uint64(),
			Slashed:                    i%2 != 0,
			ActivationEligibilityEpoch: primitives.Epoch(rand.Uint64()),
			ActivationEpoch:            primitives.Epoch(rand.Uint64()),
			ExitEpoch:                  primitives.Epoch(rand.Uint64()),
			WithdrawableEpoch:          primitives.Epoch(rand.Uint64()),
		}
		vals = append(vals, val)
	}
	return vals
}

func checkStateSaveTime(b *testing.B, saveCount int) {
	b.StopTimer()

	db := setupDB(b)
	initialSetOfValidators := validators(100000)

	// construct some states and save to randomize benchmark.
	for i := 0; i < saveCount; i++ {
		key := make([]byte, 32)
		_, err := rand.Read(key)
		require.NoError(b, err)
		st, err := util.NewBeaconState()
		require.NoError(b, err)

		// Add some more new validator to the base validator.
		validatosToAddInTest := validators(10000)
		allValidators := append(initialSetOfValidators, validatosToAddInTest...)

		// shuffle validators.
		rand.Seed(time.Now().UnixNano())
		rand.Shuffle(len(allValidators), func(i, j int) { allValidators[i], allValidators[j] = allValidators[j], allValidators[i] })

		require.NoError(b, st.SetValidators(allValidators))
		require.NoError(b, db.SaveState(context.Background(), st, bytesutil.ToBytes32(key)))
	}

	// create a state to save in benchmark
	r := [32]byte{'A'}
	st, err := util.NewBeaconState()
	require.NoError(b, err)
	require.NoError(b, st.SetValidators(initialSetOfValidators))

	b.ReportAllocs()
	b.StartTimer()
	for i := 0; i < b.N; i++ {
		require.NoError(b, db.SaveState(context.Background(), st, r))
	}
}

func checkStateReadTime(b *testing.B, saveCount int) {
	b.StopTimer()

	db := setupDB(b)
	initialSetOfValidators := validators(100000)

	// Save a state to read in benchmark
	r := [32]byte{'A'}
	st, err := util.NewBeaconState()
	require.NoError(b, err)
	require.NoError(b, st.SetValidators(initialSetOfValidators))
	require.NoError(b, db.SaveState(context.Background(), st, r))

	// construct some states and save to randomize benchmark.
	for i := 0; i < saveCount; i++ {
		key := make([]byte, 32)
		_, err := rand.Read(key)
		require.NoError(b, err)
		st, err = util.NewBeaconState()
		require.NoError(b, err)

		// Add some more new validator to the base validator.
		validatosToAddInTest := validators(10000)
		allValidators := append(initialSetOfValidators, validatosToAddInTest...)

		// shuffle validators.
		rand.Seed(time.Now().UnixNano())
		rand.Shuffle(len(allValidators), func(i, j int) { allValidators[i], allValidators[j] = allValidators[j], allValidators[i] })

		require.NoError(b, st.SetValidators(allValidators))
		require.NoError(b, db.SaveState(context.Background(), st, bytesutil.ToBytes32(key)))
	}

	b.ReportAllocs()
	b.StartTimer()
	for i := 0; i < b.N; i++ {
		_, err := db.State(context.Background(), r)
		require.NoError(b, err)
	}
}

func TestStateBellatrix_CanSaveRetrieveValidatorEntries(t *testing.T) {
	db := setupDB(t)

	// enable historical state representation flag to test this
	resetCfg := features.InitWithReset(&features.Flags{
		EnableHistoricalSpaceRepresentation: true,
	})
	defer resetCfg()

	r := [32]byte{'A'}

	require.Equal(t, false, db.HasState(context.Background(), r))

	stateValidators := validators(10)
	st, _ := util.DeterministicGenesisStateBellatrix(t, 20)
	require.NoError(t, st.SetSlot(100))
	require.NoError(t, st.SetValidators(stateValidators))

	ctx := context.Background()
	require.NoError(t, db.SaveState(ctx, st, r))
	assert.Equal(t, true, db.HasState(context.Background(), r))

	savedS, err := db.State(context.Background(), r)
	require.NoError(t, err)

	require.DeepSSZEqual(t, st.ToProtoUnsafe(), savedS.ToProtoUnsafe(), "saved state with validators and retrieved state are not matching")

	// check if the index of the second state is still present.
	err = db.db.Update(func(tx *bolt.Tx) error {
		idxBkt := tx.Bucket(blockRootValidatorHashesBucket)
		data := idxBkt.Get(r[:])
		require.NotEqual(t, 0, len(data))
		return nil
	})
	require.NoError(t, err)

	// check if all the validator entries are still intact in the validator entry bucket.
	err = db.db.Update(func(tx *bolt.Tx) error {
		valBkt := tx.Bucket(stateValidatorsBucket)
		// if any of the original validator entry is not present, then fail the test.
		for _, val := range stateValidators {
			hash, hashErr := val.HashTreeRoot()
			assert.NoError(t, hashErr)
			data := valBkt.Get(hash[:])
			require.NotNil(t, data)
			require.NotEqual(t, 0, len(data))
		}
		return nil
	})
	require.NoError(t, err)
}

func TestBellatrixState_CanSaveRetrieve(t *testing.T) {
	db := setupDB(t)

	r := [32]byte{'A'}

	require.Equal(t, false, db.HasState(context.Background(), r))

	st, _ := util.DeterministicGenesisStateBellatrix(t, 1)
	require.NoError(t, st.SetSlot(100))

	require.NoError(t, db.SaveState(context.Background(), st, r))
	require.Equal(t, true, db.HasState(context.Background(), r))

	savedS, err := db.State(context.Background(), r)
	require.NoError(t, err)

	require.DeepSSZEqual(t, st.ToProtoUnsafe(), savedS.ToProtoUnsafe())

	savedS, err = db.State(context.Background(), [32]byte{'B'})
	require.NoError(t, err)
	require.Equal(t, state.ReadOnlyBeaconState(nil), savedS, "Unsaved state should've been nil")
}

func TestBellatrixState_CanDelete(t *testing.T) {
	db := setupDB(t)

	r := [32]byte{'A'}

	require.Equal(t, false, db.HasState(context.Background(), r))

	st, _ := util.DeterministicGenesisStateBellatrix(t, 1)
	require.NoError(t, st.SetSlot(100))

	require.NoError(t, db.SaveState(context.Background(), st, r))
	require.Equal(t, true, db.HasState(context.Background(), r))

	require.NoError(t, db.DeleteState(context.Background(), r))
	savedS, err := db.State(context.Background(), r)
	require.NoError(t, err)
	require.Equal(t, state.ReadOnlyBeaconState(nil), savedS, "Unsaved state should've been nil")
}

func TestDenebState_CanSaveRetrieve(t *testing.T) {
	db := setupDB(t)

	r := [32]byte{'A'}

	require.Equal(t, false, db.HasState(context.Background(), r))

	st, _ := util.DeterministicGenesisStateDeneb(t, 1)
	require.NoError(t, st.SetSlot(100))

	require.NoError(t, db.SaveState(context.Background(), st, r))
	require.Equal(t, true, db.HasState(context.Background(), r))

	savedS, err := db.State(context.Background(), r)
	require.NoError(t, err)

	require.DeepSSZEqual(t, st.ToProtoUnsafe(), savedS.ToProtoUnsafe())

	savedS, err = db.State(context.Background(), [32]byte{'B'})
	require.NoError(t, err)
	require.Equal(t, state.ReadOnlyBeaconState(nil), savedS, "Unsaved state should've been nil")
}

func TestDenebState_CanDelete(t *testing.T) {
	db := setupDB(t)

	r := [32]byte{'A'}

	require.Equal(t, false, db.HasState(context.Background(), r))

	st, _ := util.DeterministicGenesisStateDeneb(t, 1)
	require.NoError(t, st.SetSlot(100))

	require.NoError(t, db.SaveState(context.Background(), st, r))
	require.Equal(t, true, db.HasState(context.Background(), r))

	require.NoError(t, db.DeleteState(context.Background(), r))
	savedS, err := db.State(context.Background(), r)
	require.NoError(t, err)
	require.Equal(t, state.ReadOnlyBeaconState(nil), savedS, "Unsaved state should've been nil")
}

func TestStateDeneb_CanSaveRetrieveValidatorEntries(t *testing.T) {
	db := setupDB(t)

	// enable historical state representation flag to test this
	resetCfg := features.InitWithReset(&features.Flags{
		EnableHistoricalSpaceRepresentation: true,
	})
	defer resetCfg()

	r := [32]byte{'A'}

	require.Equal(t, false, db.HasState(context.Background(), r))

	stateValidators := validators(10)
	st, _ := util.DeterministicGenesisStateDeneb(t, 20)
	require.NoError(t, st.SetSlot(100))
	require.NoError(t, st.SetValidators(stateValidators))

	ctx := context.Background()
	require.NoError(t, db.SaveState(ctx, st, r))
	assert.Equal(t, true, db.HasState(context.Background(), r))

	savedS, err := db.State(context.Background(), r)
	require.NoError(t, err)

	require.DeepSSZEqual(t, st.Validators(), savedS.Validators(), "saved state with validators and retrieved state are not matching")

	// check if the index of the second state is still present.
	err = db.db.Update(func(tx *bolt.Tx) error {
		idxBkt := tx.Bucket(blockRootValidatorHashesBucket)
		data := idxBkt.Get(r[:])
		require.NotEqual(t, 0, len(data))
		return nil
	})
	require.NoError(t, err)

	// check if all the validator entries are still intact in the validator entry bucket.
	err = db.db.Update(func(tx *bolt.Tx) error {
		valBkt := tx.Bucket(stateValidatorsBucket)
		// if any of the original validator entry is not present, then fail the test.
		for _, val := range stateValidators {
			hash, hashErr := val.HashTreeRoot()
			assert.NoError(t, hashErr)
			data := valBkt.Get(hash[:])
			require.NotNil(t, data)
			require.NotEqual(t, 0, len(data))
		}
		return nil
	})
	require.NoError(t, err)
}

func BenchmarkState_CheckStateSaveTime_1(b *testing.B)  { checkStateSaveTime(b, 1) }
func BenchmarkState_CheckStateSaveTime_10(b *testing.B) { checkStateSaveTime(b, 10) }

func BenchmarkState_CheckStateReadTime_1(b *testing.B)  { checkStateReadTime(b, 1) }
func BenchmarkState_CheckStateReadTime_10(b *testing.B) { checkStateReadTime(b, 10) }
