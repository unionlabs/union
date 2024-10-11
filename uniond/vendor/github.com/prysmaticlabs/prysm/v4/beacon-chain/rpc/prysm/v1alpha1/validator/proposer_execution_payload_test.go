package validator

import (
	"context"
	"errors"
	"math/big"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	gethtypes "github.com/ethereum/go-ethereum/core/types"
	chainMock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	dbTest "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	powtesting "github.com/prysmaticlabs/prysm/v4/beacon-chain/execution/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	pb "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func TestServer_activationEpochNotReached(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	require.Equal(t, false, activationEpochNotReached(0))

	cfg := params.BeaconConfig().Copy()
	cfg.TerminalBlockHash = common.BytesToHash(bytesutil.PadTo([]byte{0x01}, 32))
	cfg.TerminalBlockHashActivationEpoch = 1
	params.OverrideBeaconConfig(cfg)

	require.Equal(t, true, activationEpochNotReached(0))
	require.Equal(t, false, activationEpochNotReached(params.BeaconConfig().SlotsPerEpoch+1))
}

func TestServer_getExecutionPayload(t *testing.T) {
	beaconDB := dbTest.SetupDB(t)
	nonTransitionSt, _ := util.DeterministicGenesisStateBellatrix(t, 1)
	b1pb := util.NewBeaconBlock()
	b1r, err := b1pb.Block.HashTreeRoot()
	require.NoError(t, err)
	util.SaveBlock(t, context.Background(), beaconDB, b1pb)
	require.NoError(t, nonTransitionSt.SetFinalizedCheckpoint(&ethpb.Checkpoint{
		Root: b1r[:],
	}))

	transitionSt, _ := util.DeterministicGenesisStateBellatrix(t, 1)
	wrappedHeader, err := blocks.WrappedExecutionPayloadHeader(&pb.ExecutionPayloadHeader{BlockNumber: 1})
	require.NoError(t, err)
	require.NoError(t, transitionSt.SetLatestExecutionPayloadHeader(wrappedHeader))
	b2pb := util.NewBeaconBlockBellatrix()
	b2r, err := b2pb.Block.HashTreeRoot()
	require.NoError(t, err)
	util.SaveBlock(t, context.Background(), beaconDB, b2pb)
	require.NoError(t, transitionSt.SetFinalizedCheckpoint(&ethpb.Checkpoint{
		Root: b2r[:],
	}))

	capellaTransitionState, _ := util.DeterministicGenesisStateCapella(t, 1)
	wrappedHeaderCapella, err := blocks.WrappedExecutionPayloadHeaderCapella(&pb.ExecutionPayloadHeaderCapella{BlockNumber: 1}, big.NewInt(0))
	require.NoError(t, err)
	require.NoError(t, capellaTransitionState.SetLatestExecutionPayloadHeader(wrappedHeaderCapella))
	b2pbCapella := util.NewBeaconBlockCapella()
	b2rCapella, err := b2pbCapella.Block.HashTreeRoot()
	require.NoError(t, err)
	util.SaveBlock(t, context.Background(), beaconDB, b2pbCapella)
	require.NoError(t, capellaTransitionState.SetFinalizedCheckpoint(&ethpb.Checkpoint{
		Root: b2rCapella[:],
	}))

	tests := []struct {
		name              string
		st                state.BeaconState
		errString         string
		forkchoiceErr     error
		payloadID         *pb.PayloadIDBytes
		terminalBlockHash common.Hash
		activationEpoch   primitives.Epoch
		validatorIndx     primitives.ValidatorIndex
		override          bool
		wantedOverride    bool
	}{
		{
			name:          "transition completed, nil payload id",
			st:            transitionSt,
			validatorIndx: 2,
			errString:     "nil payload with block hash",
		},
		{
			name:      "transition completed, happy case (has fee recipient in Db)",
			st:        transitionSt,
			payloadID: &pb.PayloadIDBytes{0x1},
		},
		{
			name:          "transition completed, happy case (doesn't have fee recipient in Db)",
			st:            transitionSt,
			payloadID:     &pb.PayloadIDBytes{0x1},
			validatorIndx: 1,
		},
		{
			name:          "transition completed, capella, happy case (doesn't have fee recipient in Db)",
			st:            capellaTransitionState,
			payloadID:     &pb.PayloadIDBytes{0x1},
			validatorIndx: 1,
		},
		{
			name:          "transition completed, happy case, (payload ID cached)",
			st:            transitionSt,
			payloadID:     &pb.PayloadIDBytes{0x1},
			validatorIndx: 100,
		},
		{
			name:          "transition completed, could not prepare payload",
			st:            transitionSt,
			forkchoiceErr: errors.New("fork choice error"),
			errString:     "could not prepare payload",
		},
		{
			name:      "transition not-completed, latest exec block is nil",
			st:        nonTransitionSt,
			errString: "latest execution block is nil",
		},
		{
			name:              "transition not-completed, activation epoch not reached",
			st:                nonTransitionSt,
			terminalBlockHash: [32]byte{0x1},
			activationEpoch:   1,
		},
		{
			name:           "local client override",
			st:             transitionSt,
			validatorIndx:  100,
			override:       true,
			payloadID:      &pb.PayloadIDBytes{0x1},
			wantedOverride: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			cfg := params.BeaconConfig().Copy()
			cfg.TerminalBlockHash = tt.terminalBlockHash
			cfg.TerminalBlockHashActivationEpoch = tt.activationEpoch
			params.OverrideBeaconConfig(cfg)

			vs := &Server{
				ExecutionEngineCaller:  &powtesting.EngineClient{PayloadIDBytes: tt.payloadID, ErrForkchoiceUpdated: tt.forkchoiceErr, ExecutionPayload: &pb.ExecutionPayload{}, BuilderOverride: tt.override},
				HeadFetcher:            &chainMock.ChainService{State: tt.st},
				FinalizationFetcher:    &chainMock.ChainService{},
				BeaconDB:               beaconDB,
				PayloadIDCache:         cache.NewPayloadIDCache(),
				TrackedValidatorsCache: cache.NewTrackedValidatorsCache(),
			}
			vs.TrackedValidatorsCache.Set(cache.TrackedValidator{Active: true, Index: tt.validatorIndx})
			if tt.payloadID != nil {
				vs.PayloadIDCache.Set(tt.st.Slot(), [32]byte{'a'}, [8]byte(*tt.payloadID))
			}
			blk := util.NewBeaconBlockBellatrix()
			blk.Block.Slot = tt.st.Slot()
			blk.Block.ProposerIndex = tt.validatorIndx
			blk.Block.ParentRoot = bytesutil.PadTo([]byte{'a'}, 32)
			b, err := blocks.NewSignedBeaconBlock(blk)
			require.NoError(t, err)
			var gotOverride bool
			_, gotOverride, err = vs.getLocalPayload(context.Background(), b.Block(), tt.st)
			if tt.errString != "" {
				require.ErrorContains(t, tt.errString, err)
			} else {
				require.Equal(t, tt.wantedOverride, gotOverride)
				require.NoError(t, err)
			}
		})
	}
}

