package forkchoice

import (
	"context"
	"math/big"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	gethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/kzg"
	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache/depositcache"
	coreTime "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/filesystem"
	testDB "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	doublylinkedtree "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/doubly-linked-tree"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/attestations"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	payloadattribute "github.com/prysmaticlabs/prysm/v4/consensus-types/payload-attribute"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	pb "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func startChainService(t testing.TB,
	st state.BeaconState,
	block interfaces.ReadOnlySignedBeaconBlock,
	engineMock *engineMock,
) *blockchain.Service {
	ctx := context.Background()
	db := testDB.SetupDB(t)
	require.NoError(t, db.SaveBlock(ctx, block))
	r, err := block.Block().HashTreeRoot()
	require.NoError(t, err)
	require.NoError(t, db.SaveGenesisBlockRoot(ctx, r))

	cp := &ethpb.Checkpoint{
		Epoch: coreTime.CurrentEpoch(st),
		Root:  r[:],
	}
	require.NoError(t, db.SaveState(ctx, st, r))
	require.NoError(t, db.SaveJustifiedCheckpoint(ctx, cp))
	require.NoError(t, db.SaveFinalizedCheckpoint(ctx, cp))
	attPool, err := attestations.NewService(ctx, &attestations.Config{
		Pool: attestations.NewPool(),
	})
	require.NoError(t, err)

	depositCache, err := depositcache.New()
	require.NoError(t, err)

	fc := doublylinkedtree.New()
	opts := append([]blockchain.Option{},
		blockchain.WithExecutionEngineCaller(engineMock),
		blockchain.WithFinalizedStateAtStartUp(st),
		blockchain.WithDatabase(db),
		blockchain.WithAttestationService(attPool),
		blockchain.WithForkChoiceStore(fc),
		blockchain.WithStateGen(stategen.New(db, fc)),
		blockchain.WithStateNotifier(&mock.MockStateNotifier{}),
		blockchain.WithAttestationPool(attestations.NewPool()),
		blockchain.WithDepositCache(depositCache),
		blockchain.WithTrackedValidatorsCache(cache.NewTrackedValidatorsCache()),
		blockchain.WithPayloadIDCache(cache.NewPayloadIDCache()),
		blockchain.WithClockSynchronizer(startup.NewClockSynchronizer()),
		blockchain.WithBlobStorage(filesystem.NewEphemeralBlobStorage(t)),
	)
	service, err := blockchain.NewService(context.Background(), opts...)
	require.NoError(t, err)
	// force start kzg context here until Deneb fork epoch is decided
	require.NoError(t, kzg.Start())
	require.NoError(t, service.StartFromSavedState(st))
	return service
}

type engineMock struct {
	powBlocks       map[[32]byte]*ethpb.PowBlock
	latestValidHash []byte
	payloadStatus   error
}

func (m *engineMock) GetPayload(context.Context, [8]byte, primitives.Slot) (interfaces.ExecutionData, *pb.BlobsBundle, bool, error) {
	return nil, nil, false, nil
}
func (m *engineMock) GetPayloadV2(context.Context, [8]byte) (*pb.ExecutionPayloadCapella, error) {
	return nil, nil
}
func (m *engineMock) ForkchoiceUpdated(context.Context, *pb.ForkchoiceState, payloadattribute.Attributer) (*pb.PayloadIDBytes, []byte, error) {
	return nil, m.latestValidHash, m.payloadStatus
}

func (m *engineMock) NewPayload(context.Context, interfaces.ExecutionData, []common.Hash, *common.Hash) ([]byte, error) {
	return m.latestValidHash, m.payloadStatus
}

func (m *engineMock) ForkchoiceUpdatedV2(context.Context, *pb.ForkchoiceState, payloadattribute.Attributer) (*pb.PayloadIDBytes, []byte, error) {
	return nil, m.latestValidHash, m.payloadStatus
}

func (m *engineMock) LatestExecutionBlock(context.Context) (*pb.ExecutionBlock, error) {
	return nil, nil
}

func (m *engineMock) ExecutionBlockByHash(_ context.Context, hash common.Hash, _ bool) (*pb.ExecutionBlock, error) {
	b, ok := m.powBlocks[bytesutil.ToBytes32(hash.Bytes())]
	if !ok {
		return nil, nil
	}

	td := new(big.Int).SetBytes(bytesutil.ReverseByteOrder(b.TotalDifficulty))
	tdHex := hexutil.EncodeBig(td)
	return &pb.ExecutionBlock{
		Header: gethtypes.Header{
			ParentHash: common.BytesToHash(b.ParentHash),
		},
		TotalDifficulty: tdHex,
		Hash:            common.BytesToHash(b.BlockHash),
	}, nil
}

func (m *engineMock) GetTerminalBlockHash(context.Context, uint64) ([]byte, bool, error) {
	return nil, false, nil
}
