package helpers

import (
	"context"
	"strconv"
	"strings"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/grpc-ecosystem/grpc-gateway/v2/runtime"
	grpcutil "github.com/prysmaticlabs/prysm/v4/api/grpc"
	chainmock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	dbtest "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	doublylinkedtree "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/doubly-linked-tree"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/testutil"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	syncmock "github.com/prysmaticlabs/prysm/v4/beacon-chain/sync/initial-sync/testing"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"google.golang.org/grpc"
)

func TestValidateSync(t *testing.T) {
	ctx := grpc.NewContextWithServerTransportStream(context.Background(), &runtime.ServerTransportStream{})
	t.Run("syncing", func(t *testing.T) {
		syncChecker := &syncmock.Sync{
			IsSyncing: true,
		}
		headSlot := primitives.Slot(100)
		st, err := util.NewBeaconState()
		require.NoError(t, err)
		require.NoError(t, st.SetSlot(50))
		chainService := &chainmock.ChainService{
			Slot:  &headSlot,
			State: st,
		}
		err = ValidateSyncGRPC(ctx, syncChecker, chainService, chainService, chainService)
		require.NotNil(t, err)
		sts, ok := grpc.ServerTransportStreamFromContext(ctx).(*runtime.ServerTransportStream)
		require.Equal(t, true, ok, "type assertion failed")
		md := sts.Header()
		v, ok := md[strings.ToLower(grpcutil.CustomErrorMetadataKey)]
		require.Equal(t, true, ok, "could not retrieve custom error metadata value")
		assert.DeepEqual(
			t,
			[]string{`{"data":{"head_slot":"50","sync_distance":"50","is_syncing":true,"is_optimistic":false,"el_offline":false}}`},
			v,
		)
	})
	t.Run("not syncing", func(t *testing.T) {
		syncChecker := &syncmock.Sync{
			IsSyncing: false,
		}
		headSlot := primitives.Slot(100)
		st, err := util.NewBeaconState()
		require.NoError(t, err)
		require.NoError(t, st.SetSlot(50))
		chainService := &chainmock.ChainService{
			Slot:  &headSlot,
			State: st,
		}
		err = ValidateSyncGRPC(ctx, syncChecker, nil, nil, chainService)
		require.NoError(t, err)
	})
}

