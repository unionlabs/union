package baseapp_test

import (
	"bytes"
	"context"
	"crypto/sha256"
	"encoding/binary"
	"encoding/hex"
	"errors"
	"fmt"
	"math/rand"
	"strconv"
	"strings"
	"testing"
	"time"

	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
	cmtprotocrypto "github.com/cometbft/cometbft/api/cometbft/crypto/v1"
	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	"github.com/cometbft/cometbft/crypto/secp256k1"
	cmttypes "github.com/cometbft/cometbft/types"
	protoio "github.com/cosmos/gogoproto/io"
	"github.com/cosmos/gogoproto/jsonpb"
	"github.com/cosmos/gogoproto/proto"
	gogotypes "github.com/cosmos/gogoproto/types"
	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/require"

	coretesting "cosmossdk.io/core/testing"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/log"
	pruningtypes "cosmossdk.io/store/pruning/types"
	"cosmossdk.io/store/snapshots"
	snapshottypes "cosmossdk.io/store/snapshots/types"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/baseapp"
	baseapptestutil "github.com/cosmos/cosmos-sdk/baseapp/testutil"
	"github.com/cosmos/cosmos-sdk/baseapp/testutil/mock"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	"github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/mempool"
	"github.com/cosmos/cosmos-sdk/x/auth/signing"
)

const (
	failStr    = "&failOnAnte=false"
	fooStr     = "foo"
	counterStr = "counter="
)

func TestABCI_Info(t *testing.T) {
	suite := NewBaseAppSuite(t)
	ctx := suite.baseApp.NewContext(true)
	err := suite.baseApp.StoreConsensusParams(ctx, cmttypes.DefaultConsensusParams().ToProto())
	require.NoError(t, err)

	reqInfo := abci.InfoRequest{}
	res, err := suite.baseApp.Info(&reqInfo)
	require.NoError(t, err)

	emptyHash := sha256.Sum256([]byte{})
	appHash := emptyHash[:]
	require.Equal(t, "", res.Version)
	require.Equal(t, t.Name(), res.GetData())
	require.Equal(t, int64(0), res.LastBlockHeight)
	require.Equal(t, appHash, res.LastBlockAppHash)
	appVersion, err := suite.baseApp.AppVersion(ctx)
	require.NoError(t, err)
	require.Equal(t, appVersion, res.AppVersion)

	_, err = suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{Height: 1})
	require.NoError(t, err)
	_, err = suite.baseApp.Commit()
	require.NoError(t, err)
	require.NoError(t, suite.baseApp.SetAppVersion(ctx, 1))
	res, err = suite.baseApp.Info(&reqInfo)
	require.NoError(t, err)
	require.Equal(t, uint64(1), res.AppVersion)
}

func TestABCI_First_block_Height(t *testing.T) {
	suite := NewBaseAppSuite(t, baseapp.SetChainID("test-chain-id"))
	app := suite.baseApp

	_, err := app.InitChain(&abci.InitChainRequest{
		ChainId:         "test-chain-id",
		ConsensusParams: &cmtproto.ConsensusParams{Block: &cmtproto.BlockParams{MaxGas: 5000000}},
		InitialHeight:   1,
	})
	require.NoError(t, err)

	_, err = app.Commit()
	require.NoError(t, err)

	ctx := app.GetContextForCheckTx(nil)
	require.Equal(t, int64(1), ctx.BlockHeight())
}

func TestABCI_InitChain(t *testing.T) {
	name := t.Name()
	db := coretesting.NewMemDB()
	logger := log.NewTestLogger(t)
	app := baseapp.NewBaseApp(name, logger, db, nil, baseapp.SetChainID("test-chain-id"))

	capKey := storetypes.NewKVStoreKey("main")
	capKey2 := storetypes.NewKVStoreKey("key2")
	app.MountStores(capKey, capKey2)

	// set a value in the store on init chain
	key, value := []byte("hello"), []byte("goodbye")
	var initChainer sdk.InitChainer = func(ctx sdk.Context, req *abci.InitChainRequest) (*abci.InitChainResponse, error) {
		store := ctx.KVStore(capKey)
		store.Set(key, value)
		return &abci.InitChainResponse{}, nil
	}

	query := abci.QueryRequest{
		Path: "/store/main/key",
		Data: key,
	}

	// initChain is nil and chain ID is wrong - errors
	_, err := app.InitChain(&abci.InitChainRequest{ChainId: "wrong-chain-id"})
	require.Error(t, err)

	// initChain is nil - nothing happens
	_, err = app.InitChain(&abci.InitChainRequest{ChainId: "test-chain-id"})
	require.NoError(t, err)
	resQ, err := app.Query(context.TODO(), &query)
	require.NoError(t, err)
	require.Equal(t, 0, len(resQ.Value))

	// set initChainer and try again - should see the value
	app.SetInitChainer(initChainer)

	// stores are mounted and private members are set - sealing baseapp
	err = app.LoadLatestVersion() // needed to make stores non-nil
	require.Nil(t, err)
	require.Equal(t, int64(0), app.LastBlockHeight())

	initChainRes, err := app.InitChain(&abci.InitChainRequest{AppStateBytes: []byte("{}"), ChainId: "test-chain-id"}) // must have valid JSON genesis file, even if empty
	require.NoError(t, err)

	// The AppHash returned by a new chain is the sha256 hash of "".
	// $ echo -n '' | sha256sum
	// e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
	apphash, err := hex.DecodeString("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
	require.NoError(t, err)
	emptyHash := sha256.Sum256([]byte{})
	require.Equal(t, emptyHash[:], apphash)

	require.Equal(t, apphash, initChainRes.AppHash)

	// assert that chainID is set correctly in InitChain
	chainID := getFinalizeBlockStateCtx(app).ChainID()
	require.Equal(t, "test-chain-id", chainID, "ChainID in deliverState not set correctly in InitChain")

	chainID = getCheckStateCtx(app).ChainID()
	require.Equal(t, "test-chain-id", chainID, "ChainID in checkState not set correctly in InitChain")

	_, err = app.FinalizeBlock(&abci.FinalizeBlockRequest{
		Hash:   initChainRes.AppHash,
		Height: 1,
	})
	require.NoError(t, err)

	_, err = app.Commit()
	require.NoError(t, err)

	resQ, err = app.Query(context.TODO(), &query)
	require.NoError(t, err)
	require.Equal(t, int64(1), app.LastBlockHeight())
	require.Equal(t, value, resQ.Value)

	// reload app
	app = baseapp.NewBaseApp(name, logger, db, nil)
	app.SetInitChainer(initChainer)
	app.MountStores(capKey, capKey2)
	err = app.LoadLatestVersion() // needed to make stores non-nil
	require.Nil(t, err)
	require.Equal(t, int64(1), app.LastBlockHeight())

	// ensure we can still query after reloading
	resQ, err = app.Query(context.TODO(), &query)
	require.NoError(t, err)
	require.Equal(t, value, resQ.Value)

	// commit and ensure we can still query
	_, err = app.FinalizeBlock(&abci.FinalizeBlockRequest{Height: app.LastBlockHeight() + 1})
	require.NoError(t, err)
	_, err = app.Commit()
	require.NoError(t, err)

	resQ, err = app.Query(context.TODO(), &query)
	require.NoError(t, err)
	require.Equal(t, value, resQ.Value)
}

func TestABCI_InitChain_WithInitialHeight(t *testing.T) {
	name := t.Name()
	db := coretesting.NewMemDB()
	app := baseapp.NewBaseApp(name, log.NewTestLogger(t), db, nil)

	_, err := app.InitChain(
		&abci.InitChainRequest{
			InitialHeight: 3,
		},
	)
	require.NoError(t, err)

	_, err = app.Commit()
	require.NoError(t, err)
	require.Equal(t, int64(3), app.LastBlockHeight())
}

func TestABCI_FinalizeBlock_WithInitialHeight(t *testing.T) {
	name := t.Name()
	db := coretesting.NewMemDB()
	app := baseapp.NewBaseApp(name, log.NewTestLogger(t), db, nil)

	_, err := app.InitChain(
		&abci.InitChainRequest{
			InitialHeight: 3,
		},
	)
	require.NoError(t, err)

	_, err = app.FinalizeBlock(&abci.FinalizeBlockRequest{Height: 4})
	require.Error(t, err, "invalid height: 4; expected: 3")

	_, err = app.FinalizeBlock(&abci.FinalizeBlockRequest{Height: 3})
	require.NoError(t, err)
	_, err = app.Commit()
	require.NoError(t, err)
	require.Equal(t, int64(3), app.LastBlockHeight())
}

func TestABCI_FinalizeBlock_WithBeginAndEndBlocker(t *testing.T) {
	name := t.Name()
	db := coretesting.NewMemDB()
	app := baseapp.NewBaseApp(name, log.NewTestLogger(t), db, nil)

	app.SetBeginBlocker(func(ctx sdk.Context) (sdk.BeginBlock, error) {
		return sdk.BeginBlock{
			Events: []abci.Event{
				{
					Type: "sometype",
					Attributes: []abci.EventAttribute{
						{
							Key:   fooStr,
							Value: "bar",
						},
					},
				},
			},
		}, nil
	})

	app.SetEndBlocker(func(ctx sdk.Context) (sdk.EndBlock, error) {
		return sdk.EndBlock{
			Events: []abci.Event{
				{
					Type: "anothertype",
					Attributes: []abci.EventAttribute{
						{
							Key:   fooStr,
							Value: "bar",
						},
					},
				},
			},
		}, nil
	})

	_, err := app.InitChain(
		&abci.InitChainRequest{
			InitialHeight: 1,
		},
	)
	require.NoError(t, err)

	res, err := app.FinalizeBlock(&abci.FinalizeBlockRequest{Height: 1})
	require.NoError(t, err)

	require.Len(t, res.Events, 2)

	require.Equal(t, "sometype", res.Events[0].Type)
	require.Equal(t, fooStr, res.Events[0].Attributes[0].Key)
	require.Equal(t, "bar", res.Events[0].Attributes[0].Value)
	require.Equal(t, "mode", res.Events[0].Attributes[1].Key)
	require.Equal(t, "BeginBlock", res.Events[0].Attributes[1].Value)

	require.Equal(t, "anothertype", res.Events[1].Type)
	require.Equal(t, fooStr, res.Events[1].Attributes[0].Key)
	require.Equal(t, "bar", res.Events[1].Attributes[0].Value)
	require.Equal(t, "mode", res.Events[1].Attributes[1].Key)
	require.Equal(t, "EndBlock", res.Events[1].Attributes[1].Value)

	_, err = app.Commit()
	require.NoError(t, err)

	require.Equal(t, int64(1), app.LastBlockHeight())
}

func TestABCI_ExtendVote(t *testing.T) {
	name := t.Name()
	db := coretesting.NewMemDB()
	app := baseapp.NewBaseApp(name, log.NewTestLogger(t), db, nil)

	app.SetExtendVoteHandler(func(ctx sdk.Context, req *abci.ExtendVoteRequest) (*abci.ExtendVoteResponse, error) {
		voteExt := fooStr + hex.EncodeToString(req.Hash) + strconv.FormatInt(req.Height, 10)
		return &abci.ExtendVoteResponse{VoteExtension: []byte(voteExt)}, nil
	})

	app.SetVerifyVoteExtensionHandler(func(ctx sdk.Context, req *abci.VerifyVoteExtensionRequest) (*abci.VerifyVoteExtensionResponse, error) {
		// do some kind of verification here
		expectedVoteExt := fooStr + hex.EncodeToString(req.Hash) + strconv.FormatInt(req.Height, 10)
		if !bytes.Equal(req.VoteExtension, []byte(expectedVoteExt)) {
			return &abci.VerifyVoteExtensionResponse{Status: abci.VERIFY_VOTE_EXTENSION_STATUS_REJECT}, nil
		}

		return &abci.VerifyVoteExtensionResponse{Status: abci.VERIFY_VOTE_EXTENSION_STATUS_ACCEPT}, nil
	})

	app.SetParamStore(&paramStore{db: coretesting.NewMemDB()})
	_, err := app.InitChain(
		&abci.InitChainRequest{
			InitialHeight: 1,
			ConsensusParams: &cmtproto.ConsensusParams{
				Feature: &cmtproto.FeatureParams{
					VoteExtensionsEnableHeight: &gogotypes.Int64Value{Value: 200},
				},
			},
		},
	)
	require.NoError(t, err)

	// Votes not enabled yet
	_, err = app.ExtendVote(context.Background(), &abci.ExtendVoteRequest{Height: 123, Hash: []byte("thehash")})
	require.ErrorContains(t, err, "vote extensions are not enabled")

	// First vote on the first enabled height
	res, err := app.ExtendVote(context.Background(), &abci.ExtendVoteRequest{Height: 200, Hash: []byte("thehash")})
	require.NoError(t, err)
	require.Len(t, res.VoteExtension, 20)

	res, err = app.ExtendVote(context.Background(), &abci.ExtendVoteRequest{Height: 1000, Hash: []byte("thehash")})
	require.NoError(t, err)
	require.Len(t, res.VoteExtension, 21)

	// Error during vote extension should return an empty vote extension and no error
	app.SetExtendVoteHandler(func(ctx sdk.Context, req *abci.ExtendVoteRequest) (*abci.ExtendVoteResponse, error) {
		return nil, errors.New("some error")
	})
	res, err = app.ExtendVote(context.Background(), &abci.ExtendVoteRequest{Height: 1000, Hash: []byte("thehash")})
	require.NoError(t, err)
	require.Len(t, res.VoteExtension, 0)

	// Verify Vote Extensions
	_, err = app.VerifyVoteExtension(&abci.VerifyVoteExtensionRequest{Height: 123, VoteExtension: []byte("1234567")})
	require.ErrorContains(t, err, "vote extensions are not enabled")

	// First vote on the first enabled height
	vres, err := app.VerifyVoteExtension(&abci.VerifyVoteExtensionRequest{Height: 200, Hash: []byte("thehash"), VoteExtension: []byte("foo74686568617368200")})
	require.NoError(t, err)
	require.Equal(t, abci.VERIFY_VOTE_EXTENSION_STATUS_ACCEPT, vres.Status)

	vres, err = app.VerifyVoteExtension(&abci.VerifyVoteExtensionRequest{Height: 1000, Hash: []byte("thehash"), VoteExtension: []byte("foo746865686173681000")})
	require.NoError(t, err)
	require.Equal(t, abci.VERIFY_VOTE_EXTENSION_STATUS_ACCEPT, vres.Status)

	// Reject because it's just some random bytes
	vres, err = app.VerifyVoteExtension(&abci.VerifyVoteExtensionRequest{Height: 201, Hash: []byte("thehash"), VoteExtension: []byte("12345678")})
	require.NoError(t, err)
	require.Equal(t, abci.VERIFY_VOTE_EXTENSION_STATUS_REJECT, vres.Status)

	// Reject because the verification failed (no error)
	app.SetVerifyVoteExtensionHandler(func(ctx sdk.Context, req *abci.VerifyVoteExtensionRequest) (*abci.VerifyVoteExtensionResponse, error) {
		return nil, errors.New("some error")
	})
	vres, err = app.VerifyVoteExtension(&abci.VerifyVoteExtensionRequest{Height: 201, Hash: []byte("thehash"), VoteExtension: []byte("12345678")})
	require.NoError(t, err)
	require.Equal(t, abci.VERIFY_VOTE_EXTENSION_STATUS_REJECT, vres.Status)
}

// TestABCI_OnlyVerifyVoteExtension makes sure we can call VerifyVoteExtension
// without having called ExtendVote before.
func TestABCI_OnlyVerifyVoteExtension(t *testing.T) {
	name := t.Name()
	db := coretesting.NewMemDB()
	app := baseapp.NewBaseApp(name, log.NewTestLogger(t), db, nil)

	app.SetVerifyVoteExtensionHandler(func(ctx sdk.Context, req *abci.VerifyVoteExtensionRequest) (*abci.VerifyVoteExtensionResponse, error) {
		// do some kind of verification here
		expectedVoteExt := fooStr + hex.EncodeToString(req.Hash) + strconv.FormatInt(req.Height, 10)
		if !bytes.Equal(req.VoteExtension, []byte(expectedVoteExt)) {
			return &abci.VerifyVoteExtensionResponse{Status: abci.VERIFY_VOTE_EXTENSION_STATUS_REJECT}, nil
		}

		return &abci.VerifyVoteExtensionResponse{Status: abci.VERIFY_VOTE_EXTENSION_STATUS_ACCEPT}, nil
	})

	app.SetParamStore(&paramStore{db: coretesting.NewMemDB()})
	_, err := app.InitChain(
		&abci.InitChainRequest{
			InitialHeight: 1,
			ConsensusParams: &cmtproto.ConsensusParams{
				Feature: &cmtproto.FeatureParams{
					VoteExtensionsEnableHeight: &gogotypes.Int64Value{Value: 200},
				},
			},
		},
	)
	require.NoError(t, err)

	// Verify Vote Extensions
	_, err = app.VerifyVoteExtension(&abci.VerifyVoteExtensionRequest{Height: 123, VoteExtension: []byte("1234567")})
	require.ErrorContains(t, err, "vote extensions are not enabled")

	// First vote on the first enabled height
	vres, err := app.VerifyVoteExtension(&abci.VerifyVoteExtensionRequest{Height: 200, Hash: []byte("thehash"), VoteExtension: []byte("foo74686568617368200")})
	require.NoError(t, err)
	require.Equal(t, abci.VERIFY_VOTE_EXTENSION_STATUS_ACCEPT, vres.Status)

	vres, err = app.VerifyVoteExtension(&abci.VerifyVoteExtensionRequest{Height: 1000, Hash: []byte("thehash"), VoteExtension: []byte("foo746865686173681000")})
	require.NoError(t, err)
	require.Equal(t, abci.VERIFY_VOTE_EXTENSION_STATUS_ACCEPT, vres.Status)

	// Reject because it's just some random bytes
	vres, err = app.VerifyVoteExtension(&abci.VerifyVoteExtensionRequest{Height: 201, Hash: []byte("thehash"), VoteExtension: []byte("12345678")})
	require.NoError(t, err)
	require.Equal(t, abci.VERIFY_VOTE_EXTENSION_STATUS_REJECT, vres.Status)

	// Reject because the verification failed (no error)
	app.SetVerifyVoteExtensionHandler(func(ctx sdk.Context, req *abci.VerifyVoteExtensionRequest) (*abci.VerifyVoteExtensionResponse, error) {
		return nil, errors.New("some error")
	})
	vres, err = app.VerifyVoteExtension(&abci.VerifyVoteExtensionRequest{Height: 201, Hash: []byte("thehash"), VoteExtension: []byte("12345678")})
	require.NoError(t, err)
	require.Equal(t, abci.VERIFY_VOTE_EXTENSION_STATUS_REJECT, vres.Status)
}

func TestABCI_GRPCQuery(t *testing.T) {
	grpcQueryOpt := func(bapp *baseapp.BaseApp) {
		testdata.RegisterQueryServer(
			bapp.GRPCQueryRouter(),
			testdata.QueryImpl{},
		)
	}

	suite := NewBaseAppSuite(t, grpcQueryOpt)

	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		ConsensusParams: &cmtproto.ConsensusParams{},
	})
	require.NoError(t, err)

	req := testdata.SayHelloRequest{Name: fooStr}
	reqBz, err := req.Marshal()
	require.NoError(t, err)

	resQuery, err := suite.baseApp.Query(context.TODO(), &abci.QueryRequest{
		Data: reqBz,
		Path: "/testpb.Query/SayHello",
	})
	require.NoError(t, err)
	require.Equal(t, sdkerrors.ErrInvalidHeight.ABCICode(), resQuery.Code, resQuery)
	require.Contains(t, resQuery.Log, "TestABCI_GRPCQuery is not ready; please wait for first block")

	_, err = suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{Height: suite.baseApp.LastBlockHeight() + 1})
	require.NoError(t, err)

	_, err = suite.baseApp.Commit()
	require.NoError(t, err)

	reqQuery := abci.QueryRequest{
		Data: reqBz,
		Path: "/testpb.Query/SayHello",
	}

	resQuery, err = suite.baseApp.Query(context.TODO(), &reqQuery)
	require.NoError(t, err)
	require.Equal(t, abci.CodeTypeOK, resQuery.Code, resQuery)

	var res testdata.SayHelloResponse
	require.NoError(t, res.Unmarshal(resQuery.Value))
	require.Equal(t, "Hello foo!", res.Greeting)
}