func TestServer_getExecutionPayloadContextTimeout(t *testing.T) {
	beaconDB := dbTest.SetupDB(t)
	nonTransitionSt, _ := util.DeterministicGenesisStateBellatrix(t, 1)
	b1pb := util.NewBeaconBlock()
	b1r, err := b1pb.Block.HashTreeRoot()
	require.NoError(t, err)
	util.SaveBlock(t, context.Background(), beaconDB, b1pb)
	require.NoError(t, nonTransitionSt.SetFinalizedCheckpoint(&ethpb.Checkpoint{
		Root: b1r[:],
	}))

	require.NoError(t, beaconDB.SaveFeeRecipientsByValidatorIDs(context.Background(), []primitives.ValidatorIndex{0}, []common.Address{{}}))

	cfg := params.BeaconConfig().Copy()
	cfg.TerminalBlockHash = common.Hash{'a'}
	cfg.TerminalBlockHashActivationEpoch = 1
	params.OverrideBeaconConfig(cfg)

	vs := &Server{
		ExecutionEngineCaller:  &powtesting.EngineClient{PayloadIDBytes: &pb.PayloadIDBytes{}, ErrGetPayload: context.DeadlineExceeded, ExecutionPayload: &pb.ExecutionPayload{}},
		HeadFetcher:            &chainMock.ChainService{State: nonTransitionSt},
		BeaconDB:               beaconDB,
		PayloadIDCache:         cache.NewPayloadIDCache(),
		TrackedValidatorsCache: cache.NewTrackedValidatorsCache(),
	}
	vs.PayloadIDCache.Set(nonTransitionSt.Slot(), [32]byte{'a'}, [8]byte{100})

	blk := util.NewBeaconBlockBellatrix()
	blk.Block.Slot = nonTransitionSt.Slot()
	blk.Block.ProposerIndex = 100
	blk.Block.ParentRoot = bytesutil.PadTo([]byte{'a'}, 32)
	b, err := blocks.NewSignedBeaconBlock(blk)
	require.NoError(t, err)
	_, _, err = vs.getLocalPayload(context.Background(), b.Block(), nonTransitionSt)
	require.NoError(t, err)
}

