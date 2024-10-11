package validator

import (
	"context"
	"testing"
	"time"

	blockchainTest "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/builder"
	testing2 "github.com/prysmaticlabs/prysm/v4/beacon-chain/builder/testing"
	dbTest "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	doublylinkedtree "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/doubly-linked-tree"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	v1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func TestServer_circuitBreakBuilder(t *testing.T) {
	hook := logTest.NewGlobal()
	s := &Server{}
	_, err := s.circuitBreakBuilder(0)
	require.ErrorContains(t, "no fork choicer configured", err)

	s.ForkchoiceFetcher = &blockchainTest.ChainService{ForkChoiceStore: doublylinkedtree.New()}
	s.ForkchoiceFetcher.SetForkChoiceGenesisTime(uint64(time.Now().Unix()))
	b, err := s.circuitBreakBuilder(params.BeaconConfig().MaxBuilderConsecutiveMissedSlots + 1)
	require.NoError(
		t,
		err,
	)
	require.Equal(t, true, b)
	require.LogsContain(t, hook, "Circuit breaker activated due to missing consecutive slot. Ignore if mev-boost is not used")

	ojc := &ethpb.Checkpoint{Root: params.BeaconConfig().ZeroHash[:]}
	ofc := &ethpb.Checkpoint{Root: params.BeaconConfig().ZeroHash[:]}
	ctx := context.Background()
	st, blkRoot, err := createState(1, [32]byte{'a'}, [32]byte{}, params.BeaconConfig().ZeroHash, ojc, ofc)
	require.NoError(t, err)
	require.NoError(t, s.ForkchoiceFetcher.InsertNode(ctx, st, blkRoot))
	b, err = s.circuitBreakBuilder(params.BeaconConfig().MaxBuilderConsecutiveMissedSlots)
	require.NoError(t, err)
	require.Equal(t, false, b)

	params.SetupTestConfigCleanup(t)
	cfg := params.BeaconConfig().Copy()
	cfg.MaxBuilderEpochMissedSlots = 4
	params.OverrideBeaconConfig(cfg)
	st, blkRoot, err = createState(params.BeaconConfig().SlotsPerEpoch, [32]byte{'b'}, [32]byte{'a'}, params.BeaconConfig().ZeroHash, ojc, ofc)
	require.NoError(t, err)
	require.NoError(t, s.ForkchoiceFetcher.InsertNode(ctx, st, blkRoot))
	b, err = s.circuitBreakBuilder(params.BeaconConfig().SlotsPerEpoch + 1)
	require.NoError(t, err)
	require.Equal(t, true, b)
	require.LogsContain(t, hook, "Circuit breaker activated due to missing enough slots last epoch. Ignore if mev-boost is not used")

	want := params.BeaconConfig().SlotsPerEpoch - params.BeaconConfig().MaxBuilderEpochMissedSlots
	for i := primitives.Slot(2); i <= want+2; i++ {
		st, blkRoot, err = createState(i, [32]byte{byte(i)}, [32]byte{'a'}, params.BeaconConfig().ZeroHash, ojc, ofc)
		require.NoError(t, err)
		require.NoError(t, s.ForkchoiceFetcher.InsertNode(ctx, st, blkRoot))
	}
	b, err = s.circuitBreakBuilder(params.BeaconConfig().SlotsPerEpoch + 1)
	require.NoError(t, err)
	require.Equal(t, false, b)
}