func TestABCI_P2PQuery(t *testing.T) {
	addrPeerFilterOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAddrPeerFilter(func(addrport string) *abci.QueryResponse {
			require.Equal(t, "1.1.1.1:8000", addrport)
			return &abci.QueryResponse{Code: uint32(3)}
		})
	}

	idPeerFilterOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetIDPeerFilter(func(id string) *abci.QueryResponse {
			require.Equal(t, "testid", id)
			return &abci.QueryResponse{Code: uint32(4)}
		})
	}

	suite := NewBaseAppSuite(t, addrPeerFilterOpt, idPeerFilterOpt)

	addrQuery := abci.QueryRequest{
		Path: "/p2p/filter/addr/1.1.1.1:8000",
	}
	res, err := suite.baseApp.Query(context.TODO(), &addrQuery)
	require.NoError(t, err)
	require.Equal(t, uint32(3), res.Code)

	idQuery := abci.QueryRequest{
		Path: "/p2p/filter/id/testid",
	}
	res, err = suite.baseApp.Query(context.TODO(), &idQuery)
	require.NoError(t, err)
	require.Equal(t, uint32(4), res.Code)
}

func TestBaseApp_PrepareCheckState(t *testing.T) {
	db := coretesting.NewMemDB()
	name := t.Name()
	logger := log.NewTestLogger(t)

	cp := &cmtproto.ConsensusParams{
		Block: &cmtproto.BlockParams{
			MaxGas: 5000000,
		},
	}

	app := baseapp.NewBaseApp(name, logger, db, nil)
	app.SetParamStore(&paramStore{db: coretesting.NewMemDB()})
	_, err := app.InitChain(&abci.InitChainRequest{
		ConsensusParams: cp,
	})
	require.NoError(t, err)

	wasPrepareCheckStateCalled := false
	app.SetPrepareCheckStater(func(ctx sdk.Context) {
		wasPrepareCheckStateCalled = true
	})
	app.Seal()

	_, err = app.Commit()
	require.NoError(t, err)
	require.Equal(t, true, wasPrepareCheckStateCalled)
}

func TestBaseApp_Precommit(t *testing.T) {
	db := coretesting.NewMemDB()
	name := t.Name()
	logger := log.NewTestLogger(t)

	cp := &cmtproto.ConsensusParams{
		Block: &cmtproto.BlockParams{
			MaxGas: 5000000,
		},
	}

	app := baseapp.NewBaseApp(name, logger, db, nil)
	app.SetParamStore(&paramStore{db: coretesting.NewMemDB()})
	_, err := app.InitChain(&abci.InitChainRequest{
		ConsensusParams: cp,
	})
	require.NoError(t, err)

	wasPrecommiterCalled := false
	app.SetPrecommiter(func(ctx sdk.Context) {
		wasPrecommiterCalled = true
	})
	app.Seal()

	_, err = app.Commit()
	require.NoError(t, err)
	require.Equal(t, true, wasPrecommiterCalled)
}

func TestABCI_CheckTx(t *testing.T) {
	// This ante handler reads the key and checks that the value matches the
	// current counter. This ensures changes to the KVStore persist across
	// successive CheckTx runs.
	counterKey := []byte("counter-key")
	anteOpt := func(bapp *baseapp.BaseApp) { bapp.SetAnteHandler(anteHandlerTxTest(t, capKey1, counterKey)) }
	suite := NewBaseAppSuite(t, anteOpt)

	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), CounterServerImpl{t, capKey1, counterKey})

	nTxs := int64(5)
	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		ConsensusParams: &cmtproto.ConsensusParams{},
	})
	require.NoError(t, err)

	for i := int64(0); i < nTxs; i++ {
		tx := newTxCounter(t, suite.txConfig, suite.ac, i, 0) // no messages
		txBytes, err := suite.txConfig.TxEncoder()(tx)
		require.NoError(t, err)

		r, err := suite.baseApp.CheckTx(&abci.CheckTxRequest{Tx: txBytes, Type: abci.CHECK_TX_TYPE_CHECK})
		require.NoError(t, err)
		require.True(t, r.IsOK(), fmt.Sprintf("%v", r))
		require.Empty(t, r.GetEvents())
	}

	checkStateStore := getCheckStateCtx(suite.baseApp).KVStore(capKey1)
	storedCounter := getIntFromStore(t, checkStateStore, counterKey)

	// ensure AnteHandler ran
	require.Equal(t, nTxs, storedCounter)

	// if a block is committed, CheckTx state should be reset
	_, err = suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{
		Height: 1,
		Hash:   []byte("hash"),
	})
	require.NoError(t, err)

	require.NotNil(t, getCheckStateCtx(suite.baseApp).BlockGasMeter(), "block gas meter should have been set to checkState")
	require.NotEmpty(t, getCheckStateCtx(suite.baseApp).HeaderHash())

	_, err = suite.baseApp.Commit()
	require.NoError(t, err)

	checkStateStore = getCheckStateCtx(suite.baseApp).KVStore(capKey1)
	storedBytes := checkStateStore.Get(counterKey)
	require.Nil(t, storedBytes)
}

