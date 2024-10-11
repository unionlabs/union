package validator

import (
	"context"
	"math"
	"math/big"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/api/client/builder"
	blockchainTest "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	builderTest "github.com/prysmaticlabs/prysm/v4/beacon-chain/builder/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	dbTest "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	powtesting "github.com/prysmaticlabs/prysm/v4/beacon-chain/execution/testing"
	doublylinkedtree "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/doubly-linked-tree"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	consensus_types "github.com/prysmaticlabs/prysm/v4/consensus-types"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/encoding/ssz"
	v1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func TestServer_setExecutionData(t *testing.T) {
	hook := logTest.NewGlobal()

	ctx := context.Background()
	cfg := params.BeaconConfig().Copy()
	cfg.BellatrixForkEpoch = 0
	cfg.CapellaForkEpoch = 0
	params.OverrideBeaconConfig(cfg)
	params.SetupTestConfigCleanup(t)

	beaconDB := dbTest.SetupDB(t)
	capellaTransitionState, _ := util.DeterministicGenesisStateCapella(t, 1)
	wrappedHeaderCapella, err := blocks.WrappedExecutionPayloadHeaderCapella(&v1.ExecutionPayloadHeaderCapella{BlockNumber: 1}, big.NewInt(0))
	require.NoError(t, err)
	require.NoError(t, capellaTransitionState.SetLatestExecutionPayloadHeader(wrappedHeaderCapella))
	b2pbCapella := util.NewBeaconBlockCapella()
	b2rCapella, err := b2pbCapella.Block.HashTreeRoot()
	require.NoError(t, err)
	util.SaveBlock(t, context.Background(), beaconDB, b2pbCapella)
	require.NoError(t, capellaTransitionState.SetFinalizedCheckpoint(&ethpb.Checkpoint{
		Root: b2rCapella[:],
	}))
	require.NoError(t, beaconDB.SaveFeeRecipientsByValidatorIDs(context.Background(), []primitives.ValidatorIndex{0}, []common.Address{{}}))

	denebTransitionState, _ := util.DeterministicGenesisStateDeneb(t, 1)
	wrappedHeaderDeneb, err := blocks.WrappedExecutionPayloadHeaderDeneb(&v1.ExecutionPayloadHeaderDeneb{BlockNumber: 2}, big.NewInt(0))
	require.NoError(t, err)
	require.NoError(t, denebTransitionState.SetLatestExecutionPayloadHeader(wrappedHeaderDeneb))
	b2pbDeneb := util.NewBeaconBlockDeneb()
	b2rDeneb, err := b2pbDeneb.Block.HashTreeRoot()
	require.NoError(t, err)
	util.SaveBlock(t, context.Background(), beaconDB, b2pbDeneb)
	require.NoError(t, denebTransitionState.SetFinalizedCheckpoint(&ethpb.Checkpoint{
		Root: b2rDeneb[:],
	}))

	withdrawals := []*v1.Withdrawal{{
		Index:          1,
		ValidatorIndex: 2,
		Address:        make([]byte, fieldparams.FeeRecipientLength),
		Amount:         3,
	}}
	id := &v1.PayloadIDBytes{0x1}
	vs := &Server{
		ExecutionEngineCaller:  &powtesting.EngineClient{PayloadIDBytes: id, ExecutionPayloadCapella: &v1.ExecutionPayloadCapella{BlockNumber: 1, Withdrawals: withdrawals}, BlockValue: 0},
		HeadFetcher:            &blockchainTest.ChainService{State: capellaTransitionState},
		FinalizationFetcher:    &blockchainTest.ChainService{},
		BeaconDB:               beaconDB,
		PayloadIDCache:         cache.NewPayloadIDCache(),
		BlockBuilder:           &builderTest.MockBuilderService{HasConfigured: true, Cfg: &builderTest.Config{BeaconDB: beaconDB}},
		ForkchoiceFetcher:      &blockchainTest.ChainService{},
		TrackedValidatorsCache: cache.NewTrackedValidatorsCache(),
	}

	t.Run("No builder configured. Use local block", func(t *testing.T) {
		blk, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlockCapella())
		require.NoError(t, err)
		b := blk.Block()
		localPayload, _, err := vs.getLocalPayload(ctx, b, capellaTransitionState)
		require.NoError(t, err)
		builderPayload, builderKzgCommitments, err := vs.getBuilderPayloadAndBlobs(ctx, b.Slot(), b.ProposerIndex())
		require.NoError(t, err)
		require.DeepEqual(t, [][]uint8(nil), builderKzgCommitments)
		require.NoError(t, setExecutionData(context.Background(), blk, localPayload, builderPayload, builderKzgCommitments, defaultBuilderBoostFactor))
		e, err := blk.Block().Body().Execution()
		require.NoError(t, err)
		require.Equal(t, uint64(1), e.BlockNumber()) // Local block
	})
	t.Run("Builder configured. Builder Block has higher value. Incorrect withdrawals", func(t *testing.T) {
		blk, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlockCapella())
		require.NoError(t, err)
		require.NoError(t, vs.BeaconDB.SaveRegistrationsByValidatorIDs(ctx, []primitives.ValidatorIndex{blk.Block().ProposerIndex()},
			[]*ethpb.ValidatorRegistrationV1{{FeeRecipient: make([]byte, fieldparams.FeeRecipientLength), Timestamp: uint64(time.Now().Unix()), Pubkey: make([]byte, fieldparams.BLSPubkeyLength)}}))
		ti, err := slots.ToTime(uint64(time.Now().Unix()), 0)
		require.NoError(t, err)
		sk, err := bls.RandKey()
		require.NoError(t, err)
		bid := &ethpb.BuilderBidCapella{
			Header: &v1.ExecutionPayloadHeaderCapella{
				ParentHash:       params.BeaconConfig().ZeroHash[:],
				FeeRecipient:     make([]byte, fieldparams.FeeRecipientLength),
				StateRoot:        make([]byte, fieldparams.RootLength),
				ReceiptsRoot:     make([]byte, fieldparams.RootLength),
				LogsBloom:        make([]byte, fieldparams.LogsBloomLength),
				PrevRandao:       make([]byte, fieldparams.RootLength),
				BlockNumber:      2,
				Timestamp:        uint64(ti.Unix()),
				ExtraData:        make([]byte, 0),
				BaseFeePerGas:    make([]byte, fieldparams.RootLength),
				BlockHash:        make([]byte, fieldparams.RootLength),
				TransactionsRoot: bytesutil.PadTo([]byte{1}, fieldparams.RootLength),
				WithdrawalsRoot:  make([]byte, fieldparams.RootLength),
			},
			Pubkey: sk.PublicKey().Marshal(),
			Value:  bytesutil.PadTo([]byte{1}, 32),
		}
		d := params.BeaconConfig().DomainApplicationBuilder
		domain, err := signing.ComputeDomain(d, nil, nil)
		require.NoError(t, err)
		sr, err := signing.ComputeSigningRoot(bid, domain)
		require.NoError(t, err)
		sBid := &ethpb.SignedBuilderBidCapella{
			Message:   bid,
			Signature: sk.Sign(sr[:]).Marshal(),
		}
		vs.BlockBuilder = &builderTest.MockBuilderService{
			BidCapella:    sBid,
			HasConfigured: true,
			Cfg:           &builderTest.Config{BeaconDB: beaconDB},
		}
		wb, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlockBellatrix())
		require.NoError(t, err)
		chain := &blockchainTest.ChainService{ForkChoiceStore: doublylinkedtree.New(), Genesis: time.Now(), Block: wb}
		vs.ForkchoiceFetcher = chain
		vs.ForkchoiceFetcher.SetForkChoiceGenesisTime(uint64(time.Now().Unix()))
		vs.TimeFetcher = chain
		vs.HeadFetcher = chain
		b := blk.Block()

		localPayload, _, err := vs.getLocalPayload(ctx, b, capellaTransitionState)
		require.NoError(t, err)
		builderPayload, builderKzgCommitments, err := vs.getBuilderPayloadAndBlobs(ctx, b.Slot(), b.ProposerIndex())
		require.NoError(t, err)
		require.DeepEqual(t, [][]uint8(nil), builderKzgCommitments)
		require.NoError(t, setExecutionData(context.Background(), blk, localPayload, builderPayload, builderKzgCommitments, defaultBuilderBoostFactor))
		e, err := blk.Block().Body().Execution()
		require.NoError(t, err)
		require.Equal(t, uint64(1), e.BlockNumber()) // Local block because incorrect withdrawals
	})
	t.Run("Builder configured. Builder Block has higher value. Correct withdrawals.", func(t *testing.T) {
		blk, err := blocks.NewSignedBeaconBlock(util.NewBlindedBeaconBlockCapella())
		require.NoError(t, err)
		require.NoError(t, vs.BeaconDB.SaveRegistrationsByValidatorIDs(ctx, []primitives.ValidatorIndex{blk.Block().ProposerIndex()},
			[]*ethpb.ValidatorRegistrationV1{{FeeRecipient: make([]byte, fieldparams.FeeRecipientLength), Timestamp: uint64(time.Now().Unix()), Pubkey: make([]byte, fieldparams.BLSPubkeyLength)}}))
		ti, err := slots.ToTime(uint64(time.Now().Unix()), 0)
		require.NoError(t, err)
		sk, err := bls.RandKey()
		require.NoError(t, err)
		wr, err := ssz.WithdrawalSliceRoot(withdrawals, fieldparams.MaxWithdrawalsPerPayload)
		require.NoError(t, err)
		builderValue := bytesutil.ReverseByteOrder(big.NewInt(1e9).Bytes())
		bid := &ethpb.BuilderBidCapella{
			Header: &v1.ExecutionPayloadHeaderCapella{
				ParentHash:       params.BeaconConfig().ZeroHash[:],
				FeeRecipient:     make([]byte, fieldparams.FeeRecipientLength),
				StateRoot:        make([]byte, fieldparams.RootLength),
				ReceiptsRoot:     make([]byte, fieldparams.RootLength),
				LogsBloom:        make([]byte, fieldparams.LogsBloomLength),
				PrevRandao:       make([]byte, fieldparams.RootLength),
				BlockNumber:      2,
				Timestamp:        uint64(ti.Unix()),
				ExtraData:        make([]byte, 0),
				BaseFeePerGas:    make([]byte, fieldparams.RootLength),
				BlockHash:        make([]byte, fieldparams.RootLength),
				TransactionsRoot: bytesutil.PadTo([]byte{1}, fieldparams.RootLength),
				WithdrawalsRoot:  wr[:],
			},
			Pubkey: sk.PublicKey().Marshal(),
			Value:  bytesutil.PadTo(builderValue, 32),
		}
		d := params.BeaconConfig().DomainApplicationBuilder
		domain, err := signing.ComputeDomain(d, nil, nil)
		require.NoError(t, err)
		sr, err := signing.ComputeSigningRoot(bid, domain)
		require.NoError(t, err)
		sBid := &ethpb.SignedBuilderBidCapella{
			Message:   bid,
			Signature: sk.Sign(sr[:]).Marshal(),
		}
		vs.BlockBuilder = &builderTest.MockBuilderService{
			BidCapella:    sBid,
			HasConfigured: true,
			Cfg:           &builderTest.Config{BeaconDB: beaconDB},
		}
		wb, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlockCapella())
		require.NoError(t, err)
		chain := &blockchainTest.ChainService{ForkChoiceStore: doublylinkedtree.New(), Genesis: time.Now(), Block: wb}
		vs.ForkFetcher = chain
		vs.ForkchoiceFetcher.SetForkChoiceGenesisTime(uint64(time.Now().Unix()))
		vs.TimeFetcher = chain
		vs.HeadFetcher = chain

		b := blk.Block()
		localPayload, _, err := vs.getLocalPayload(ctx, b, capellaTransitionState)
		require.NoError(t, err)
		builderPayload, builderKzgCommitments, err := vs.getBuilderPayloadAndBlobs(ctx, b.Slot(), b.ProposerIndex())
		require.NoError(t, err)
		require.DeepEqual(t, [][]uint8(nil), builderKzgCommitments)
		require.NoError(t, setExecutionData(context.Background(), blk, localPayload, builderPayload, builderKzgCommitments, defaultBuilderBoostFactor))
		e, err := blk.Block().Body().Execution()
		require.NoError(t, err)
		require.Equal(t, uint64(2), e.BlockNumber()) // Builder block
	})
	t.Run("Max builder boost factor should return builder", func(t *testing.T) {
		blk, err := blocks.NewSignedBeaconBlock(util.NewBlindedBeaconBlockCapella())
		require.NoError(t, err)
		require.NoError(t, vs.BeaconDB.SaveRegistrationsByValidatorIDs(ctx, []primitives.ValidatorIndex{blk.Block().ProposerIndex()},
			[]*ethpb.ValidatorRegistrationV1{{FeeRecipient: make([]byte, fieldparams.FeeRecipientLength), Timestamp: uint64(time.Now().Unix()), Pubkey: make([]byte, fieldparams.BLSPubkeyLength)}}))
		ti, err := slots.ToTime(uint64(time.Now().Unix()), 0)
		require.NoError(t, err)
		sk, err := bls.RandKey()
		require.NoError(t, err)
		wr, err := ssz.WithdrawalSliceRoot(withdrawals, fieldparams.MaxWithdrawalsPerPayload)
		require.NoError(t, err)
		builderValue := bytesutil.ReverseByteOrder(big.NewInt(1e9).Bytes())
		bid := &ethpb.BuilderBidCapella{
			Header: &v1.ExecutionPayloadHeaderCapella{
				FeeRecipient:     make([]byte, fieldparams.FeeRecipientLength),
				StateRoot:        make([]byte, fieldparams.RootLength),
				ReceiptsRoot:     make([]byte, fieldparams.RootLength),
				LogsBloom:        make([]byte, fieldparams.LogsBloomLength),
				PrevRandao:       make([]byte, fieldparams.RootLength),
				BaseFeePerGas:    make([]byte, fieldparams.RootLength),
				BlockHash:        make([]byte, fieldparams.RootLength),
				TransactionsRoot: bytesutil.PadTo([]byte{1}, fieldparams.RootLength),
				ParentHash:       params.BeaconConfig().ZeroHash[:],
				Timestamp:        uint64(ti.Unix()),
				BlockNumber:      2,
				WithdrawalsRoot:  wr[:],
			},
			Pubkey: sk.PublicKey().Marshal(),
			Value:  bytesutil.PadTo(builderValue, 32),
		}
		d := params.BeaconConfig().DomainApplicationBuilder
		domain, err := signing.ComputeDomain(d, nil, nil)
		require.NoError(t, err)
		sr, err := signing.ComputeSigningRoot(bid, domain)
		require.NoError(t, err)
		sBid := &ethpb.SignedBuilderBidCapella{
			Message:   bid,
			Signature: sk.Sign(sr[:]).Marshal(),
		}
		vs.BlockBuilder = &builderTest.MockBuilderService{
			BidCapella:    sBid,
			HasConfigured: true,
			Cfg:           &builderTest.Config{BeaconDB: beaconDB},
		}
		wb, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlockCapella())
		require.NoError(t, err)
		chain := &blockchainTest.ChainService{ForkChoiceStore: doublylinkedtree.New(), Genesis: time.Now(), Block: wb}
		vs.ForkFetcher = chain
		vs.ForkchoiceFetcher.SetForkChoiceGenesisTime(uint64(time.Now().Unix()))
		vs.TimeFetcher = chain
		vs.HeadFetcher = chain

		b := blk.Block()
		localPayload, _, err := vs.getLocalPayload(ctx, b, capellaTransitionState)
		require.NoError(t, err)
		builderPayload, builderKzgCommitments, err := vs.getBuilderPayloadAndBlobs(ctx, b.Slot(), b.ProposerIndex())
		require.NoError(t, err)
		require.DeepEqual(t, [][]uint8(nil), builderKzgCommitments)
		require.NoError(t, setExecutionData(context.Background(), blk, localPayload, builderPayload, builderKzgCommitments, math.MaxUint64))
		e, err := blk.Block().Body().Execution()
		require.NoError(t, err)
		require.Equal(t, uint64(2), e.BlockNumber()) // builder block
	})
	t.Run("Builder builder has higher value but forced to local payload with builder boost factor", func(t *testing.T) {
		blk, err := blocks.NewSignedBeaconBlock(util.NewBlindedBeaconBlockCapella())
		require.NoError(t, err)
		require.NoError(t, vs.BeaconDB.SaveRegistrationsByValidatorIDs(ctx, []primitives.ValidatorIndex{blk.Block().ProposerIndex()},
			[]*ethpb.ValidatorRegistrationV1{{FeeRecipient: make([]byte, fieldparams.FeeRecipientLength), Timestamp: uint64(time.Now().Unix()), Pubkey: make([]byte, fieldparams.BLSPubkeyLength)}}))
		ti, err := slots.ToTime(uint64(time.Now().Unix()), 0)
		require.NoError(t, err)
		sk, err := bls.RandKey()
		require.NoError(t, err)
		wr, err := ssz.WithdrawalSliceRoot(withdrawals, fieldparams.MaxWithdrawalsPerPayload)
		require.NoError(t, err)
		builderValue := bytesutil.ReverseByteOrder(big.NewInt(1e9).Bytes())
		bid := &ethpb.BuilderBidCapella{
			Header: &v1.ExecutionPayloadHeaderCapella{
				FeeRecipient:     make([]byte, fieldparams.FeeRecipientLength),
				StateRoot:        make([]byte, fieldparams.RootLength),
				ReceiptsRoot:     make([]byte, fieldparams.RootLength),
				LogsBloom:        make([]byte, fieldparams.LogsBloomLength),
				PrevRandao:       make([]byte, fieldparams.RootLength),
				BaseFeePerGas:    make([]byte, fieldparams.RootLength),
				BlockHash:        make([]byte, fieldparams.RootLength),
				TransactionsRoot: bytesutil.PadTo([]byte{1}, fieldparams.RootLength),
				ParentHash:       params.BeaconConfig().ZeroHash[:],
				Timestamp:        uint64(ti.Unix()),
				BlockNumber:      2,
				WithdrawalsRoot:  wr[:],
			},
			Pubkey: sk.PublicKey().Marshal(),
			Value:  bytesutil.PadTo(builderValue, 32),
		}
		d := params.BeaconConfig().DomainApplicationBuilder
		domain, err := signing.ComputeDomain(d, nil, nil)
		require.NoError(t, err)
		sr, err := signing.ComputeSigningRoot(bid, domain)
		require.NoError(t, err)
		sBid := &ethpb.SignedBuilderBidCapella{
			Message:   bid,
			Signature: sk.Sign(sr[:]).Marshal(),
		}
		vs.BlockBuilder = &builderTest.MockBuilderService{
			BidCapella:    sBid,
			HasConfigured: true,
			Cfg:           &builderTest.Config{BeaconDB: beaconDB},
		}
		wb, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlockCapella())
		require.NoError(t, err)
		chain := &blockchainTest.ChainService{ForkChoiceStore: doublylinkedtree.New(), Genesis: time.Now(), Block: wb}
		vs.ForkFetcher = chain
		vs.ForkchoiceFetcher.SetForkChoiceGenesisTime(uint64(time.Now().Unix()))
		vs.TimeFetcher = chain
		vs.HeadFetcher = chain

		b := blk.Block()
		localPayload, _, err := vs.getLocalPayload(ctx, b, capellaTransitionState)
		require.NoError(t, err)
		builderPayload, builderKzgCommitments, err := vs.getBuilderPayloadAndBlobs(ctx, b.Slot(), b.ProposerIndex())
		require.NoError(t, err)
		require.DeepEqual(t, [][]uint8(nil), builderKzgCommitments)
		require.NoError(t, setExecutionData(context.Background(), blk, localPayload, builderPayload, builderKzgCommitments, 0))
		e, err := blk.Block().Body().Execution()
		require.NoError(t, err)
		require.Equal(t, uint64(1), e.BlockNumber()) // local block
	})
	t.Run("Builder configured. Local block has higher value", func(t *testing.T) {
		blk, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlockCapella())
		require.NoError(t, err)
		vs.ExecutionEngineCaller = &powtesting.EngineClient{PayloadIDBytes: id, ExecutionPayloadCapella: &v1.ExecutionPayloadCapella{BlockNumber: 3}, BlockValue: 2 * 1e9}
		b := blk.Block()
		localPayload, _, err := vs.getLocalPayload(ctx, b, capellaTransitionState)
		require.NoError(t, err)
		builderPayload, builderKzgCommitments, err := vs.getBuilderPayloadAndBlobs(ctx, b.Slot(), b.ProposerIndex())
		require.NoError(t, err)
		require.DeepEqual(t, [][]uint8(nil), builderKzgCommitments)
		require.NoError(t, setExecutionData(context.Background(), blk, localPayload, builderPayload, builderKzgCommitments, defaultBuilderBoostFactor))
		e, err := blk.Block().Body().Execution()
		require.NoError(t, err)
		require.Equal(t, uint64(3), e.BlockNumber()) // Local block

		require.LogsContain(t, hook, "builderGweiValue=1 localBoostPercentage=0 localGweiValue=2")
	})
	t.Run("Builder configured. Local block and local boost has higher value", func(t *testing.T) {
		cfg := params.BeaconConfig().Copy()
		cfg.LocalBlockValueBoost = 1 // Boost 1%.
		params.OverrideBeaconConfig(cfg)

		blk, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlockCapella())
		require.NoError(t, err)
		vs.ExecutionEngineCaller = &powtesting.EngineClient{PayloadIDBytes: id, ExecutionPayloadCapella: &v1.ExecutionPayloadCapella{BlockNumber: 3}, BlockValue: 1 * 1e9}
		b := blk.Block()
		localPayload, _, err := vs.getLocalPayload(ctx, b, capellaTransitionState)
		require.NoError(t, err)
		builderPayload, builderKzgCommitments, err := vs.getBuilderPayloadAndBlobs(ctx, b.Slot(), b.ProposerIndex())
		require.NoError(t, err)
		require.DeepEqual(t, [][]uint8(nil), builderKzgCommitments)
		require.NoError(t, setExecutionData(context.Background(), blk, localPayload, builderPayload, builderKzgCommitments, defaultBuilderBoostFactor))
		e, err := blk.Block().Body().Execution()
		require.NoError(t, err)
		require.Equal(t, uint64(3), e.BlockNumber()) // Local block

		require.LogsContain(t, hook, "builderGweiValue=1 localBoostPercentage=1 localGweiValue=1")
	})
	t.Run("Builder configured. Builder returns fault. Use local block", func(t *testing.T) {
		blk, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlockCapella())
		require.NoError(t, err)
		vs.BlockBuilder = &builderTest.MockBuilderService{
			ErrGetHeader:  errors.New("fault"),
			HasConfigured: true,
			Cfg:           &builderTest.Config{BeaconDB: beaconDB},
		}
		vs.ExecutionEngineCaller = &powtesting.EngineClient{PayloadIDBytes: id, ExecutionPayloadCapella: &v1.ExecutionPayloadCapella{BlockNumber: 4}, BlockValue: 0}
		b := blk.Block()
		localPayload, _, err := vs.getLocalPayload(ctx, b, capellaTransitionState)
		require.NoError(t, err)
		builderPayload, builderKzgCommitments, err := vs.getBuilderPayloadAndBlobs(ctx, b.Slot(), b.ProposerIndex())
		require.ErrorIs(t, consensus_types.ErrNilObjectWrapped, err) // Builder returns fault. Use local block
		require.DeepEqual(t, [][]uint8(nil), builderKzgCommitments)
		require.NoError(t, setExecutionData(context.Background(), blk, localPayload, builderPayload, builderKzgCommitments, defaultBuilderBoostFactor))
		e, err := blk.Block().Body().Execution()
		require.NoError(t, err)
		require.Equal(t, uint64(4), e.BlockNumber()) // Local block
	})
	t.Run("Can get local payload and blobs Deneb", func(t *testing.T) {
		cfg := params.BeaconConfig().Copy()
		cfg.DenebForkEpoch = 0
		params.OverrideBeaconConfig(cfg)
		params.SetupTestConfigCleanup(t)

		blk, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlockDeneb())
		blk.SetSlot(1)
		require.NoError(t, err)
		vs.BlockBuilder = &builderTest.MockBuilderService{
			HasConfigured: false,
		}
		blobsBundle := &v1.BlobsBundle{
			KzgCommitments: [][]byte{{1, 2, 3}},
			Proofs:         [][]byte{{4, 5, 6}},
			Blobs:          [][]byte{{7, 8, 9}},
		}
		vs.ExecutionEngineCaller = &powtesting.EngineClient{
			PayloadIDBytes:        id,
			BlobsBundle:           blobsBundle,
			ExecutionPayloadDeneb: &v1.ExecutionPayloadDeneb{BlockNumber: 4},
			BlockValue:            0}
		blk.SetSlot(primitives.Slot(params.BeaconConfig().DenebForkEpoch) * params.BeaconConfig().SlotsPerEpoch)
		localPayload, _, err := vs.getLocalPayload(ctx, blk.Block(), capellaTransitionState)
		require.NoError(t, err)
		require.Equal(t, uint64(4), localPayload.BlockNumber())
		cachedBundle := bundleCache.get(blk.Block().Slot())
		require.DeepEqual(t, cachedBundle, blobsBundle)
	})
	t.Run("Can get builder payload and blobs in Deneb", func(t *testing.T) {
		cfg := params.BeaconConfig().Copy()
		cfg.DenebForkEpoch = 0
		params.OverrideBeaconConfig(cfg)
		params.SetupTestConfigCleanup(t)

		blk, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlockDeneb())
		require.NoError(t, err)
		ti, err := slots.ToTime(uint64(time.Now().Unix()), 0)
		require.NoError(t, err)
		sk, err := bls.RandKey()
		require.NoError(t, err)
		wr, err := ssz.WithdrawalSliceRoot(withdrawals, fieldparams.MaxWithdrawalsPerPayload)
		require.NoError(t, err)
		builderValue := bytesutil.ReverseByteOrder(big.NewInt(1e9).Bytes())

		bid := &ethpb.BuilderBidDeneb{
			Header: &v1.ExecutionPayloadHeaderDeneb{
				FeeRecipient:     make([]byte, fieldparams.FeeRecipientLength),
				StateRoot:        make([]byte, fieldparams.RootLength),
				ReceiptsRoot:     make([]byte, fieldparams.RootLength),
				LogsBloom:        make([]byte, fieldparams.LogsBloomLength),
				PrevRandao:       make([]byte, fieldparams.RootLength),
				BaseFeePerGas:    make([]byte, fieldparams.RootLength),
				BlockHash:        make([]byte, fieldparams.RootLength),
				TransactionsRoot: bytesutil.PadTo([]byte{1}, fieldparams.RootLength),
				ParentHash:       params.BeaconConfig().ZeroHash[:],
				Timestamp:        uint64(ti.Unix()),
				BlockNumber:      2,
				WithdrawalsRoot:  wr[:],
				BlobGasUsed:      123,
				ExcessBlobGas:    456,
			},
			Pubkey:             sk.PublicKey().Marshal(),
			Value:              bytesutil.PadTo(builderValue, 32),
			BlobKzgCommitments: [][]byte{bytesutil.PadTo([]byte{2}, fieldparams.BLSPubkeyLength), bytesutil.PadTo([]byte{5}, fieldparams.BLSPubkeyLength)},
		}

		d := params.BeaconConfig().DomainApplicationBuilder
		domain, err := signing.ComputeDomain(d, nil, nil)
		require.NoError(t, err)
		sr, err := signing.ComputeSigningRoot(bid, domain)
		require.NoError(t, err)
		sBid := &ethpb.SignedBuilderBidDeneb{
			Message:   bid,
			Signature: sk.Sign(sr[:]).Marshal(),
		}
		vs.BlockBuilder = &builderTest.MockBuilderService{
			BidDeneb:      sBid,
			HasConfigured: true,
			Cfg:           &builderTest.Config{BeaconDB: beaconDB},
		}
		require.NoError(t, beaconDB.SaveRegistrationsByValidatorIDs(ctx, []primitives.ValidatorIndex{blk.Block().ProposerIndex()},
			[]*ethpb.ValidatorRegistrationV1{{FeeRecipient: make([]byte, fieldparams.FeeRecipientLength), Timestamp: uint64(time.Now().Unix()), Pubkey: make([]byte, fieldparams.BLSPubkeyLength)}}))

		wb, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlockDeneb())
		require.NoError(t, err)
		chain := &blockchainTest.ChainService{ForkChoiceStore: doublylinkedtree.New(), Genesis: time.Now(), Block: wb}
		vs.ForkFetcher = chain
		vs.ForkchoiceFetcher.SetForkChoiceGenesisTime(uint64(time.Now().Unix()))
		vs.TimeFetcher = chain
		vs.HeadFetcher = chain
		vs.ExecutionEngineCaller = &powtesting.EngineClient{PayloadIDBytes: id, ExecutionPayloadDeneb: &v1.ExecutionPayloadDeneb{BlockNumber: 4, Withdrawals: withdrawals}, BlockValue: 0}

		require.NoError(t, err)
		blk.SetSlot(primitives.Slot(params.BeaconConfig().DenebForkEpoch) * params.BeaconConfig().SlotsPerEpoch)
		require.NoError(t, err)
		builderPayload, builderKzgCommitments, err := vs.getBuilderPayloadAndBlobs(ctx, blk.Block().Slot(), blk.Block().ProposerIndex())
		require.NoError(t, err)
		require.DeepEqual(t, bid.BlobKzgCommitments, builderKzgCommitments)
		require.Equal(t, bid.Header.BlockNumber, builderPayload.BlockNumber()) // header should be the same from block

		localPayload, _, err := vs.getLocalPayload(ctx, blk.Block(), denebTransitionState)
		require.NoError(t, err)
		require.NoError(t, setExecutionData(context.Background(), blk, localPayload, builderPayload, builderKzgCommitments, defaultBuilderBoostFactor))

		got, err := blk.Block().Body().BlobKzgCommitments()
		require.NoError(t, err)
		require.DeepEqual(t, bid.BlobKzgCommitments, got)
	})
}