func TestServer_validatorRegistered(t *testing.T) {
	b, err := builder.NewService(context.Background())
	require.NoError(t, err)
	proposerServer := &Server{
		BlockBuilder: b,
	}
	ctx := context.Background()

	reg, err := proposerServer.validatorRegistered(ctx, 0)
	require.ErrorContains(t, "nil beacon db", err)
	require.Equal(t, false, reg)
	db := dbTest.SetupDB(t)
	realBuilder, err := builder.NewService(context.Background(), builder.WithDatabase(db))
	require.NoError(t, err)
	proposerServer.BlockBuilder = realBuilder
	reg, err = proposerServer.validatorRegistered(ctx, 0)
	require.NoError(t, err)
	require.Equal(t, false, reg)

	f := bytesutil.PadTo([]byte{}, fieldparams.FeeRecipientLength)
	p := bytesutil.PadTo([]byte{}, fieldparams.BLSPubkeyLength)
	require.NoError(t, db.SaveRegistrationsByValidatorIDs(ctx, []primitives.ValidatorIndex{0, 1},
		[]*ethpb.ValidatorRegistrationV1{{FeeRecipient: f, Timestamp: uint64(time.Now().Unix()), Pubkey: p}, {FeeRecipient: f, Timestamp: uint64(time.Now().Unix()), Pubkey: p}}))

	reg, err = proposerServer.validatorRegistered(ctx, 0)
	require.NoError(t, err)
	require.Equal(t, true, reg)
	reg, err = proposerServer.validatorRegistered(ctx, 1)
	require.NoError(t, err)
	require.Equal(t, true, reg)

}

func TestServer_canUseBuilder(t *testing.T) {
	proposerServer := &Server{
		BlockBuilder: &testing2.MockBuilderService{
			HasConfigured: false,
		},
	}
	reg, err := proposerServer.canUseBuilder(context.Background(), 0, 0)
	require.NoError(t, err)
	require.Equal(t, false, reg)

	ctx := context.Background()

	proposerServer.ForkchoiceFetcher = &blockchainTest.ChainService{ForkChoiceStore: doublylinkedtree.New()}
	proposerServer.ForkchoiceFetcher.SetForkChoiceGenesisTime(uint64(time.Now().Unix()))
	reg, err = proposerServer.canUseBuilder(ctx, params.BeaconConfig().MaxBuilderConsecutiveMissedSlots+1, 0)
	require.NoError(t, err)
	require.Equal(t, false, reg)
	db := dbTest.SetupDB(t)

	proposerServer.BlockBuilder = &testing2.MockBuilderService{
		HasConfigured: true,
		Cfg:           &testing2.Config{BeaconDB: db},
	}

	reg, err = proposerServer.canUseBuilder(ctx, 1, 0)
	require.NoError(t, err)
	require.Equal(t, false, reg)

	f := bytesutil.PadTo([]byte{}, fieldparams.FeeRecipientLength)
	p := bytesutil.PadTo([]byte{}, fieldparams.BLSPubkeyLength)
	require.NoError(t, db.SaveRegistrationsByValidatorIDs(ctx, []primitives.ValidatorIndex{0},
		[]*ethpb.ValidatorRegistrationV1{{FeeRecipient: f, Timestamp: uint64(time.Now().Unix()), Pubkey: p}}))

	reg, err = proposerServer.canUseBuilder(ctx, params.BeaconConfig().MaxBuilderConsecutiveMissedSlots-1, 0)
	require.NoError(t, err)
	require.Equal(t, true, reg)
}

func createState(
	slot primitives.Slot,
	blockRoot [32]byte,
	parentRoot [32]byte,
	payloadHash [32]byte,
	justified *ethpb.Checkpoint,
	finalized *ethpb.Checkpoint,
) (state.BeaconState, [32]byte, error) {

	base := &ethpb.BeaconStateBellatrix{
		Slot:                       slot,
		RandaoMixes:                make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
		BlockRoots:                 make([][]byte, 1),
		CurrentJustifiedCheckpoint: justified,
		FinalizedCheckpoint:        finalized,
		LatestExecutionPayloadHeader: &v1.ExecutionPayloadHeader{
			BlockHash: payloadHash[:],
		},
		LatestBlockHeader: &ethpb.BeaconBlockHeader{
			ParentRoot: parentRoot[:],
		},
	}

	base.BlockRoots[0] = append(base.BlockRoots[0], blockRoot[:]...)
	st, err := state_native.InitializeFromProtoBellatrix(base)
	return st, blockRoot, err
}