func TestABCI_FinalizeBlock_DeliverTx(t *testing.T) {
	anteKey := []byte("ante-key")
	anteOpt := func(bapp *baseapp.BaseApp) { bapp.SetAnteHandler(anteHandlerTxTest(t, capKey1, anteKey)) }
	suite := NewBaseAppSuite(t, anteOpt)

	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		ConsensusParams: &cmtproto.ConsensusParams{},
	})
	require.NoError(t, err)

	deliverKey := []byte("deliver-key")
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), CounterServerImpl{t, capKey1, deliverKey})

	nBlocks := 3
	txPerHeight := 5

	for blockN := 0; blockN < nBlocks; blockN++ {

		txs := [][]byte{}
		for i := 0; i < txPerHeight; i++ {
			counter := int64(blockN*txPerHeight + i)
			tx := newTxCounter(t, suite.txConfig, suite.ac, counter, counter)

			txBytes, err := suite.txConfig.TxEncoder()(tx)
			require.NoError(t, err)

			txs = append(txs, txBytes)
		}

		res, err := suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{
			Height: int64(blockN) + 1,
			Txs:    txs,
		})
		require.NoError(t, err)

		for i := 0; i < txPerHeight; i++ {
			counter := int64(blockN*txPerHeight + i)
			require.True(t, res.TxResults[i].IsOK(), fmt.Sprintf("%v", res))

			events := res.TxResults[i].GetEvents()
			require.Len(t, events, 3, "should contain ante handler, message type and counter events respectively")
			require.Equal(t, sdk.MarkEventsToIndex(counterEvent("ante_handler", counter).ToABCIEvents(), map[string]struct{}{})[0].Attributes[0], events[0].Attributes[0], "ante handler event")
			require.Equal(t, sdk.MarkEventsToIndex(counterEvent(sdk.EventTypeMessage, counter).ToABCIEvents(), map[string]struct{}{})[0].Attributes[0], events[2].Attributes[0], "msg handler update counter event")
		}

		_, err = suite.baseApp.Commit()
		require.NoError(t, err)

	}
}

func TestABCI_FinalizeBlock_MultiMsg(t *testing.T) {
	anteKey := []byte("ante-key")
	anteOpt := func(bapp *baseapp.BaseApp) { bapp.SetAnteHandler(anteHandlerTxTest(t, capKey1, anteKey)) }
	suite := NewBaseAppSuite(t, anteOpt)

	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		ConsensusParams: &cmtproto.ConsensusParams{},
	})
	require.NoError(t, err)

	deliverKey := []byte("deliver-key")
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), CounterServerImpl{t, capKey1, deliverKey})

	deliverKey2 := []byte("deliver-key2")
	baseapptestutil.RegisterCounter2Server(suite.baseApp.MsgServiceRouter(), Counter2ServerImpl{t, capKey1, deliverKey2})

	// run a multi-msg tx
	// with all msgs the same route
	tx := newTxCounter(t, suite.txConfig, suite.ac, 0, 0, 1, 2)
	txBytes, err := suite.txConfig.TxEncoder()(tx)
	require.NoError(t, err)

	_, err = suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{
		Height: 1,
		Txs:    [][]byte{txBytes},
	})
	require.NoError(t, err)

	store := getFinalizeBlockStateCtx(suite.baseApp).KVStore(capKey1)

	// tx counter only incremented once
	txCounter := getIntFromStore(t, store, anteKey)
	require.Equal(t, int64(1), txCounter)

	// msg counter incremented three times
	msgCounter := getIntFromStore(t, store, deliverKey)
	require.Equal(t, int64(3), msgCounter)

	// replace the second message with a Counter2
	tx = newTxCounter(t, suite.txConfig, suite.ac, 1, 3)

	builder := suite.txConfig.NewTxBuilder()
	msgs := tx.GetMsgs()
	_, _, addr := testdata.KeyTestPubAddr()
	addrStr, err := suite.ac.BytesToString(addr)
	require.NoError(t, err)
	msgs = append(msgs, &baseapptestutil.MsgCounter2{Counter: 0, Signer: addrStr})
	msgs = append(msgs, &baseapptestutil.MsgCounter2{Counter: 1, Signer: addrStr})

	err = builder.SetMsgs(msgs...)
	require.NoError(t, err)

	builder.SetMemo(tx.GetMemo())
	setTxSignature(t, builder, 0)

	txBytes, err = suite.txConfig.TxEncoder()(builder.GetTx())
	require.NoError(t, err)

	_, err = suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{
		Height: 1,
		Txs:    [][]byte{txBytes},
	})
	require.NoError(t, err)

	store = getFinalizeBlockStateCtx(suite.baseApp).KVStore(capKey1)

	// tx counter only incremented once
	txCounter = getIntFromStore(t, store, anteKey)
	require.Equal(t, int64(2), txCounter)

	// original counter increments by one
	// new counter increments by two
	msgCounter = getIntFromStore(t, store, deliverKey)
	require.Equal(t, int64(4), msgCounter)

	msgCounter2 := getIntFromStore(t, store, deliverKey2)
	require.Equal(t, int64(2), msgCounter2)
}

func TestABCI_Query_SimulateTx(t *testing.T) {
	gasConsumed := uint64(5)
	anteOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAnteHandler(func(ctx sdk.Context, tx sdk.Tx, simulate bool) (newCtx sdk.Context, err error) {
			newCtx = ctx.WithGasMeter(storetypes.NewGasMeter(gasConsumed))
			return
		})
	}
	suite := NewBaseAppSuite(t, anteOpt)

	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		ConsensusParams: &cmtproto.ConsensusParams{},
	})
	require.NoError(t, err)

	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), CounterServerImplGasMeterOnly{gasConsumed})

	nBlocks := 3
	for blockN := 0; blockN < nBlocks; blockN++ {
		count := int64(blockN + 1)

		tx := newTxCounter(t, suite.txConfig, suite.ac, count, count)

		txBytes, err := suite.txConfig.TxEncoder()(tx)
		require.Nil(t, err)

		// simulate a message, check gas reported
		gInfo, result, err := suite.baseApp.Simulate(txBytes)
		require.NoError(t, err)
		require.NotNil(t, result)
		require.Equal(t, gasConsumed, gInfo.GasUsed)

		// simulate again, same result
		gInfo, result, err = suite.baseApp.Simulate(txBytes)
		require.NoError(t, err)
		require.NotNil(t, result)
		require.Equal(t, gasConsumed, gInfo.GasUsed)

		// simulate by calling Query with encoded tx
		query := abci.QueryRequest{
			Path: "/app/simulate",
			Data: txBytes,
		}
		queryResult, err := suite.baseApp.Query(context.TODO(), &query)
		require.NoError(t, err)
		require.True(t, queryResult.IsOK(), queryResult.Log)

		var simRes sdk.SimulationResponse
		require.NoError(t, jsonpb.Unmarshal(strings.NewReader(string(queryResult.Value)), &simRes))

		require.Equal(t, gInfo, simRes.GasInfo)
		require.Equal(t, result.Log, simRes.Result.Log)
		require.Equal(t, result.Events, simRes.Result.Events)
		require.True(t, bytes.Equal(result.Data, simRes.Result.Data))

		_, err = suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{Height: count})
		require.NoError(t, err)
		_, err = suite.baseApp.Commit()
		require.NoError(t, err)
	}
}

func TestABCI_InvalidTransaction(t *testing.T) {
	anteOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAnteHandler(func(ctx sdk.Context, tx sdk.Tx, simulate bool) (newCtx sdk.Context, err error) {
			return
		})
	}

	suite := NewBaseAppSuite(t, anteOpt)
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), CounterServerImplGasMeterOnly{})

	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		ConsensusParams: &cmtproto.ConsensusParams{},
	})
	require.NoError(t, err)
	_, err = suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{
		Height: 1,
	})
	require.NoError(t, err)
	// malformed transaction bytes
	{
		bz := []byte("example vote extension")
		result, err := suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{
			Height: 1,
			Txs:    [][]byte{bz},
		})

		require.EqualValues(t, sdkerrors.ErrTxDecode.Codespace(), result.TxResults[0].Codespace, err)
		require.EqualValues(t, sdkerrors.ErrTxDecode.ABCICode(), result.TxResults[0].Code, err)
		require.EqualValues(t, 0, result.TxResults[0].GasUsed, err)
		require.EqualValues(t, 0, result.TxResults[0].GasWanted, err)
	}
	// transaction with no messages
	{
		emptyTx := suite.txConfig.NewTxBuilder().GetTx()
		bz, err := suite.txConfig.TxEncoder()(emptyTx)
		require.NoError(t, err)
		result, err := suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{
			Height: 1,
			Txs:    [][]byte{bz},
		})
		require.EqualValues(t, sdkerrors.ErrInvalidRequest.Codespace(), result.TxResults[0].Codespace, err)
		require.EqualValues(t, sdkerrors.ErrInvalidRequest.ABCICode(), result.TxResults[0].Code, err)
	}

	// transaction where ValidateBasic fails
	{
		testCases := []struct {
			tx   signing.Tx
			fail bool
		}{
			{newTxCounter(t, suite.txConfig, suite.ac, 0, 0), false},
			{newTxCounter(t, suite.txConfig, suite.ac, -1, 0), false},
			{newTxCounter(t, suite.txConfig, suite.ac, 100, 100), false},
			{newTxCounter(t, suite.txConfig, suite.ac, 100, 5, 4, 3, 2, 1), false},

			{newTxCounter(t, suite.txConfig, suite.ac, 0, -1), true},
			{newTxCounter(t, suite.txConfig, suite.ac, 0, 1, -2), true},
			{newTxCounter(t, suite.txConfig, suite.ac, 0, 1, 2, -10, 5), true},
		}

		for _, testCase := range testCases {
			tx := testCase.tx
			_, result, err := suite.baseApp.SimDeliver(suite.txConfig.TxEncoder(), tx)

			if testCase.fail {
				require.Error(t, err)

				space, code, _ := errorsmod.ABCIInfo(err, false)
				require.EqualValues(t, sdkerrors.ErrInvalidSequence.Codespace(), space, err)
				require.EqualValues(t, sdkerrors.ErrInvalidSequence.ABCICode(), code, err)
			} else {
				require.NotNil(t, result)
			}
		}
	}

	// transaction with no known route
	{
		txBuilder := suite.txConfig.NewTxBuilder()
		_, _, addr := testdata.KeyTestPubAddr()
		addrStr, err := suite.ac.BytesToString(addr)
		require.NoError(t, err)
		err = txBuilder.SetMsgs(&baseapptestutil.MsgCounter2{Signer: addrStr})
		require.NoError(t, err)
		setTxSignature(t, txBuilder, 0)
		unknownRouteTx := txBuilder.GetTx()

		_, result, err := suite.baseApp.SimDeliver(suite.txConfig.TxEncoder(), unknownRouteTx)
		require.Error(t, err)
		require.Nil(t, result)

		space, code, _ := errorsmod.ABCIInfo(err, false)
		require.EqualValues(t, sdkerrors.ErrUnknownRequest.Codespace(), space, err)
		require.EqualValues(t, sdkerrors.ErrUnknownRequest.ABCICode(), code, err)

		txBuilder = suite.txConfig.NewTxBuilder()
		err = txBuilder.SetMsgs(
			&baseapptestutil.MsgCounter{Signer: addrStr},
			&baseapptestutil.MsgCounter2{Signer: addrStr},
		)
		require.NoError(t, err)
		setTxSignature(t, txBuilder, 0)
		unknownRouteTx = txBuilder.GetTx()

		_, result, err = suite.baseApp.SimDeliver(suite.txConfig.TxEncoder(), unknownRouteTx)
		require.Error(t, err)
		require.Nil(t, result)

		space, code, _ = errorsmod.ABCIInfo(err, false)
		require.EqualValues(t, sdkerrors.ErrUnknownRequest.Codespace(), space, err)
		require.EqualValues(t, sdkerrors.ErrUnknownRequest.ABCICode(), code, err)
	}
}