func TestServer_getExecutionPayload_UnexpectedFeeRecipient(t *testing.T) {
	hook := logTest.NewGlobal()
	beaconDB := dbTest.SetupDB(t)
	nonTransitionSt, _ := util.DeterministicGenesisStateBellatrix(t, 1)
	b1pb := util.NewBeaconBlock()
	b1r, err := b1pb.Block.HashTreeRoot()
	require.NoError(t, err)
	util.SaveBlock(t, context.Background(), beaconDB, b1pb)
	require.NoError(t, nonTransitionSt.SetFinalizedCheckpoint(&ethpb.Checkpoint{
		Root: b1r[:],
	}))

	transitionSt, _ := util.DeterministicGenesisStateBellatrix(t, 1)
	wrappedHeader, err := blocks.WrappedExecutionPayloadHeader(&pb.ExecutionPayloadHeader{BlockNumber: 1})
	require.NoError(t, err)
	require.NoError(t, transitionSt.SetLatestExecutionPayloadHeader(wrappedHeader))
	b2pb := util.NewBeaconBlockBellatrix()
	b2r, err := b2pb.Block.HashTreeRoot()
	require.NoError(t, err)
	util.SaveBlock(t, context.Background(), beaconDB, b2pb)
	require.NoError(t, transitionSt.SetFinalizedCheckpoint(&ethpb.Checkpoint{
		Root: b2r[:],
	}))

	feeRecipient := common.BytesToAddress([]byte("a"))
	payloadID := &pb.PayloadIDBytes{0x1}
	payload := emptyPayload()
	payload.FeeRecipient = feeRecipient[:]
	vs := &Server{
		ExecutionEngineCaller: &powtesting.EngineClient{
			PayloadIDBytes:   payloadID,
			ExecutionPayload: payload,
		},
		HeadFetcher:            &chainMock.ChainService{State: transitionSt},
		FinalizationFetcher:    &chainMock.ChainService{},
		BeaconDB:               beaconDB,
		PayloadIDCache:         cache.NewPayloadIDCache(),
		TrackedValidatorsCache: cache.NewTrackedValidatorsCache(),
	}
	val := cache.TrackedValidator{
		Active:       true,
		FeeRecipient: primitives.ExecutionAddress(feeRecipient),
		Index:        0,
	}
	vs.TrackedValidatorsCache.Set(val)

	blk := util.NewBeaconBlockBellatrix()
	blk.Block.Slot = transitionSt.Slot()
	blk.Block.ParentRoot = bytesutil.PadTo([]byte{}, 32)
	b, err := blocks.NewSignedBeaconBlock(blk)
	require.NoError(t, err)
	gotPayload, _, err := vs.getLocalPayload(context.Background(), b.Block(), transitionSt)
	require.NoError(t, err)
	require.NotNil(t, gotPayload)
	require.Equal(t, common.Address(gotPayload.FeeRecipient()), feeRecipient)

	// We should NOT be getting the warning.
	require.LogsDoNotContain(t, hook, "Fee recipient address from execution client is not what was expected")
	hook.Reset()

	evilRecipientAddress := common.BytesToAddress([]byte("evil"))
	payload.FeeRecipient = evilRecipientAddress[:]
	vs.PayloadIDCache = cache.NewPayloadIDCache()

	gotPayload, _, err = vs.getLocalPayload(context.Background(), b.Block(), transitionSt)
	require.NoError(t, err)
	require.NotNil(t, gotPayload)

	// Users should be warned.
	require.LogsContain(t, hook, "Fee recipient address from execution client is not what was expected")
}