func TestServer_getPayloadHeader(t *testing.T) {
	genesis := time.Now().Add(-time.Duration(params.BeaconConfig().SlotsPerEpoch) * time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Second)
	params.SetupTestConfigCleanup(t)
	bc := params.BeaconConfig()
	bc.BellatrixForkEpoch = 1
	params.OverrideBeaconConfig(bc)
	fakeCapellaEpoch := primitives.Epoch(10)
	params.SetupTestConfigCleanup(t)
	cfg := params.BeaconConfig().Copy()
	cfg.CapellaForkVersion = []byte{'A', 'B', 'C', 'Z'}
	cfg.CapellaForkEpoch = fakeCapellaEpoch
	cfg.InitializeForkSchedule()
	params.OverrideBeaconConfig(cfg)
	emptyRoot, err := ssz.TransactionsRoot([][]byte{})
	require.NoError(t, err)
	ti, err := slots.ToTime(uint64(time.Now().Unix()), 0)
	require.NoError(t, err)

	sk, err := bls.RandKey()
	require.NoError(t, err)
	bid := &ethpb.BuilderBid{
		Header: &v1.ExecutionPayloadHeader{
			FeeRecipient:     make([]byte, fieldparams.FeeRecipientLength),
			StateRoot:        make([]byte, fieldparams.RootLength),
			ReceiptsRoot:     make([]byte, fieldparams.RootLength),
			LogsBloom:        make([]byte, fieldparams.LogsBloomLength),
			PrevRandao:       make([]byte, fieldparams.RootLength),
			BaseFeePerGas:    make([]byte, fieldparams.RootLength),
			BlockHash:        make([]byte, fieldparams.RootLength),
			TransactionsRoot: bytesutil.PadTo([]byte{1}, fieldparams.RootLength),
			ParentHash:       params.BeaconConfig().ZeroHash[:],
			Timestamp:        uint64(ti.Unix()),
		},
		Pubkey: sk.PublicKey().Marshal(),
		Value:  bytesutil.PadTo([]byte{1, 2, 3}, 32),
	}
	d := params.BeaconConfig().DomainApplicationBuilder
	domain, err := signing.ComputeDomain(d, nil, nil)
	require.NoError(t, err)
	sr, err := signing.ComputeSigningRoot(bid, domain)
	require.NoError(t, err)
	sBid := &ethpb.SignedBuilderBid{
		Message:   bid,
		Signature: sk.Sign(sr[:]).Marshal(),
	}
	withdrawals := []*v1.Withdrawal{{
		Index:          1,
		ValidatorIndex: 2,
		Address:        make([]byte, fieldparams.FeeRecipientLength),
		Amount:         3,
	}}
	wr, err := ssz.WithdrawalSliceRoot(withdrawals, fieldparams.MaxWithdrawalsPerPayload)
	require.NoError(t, err)

	tiCapella, err := slots.ToTime(uint64(genesis.Unix()), primitives.Slot(fakeCapellaEpoch)*params.BeaconConfig().SlotsPerEpoch)
	require.NoError(t, err)
	bidCapella := &ethpb.BuilderBidCapella{
		Header: &v1.ExecutionPayloadHeaderCapella{
			FeeRecipient:     make([]byte, fieldparams.FeeRecipientLength),
			StateRoot:        make([]byte, fieldparams.RootLength),
			ReceiptsRoot:     make([]byte, fieldparams.RootLength),
			LogsBloom:        make([]byte, fieldparams.LogsBloomLength),
			PrevRandao:       make([]byte, fieldparams.RootLength),
			BaseFeePerGas:    make([]byte, fieldparams.RootLength),
			BlockHash:        make([]byte, fieldparams.RootLength),
			TransactionsRoot: bytesutil.PadTo([]byte{1}, fieldparams.RootLength),
			ParentHash:       params.BeaconConfig().ZeroHash[:],
			Timestamp:        uint64(tiCapella.Unix()),
			WithdrawalsRoot:  wr[:],
		},
		Pubkey: sk.PublicKey().Marshal(),
		Value:  bytesutil.PadTo([]byte{1, 2, 3}, 32),
	}
	srCapella, err := signing.ComputeSigningRoot(bidCapella, domain)
	require.NoError(t, err)
	sBidCapella := &ethpb.SignedBuilderBidCapella{
		Message:   bidCapella,
		Signature: sk.Sign(srCapella[:]).Marshal(),
	}

	require.NoError(t, err)
	tests := []struct {
		name                  string
		head                  interfaces.ReadOnlySignedBeaconBlock
		mock                  *builderTest.MockBuilderService
		fetcher               *blockchainTest.ChainService
		err                   string
		returnedHeader        *v1.ExecutionPayloadHeader
		returnedHeaderCapella *v1.ExecutionPayloadHeaderCapella
	}{
		{
			name: "can't request before bellatrix epoch",
			mock: &builderTest.MockBuilderService{},
			fetcher: &blockchainTest.ChainService{
				Block: func() interfaces.ReadOnlySignedBeaconBlock {
					wb, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlock())
					require.NoError(t, err)
					return wb
				}(),
			},
			err: "can't get payload header from builder before bellatrix epoch",
		},
		{
			name: "get header failed",
			mock: &builderTest.MockBuilderService{
				ErrGetHeader: errors.New("can't get header"),
				Bid:          sBid,
			},
			fetcher: &blockchainTest.ChainService{
				Block: func() interfaces.ReadOnlySignedBeaconBlock {
					wb, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlockBellatrix())
					require.NoError(t, err)
					wb.SetSlot(primitives.Slot(params.BeaconConfig().BellatrixForkEpoch) * params.BeaconConfig().SlotsPerEpoch)
					return wb
				}(),
			},
			err: "can't get header",
		},
		{
			name: "0 bid",
			mock: &builderTest.MockBuilderService{
				Bid: &ethpb.SignedBuilderBid{
					Message: &ethpb.BuilderBid{
						Header: &v1.ExecutionPayloadHeader{
							BlockNumber: 123,
						},
					},
				},
			},
			fetcher: &blockchainTest.ChainService{
				Block: func() interfaces.ReadOnlySignedBeaconBlock {
					wb, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlockBellatrix())
					require.NoError(t, err)
					wb.SetSlot(primitives.Slot(params.BeaconConfig().BellatrixForkEpoch) * params.BeaconConfig().SlotsPerEpoch)
					return wb
				}(),
			},
			err: "builder returned header with 0 bid amount",
		},
		{
			name: "invalid tx root",
			mock: &builderTest.MockBuilderService{
				Bid: &ethpb.SignedBuilderBid{
					Message: &ethpb.BuilderBid{
						Value: []byte{1},
						Header: &v1.ExecutionPayloadHeader{
							BlockNumber:      123,
							TransactionsRoot: emptyRoot[:],
						},
					},
				},
			},
			fetcher: &blockchainTest.ChainService{
				Block: func() interfaces.ReadOnlySignedBeaconBlock {
					wb, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlockBellatrix())
					require.NoError(t, err)
					wb.SetSlot(primitives.Slot(params.BeaconConfig().BellatrixForkEpoch) * params.BeaconConfig().SlotsPerEpoch)
					return wb
				}(),
			},
			err: "builder returned header with an empty tx root",
		},
		{
			name: "can get header",
			mock: &builderTest.MockBuilderService{
				Bid: sBid,
			},
			fetcher: &blockchainTest.ChainService{
				Block: func() interfaces.ReadOnlySignedBeaconBlock {
					wb, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlockBellatrix())
					require.NoError(t, err)
					wb.SetSlot(primitives.Slot(params.BeaconConfig().BellatrixForkEpoch) * params.BeaconConfig().SlotsPerEpoch)
					return wb
				}(),
			},
			returnedHeader: bid.Header,
		},
		{
			name: "wrong bid version",
			mock: &builderTest.MockBuilderService{
				BidCapella: sBidCapella,
			},
			fetcher: &blockchainTest.ChainService{
				Block: func() interfaces.ReadOnlySignedBeaconBlock {
					wb, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlockBellatrix())
					require.NoError(t, err)
					wb.SetSlot(primitives.Slot(params.BeaconConfig().BellatrixForkEpoch) * params.BeaconConfig().SlotsPerEpoch)
					return wb
				}(),
			},
			err: "is different from head block version",
		},
		{
			name: "different bid version during hard fork",
			mock: &builderTest.MockBuilderService{
				BidCapella: sBidCapella,
			},
			fetcher: &blockchainTest.ChainService{
				Block: func() interfaces.ReadOnlySignedBeaconBlock {
					wb, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlockBellatrix())
					require.NoError(t, err)
					wb.SetSlot(primitives.Slot(fakeCapellaEpoch) * params.BeaconConfig().SlotsPerEpoch)
					return wb
				}(),
			},
			returnedHeaderCapella: bidCapella.Header,
		},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			vs := &Server{BlockBuilder: tc.mock, HeadFetcher: tc.fetcher, TimeFetcher: &blockchainTest.ChainService{
				Genesis: genesis,
			}}
			hb, err := vs.HeadFetcher.HeadBlock(context.Background())
			require.NoError(t, err)
			h, _, err := vs.getPayloadHeaderFromBuilder(context.Background(), hb.Block().Slot(), 0)
			if tc.err != "" {
				require.ErrorContains(t, tc.err, err)
			} else {
				require.NoError(t, err)
				if tc.returnedHeader != nil {
					want, err := blocks.WrappedExecutionPayloadHeader(tc.returnedHeader)
					require.NoError(t, err)
					require.DeepEqual(t, want, h)
				}
				if tc.returnedHeaderCapella != nil {
					want, err := blocks.WrappedExecutionPayloadHeaderCapella(tc.returnedHeaderCapella, big.NewInt(197121)) // value is a mock
					require.NoError(t, err)
					require.DeepEqual(t, want, h)
				}
			}
		})
	}
}