func TestABCI_TxGasLimits(t *testing.T) {
	gasGranted := uint64(10)
	anteOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAnteHandler(func(ctx sdk.Context, tx sdk.Tx, simulate bool) (newCtx sdk.Context, err error) {
			newCtx = ctx.WithGasMeter(storetypes.NewGasMeter(gasGranted))

			// AnteHandlers must have their own defer/recover in order for the BaseApp
			// to know how much gas was used! This is because the GasMeter is created in
			// the AnteHandler, but if it panics the context won't be set properly in
			// runTx's recover call.
			defer func() {
				if r := recover(); r != nil {
					switch rType := r.(type) {
					case storetypes.ErrorOutOfGas:
						err = errorsmod.Wrapf(sdkerrors.ErrOutOfGas, "out of gas in location: %v", rType.Descriptor)
					default:
						panic(r)
					}
				}
			}()

			count, _ := parseTxMemo(t, tx)
			newCtx.GasMeter().ConsumeGas(uint64(count), "counter-ante")

			return newCtx, nil
		})
	}

	suite := NewBaseAppSuite(t, anteOpt)
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), CounterServerImplGasMeterOnly{})

	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		ConsensusParams: &cmtproto.ConsensusParams{},
	})
	require.NoError(t, err)

	_, err = suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{
		Height: 1,
	})
	require.NoError(t, err)

	_, err = suite.baseApp.Commit()
	require.NoError(t, err)

	testCases := []struct {
		tx      signing.Tx
		gasUsed int64
		fail    bool
	}{
		{newTxCounter(t, suite.txConfig, suite.ac, 0, 0), 0, false},
		{newTxCounter(t, suite.txConfig, suite.ac, 1, 1), 2, false},
		{newTxCounter(t, suite.txConfig, suite.ac, 9, 1), 10, false},
		{newTxCounter(t, suite.txConfig, suite.ac, 1, 9), 10, false},
		{newTxCounter(t, suite.txConfig, suite.ac, 10, 0), 10, false},

		{newTxCounter(t, suite.txConfig, suite.ac, 9, 2), 11, true},
		{newTxCounter(t, suite.txConfig, suite.ac, 2, 9), 11, true},
		// {newTxCounter(t, suite.txConfig, suite.ac, 9, 1, 1), 11, true},
		// {newTxCounter(t, suite.txConfig, suite.ac, 1, 8, 1, 1), 11, true},
		//  {newTxCounter(t, suite.txConfig, suite.ac, 11, 0), 11, true},
		//  {newTxCounter(t, suite.txConfig, suite.ac, 0, 11), 11, true},
		//  {newTxCounter(t, suite.txConfig, suite.ac, 0, 5, 11), 16, true},
	}

	txs := [][]byte{}
	for _, tc := range testCases {
		tx := tc.tx
		bz, err := suite.txConfig.TxEncoder()(tx)
		require.NoError(t, err)
		txs = append(txs, bz)
	}

	// Deliver the txs
	res, err := suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{
		Height: 2,
		Txs:    txs,
	})

	require.NoError(t, err)

	for i, tc := range testCases {

		result := res.TxResults[i]

		require.Equal(t, tc.gasUsed, result.GasUsed, fmt.Sprintf("tc #%d; gas: %v, result: %v, err: %s", i, result.GasUsed, result, err))

		// check for out of gas
		if !tc.fail {
			require.NotNil(t, result, fmt.Sprintf("%d: %v, %v", i, tc, err))
		} else {
			require.EqualValues(t, sdkerrors.ErrOutOfGas.Codespace(), result.Codespace, err)
			require.EqualValues(t, sdkerrors.ErrOutOfGas.ABCICode(), result.Code, err)
		}
	}
}

func TestABCI_MaxBlockGasLimits(t *testing.T) {
	gasGranted := uint64(10)
	anteOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAnteHandler(func(ctx sdk.Context, tx sdk.Tx, simulate bool) (newCtx sdk.Context, err error) {
			newCtx = ctx.WithGasMeter(storetypes.NewGasMeter(gasGranted))

			defer func() {
				if r := recover(); r != nil {
					switch rType := r.(type) {
					case storetypes.ErrorOutOfGas:
						err = errorsmod.Wrapf(sdkerrors.ErrOutOfGas, "out of gas in location: %v", rType.Descriptor)
					default:
						panic(r)
					}
				}
			}()

			count, _ := parseTxMemo(t, tx)
			newCtx.GasMeter().ConsumeGas(uint64(count), "counter-ante")

			return
		})
	}

	suite := NewBaseAppSuite(t, anteOpt)
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), CounterServerImplGasMeterOnly{})

	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		ConsensusParams: &cmtproto.ConsensusParams{
			Block: &cmtproto.BlockParams{
				MaxGas: 100,
			},
		},
	})
	require.NoError(t, err)

	_, err = suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{Height: 1})
	require.NoError(t, err)

	testCases := []struct {
		tx                signing.Tx
		numDelivers       int
		gasUsedPerDeliver uint64
		fail              bool
		failAfterDeliver  int
	}{
		{newTxCounter(t, suite.txConfig, suite.ac, 0, 0), 0, 0, false, 0},
		{newTxCounter(t, suite.txConfig, suite.ac, 9, 1), 2, 10, false, 0},
		{newTxCounter(t, suite.txConfig, suite.ac, 10, 0), 3, 10, false, 0},
		{newTxCounter(t, suite.txConfig, suite.ac, 10, 0), 10, 10, false, 0},
		{newTxCounter(t, suite.txConfig, suite.ac, 2, 7), 11, 9, false, 0},
		// {newTxCounter(t, suite.txConfig, suite.ac, 10, 0), 10, 10, false, 0}, // hit the limit but pass

		// {newTxCounter(t, suite.txConfig, suite.ac, 10, 0), 11, 10, true, 10},
		// {newTxCounter(t, suite.txConfig, suite.ac, 10, 0), 15, 10, true, 10},
		// {newTxCounter(t, suite.txConfig, suite.ac, 9, 0), 12, 9, true, 11}, // fly past the limit
	}

	for i, tc := range testCases {
		tx := tc.tx

		// reset block gas
		_, err := suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{Height: suite.baseApp.LastBlockHeight() + 1})
		require.NoError(t, err)

		// execute the transaction multiple times
		for j := 0; j < tc.numDelivers; j++ {

			_, result, err := suite.baseApp.SimDeliver(suite.txConfig.TxEncoder(), tx)

			ctx := getFinalizeBlockStateCtx(suite.baseApp)

			// check for failed transactions
			if tc.fail && (j+1) > tc.failAfterDeliver {
				require.Error(t, err, fmt.Sprintf("tc #%d; result: %v, err: %s", i, result, err))
				require.Nil(t, tx, fmt.Sprintf("tc #%d; result: %v, err: %s", i, result, err))

				space, code, _ := errorsmod.ABCIInfo(err, false)
				require.EqualValues(t, sdkerrors.ErrOutOfGas.Codespace(), space, err)
				require.EqualValues(t, sdkerrors.ErrOutOfGas.ABCICode(), code, err)
				require.True(t, ctx.BlockGasMeter().IsOutOfGas())
			} else {
				// check gas used and wanted
				blockGasUsed := ctx.BlockGasMeter().GasConsumed()
				expBlockGasUsed := tc.gasUsedPerDeliver * uint64(j+1)
				require.Equal(
					t, expBlockGasUsed, blockGasUsed,
					fmt.Sprintf("%d,%d: %v, %v, %v, %v", i, j, tc, expBlockGasUsed, blockGasUsed, result),
				)

				require.NotNil(t, tx, fmt.Sprintf("tc #%d; currDeliver: %d, result: %v, err: %s", i, j, result, err))
				require.False(t, ctx.BlockGasMeter().IsPastLimit())
			}
		}
	}
}

func TestABCI_GasConsumptionBadTx(t *testing.T) {
	gasWanted := uint64(5)
	anteOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAnteHandler(func(ctx sdk.Context, tx sdk.Tx, simulate bool) (newCtx sdk.Context, err error) {
			newCtx = ctx.WithGasMeter(storetypes.NewGasMeter(gasWanted))

			defer func() {
				if r := recover(); r != nil {
					switch rType := r.(type) {
					case storetypes.ErrorOutOfGas:
						log := fmt.Sprintf("out of gas in location: %v", rType.Descriptor)
						err = errorsmod.Wrap(sdkerrors.ErrOutOfGas, log)
					default:
						panic(r)
					}
				}
			}()

			counter, failOnAnte := parseTxMemo(t, tx)
			newCtx.GasMeter().ConsumeGas(uint64(counter), "counter-ante")
			if failOnAnte {
				return newCtx, errorsmod.Wrap(sdkerrors.ErrUnauthorized, "ante handler failure")
			}

			return
		})
	}

	suite := NewBaseAppSuite(t, anteOpt)
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), CounterServerImplGasMeterOnly{})

	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		ConsensusParams: &cmtproto.ConsensusParams{
			Block: &cmtproto.BlockParams{
				MaxGas: 9,
			},
		},
	})
	require.NoError(t, err)

	tx := newTxCounter(t, suite.txConfig, suite.ac, 5, 0)
	tx = setFailOnAnte(t, suite.txConfig, tx, true)
	txBytes, err := suite.txConfig.TxEncoder()(tx)
	require.NoError(t, err)

	// require next tx to fail due to black gas limit
	tx = newTxCounter(t, suite.txConfig, suite.ac, 5, 0)
	txBytes2, err := suite.txConfig.TxEncoder()(tx)
	require.NoError(t, err)

	_, err = suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{
		Height: suite.baseApp.LastBlockHeight() + 1,
		Txs:    [][]byte{txBytes, txBytes2},
	})
	require.NoError(t, err)
}

func TestABCI_Query(t *testing.T) {
	key, value := []byte("hello"), []byte("goodbye")
	anteOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAnteHandler(func(ctx sdk.Context, tx sdk.Tx, simulate bool) (newCtx sdk.Context, err error) {
			store := ctx.KVStore(capKey1)
			store.Set(key, value)
			return
		})
	}

	suite := NewBaseAppSuite(t, anteOpt)
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), CounterServerImplGasMeterOnly{})

	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		ConsensusParams: &cmtproto.ConsensusParams{},
	})
	require.NoError(t, err)

	// NOTE: "/store/key1" tells us KVStore
	// and the final "/key" says to use the data as the
	// key in the given KVStore ...
	query := abci.QueryRequest{
		Path: "/store/key1/key",
		Data: key,
	}
	tx := newTxCounter(t, suite.txConfig, suite.ac, 0, 0)

	// query is empty before we do anything
	res, err := suite.baseApp.Query(context.TODO(), &query)
	require.NoError(t, err)
	require.Equal(t, 0, len(res.Value))

	// query is still empty after a CheckTx
	_, resTx, err := suite.baseApp.SimCheck(suite.txConfig.TxEncoder(), tx)
	require.NoError(t, err)
	require.NotNil(t, resTx)

	res, err = suite.baseApp.Query(context.TODO(), &query)
	require.NoError(t, err)
	require.Equal(t, 0, len(res.Value))

	bz, err := suite.txConfig.TxEncoder()(tx)
	require.NoError(t, err)

	_, err = suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{
		Height: 1,
		Txs:    [][]byte{bz},
	})
	require.NoError(t, err)

	res, err = suite.baseApp.Query(context.TODO(), &query)
	require.NoError(t, err)
	require.Equal(t, 0, len(res.Value))

	// query returns correct value after Commit
	_, err = suite.baseApp.Commit()
	require.NoError(t, err)

	res, err = suite.baseApp.Query(context.TODO(), &query)
	require.NoError(t, err)
	require.Equal(t, value, res.Value)
}