func TestIsOptimistic(t *testing.T) {
	ctx := context.Background()

	t.Run("head optimistic", func(t *testing.T) {
		cs := &chainmock.ChainService{Optimistic: true}
		o, err := IsOptimistic(ctx, []byte("head"), cs, nil, nil, nil)
		require.NoError(t, err)
		assert.Equal(t, true, o)
	})
	t.Run("head not optimistic", func(t *testing.T) {
		cs := &chainmock.ChainService{Optimistic: false}
		o, err := IsOptimistic(ctx, []byte("head"), cs, nil, nil, nil)
		require.NoError(t, err)
		assert.Equal(t, false, o)
	})
	t.Run("genesis", func(t *testing.T) {
		o, err := IsOptimistic(ctx, []byte("genesis"), nil, nil, nil, nil)
		require.NoError(t, err)
		assert.Equal(t, false, o)
	})
	t.Run("finalized", func(t *testing.T) {
		t.Run("finalized checkpoint is optimistic", func(t *testing.T) {
			st, err := util.NewBeaconState()
			require.NoError(t, err)
			cs := &chainmock.ChainService{Optimistic: true, FinalizedCheckPoint: &eth.Checkpoint{}, OptimisticRoots: map[[32]byte]bool{[32]byte{}: true}}
			mf := &testutil.MockStater{BeaconState: st}
			o, err := IsOptimistic(ctx, []byte("finalized"), cs, mf, cs, nil)
			require.NoError(t, err)
			assert.Equal(t, true, o)
		})
		t.Run("finalized checkpoint is not optimistic", func(t *testing.T) {
			st, err := util.NewBeaconState()
			require.NoError(t, err)
			cs := &chainmock.ChainService{Optimistic: true, FinalizedCheckPoint: &eth.Checkpoint{}}
			mf := &testutil.MockStater{BeaconState: st}
			o, err := IsOptimistic(ctx, []byte("finalized"), cs, mf, cs, nil)
			require.NoError(t, err)
			assert.Equal(t, false, o)
		})
	})
	t.Run("justified", func(t *testing.T) {
		t.Run("justified checkpoint is optimistic", func(t *testing.T) {
			st, err := util.NewBeaconState()
			require.NoError(t, err)
			cs := &chainmock.ChainService{Optimistic: true, CurrentJustifiedCheckPoint: &eth.Checkpoint{}, OptimisticRoots: map[[32]byte]bool{[32]byte{}: true}}
			mf := &testutil.MockStater{BeaconState: st}
			o, err := IsOptimistic(ctx, []byte("justified"), cs, mf, cs, nil)
			require.NoError(t, err)
			assert.Equal(t, true, o)
		})
		t.Run("justified checkpoint is not optimistic", func(t *testing.T) {
			st, err := util.NewBeaconState()
			require.NoError(t, err)
			cs := &chainmock.ChainService{Optimistic: true, CurrentJustifiedCheckPoint: &eth.Checkpoint{}}
			mf := &testutil.MockStater{BeaconState: st}
			o, err := IsOptimistic(ctx, []byte("justified"), cs, mf, cs, nil)
			require.NoError(t, err)
			assert.Equal(t, false, o)
		})
	})
	t.Run("root", func(t *testing.T) {
		t.Run("is head and head is optimistic", func(t *testing.T) {
			st, err := util.NewBeaconState()
			require.NoError(t, err)
			cs := &chainmock.ChainService{Optimistic: true}
			mf := &testutil.MockStater{BeaconState: st}
			o, err := IsOptimistic(ctx, bytesutil.PadTo([]byte("root"), 32), cs, mf, cs, nil)
			require.NoError(t, err)
			assert.Equal(t, true, o)
		})
		t.Run("is head and head is not optimistic", func(t *testing.T) {
			st, err := util.NewBeaconState()
			require.NoError(t, err)
			cs := &chainmock.ChainService{Optimistic: false}
			mf := &testutil.MockStater{BeaconState: st}
			o, err := IsOptimistic(ctx, bytesutil.PadTo([]byte("root"), 32), cs, mf, cs, nil)
			require.NoError(t, err)
			assert.Equal(t, false, o)
		})
		t.Run("root is optimistic", func(t *testing.T) {
			b, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlock())
			require.NoError(t, err)
			b.SetStateRoot(bytesutil.PadTo([]byte("root"), 32))
			db := dbtest.SetupDB(t)
			require.NoError(t, db.SaveBlock(ctx, b))
			fetcherSt, err := util.NewBeaconState()
			require.NoError(t, err)
			chainSt, err := util.NewBeaconState()
			require.NoError(t, err)
			require.NoError(t, chainSt.SetSlot(fieldparams.SlotsPerEpoch))
			bRoot, err := b.Block().HashTreeRoot()
			require.NoError(t, err)
			cs := &chainmock.ChainService{State: chainSt, OptimisticRoots: map[[32]byte]bool{bRoot: true}}
			mf := &testutil.MockStater{BeaconState: fetcherSt}
			o, err := IsOptimistic(ctx, bytesutil.PadTo([]byte("root"), 32), cs, mf, cs, db)
			require.NoError(t, err)
			assert.Equal(t, true, o)
		})
		t.Run("root is not optimistic", func(t *testing.T) {
			b, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlock())
			require.NoError(t, err)
			b.SetStateRoot(bytesutil.PadTo([]byte("root"), 32))
			db := dbtest.SetupDB(t)
			require.NoError(t, db.SaveBlock(ctx, b))
			fetcherSt, err := util.NewBeaconState()
			require.NoError(t, err)
			chainSt, err := util.NewBeaconState()
			require.NoError(t, err)
			require.NoError(t, chainSt.SetSlot(fieldparams.SlotsPerEpoch))
			cs := &chainmock.ChainService{State: chainSt}
			mf := &testutil.MockStater{BeaconState: fetcherSt}
			o, err := IsOptimistic(ctx, bytesutil.PadTo([]byte("root"), 32), cs, mf, cs, db)
			require.NoError(t, err)
			assert.Equal(t, false, o)
		})
		t.Run("no canonical blocks", func(t *testing.T) {
			b, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlock())
			require.NoError(t, err)
			db := dbtest.SetupDB(t)
			require.NoError(t, db.SaveBlock(ctx, b))
			fetcherSt, err := util.NewBeaconState()
			require.NoError(t, err)
			chainSt, err := util.NewBeaconState()
			require.NoError(t, err)
			require.NoError(t, chainSt.SetSlot(fieldparams.SlotsPerEpoch))
			cs := &chainmock.ChainService{Optimistic: false, State: chainSt, CanonicalRoots: map[[32]byte]bool{}}
			mf := &testutil.MockStater{BeaconState: fetcherSt}
			o, err := IsOptimistic(ctx, bytesutil.PadTo([]byte("root"), 32), nil, mf, cs, db)
			require.NoError(t, err)
			assert.Equal(t, true, o)
		})
	})
	t.Run("hex", func(t *testing.T) {
		t.Run("is head and head is optimistic", func(t *testing.T) {
			st, err := util.NewBeaconState()
			require.NoError(t, err)
			cs := &chainmock.ChainService{Optimistic: true}
			mf := &testutil.MockStater{BeaconState: st}
			o, err := IsOptimistic(ctx, []byte(hexutil.Encode(bytesutil.PadTo([]byte("root"), 32))), cs, mf, cs, nil)
			require.NoError(t, err)
			assert.Equal(t, true, o)
		})
		t.Run("is head and head is not optimistic", func(t *testing.T) {
			st, err := util.NewBeaconState()
			require.NoError(t, err)
			cs := &chainmock.ChainService{Optimistic: false}
			mf := &testutil.MockStater{BeaconState: st}
			o, err := IsOptimistic(ctx, []byte(hexutil.Encode(bytesutil.PadTo([]byte("root"), 32))), cs, mf, cs, nil)
			require.NoError(t, err)
			assert.Equal(t, false, o)
		})
		t.Run("root is optimistic", func(t *testing.T) {
			b, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlock())
			require.NoError(t, err)
			b.SetStateRoot(bytesutil.PadTo([]byte("root"), 32))
			db := dbtest.SetupDB(t)
			require.NoError(t, db.SaveBlock(ctx, b))
			fetcherSt, err := util.NewBeaconState()
			require.NoError(t, err)
			chainSt, err := util.NewBeaconState()
			require.NoError(t, err)
			require.NoError(t, chainSt.SetSlot(fieldparams.SlotsPerEpoch))
			bRoot, err := b.Block().HashTreeRoot()
			require.NoError(t, err)
			cs := &chainmock.ChainService{State: chainSt, OptimisticRoots: map[[32]byte]bool{bRoot: true}}
			mf := &testutil.MockStater{BeaconState: fetcherSt}
			o, err := IsOptimistic(ctx, []byte(hexutil.Encode(bytesutil.PadTo([]byte("root"), 32))), cs, mf, cs, db)
			require.NoError(t, err)
			assert.Equal(t, true, o)
		})
		t.Run("root is not optimistic", func(t *testing.T) {
			b, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlock())
			require.NoError(t, err)
			b.SetStateRoot(bytesutil.PadTo([]byte("root"), 32))
			db := dbtest.SetupDB(t)
			require.NoError(t, db.SaveBlock(ctx, b))
			fetcherSt, err := util.NewBeaconState()
			require.NoError(t, err)
			chainSt, err := util.NewBeaconState()
			require.NoError(t, err)
			require.NoError(t, chainSt.SetSlot(fieldparams.SlotsPerEpoch))
			cs := &chainmock.ChainService{State: chainSt}
			mf := &testutil.MockStater{BeaconState: fetcherSt}
			o, err := IsOptimistic(ctx, []byte(hexutil.Encode(bytesutil.PadTo([]byte("root"), 32))), cs, mf, cs, db)
			require.NoError(t, err)
			assert.Equal(t, false, o)
		})
		t.Run("no canonical blocks", func(t *testing.T) {
			b, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlock())
			require.NoError(t, err)
			db := dbtest.SetupDB(t)
			require.NoError(t, db.SaveBlock(ctx, b))
			fetcherSt, err := util.NewBeaconState()
			require.NoError(t, err)
			chainSt, err := util.NewBeaconState()
			require.NoError(t, err)
			require.NoError(t, chainSt.SetSlot(fieldparams.SlotsPerEpoch))
			cs := &chainmock.ChainService{Optimistic: false, State: chainSt, CanonicalRoots: map[[32]byte]bool{}}
			mf := &testutil.MockStater{BeaconState: fetcherSt}
			o, err := IsOptimistic(ctx, []byte(hexutil.Encode(bytesutil.PadTo([]byte("root"), 32))), nil, mf, cs, db)
			require.NoError(t, err)
			assert.Equal(t, true, o)
		})
	})
	t.Run("slot", func(t *testing.T) {
		t.Run("head is not optimistic", func(t *testing.T) {
			cs := &chainmock.ChainService{Optimistic: false}
			o, err := IsOptimistic(ctx, []byte("0"), cs, nil, nil, nil)
			require.NoError(t, err)
			assert.Equal(t, false, o)
		})
		t.Run("is before validated slot when head is optimistic", func(t *testing.T) {
			db := dbtest.SetupDB(t)
			require.NoError(t, db.SaveStateSummary(ctx, &eth.StateSummary{Slot: fieldparams.SlotsPerEpoch, Root: []byte("root")}))
			require.NoError(t, db.SaveLastValidatedCheckpoint(ctx, &eth.Checkpoint{Epoch: 1, Root: []byte("root")}))
			cs := &chainmock.ChainService{Optimistic: true, FinalizedCheckPoint: &eth.Checkpoint{Epoch: 1}}
			o, err := IsOptimistic(ctx, []byte("0"), cs, nil, cs, db)
			require.NoError(t, err)
			assert.Equal(t, false, o)
		})
		t.Run("is equal to validated slot when head is optimistic", func(t *testing.T) {
			db := dbtest.SetupDB(t)
			require.NoError(t, db.SaveStateSummary(ctx, &eth.StateSummary{Slot: fieldparams.SlotsPerEpoch, Root: []byte("root")}))
			require.NoError(t, db.SaveLastValidatedCheckpoint(ctx, &eth.Checkpoint{Epoch: 1, Root: []byte("root")}))
			cs := &chainmock.ChainService{Optimistic: true, FinalizedCheckPoint: &eth.Checkpoint{Epoch: 1}}
			o, err := IsOptimistic(ctx, []byte("32"), cs, nil, cs, db)
			require.NoError(t, err)
			assert.Equal(t, false, o)
		})
		t.Run("is after validated slot and validated slot is before finalized slot", func(t *testing.T) {
			db := dbtest.SetupDB(t)
			require.NoError(t, db.SaveStateSummary(ctx, &eth.StateSummary{Slot: fieldparams.SlotsPerEpoch, Root: []byte("root")}))
			require.NoError(t, db.SaveLastValidatedCheckpoint(ctx, &eth.Checkpoint{Epoch: 1, Root: []byte("root")}))
			cs := &chainmock.ChainService{Optimistic: true, FinalizedCheckPoint: &eth.Checkpoint{Epoch: 2}}
			o, err := IsOptimistic(ctx, []byte("33"), cs, nil, cs, db)
			require.NoError(t, err)
			assert.Equal(t, true, o)
		})
		t.Run("is head", func(t *testing.T) {
			db := dbtest.SetupDB(t)
			require.NoError(t, db.SaveStateSummary(ctx, &eth.StateSummary{Slot: fieldparams.SlotsPerEpoch, Root: []byte("root")}))
			require.NoError(t, db.SaveLastValidatedCheckpoint(ctx, &eth.Checkpoint{Epoch: 1, Root: []byte("root")}))
			fetcherSt, err := util.NewBeaconState()
			require.NoError(t, err)
			chainSt, err := util.NewBeaconState()
			require.NoError(t, err)
			require.NoError(t, chainSt.SetSlot(fieldparams.SlotsPerEpoch*2))
			cs := &chainmock.ChainService{Optimistic: true, State: chainSt, FinalizedCheckPoint: &eth.Checkpoint{Epoch: 0}}
			mf := &testutil.MockStater{BeaconState: fetcherSt}
			o, err := IsOptimistic(ctx, []byte(strconv.Itoa(fieldparams.SlotsPerEpoch*2)), cs, mf, cs, db)
			require.NoError(t, err)
			assert.Equal(t, true, o)
		})
		t.Run("ancestor is optimistic", func(t *testing.T) {
			db := dbtest.SetupDB(t)
			require.NoError(t, db.SaveStateSummary(ctx, &eth.StateSummary{Slot: fieldparams.SlotsPerEpoch, Root: []byte("root")}))
			require.NoError(t, db.SaveLastValidatedCheckpoint(ctx, &eth.Checkpoint{Epoch: 1, Root: []byte("root")}))
			r := bytesutil.ToBytes32([]byte("root"))
			fcs := doublylinkedtree.New()
			finalizedCheckpt := &eth.Checkpoint{Epoch: 0}
			st, root, err := prepareForkchoiceState(fieldparams.SlotsPerEpoch*2, r, [32]byte{}, [32]byte{}, finalizedCheckpt, finalizedCheckpt)
			require.NoError(t, err)
			require.NoError(t, fcs.InsertNode(ctx, st, root))
			headRoot := [32]byte{'r'}
			st, root, err = prepareForkchoiceState(fieldparams.SlotsPerEpoch*2+1, headRoot, r, [32]byte{}, finalizedCheckpt, finalizedCheckpt)
			require.NoError(t, err)
			require.NoError(t, fcs.InsertNode(ctx, st, root))
			cs := &chainmock.ChainService{Root: headRoot[:], Optimistic: true, ForkChoiceStore: fcs, OptimisticRoots: map[[32]byte]bool{r: true}, FinalizedCheckPoint: finalizedCheckpt}
			mf := &testutil.MockStater{BeaconState: st}
			o, err := IsOptimistic(ctx, []byte(strconv.Itoa(fieldparams.SlotsPerEpoch*2)), cs, mf, cs, db)
			require.NoError(t, err)
			assert.Equal(t, true, o)
		})
		t.Run("ancestor is not optimistic", func(t *testing.T) {
			db := dbtest.SetupDB(t)
			require.NoError(t, db.SaveStateSummary(ctx, &eth.StateSummary{Slot: fieldparams.SlotsPerEpoch, Root: []byte("root")}))
			require.NoError(t, db.SaveLastValidatedCheckpoint(ctx, &eth.Checkpoint{Epoch: 1, Root: []byte("root")}))
			r := bytesutil.ToBytes32([]byte("root"))
			fcs := doublylinkedtree.New()
			finalizedCheckpt := &eth.Checkpoint{Epoch: 0}
			st, root, err := prepareForkchoiceState(fieldparams.SlotsPerEpoch*2, r, [32]byte{}, [32]byte{}, finalizedCheckpt, finalizedCheckpt)
			require.NoError(t, err)
			require.NoError(t, fcs.InsertNode(ctx, st, root))
			headRoot := [32]byte{'r'}
			st, root, err = prepareForkchoiceState(fieldparams.SlotsPerEpoch*2+1, headRoot, r, [32]byte{}, finalizedCheckpt, finalizedCheckpt)
			require.NoError(t, err)
			require.NoError(t, fcs.InsertNode(ctx, st, root))
			cs := &chainmock.ChainService{Root: headRoot[:], Optimistic: true, ForkChoiceStore: fcs, OptimisticRoots: map[[32]byte]bool{r: false}, FinalizedCheckPoint: finalizedCheckpt}
			mf := &testutil.MockStater{BeaconState: st}
			o, err := IsOptimistic(ctx, []byte(strconv.Itoa(fieldparams.SlotsPerEpoch*2)), cs, mf, cs, db)
			require.NoError(t, err)
			assert.Equal(t, false, o)
		})
	})
}

// prepareForkchoiceState prepares a beacon state with the given data to mock
// insert into forkchoice
func prepareForkchoiceState(
	slot primitives.Slot,
	blockRoot [32]byte,
	parentRoot [32]byte,
	payloadHash [32]byte,
	justified *eth.Checkpoint,
	finalized *eth.Checkpoint,
) (state.BeaconState, [32]byte, error) {
	blockHeader := &eth.BeaconBlockHeader{
		ParentRoot: parentRoot[:],
	}

	executionHeader := &enginev1.ExecutionPayloadHeader{
		BlockHash: payloadHash[:],
	}

	base := &eth.BeaconStateBellatrix{
		Slot:                         slot,
		RandaoMixes:                  make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
		BlockRoots:                   make([][]byte, 1),
		CurrentJustifiedCheckpoint:   justified,
		FinalizedCheckpoint:          finalized,
		LatestExecutionPayloadHeader: executionHeader,
		LatestBlockHeader:            blockHeader,
	}

	base.BlockRoots[0] = append(base.BlockRoots[0], blockRoot[:]...)
	st, err := state_native.InitializeFromProtoBellatrix(base)
	return st, blockRoot, err
}