func TestServer_validateBuilderSignature(t *testing.T) {
	sk, err := bls.RandKey()
	require.NoError(t, err)
	bid := &ethpb.BuilderBid{
		Header: &v1.ExecutionPayloadHeader{
			ParentHash:       make([]byte, fieldparams.RootLength),
			FeeRecipient:     make([]byte, fieldparams.FeeRecipientLength),
			StateRoot:        make([]byte, fieldparams.RootLength),
			ReceiptsRoot:     make([]byte, fieldparams.RootLength),
			LogsBloom:        make([]byte, fieldparams.LogsBloomLength),
			PrevRandao:       make([]byte, fieldparams.RootLength),
			BaseFeePerGas:    make([]byte, fieldparams.RootLength),
			BlockHash:        make([]byte, fieldparams.RootLength),
			TransactionsRoot: make([]byte, fieldparams.RootLength),
			BlockNumber:      1,
		},
		Pubkey: sk.PublicKey().Marshal(),
		Value:  bytesutil.PadTo([]byte{1, 2, 3}, 32),
	}
	d := params.BeaconConfig().DomainApplicationBuilder
	domain, err := signing.ComputeDomain(d, nil, nil)
	require.NoError(t, err)
	sr, err := signing.ComputeSigningRoot(bid, domain)
	require.NoError(t, err)
	pbBid := &ethpb.SignedBuilderBid{
		Message:   bid,
		Signature: sk.Sign(sr[:]).Marshal(),
	}
	sBid, err := builder.WrappedSignedBuilderBid(pbBid)
	require.NoError(t, err)
	require.NoError(t, validateBuilderSignature(sBid))

	pbBid.Message.Value = make([]byte, 32)
	sBid, err = builder.WrappedSignedBuilderBid(pbBid)
	require.NoError(t, err)
	require.ErrorIs(t, validateBuilderSignature(sBid), signing.ErrSigFailedToVerify)
}