func TestABCI_GetBlockRetentionHeight(t *testing.T) {
	logger := log.NewTestLogger(t)
	db := coretesting.NewMemDB()
	name := t.Name()

	snapshotStore, err := snapshots.NewStore(coretesting.NewMemDB(), testutil.GetTempDir(t))
	require.NoError(t, err)

	testCases := map[string]struct {
		bapp         *baseapp.BaseApp
		maxAgeBlocks int64
		commitHeight int64
		expected     int64
	}{
		"defaults": {
			bapp:         baseapp.NewBaseApp(name, logger, db, nil),
			maxAgeBlocks: 0,
			commitHeight: 499000,
			expected:     0,
		},
		"pruning unbonding time only": {
			bapp:         baseapp.NewBaseApp(name, logger, db, nil, baseapp.SetMinRetainBlocks(1)),
			maxAgeBlocks: 362880,
			commitHeight: 499000,
			expected:     136120,
		},
		"pruning iavl snapshot only": {
			bapp: baseapp.NewBaseApp(
				name, logger, db, nil,
				baseapp.SetPruning(pruningtypes.NewPruningOptions(pruningtypes.PruningNothing)),
				baseapp.SetMinRetainBlocks(1),
				baseapp.SetSnapshot(snapshotStore, snapshottypes.NewSnapshotOptions(10000, 1)),
			),
			maxAgeBlocks: 0,
			commitHeight: 499000,
			expected:     489000,
		},
		"pruning state sync snapshot only": {
			bapp: baseapp.NewBaseApp(
				name, logger, db, nil,
				baseapp.SetSnapshot(snapshotStore, snapshottypes.NewSnapshotOptions(50000, 3)),
				baseapp.SetMinRetainBlocks(1),
			),
			maxAgeBlocks: 0,
			commitHeight: 499000,
			expected:     349000,
		},
		"pruning min retention only": {
			bapp: baseapp.NewBaseApp(
				name, logger, db, nil,
				baseapp.SetMinRetainBlocks(400000),
			),
			maxAgeBlocks: 0,
			commitHeight: 499000,
			expected:     99000,
		},
		"pruning all conditions": {
			bapp: baseapp.NewBaseApp(
				name, logger, db, nil,
				baseapp.SetPruning(pruningtypes.NewCustomPruningOptions(0, 0)),
				baseapp.SetMinRetainBlocks(400000),
				baseapp.SetSnapshot(snapshotStore, snapshottypes.NewSnapshotOptions(50000, 3)),
			),
			maxAgeBlocks: 362880,
			commitHeight: 499000,
			expected:     99000,
		},
		"no pruning due to no persisted state": {
			bapp: baseapp.NewBaseApp(
				name, logger, db, nil,
				baseapp.SetPruning(pruningtypes.NewCustomPruningOptions(0, 0)),
				baseapp.SetMinRetainBlocks(400000),
				baseapp.SetSnapshot(snapshotStore, snapshottypes.NewSnapshotOptions(50000, 3)),
			),
			maxAgeBlocks: 362880,
			commitHeight: 10000,
			expected:     0,
		},
		"disable pruning": {
			bapp: baseapp.NewBaseApp(
				name, logger, db, nil,
				baseapp.SetPruning(pruningtypes.NewCustomPruningOptions(0, 0)),
				baseapp.SetMinRetainBlocks(0),
				baseapp.SetSnapshot(snapshotStore, snapshottypes.NewSnapshotOptions(50000, 3)),
			),
			maxAgeBlocks: 362880,
			commitHeight: 499000,
			expected:     0,
		},
	}

	for name, tc := range testCases {

		tc.bapp.SetParamStore(&paramStore{db: coretesting.NewMemDB()})
		_, err := tc.bapp.InitChain(&abci.InitChainRequest{
			ConsensusParams: &cmtproto.ConsensusParams{
				Evidence: &cmtproto.EvidenceParams{
					MaxAgeNumBlocks: tc.maxAgeBlocks,
				},
			},
		})
		require.NoError(t, err)

		t.Run(name, func(t *testing.T) {
			require.Equal(t, tc.expected, tc.bapp.GetBlockRetentionHeight(tc.commitHeight))
		})
	}
}

// Verifies that PrepareCheckState is called with the checkState.
func TestPrepareCheckStateCalledWithCheckState(t *testing.T) {
	t.Parallel()

	logger := log.NewTestLogger(t)
	db := coretesting.NewMemDB()
	name := t.Name()
	app := baseapp.NewBaseApp(name, logger, db, nil)

	wasPrepareCheckStateCalled := false
	app.SetPrepareCheckStater(func(ctx sdk.Context) {
		require.Equal(t, true, ctx.IsCheckTx())
		wasPrepareCheckStateCalled = true
	})

	_, err := app.FinalizeBlock(&abci.FinalizeBlockRequest{Height: 1})
	require.NoError(t, err)
	_, err = app.Commit()
	require.NoError(t, err)

	require.Equal(t, true, wasPrepareCheckStateCalled)
}

// Verifies that the Precommiter is called with the deliverState.
func TestPrecommiterCalledWithDeliverState(t *testing.T) {
	t.Parallel()

	logger := log.NewTestLogger(t)
	db := coretesting.NewMemDB()
	name := t.Name()
	app := baseapp.NewBaseApp(name, logger, db, nil)

	wasPrecommiterCalled := false
	app.SetPrecommiter(func(ctx sdk.Context) {
		require.Equal(t, false, ctx.IsCheckTx())
		require.Equal(t, false, ctx.IsReCheckTx())
		wasPrecommiterCalled = true
	})

	_, err := app.FinalizeBlock(&abci.FinalizeBlockRequest{Height: 1})
	require.NoError(t, err)
	_, err = app.Commit()
	require.NoError(t, err)

	require.Equal(t, true, wasPrecommiterCalled)
}

func TestABCI_Proposal_HappyPath(t *testing.T) {
	anteKey := []byte("ante-key")
	pool := mempool.NewSenderNonceMempool(mempool.SenderNonceMaxTxOpt(5000))
	anteOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAnteHandler(anteHandlerTxTest(t, capKey1, anteKey))
	}

	suite := NewBaseAppSuite(t, anteOpt, baseapp.SetMempool(pool))
	baseapptestutil.RegisterKeyValueServer(suite.baseApp.MsgServiceRouter(), MsgKeyValueImpl{})
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), NoopCounterServerImpl{})

	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		ConsensusParams: &cmtproto.ConsensusParams{},
	})
	require.NoError(t, err)

	tx := newTxCounter(t, suite.txConfig, suite.ac, 0, 1)
	txBytes, err := suite.txConfig.TxEncoder()(tx)
	require.NoError(t, err)

	reqCheckTx := abci.CheckTxRequest{
		Tx:   txBytes,
		Type: abci.CHECK_TX_TYPE_CHECK,
	}
	_, err = suite.baseApp.CheckTx(&reqCheckTx)
	require.NoError(t, err)

	tx2 := newTxCounter(t, suite.txConfig, suite.ac, 1, 1)

	tx2Bytes, err := suite.txConfig.TxEncoder()(tx2)
	require.NoError(t, err)

	err = pool.Insert(sdk.Context{}, tx2)
	require.NoError(t, err)

	reqPrepareProposal := abci.PrepareProposalRequest{
		MaxTxBytes: 1000,
		Height:     1,
	}
	resPrepareProposal, err := suite.baseApp.PrepareProposal(&reqPrepareProposal)
	require.NoError(t, err)
	require.Equal(t, 2, len(resPrepareProposal.Txs))

	reqProposalTxBytes := [2][]byte{
		txBytes,
		tx2Bytes,
	}
	reqProcessProposal := abci.ProcessProposalRequest{
		Txs:    reqProposalTxBytes[:],
		Height: reqPrepareProposal.Height,
	}

	resProcessProposal, err := suite.baseApp.ProcessProposal(&reqProcessProposal)
	require.NoError(t, err)
	require.Equal(t, abci.PROCESS_PROPOSAL_STATUS_ACCEPT, resProcessProposal.Status)

	// the same txs as in PrepareProposal
	res, err := suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{
		Height: suite.baseApp.LastBlockHeight() + 1,
		Txs:    reqProposalTxBytes[:],
	})
	require.NoError(t, err)

	require.Equal(t, 0, pool.CountTx())

	require.NotEmpty(t, res.TxResults[0].Events)
	require.True(t, res.TxResults[0].IsOK(), fmt.Sprintf("%v", res))
}

func TestABCI_Proposal_Read_State_PrepareProposal(t *testing.T) {
	someKey := []byte("some-key")

	setInitChainerOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetInitChainer(func(ctx sdk.Context, req *abci.InitChainRequest) (*abci.InitChainResponse, error) {
			ctx.KVStore(capKey1).Set(someKey, []byte(fooStr))
			return &abci.InitChainResponse{}, nil
		})
	}

	prepareOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetPrepareProposal(func(ctx sdk.Context, req *abci.PrepareProposalRequest) (*abci.PrepareProposalResponse, error) {
			value := ctx.KVStore(capKey1).Get(someKey)
			// We should be able to access any state written in InitChain
			require.Equal(t, fooStr, string(value))
			return &abci.PrepareProposalResponse{Txs: req.Txs}, nil
		})
	}

	suite := NewBaseAppSuite(t, setInitChainerOpt, prepareOpt)

	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		InitialHeight:   1,
		ConsensusParams: &cmtproto.ConsensusParams{},
	})
	require.NoError(t, err)

	reqPrepareProposal := abci.PrepareProposalRequest{
		MaxTxBytes: 1000,
		Height:     1, // this value can't be 0
	}
	resPrepareProposal, err := suite.baseApp.PrepareProposal(&reqPrepareProposal)
	require.NoError(t, err)
	require.Equal(t, 0, len(resPrepareProposal.Txs))

	reqProposalTxBytes := [][]byte{}
	reqProcessProposal := abci.ProcessProposalRequest{
		Txs:    reqProposalTxBytes,
		Height: reqPrepareProposal.Height,
	}

	resProcessProposal, err := suite.baseApp.ProcessProposal(&reqProcessProposal)
	require.NoError(t, err)
	require.Equal(t, abci.PROCESS_PROPOSAL_STATUS_ACCEPT, resProcessProposal.Status)
}

func TestABCI_Proposals_WithVE(t *testing.T) {
	someVoteExtension := []byte("some-vote-extension")

	setInitChainerOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetInitChainer(func(ctx sdk.Context, req *abci.InitChainRequest) (*abci.InitChainResponse, error) {
			return &abci.InitChainResponse{}, nil
		})
	}

	prepareOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetPrepareProposal(func(ctx sdk.Context, req *abci.PrepareProposalRequest) (*abci.PrepareProposalResponse, error) {
			// Inject the vote extension to the beginning of the proposal
			txs := make([][]byte, len(req.Txs)+1)
			txs[0] = someVoteExtension
			copy(txs[1:], req.Txs)

			return &abci.PrepareProposalResponse{Txs: txs}, nil
		})

		bapp.SetProcessProposal(func(ctx sdk.Context, req *abci.ProcessProposalRequest) (*abci.ProcessProposalResponse, error) {
			// Check that the vote extension is still there
			require.Equal(t, someVoteExtension, req.Txs[0])
			return &abci.ProcessProposalResponse{Status: abci.PROCESS_PROPOSAL_STATUS_ACCEPT}, nil
		})
	}

	suite := NewBaseAppSuite(t, setInitChainerOpt, prepareOpt)

	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		InitialHeight:   1,
		ConsensusParams: &cmtproto.ConsensusParams{},
	})
	require.NoError(t, err)
	reqPrepareProposal := abci.PrepareProposalRequest{
		MaxTxBytes: 100000,
		Height:     1, // this value can't be 0
	}
	resPrepareProposal, err := suite.baseApp.PrepareProposal(&reqPrepareProposal)
	require.NoError(t, err)
	require.Equal(t, 1, len(resPrepareProposal.Txs))

	reqProcessProposal := abci.ProcessProposalRequest{
		Txs:    resPrepareProposal.Txs,
		Height: reqPrepareProposal.Height,
	}
	resProcessProposal, err := suite.baseApp.ProcessProposal(&reqProcessProposal)
	require.NoError(t, err)
	require.Equal(t, abci.PROCESS_PROPOSAL_STATUS_ACCEPT, resProcessProposal.Status)

	// Run finalize block and ensure that the vote extension is still there and that
	// the proposal is accepted
	result, err := suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{
		Txs:    resPrepareProposal.Txs,
		Height: reqPrepareProposal.Height,
	})
	require.NoError(t, err)
	require.Equal(t, 1, len(result.TxResults))
	require.EqualValues(t, sdkerrors.ErrTxDecode.Codespace(), result.TxResults[0].Codespace, err)
	require.EqualValues(t, sdkerrors.ErrTxDecode.ABCICode(), result.TxResults[0].Code, err)
	require.EqualValues(t, 0, result.TxResults[0].GasUsed, err)
	require.EqualValues(t, 0, result.TxResults[0].GasWanted, err)
}