func TestServer_getTerminalBlockHashIfExists(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	tests := []struct {
		name                  string
		paramsTerminalHash    []byte
		paramsTd              string
		currentPowBlock       *pb.ExecutionBlock
		parentPowBlock        *pb.ExecutionBlock
		wantTerminalBlockHash []byte
		wantExists            bool
		errString             string
	}{
		{
			name:               "use terminal block hash, doesn't exist",
			paramsTerminalHash: common.BytesToHash([]byte("a")).Bytes(),
			errString:          "could not fetch height for hash",
		},
		{
			name: "use terminal block hash, exists",
			paramsTerminalHash: []byte{0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
				0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff},
			wantExists: true,
			wantTerminalBlockHash: []byte{0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
				0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff},
		},
		{
			name:     "use terminal total difficulty",
			paramsTd: "2",
			currentPowBlock: &pb.ExecutionBlock{
				Hash: common.BytesToHash([]byte("a")),
				Header: gethtypes.Header{
					ParentHash: common.BytesToHash([]byte("b")),
				},
				TotalDifficulty: "0x3",
			},
			parentPowBlock: &pb.ExecutionBlock{
				Hash: common.BytesToHash([]byte("b")),
				Header: gethtypes.Header{
					ParentHash: common.BytesToHash([]byte("c")),
				},
				TotalDifficulty: "0x1",
			},
			wantExists:            true,
			wantTerminalBlockHash: common.BytesToHash([]byte("a")).Bytes(),
		},
		{
			name:     "use terminal total difficulty but fails timestamp",
			paramsTd: "2",
			currentPowBlock: &pb.ExecutionBlock{
				Hash: common.BytesToHash([]byte("a")),
				Header: gethtypes.Header{
					ParentHash: common.BytesToHash([]byte("b")),
					Time:       1,
				},
				TotalDifficulty: "0x3",
			},
			parentPowBlock: &pb.ExecutionBlock{
				Hash: common.BytesToHash([]byte("b")),
				Header: gethtypes.Header{
					ParentHash: common.BytesToHash([]byte("c")),
				},
				TotalDifficulty: "0x1",
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			cfg := params.BeaconConfig().Copy()
			cfg.TerminalTotalDifficulty = tt.paramsTd
			cfg.TerminalBlockHash = common.BytesToHash(tt.paramsTerminalHash)
			params.OverrideBeaconConfig(cfg)
			var m map[[32]byte]*pb.ExecutionBlock
			if tt.parentPowBlock != nil {
				m = map[[32]byte]*pb.ExecutionBlock{
					tt.parentPowBlock.Hash: tt.parentPowBlock,
				}
			}
			c := powtesting.New()
			c.HashesByHeight[0] = tt.wantTerminalBlockHash
			vs := &Server{
				Eth1BlockFetcher: c,
				ExecutionEngineCaller: &powtesting.EngineClient{
					ExecutionBlock: tt.currentPowBlock,
					BlockByHashMap: m,
				},
			}
			b, e, err := vs.getTerminalBlockHashIfExists(context.Background(), 1)
			if tt.errString != "" {
				require.ErrorContains(t, tt.errString, err)
				require.DeepEqual(t, tt.wantExists, e)
			} else {
				require.NoError(t, err)
				require.DeepEqual(t, tt.wantExists, e)
				require.DeepEqual(t, tt.wantTerminalBlockHash, b)
			}
		})
	}
}