func Test_matchingWithdrawalsRoot(t *testing.T) {
	t.Run("could not get local withdrawals", func(t *testing.T) {
		local := &v1.ExecutionPayload{}
		p, err := blocks.WrappedExecutionPayload(local)
		require.NoError(t, err)
		_, err = matchingWithdrawalsRoot(p, p)
		require.ErrorContains(t, "could not get local withdrawals", err)
	})
	t.Run("could not get builder withdrawals root", func(t *testing.T) {
		local := &v1.ExecutionPayloadCapella{}
		p, err := blocks.WrappedExecutionPayloadCapella(local, big.NewInt(0))
		require.NoError(t, err)
		header := &v1.ExecutionPayloadHeader{}
		h, err := blocks.WrappedExecutionPayloadHeader(header)
		require.NoError(t, err)
		_, err = matchingWithdrawalsRoot(p, h)
		require.ErrorContains(t, "could not get builder withdrawals root", err)
	})
	t.Run("withdrawals mismatch", func(t *testing.T) {
		local := &v1.ExecutionPayloadCapella{}
		p, err := blocks.WrappedExecutionPayloadCapella(local, big.NewInt(0))
		require.NoError(t, err)
		header := &v1.ExecutionPayloadHeaderCapella{}
		h, err := blocks.WrappedExecutionPayloadHeaderCapella(header, big.NewInt(0))
		require.NoError(t, err)
		matched, err := matchingWithdrawalsRoot(p, h)
		require.NoError(t, err)
		require.Equal(t, false, matched)
	})
	t.Run("withdrawals match", func(t *testing.T) {
		wds := []*v1.Withdrawal{{
			Index:          1,
			ValidatorIndex: 2,
			Address:        make([]byte, fieldparams.FeeRecipientLength),
			Amount:         3,
		}}
		local := &v1.ExecutionPayloadCapella{Withdrawals: wds}
		p, err := blocks.WrappedExecutionPayloadCapella(local, big.NewInt(0))
		require.NoError(t, err)
		header := &v1.ExecutionPayloadHeaderCapella{}
		wr, err := ssz.WithdrawalSliceRoot(wds, fieldparams.MaxWithdrawalsPerPayload)
		require.NoError(t, err)
		header.WithdrawalsRoot = wr[:]
		h, err := blocks.WrappedExecutionPayloadHeaderCapella(header, big.NewInt(0))
		require.NoError(t, err)
		matched, err := matchingWithdrawalsRoot(p, h)
		require.NoError(t, err)
		require.Equal(t, true, matched)
	})
}

func TestEmptyTransactionsRoot(t *testing.T) {
	r, err := ssz.TransactionsRoot([][]byte{})
	require.NoError(t, err)
	require.DeepEqual(t, r, emptyTransactionsRoot)
}