func TestABCI_PrepareProposal_ReachedMaxBytes(t *testing.T) {
	anteKey := []byte("ante-key")
	pool := mempool.NewSenderNonceMempool(mempool.SenderNonceMaxTxOpt(5000))
	anteOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAnteHandler(anteHandlerTxTest(t, capKey1, anteKey))
	}

	suite := NewBaseAppSuite(t, anteOpt, baseapp.SetMempool(pool))
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), NoopCounterServerImpl{})

	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		ConsensusParams: &cmtproto.ConsensusParams{},
	})
	require.NoError(t, err)

	expectedTxs := 8
	var expectedTxBytes int64

	for i := 0; i < 100; i++ {
		tx2 := newTxCounter(t, suite.txConfig, suite.ac, int64(i), int64(i))
		err := pool.Insert(sdk.Context{}, tx2)
		require.NoError(t, err)

		txBz, err := suite.txConfig.TxEncoder()(tx2)
		require.NoError(t, err)
		txDataSize := int(cmttypes.ComputeProtoSizeForTxs([]cmttypes.Tx{txBz}))
		if i < expectedTxs {
			expectedTxBytes += int64(txDataSize)
		}
	}

	reqPrepareProposal := abci.PrepareProposalRequest{
		MaxTxBytes: expectedTxBytes,
		Height:     1,
	}
	resPrepareProposal, err := suite.baseApp.PrepareProposal(&reqPrepareProposal)
	require.NoError(t, err)
	require.Equal(t, expectedTxs, len(resPrepareProposal.Txs))
}

func TestABCI_PrepareProposal_BadEncoding(t *testing.T) {
	anteKey := []byte("ante-key")
	pool := mempool.NewSenderNonceMempool(mempool.SenderNonceMaxTxOpt(5000))
	anteOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAnteHandler(anteHandlerTxTest(t, capKey1, anteKey))
	}

	suite := NewBaseAppSuite(t, anteOpt, baseapp.SetMempool(pool))
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), NoopCounterServerImpl{})

	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		ConsensusParams: &cmtproto.ConsensusParams{},
	})
	require.NoError(t, err)

	tx := newTxCounter(t, suite.txConfig, suite.ac, 0, 0)
	err = pool.Insert(sdk.Context{}, tx)
	require.NoError(t, err)

	reqPrepareProposal := abci.PrepareProposalRequest{
		MaxTxBytes: 1000,
		Height:     1,
	}
	resPrepareProposal, err := suite.baseApp.PrepareProposal(&reqPrepareProposal)
	require.NoError(t, err)
	require.Equal(t, 1, len(resPrepareProposal.Txs))
}

func TestABCI_PrepareProposal_OverGasUnderBytes(t *testing.T) {
	pool := mempool.NewSenderNonceMempool(mempool.SenderNonceMaxTxOpt(5000))
	suite := NewBaseAppSuite(t, baseapp.SetMempool(pool))
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), NoopCounterServerImpl{})

	// set max block gas limit to 99, this will allow 9 txs of 10 gas each.
	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		ConsensusParams: &cmtproto.ConsensusParams{
			Block: &cmtproto.BlockParams{MaxGas: 99},
		},
	})

	require.NoError(t, err)
	// insert 100 txs, each with a gas limit of 10
	_, _, addr := testdata.KeyTestPubAddr()
	addrStr, err := suite.ac.BytesToString(addr)
	require.NoError(t, err)
	for i := int64(0); i < 100; i++ {
		msg := &baseapptestutil.MsgCounter{Counter: i, FailOnHandler: false, Signer: addrStr}
		msgs := []sdk.Msg{msg}

		builder := suite.txConfig.NewTxBuilder()
		err = builder.SetMsgs(msgs...)
		require.NoError(t, err)
		builder.SetMemo(counterStr + strconv.FormatInt(i, 10) + failStr)
		builder.SetGasLimit(10)
		setTxSignature(t, builder, uint64(i))

		err := pool.Insert(sdk.Context{}, builder.GetTx())
		require.NoError(t, err)
	}

	// ensure we only select transactions that fit within the block gas limit
	res, err := suite.baseApp.PrepareProposal(&abci.PrepareProposalRequest{
		MaxTxBytes: 1_000_000, // large enough to ignore restriction
		Height:     1,
	})
	require.NoError(t, err)

	// Should include 9 transactions
	require.Len(t, res.Txs, 9, "invalid number of transactions returned")
}

func TestABCI_PrepareProposal_MaxGas(t *testing.T) {
	pool := mempool.NewSenderNonceMempool(mempool.SenderNonceMaxTxOpt(5000))
	suite := NewBaseAppSuite(t, baseapp.SetMempool(pool))
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), NoopCounterServerImpl{})

	// set max block gas limit to 100
	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		ConsensusParams: &cmtproto.ConsensusParams{
			Block: &cmtproto.BlockParams{MaxGas: 100},
		},
	})
	require.NoError(t, err)
	// insert 100 txs, each with a gas limit of 10
	_, _, addr := testdata.KeyTestPubAddr()
	addrStr, err := suite.ac.BytesToString(addr)
	require.NoError(t, err)
	for i := int64(0); i < 100; i++ {
		msg := &baseapptestutil.MsgCounter{Counter: i, FailOnHandler: false, Signer: addrStr}
		msgs := []sdk.Msg{msg}

		builder := suite.txConfig.NewTxBuilder()
		err = builder.SetMsgs(msgs...)
		require.NoError(t, err)
		builder.SetMemo(counterStr + strconv.FormatInt(i, 10) + failStr)
		builder.SetGasLimit(10)
		setTxSignature(t, builder, uint64(i))

		err := pool.Insert(sdk.Context{}, builder.GetTx())
		require.NoError(t, err)
	}

	// ensure we only select transactions that fit within the block gas limit
	res, err := suite.baseApp.PrepareProposal(&abci.PrepareProposalRequest{
		MaxTxBytes: 1_000_000, // large enough to ignore restriction
		Height:     1,
	})
	require.NoError(t, err)
	require.Len(t, res.Txs, 10, "invalid number of transactions returned")
}

func TestABCI_PrepareProposal_Failures(t *testing.T) {
	anteKey := []byte("ante-key")
	pool := mempool.NewSenderNonceMempool(mempool.SenderNonceMaxTxOpt(5000))
	anteOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAnteHandler(anteHandlerTxTest(t, capKey1, anteKey))
	}

	suite := NewBaseAppSuite(t, anteOpt, baseapp.SetMempool(pool))
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), NoopCounterServerImpl{})

	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		ConsensusParams: &cmtproto.ConsensusParams{},
	})
	require.NoError(t, err)

	tx := newTxCounter(t, suite.txConfig, suite.ac, 0, 0)
	txBytes, err := suite.txConfig.TxEncoder()(tx)
	require.NoError(t, err)

	reqCheckTx := abci.CheckTxRequest{
		Tx:   txBytes,
		Type: abci.CHECK_TX_TYPE_CHECK,
	}
	checkTxRes, err := suite.baseApp.CheckTx(&reqCheckTx)
	require.NoError(t, err)
	require.True(t, checkTxRes.IsOK())

	failTx := newTxCounter(t, suite.txConfig, suite.ac, 1, 1)
	failTx = setFailOnAnte(t, suite.txConfig, failTx, true)

	err = pool.Insert(sdk.Context{}, failTx)
	require.NoError(t, err)
	require.Equal(t, 2, pool.CountTx())

	req := abci.PrepareProposalRequest{
		MaxTxBytes: 1000,
		Height:     1,
	}
	res, err := suite.baseApp.PrepareProposal(&req)
	require.NoError(t, err)
	require.Equal(t, 1, len(res.Txs))
}

func TestABCI_PrepareProposal_PanicRecovery(t *testing.T) {
	prepareOpt := func(app *baseapp.BaseApp) {
		app.SetPrepareProposal(func(ctx sdk.Context, rpp *abci.PrepareProposalRequest) (*abci.PrepareProposalResponse, error) {
			panic(errors.New("test"))
		})
	}
	suite := NewBaseAppSuite(t, prepareOpt)

	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		ConsensusParams: &cmtproto.ConsensusParams{},
	})
	require.NoError(t, err)

	req := abci.PrepareProposalRequest{
		MaxTxBytes: 1000,
		Height:     1,
	}

	require.NotPanics(t, func() {
		res, err := suite.baseApp.PrepareProposal(&req)
		require.NoError(t, err)
		require.Equal(t, req.Txs, res.Txs)
	})
}

func TestABCI_PrepareProposal_VoteExtensions(t *testing.T) {
	// set up mocks
	ctrl := gomock.NewController(t)
	valStore := mock.NewMockValidatorStore(ctrl)
	privkey := secp256k1.GenPrivKey()
	pubkey := privkey.PubKey()
	addr := sdk.AccAddress(pubkey.Address())
	tmPk := cmtprotocrypto.PublicKey{
		Sum: &cmtprotocrypto.PublicKey_Secp256K1{
			Secp256K1: pubkey.Bytes(),
		},
	}

	pk, err := cryptocodec.FromCmtProtoPublicKey(tmPk)
	require.NoError(t, err)

	consAddr := sdk.ConsAddress(addr)
	valStore.EXPECT().GetPubKeyByConsAddr(gomock.Any(), consAddr.Bytes()).Return(pk, nil)

	// set up baseapp
	prepareOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetPrepareProposal(func(ctx sdk.Context, req *abci.PrepareProposalRequest) (*abci.PrepareProposalResponse, error) {
			ctx = ctx.WithBlockHeight(req.Height).WithChainID(bapp.ChainID())
			_, info := extendedCommitToLastCommit(req.LocalLastCommit)
			ctx = ctx.WithCometInfo(info)
			err := baseapp.ValidateVoteExtensions(ctx, valStore, req.LocalLastCommit)
			if err != nil {
				return nil, err
			}

			cp := ctx.ConsensusParams() //nolint:staticcheck // ignore linting error
			extsEnabled := cp.Feature.VoteExtensionsEnableHeight != nil && req.Height >= cp.Feature.VoteExtensionsEnableHeight.Value && cp.Feature.VoteExtensionsEnableHeight.Value != 0
			if !extsEnabled {
				// check abci params
				extsEnabled = cp.Abci != nil && req.Height >= cp.Abci.VoteExtensionsEnableHeight && cp.Abci.VoteExtensionsEnableHeight != 0
			}
			if extsEnabled {
				req.Txs = append(req.Txs, []byte("some-tx-that-does-something-from-votes"))
			}
			return &abci.PrepareProposalResponse{Txs: req.Txs}, nil
		})
	}

	suite := NewBaseAppSuite(t, prepareOpt)

	_, err = suite.baseApp.InitChain(&abci.InitChainRequest{
		InitialHeight: 1,
		ConsensusParams: &cmtproto.ConsensusParams{
			Feature: &cmtproto.FeatureParams{
				VoteExtensionsEnableHeight: &gogotypes.Int64Value{Value: 2},
			},
		},
	})
	require.NoError(t, err)

	// first test without vote extensions, no new txs should be added
	reqPrepareProposal := abci.PrepareProposalRequest{
		MaxTxBytes: 1000,
		Height:     1, // this value can't be 0
	}
	resPrepareProposal, err := suite.baseApp.PrepareProposal(&reqPrepareProposal)
	require.NoError(t, err)
	require.Equal(t, 0, len(resPrepareProposal.Txs))

	// now we try with vote extensions, a new tx should show up
	marshalDelimitedFn := func(msg proto.Message) ([]byte, error) {
		var buf bytes.Buffer
		if err := protoio.NewDelimitedWriter(&buf).WriteMsg(msg); err != nil {
			return nil, err
		}

		return buf.Bytes(), nil
	}

	ext := []byte("something")
	cve := cmtproto.CanonicalVoteExtension{
		Extension: ext,
		Height:    2, // the vote extension was signed in the previous height
		Round:     int64(0),
		ChainId:   suite.baseApp.ChainID(),
	}

	bz, err := marshalDelimitedFn(&cve)
	require.NoError(t, err)

	extSig, err := privkey.Sign(bz)
	require.NoError(t, err)

	reqPrepareProposal = abci.PrepareProposalRequest{
		MaxTxBytes: 1000,
		Height:     3, // this value can't be 0
		LocalLastCommit: abci.ExtendedCommitInfo{
			Round: 0,
			Votes: []abci.ExtendedVoteInfo{
				{
					Validator: abci.Validator{
						Address: consAddr.Bytes(),
						Power:   666,
					},
					VoteExtension:      ext,
					ExtensionSignature: extSig,
					BlockIdFlag:        cmtproto.BlockIDFlagCommit,
				},
			},
		},
	}
	resPrepareProposal, err = suite.baseApp.PrepareProposal(&reqPrepareProposal)
	require.NoError(t, err)
	require.Equal(t, 1, len(resPrepareProposal.Txs))

	// now vote extensions but our sole voter doesn't reach majority
	reqPrepareProposal = abci.PrepareProposalRequest{
		MaxTxBytes: 1000,
		Height:     3, // this value can't be 0
		LocalLastCommit: abci.ExtendedCommitInfo{
			Round: 0,
			Votes: []abci.ExtendedVoteInfo{
				{
					Validator: abci.Validator{
						Address: consAddr.Bytes(),
						Power:   666,
					},
					VoteExtension:      ext,
					ExtensionSignature: extSig,
					BlockIdFlag:        cmtproto.BlockIDFlagNil, // This will ignore the vote extension
				},
			},
		},
	}
	resPrepareProposal, err = suite.baseApp.PrepareProposal(&reqPrepareProposal)
	require.NoError(t, err)
	require.Equal(t, 0, len(resPrepareProposal.Txs))
}

func TestABCI_ProcessProposal_PanicRecovery(t *testing.T) {
	processOpt := func(app *baseapp.BaseApp) {
		app.SetProcessProposal(func(ctx sdk.Context, rpp *abci.ProcessProposalRequest) (*abci.ProcessProposalResponse, error) {
			panic(errors.New("test"))
		})
	}
	suite := NewBaseAppSuite(t, processOpt)

	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		ConsensusParams: &cmtproto.ConsensusParams{},
	})
	require.NoError(t, err)

	require.NotPanics(t, func() {
		res, err := suite.baseApp.ProcessProposal(&abci.ProcessProposalRequest{Height: 1})
		require.NoError(t, err)
		require.Equal(t, res.Status, abci.PROCESS_PROPOSAL_STATUS_REJECT)
	})
}

// TestABCI_Proposal_Reset_State ensures that state is reset between runs of
// PrepareProposal and ProcessProposal in case they are called multiple times.
// This is only valid for heights > 1, given that on height 1 we always set the
// state to be deliverState.
func TestABCI_Proposal_Reset_State_Between_Calls(t *testing.T) {
	someKey := []byte("some-key")

	prepareOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetPrepareProposal(func(ctx sdk.Context, req *abci.PrepareProposalRequest) (*abci.PrepareProposalResponse, error) {
			// This key should not exist given that we reset the state on every call.
			require.False(t, ctx.KVStore(capKey1).Has(someKey))
			ctx.KVStore(capKey1).Set(someKey, someKey)
			return &abci.PrepareProposalResponse{Txs: req.Txs}, nil
		})
	}

	processOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetProcessProposal(func(ctx sdk.Context, req *abci.ProcessProposalRequest) (*abci.ProcessProposalResponse, error) {
			// This key should not exist given that we reset the state on every call.
			require.False(t, ctx.KVStore(capKey1).Has(someKey))
			ctx.KVStore(capKey1).Set(someKey, someKey)
			return &abci.ProcessProposalResponse{Status: abci.PROCESS_PROPOSAL_STATUS_ACCEPT}, nil
		})
	}

	suite := NewBaseAppSuite(t, prepareOpt, processOpt)

	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		ConsensusParams: &cmtproto.ConsensusParams{},
	})
	require.NoError(t, err)

	reqPrepareProposal := abci.PrepareProposalRequest{
		MaxTxBytes: 1000,
		Height:     2, // this value can't be 0
	}

	// Let's pretend something happened and PrepareProposal gets called many
	// times, this must be safe to do.
	for i := 0; i < 5; i++ {
		resPrepareProposal, err := suite.baseApp.PrepareProposal(&reqPrepareProposal)
		require.NoError(t, err)
		require.Equal(t, 0, len(resPrepareProposal.Txs))
	}

	reqProposalTxBytes := [][]byte{}
	reqProcessProposal := abci.ProcessProposalRequest{
		Txs:    reqProposalTxBytes,
		Height: 2,
	}

	// Let's pretend something happened and ProcessProposal gets called many
	// times, this must be safe to do.
	for i := 0; i < 5; i++ {
		resProcessProposal, err := suite.baseApp.ProcessProposal(&reqProcessProposal)
		require.NoError(t, err)
		require.Equal(t, abci.PROCESS_PROPOSAL_STATUS_ACCEPT, resProcessProposal.Status)
	}
}

func TestABCI_HaltChain(t *testing.T) {
	testCases := []struct {
		name        string
		haltHeight  uint64
		haltTime    uint64
		blockHeight int64
		blockTime   int64
		expHalt     bool
	}{
		{"default", 0, 0, 10, 0, false},
		{"halt-height-edge", 11, 0, 10, 0, false},
		{"halt-height-equal", 10, 0, 10, 0, true},
		{"halt-height", 10, 0, 10, 0, true},
		{"halt-time-edge", 0, 11, 1, 10, false},
		{"halt-time-equal", 0, 10, 1, 10, true},
		{"halt-time", 0, 10, 1, 11, true},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			suite := NewBaseAppSuite(t, baseapp.SetHaltHeight(tc.haltHeight), baseapp.SetHaltTime(tc.haltTime))
			_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
				ConsensusParams: &cmtproto.ConsensusParams{},
				InitialHeight:   tc.blockHeight,
			})
			require.NoError(t, err)
			app := suite.baseApp
			_, err = app.FinalizeBlock(&abci.FinalizeBlockRequest{
				Height: tc.blockHeight,
				Time:   time.Unix(tc.blockTime, 0),
			})
			if !tc.expHalt {
				require.NoError(t, err)
			} else {
				require.Error(t, err)
				require.True(t, strings.HasPrefix(err.Error(), "halt per configuration"))
			}
		})
	}
}

func TestBaseApp_PreBlocker(t *testing.T) {
	db := coretesting.NewMemDB()
	name := t.Name()
	logger := log.NewTestLogger(t)

	app := baseapp.NewBaseApp(name, logger, db, nil)
	_, err := app.InitChain(&abci.InitChainRequest{})
	require.NoError(t, err)

	wasHookCalled := false
	app.SetPreBlocker(func(ctx sdk.Context, req *abci.FinalizeBlockRequest) error {
		wasHookCalled = true
		ctx.EventManager().EmitEvent(sdk.NewEvent("preblockertest", sdk.NewAttribute("height", fmt.Sprintf("%d", req.Height))))
		return nil
	})
	app.Seal()

	res, err := app.FinalizeBlock(&abci.FinalizeBlockRequest{Height: 1})
	require.NoError(t, err)
	require.Equal(t, true, wasHookCalled)
	require.Len(t, res.Events, 1)
	require.Equal(t, "preblockertest", res.Events[0].Type)

	// Now try erroring
	app = baseapp.NewBaseApp(name, logger, db, nil)
	_, err = app.InitChain(&abci.InitChainRequest{})
	require.NoError(t, err)

	app.SetPreBlocker(func(ctx sdk.Context, req *abci.FinalizeBlockRequest) error {
		return errors.New("some error")
	})
	app.Seal()

	_, err = app.FinalizeBlock(&abci.FinalizeBlockRequest{Height: 1})
	require.Error(t, err)
}

// TestBaseApp_VoteExtensions tests vote extensions using a price as an example.
func TestBaseApp_VoteExtensions(t *testing.T) {
	ctrl := gomock.NewController(t)
	valStore := mock.NewMockValidatorStore(ctrl)

	// 10 good vote extensions, 2 bad ones from 12 total validators
	numVals := 12
	privKeys := make([]secp256k1.PrivKey, numVals)
	vals := make([]sdk.ConsAddress, numVals)
	for i := 0; i < numVals; i++ {
		privKey := secp256k1.GenPrivKey()
		privKeys[i] = privKey

		pubKey := privKey.PubKey()
		val := sdk.ConsAddress(pubKey.Bytes())
		vals[i] = val

		tmPk := cmtprotocrypto.PublicKey{
			Sum: &cmtprotocrypto.PublicKey_Secp256K1{
				Secp256K1: pubKey.Bytes(),
			},
		}

		pk, err := cryptocodec.FromCmtProtoPublicKey(tmPk)
		require.NoError(t, err)
		valStore.EXPECT().GetPubKeyByConsAddr(gomock.Any(), val).Return(pk, nil)
	}

	baseappOpts := func(app *baseapp.BaseApp) {
		app.SetExtendVoteHandler(func(sdk.Context, *abci.ExtendVoteRequest) (*abci.ExtendVoteResponse, error) {
			// here we would have a process to get the price from an external source
			price := 10000000 + rand.Int63n(1000000)
			ve := make([]byte, 8)
			binary.BigEndian.PutUint64(ve, uint64(price))
			return &abci.ExtendVoteResponse{VoteExtension: ve}, nil
		})

		app.SetVerifyVoteExtensionHandler(func(_ sdk.Context, req *abci.VerifyVoteExtensionRequest) (*abci.VerifyVoteExtensionResponse, error) {
			vePrice := binary.BigEndian.Uint64(req.VoteExtension)
			// here we would do some price validation, must not be 0 and not too high
			if vePrice > 11000000 || vePrice == 0 {
				// usually application should always return ACCEPT unless they really want to discard the entire vote
				return &abci.VerifyVoteExtensionResponse{Status: abci.VERIFY_VOTE_EXTENSION_STATUS_REJECT}, nil
			}

			return &abci.VerifyVoteExtensionResponse{Status: abci.VERIFY_VOTE_EXTENSION_STATUS_ACCEPT}, nil
		})

		app.SetPrepareProposal(func(ctx sdk.Context, req *abci.PrepareProposalRequest) (*abci.PrepareProposalResponse, error) {
			txs := [][]byte{}
			ctx = ctx.WithBlockHeight(req.Height).WithChainID(app.ChainID())
			_, info := extendedCommitToLastCommit(req.LocalLastCommit)
			ctx = ctx.WithCometInfo(info)
			if err := baseapp.ValidateVoteExtensions(ctx, valStore, req.LocalLastCommit); err != nil {
				return nil, err
			}
			// add all VE as txs (in a real scenario we would need to check signatures too)
			for _, v := range req.LocalLastCommit.Votes {
				if len(v.VoteExtension) == 8 {
					// pretend this is a way to check if the VE is valid
					if binary.BigEndian.Uint64(v.VoteExtension) < 11000000 && binary.BigEndian.Uint64(v.VoteExtension) > 0 {
						txs = append(txs, v.VoteExtension)
					}
				}
			}

			return &abci.PrepareProposalResponse{Txs: txs}, nil
		})

		app.SetProcessProposal(func(ctx sdk.Context, req *abci.ProcessProposalRequest) (*abci.ProcessProposalResponse, error) {
			// here we check if the proposal is valid, mainly if the vote extensions appended to the txs are valid
			for _, v := range req.Txs {
				// pretend this is a way to check if the tx is actually a VE
				if len(v) == 8 {
					// pretend this is a way to check if the VE is valid
					if binary.BigEndian.Uint64(v) > 11000000 || binary.BigEndian.Uint64(v) == 0 {
						return &abci.ProcessProposalResponse{Status: abci.PROCESS_PROPOSAL_STATUS_REJECT}, nil
					}
				}
			}

			return &abci.ProcessProposalResponse{Status: abci.PROCESS_PROPOSAL_STATUS_ACCEPT}, nil
		})

		app.SetPreBlocker(func(ctx sdk.Context, req *abci.FinalizeBlockRequest) error {
			count := uint64(0)
			pricesSum := uint64(0)
			for _, v := range req.Txs {
				// pretend this is a way to check if the tx is actually a VE
				if len(v) == 8 {
					count++
					pricesSum += binary.BigEndian.Uint64(v)
				}
			}

			if count > 0 {
				// we process the average price and store it in the context to make it available for FinalizeBlock
				avgPrice := pricesSum / count
				buf := make([]byte, 8)
				binary.BigEndian.PutUint64(buf, avgPrice)
				ctx.KVStore(capKey1).Set([]byte("avgPrice"), buf)
			}

			return nil
		})
	}

	suite := NewBaseAppSuite(t, baseappOpts)

	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		ConsensusParams: &cmtproto.ConsensusParams{
			Feature: &cmtproto.FeatureParams{
				VoteExtensionsEnableHeight: &gogotypes.Int64Value{Value: 1},
			},
		},
	})
	require.NoError(t, err)

	allVEs := [][]byte{}
	// simulate getting 10 vote extensions from 10 validators
	for i := 0; i < 10; i++ {
		ve, err := suite.baseApp.ExtendVote(context.TODO(), &abci.ExtendVoteRequest{Height: 1})
		require.NoError(t, err)
		allVEs = append(allVEs, ve.VoteExtension)
	}

	// add a couple of invalid vote extensions (in what regards to the check we are doing in VerifyVoteExtension/ProcessProposal)
	// add a 0 price
	ve := make([]byte, 8)
	binary.BigEndian.PutUint64(ve, uint64(0))
	allVEs = append(allVEs, ve)

	// add a price too high
	ve = make([]byte, 8)
	binary.BigEndian.PutUint64(ve, uint64(13000000))
	allVEs = append(allVEs, ve)

	// verify all votes, only 10 should be accepted
	successful := 0
	for _, v := range allVEs {
		res, err := suite.baseApp.VerifyVoteExtension(&abci.VerifyVoteExtensionRequest{
			Height:        1,
			VoteExtension: v,
		})
		require.NoError(t, err)
		if res.Status == abci.VERIFY_VOTE_EXTENSION_STATUS_ACCEPT {
			successful++
		}
	}
	require.Equal(t, 10, successful)

	extVotes := []abci.ExtendedVoteInfo{}
	for _, val := range vals {
		extVotes = append(extVotes, abci.ExtendedVoteInfo{
			VoteExtension:      allVEs[0],
			BlockIdFlag:        cmtproto.BlockIDFlagCommit,
			ExtensionSignature: []byte{},
			Validator: abci.Validator{
				Address: val.Bytes(),
				Power:   666,
			},
		},
		)
	}

	prepPropReq := &abci.PrepareProposalRequest{
		Height: 1,
		LocalLastCommit: abci.ExtendedCommitInfo{
			Round: 0,
			Votes: extVotes,
		},
	}

	// add all VEs to the local last commit, which will make PrepareProposal fail
	// because it's not expecting to receive vote extensions when height == VoteExtensionsEnableHeight
	for _, ve := range allVEs {
		prepPropReq.LocalLastCommit.Votes = append(prepPropReq.LocalLastCommit.Votes, abci.ExtendedVoteInfo{
			VoteExtension:      ve,
			BlockIdFlag:        cmtproto.BlockIDFlagCommit,
			ExtensionSignature: []byte{}, // doesn't matter, it's just to make the next PrepareProposal fail
		})
	}
	resp, err := suite.baseApp.PrepareProposal(prepPropReq)
	require.Len(t, resp.Txs, 0) // this is actually a failure, but we don't want to halt the chain
	require.NoError(t, err)     // we don't error here

	prepPropReq.LocalLastCommit.Votes = []abci.ExtendedVoteInfo{} // reset votes
	resp, err = suite.baseApp.PrepareProposal(prepPropReq)
	require.NoError(t, err)
	require.Len(t, resp.Txs, 0)

	procPropRes, err := suite.baseApp.ProcessProposal(&abci.ProcessProposalRequest{Height: 1, Txs: resp.Txs})
	require.NoError(t, err)
	require.Equal(t, abci.PROCESS_PROPOSAL_STATUS_ACCEPT, procPropRes.Status)

	_, err = suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{Height: 1, Txs: resp.Txs})
	require.NoError(t, err)

	// The average price will be nil during the first block, given that we don't have
	// any vote extensions on block 1 in PrepareProposal
	avgPrice := getFinalizeBlockStateCtx(suite.baseApp).KVStore(capKey1).Get([]byte("avgPrice"))
	require.Nil(t, avgPrice)
	_, err = suite.baseApp.Commit()
	require.NoError(t, err)

	// Now onto the second block, this time we process vote extensions from the
	// previous block (which we sign now)
	for i, ve := range allVEs {
		cve := cmtproto.CanonicalVoteExtension{
			Extension: ve,
			Height:    1,
			Round:     int64(0),
			ChainId:   suite.baseApp.ChainID(),
		}

		bz, err := marshalDelimitedFn(&cve)
		require.NoError(t, err)

		privKey := privKeys[i]
		extSig, err := privKey.Sign(bz)
		require.NoError(t, err)

		prepPropReq.LocalLastCommit.Votes = append(prepPropReq.LocalLastCommit.Votes, abci.ExtendedVoteInfo{
			VoteExtension:      ve,
			BlockIdFlag:        cmtproto.BlockIDFlagCommit,
			ExtensionSignature: extSig,
			Validator: abci.Validator{
				Address: vals[i].Bytes(),
				Power:   666,
			},
		})
	}

	prepPropReq.Height = 2
	resp, err = suite.baseApp.PrepareProposal(prepPropReq)
	require.NoError(t, err)
	require.Len(t, resp.Txs, 10)

	procPropRes, err = suite.baseApp.ProcessProposal(&abci.ProcessProposalRequest{Height: 2, Txs: resp.Txs})
	require.NoError(t, err)
	require.Equal(t, abci.PROCESS_PROPOSAL_STATUS_ACCEPT, procPropRes.Status)

	_, err = suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{Height: 2, Txs: resp.Txs})
	require.NoError(t, err)

	// Check if the average price was available in FinalizeBlock's context
	avgPrice = getFinalizeBlockStateCtx(suite.baseApp).KVStore(capKey1).Get([]byte("avgPrice"))
	require.NotNil(t, avgPrice)
	require.GreaterOrEqual(t, binary.BigEndian.Uint64(avgPrice), uint64(10000000))
	require.Less(t, binary.BigEndian.Uint64(avgPrice), uint64(11000000))

	_, err = suite.baseApp.Commit()
	require.NoError(t, err)

	// check if avgPrice was committed
	committedAvgPrice := suite.baseApp.NewContext(true).KVStore(capKey1).Get([]byte("avgPrice"))
	require.Equal(t, avgPrice, committedAvgPrice)
}

func TestABCI_PrepareProposal_Panic(t *testing.T) {
	prepareOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetPrepareProposal(func(ctx sdk.Context, req *abci.PrepareProposalRequest) (*abci.PrepareProposalResponse, error) {
			if len(req.Txs) == 3 {
				panic("i don't like number 3, panic")
			}
			// return empty if no panic
			return &abci.PrepareProposalResponse{}, nil
		})
	}

	suite := NewBaseAppSuite(t, prepareOpt)

	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		InitialHeight:   1,
		ConsensusParams: &cmtproto.ConsensusParams{},
	})
	require.NoError(t, err)

	txs := [][]byte{{1}, {2}}
	reqPrepareProposal := abci.PrepareProposalRequest{
		MaxTxBytes: 1000,
		Height:     1, // this value can't be 0
		Txs:        txs,
	}
	resPrepareProposal, err := suite.baseApp.PrepareProposal(&reqPrepareProposal)
	require.NoError(t, err)
	require.Equal(t, 0, len(resPrepareProposal.Txs))

	// make it panic, and check if it returns 3 txs (because of panic recovery)
	txs = [][]byte{{1}, {2}, {3}}
	reqPrepareProposal.Txs = txs
	resPrepareProposal, err = suite.baseApp.PrepareProposal(&reqPrepareProposal)
	require.NoError(t, err)
	require.Equal(t, 3, len(resPrepareProposal.Txs))
}

func TestOptimisticExecution(t *testing.T) {
	suite := NewBaseAppSuite(t, baseapp.SetOptimisticExecution())

	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		ConsensusParams: &cmtproto.ConsensusParams{},
	})
	require.NoError(t, err)

	// run 50 blocks
	for i := 0; i < 50; i++ {
		tx := newTxCounter(t, suite.txConfig, suite.ac, 0, 1)
		txBytes, err := suite.txConfig.TxEncoder()(tx)
		require.NoError(t, err)

		reqProcProp := abci.ProcessProposalRequest{
			Txs:    [][]byte{txBytes},
			Height: suite.baseApp.LastBlockHeight() + 1,
			Hash:   []byte("some-hash" + strconv.FormatInt(suite.baseApp.LastBlockHeight()+1, 10)),
		}

		respProcProp, err := suite.baseApp.ProcessProposal(&reqProcProp)
		require.Equal(t, abci.PROCESS_PROPOSAL_STATUS_ACCEPT, respProcProp.Status)
		require.NoError(t, err)

		reqFinalizeBlock := abci.FinalizeBlockRequest{
			Height: reqProcProp.Height,
			Txs:    reqProcProp.Txs,
			Hash:   reqProcProp.Hash,
		}

		respFinalizeBlock, err := suite.baseApp.FinalizeBlock(&reqFinalizeBlock)
		require.NoError(t, err)
		require.Len(t, respFinalizeBlock.TxResults, 1)

		_, err = suite.baseApp.Commit()
		require.NoError(t, err)
	}

	require.Equal(t, int64(50), suite.baseApp.LastBlockHeight())
}

func TestABCI_Proposal_FailReCheckTx(t *testing.T) {
	pool := mempool.NewPriorityMempool[int64](mempool.PriorityNonceMempoolConfig[int64]{
		TxPriority:      mempool.NewDefaultTxPriority(),
		MaxTx:           0,
		SignerExtractor: mempool.NewDefaultSignerExtractionAdapter(),
	})

	anteOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAnteHandler(func(ctx sdk.Context, tx sdk.Tx, simulate bool) (sdk.Context, error) {
			// always fail on recheck, just to test the recheck logic
			if ctx.IsReCheckTx() {
				return ctx, errors.New("recheck failed in ante handler")
			}

			return ctx, nil
		})
	}

	suite := NewBaseAppSuite(t, anteOpt, baseapp.SetMempool(pool))
	baseapptestutil.RegisterKeyValueServer(suite.baseApp.MsgServiceRouter(), MsgKeyValueImpl{})
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), NoopCounterServerImpl{})

	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		ConsensusParams: &cmtproto.ConsensusParams{},
	})
	require.NoError(t, err)

	tx := newTxCounter(t, suite.txConfig, suite.ac, 0, 1)
	txBytes, err := suite.txConfig.TxEncoder()(tx)
	require.NoError(t, err)

	reqCheckTx := abci.CheckTxRequest{
		Tx:   txBytes,
		Type: abci.CHECK_TX_TYPE_CHECK,
	}
	_, err = suite.baseApp.CheckTx(&reqCheckTx)
	require.NoError(t, err)

	tx2 := newTxCounter(t, suite.txConfig, suite.ac, 1, 1)

	tx2Bytes, err := suite.txConfig.TxEncoder()(tx2)
	require.NoError(t, err)

	err = pool.Insert(sdk.Context{}, tx2)
	require.NoError(t, err)

	require.Equal(t, 2, pool.CountTx())

	// call prepareProposal before calling recheck tx, just as a sanity check
	reqPrepareProposal := abci.PrepareProposalRequest{
		MaxTxBytes: 1000,
		Height:     1,
	}
	resPrepareProposal, err := suite.baseApp.PrepareProposal(&reqPrepareProposal)
	require.NoError(t, err)
	require.Equal(t, 2, len(resPrepareProposal.Txs))

	// call recheck on the first tx, it MUST return an error
	reqReCheckTx := abci.CheckTxRequest{
		Tx:   txBytes,
		Type: abci.CHECK_TX_TYPE_RECHECK,
	}
	resp, err := suite.baseApp.CheckTx(&reqReCheckTx)
	require.NoError(t, err)
	require.True(t, resp.IsErr())
	require.Equal(t, "recheck failed in ante handler", resp.Log)

	// call prepareProposal again, should return only the second tx
	resPrepareProposal, err = suite.baseApp.PrepareProposal(&reqPrepareProposal)
	require.NoError(t, err)
	require.Equal(t, 1, len(resPrepareProposal.Txs))
	require.Equal(t, tx2Bytes, resPrepareProposal.Txs[0])

	// check the mempool, it should have only the second tx
	require.Equal(t, 1, pool.CountTx())

	reqProposalTxBytes := [][]byte{
		tx2Bytes,
	}
	reqProcessProposal := abci.ProcessProposalRequest{
		Txs:    reqProposalTxBytes,
		Height: reqPrepareProposal.Height,
	}

	resProcessProposal, err := suite.baseApp.ProcessProposal(&reqProcessProposal)
	require.NoError(t, err)
	require.Equal(t, abci.PROCESS_PROPOSAL_STATUS_ACCEPT, resProcessProposal.Status)

	// the same txs as in PrepareProposal
	res, err := suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{
		Height: suite.baseApp.LastBlockHeight() + 1,
		Txs:    reqProposalTxBytes,
	})
	require.NoError(t, err)

	require.Equal(t, 0, pool.CountTx())

	require.NotEmpty(t, res.TxResults[0].Events)
	require.True(t, res.TxResults[0].IsOK(), fmt.Sprintf("%v", res))
}
