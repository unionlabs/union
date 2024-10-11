package inspect_test

import (
	"context"
	"fmt"
	"net"
	"os"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/fortytw2/leaktest"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/require"

	abcitypes "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/config"
	"github.com/cometbft/cometbft/internal/inspect"
	"github.com/cometbft/cometbft/internal/test"
	"github.com/cometbft/cometbft/libs/pubsub/query"
	httpclient "github.com/cometbft/cometbft/rpc/client/http"
	indexermocks "github.com/cometbft/cometbft/state/indexer/mocks"
	statemocks "github.com/cometbft/cometbft/state/mocks"
	txindexmocks "github.com/cometbft/cometbft/state/txindex/mocks"
	"github.com/cometbft/cometbft/types"
)

func TestInspectConstructor(t *testing.T) {
	cfg := test.ResetTestRoot("test")
	t.Cleanup(leaktest.Check(t))
	defer func() { _ = os.RemoveAll(cfg.RootDir) }()
	t.Run("from config", func(t *testing.T) {
		d, err := inspect.NewFromConfig(cfg)
		require.NoError(t, err)
		require.NotNil(t, d)
	})
}

func TestInspectRun(t *testing.T) {
	cfg := test.ResetTestRoot("test")
	t.Cleanup(leaktest.Check(t))
	defer func() { _ = os.RemoveAll(cfg.RootDir) }()
	t.Run("from config", func(t *testing.T) {
		d, err := inspect.NewFromConfig(cfg)
		require.NoError(t, err)
		ctx, cancel := context.WithCancel(context.Background())
		stoppedWG := &sync.WaitGroup{}
		stoppedWG.Add(1)
		go func() {
			require.NoError(t, d.Run(ctx))
			stoppedWG.Done()
		}()
		cancel()
		stoppedWG.Wait()
	})
}

func TestBlock(t *testing.T) {
	testHeight := int64(1)
	testBlock := new(types.Block)
	testBlock.Header.Height = testHeight
	testBlock.Header.LastCommitHash = []byte("test hash")
	stateStoreMock := &statemocks.Store{}
	stateStoreMock.On("Close").Return(nil)

	blockStoreMock := &statemocks.BlockStore{}
	blockStoreMock.On("Height").Return(testHeight)
	blockStoreMock.On("Base").Return(int64(0))
	blockStoreMock.On("LoadBlock", testHeight).Return(testBlock, &types.BlockMeta{})
	blockStoreMock.On("Close").Return(nil)

	txIndexerMock := &txindexmocks.TxIndexer{}
	blkIdxMock := &indexermocks.BlockIndexer{}

	rpcConfig := config.TestRPCConfig()
	d := inspect.New(rpcConfig, blockStoreMock, stateStoreMock, txIndexerMock, blkIdxMock)
	ctx, cancel := context.WithCancel(context.Background())
	wg := &sync.WaitGroup{}
	wg.Add(1)

	startedWG := &sync.WaitGroup{}
	startedWG.Add(1)
	go func() {
		startedWG.Done()
		defer wg.Done()
		require.NoError(t, d.Run(ctx))
	}()
	// FIXME: used to induce context switch.
	// Determine more deterministic method for prompting a context switch
	startedWG.Wait()
	requireConnect(t, rpcConfig.ListenAddress)
	cli, err := httpclient.New(rpcConfig.ListenAddress + "/v1")
	require.NoError(t, err)
	resultBlock, err := cli.Block(context.Background(), &testHeight)
	require.NoError(t, err)
	require.Equal(t, testBlock.Height, resultBlock.Block.Height)
	require.Equal(t, testBlock.LastCommitHash, resultBlock.Block.LastCommitHash)
	cancel()
	wg.Wait()

	blockStoreMock.AssertExpectations(t)
	stateStoreMock.AssertExpectations(t)
}

func TestTxSearch(t *testing.T) {
	testHash := []byte("test")
	testTx := []byte("tx")
	testQuery := fmt.Sprintf("tx.hash='%s'", string(testHash))
	testTxResult := &abcitypes.TxResult{
		Height: 1,
		Index:  100,
		Tx:     testTx,
	}

	stateStoreMock := &statemocks.Store{}
	stateStoreMock.On("Close").Return(nil)
	blockStoreMock := &statemocks.BlockStore{}
	blockStoreMock.On("Close").Return(nil)
	txIndexerMock := &txindexmocks.TxIndexer{}
	blkIdxMock := &indexermocks.BlockIndexer{}
	txIndexerMock.On("Search", mock.Anything,
		mock.MatchedBy(func(q *query.Query) bool {
			return testQuery == strings.ReplaceAll(q.String(), " ", "")
		}), mock.Anything).
		Return([]*abcitypes.TxResult{testTxResult}, 1, nil)

	rpcConfig := config.TestRPCConfig()
	d := inspect.New(rpcConfig, blockStoreMock, stateStoreMock, txIndexerMock, blkIdxMock)
	ctx, cancel := context.WithCancel(context.Background())
	wg := &sync.WaitGroup{}
	wg.Add(1)

	startedWG := &sync.WaitGroup{}
	startedWG.Add(1)
	go func() {
		startedWG.Done()
		defer wg.Done()
		require.NoError(t, d.Run(ctx))
	}()
	// FIXME: used to induce context switch.
	// Determine more deterministic method for prompting a context switch
	startedWG.Wait()
	requireConnect(t, rpcConfig.ListenAddress)
	cli, err := httpclient.New(rpcConfig.ListenAddress + "/v1")
	require.NoError(t, err)

	page := 1
	resultTxSearch, err := cli.TxSearch(context.Background(), testQuery, false, &page, &page, "")
	require.NoError(t, err)
	require.Len(t, resultTxSearch.Txs, 1)
	require.Equal(t, types.Tx(testTx), resultTxSearch.Txs[0].Tx)

	cancel()
	wg.Wait()

	txIndexerMock.AssertExpectations(t)
	stateStoreMock.AssertExpectations(t)
	blockStoreMock.AssertExpectations(t)
}

func TestTx(t *testing.T) {
	testHash := []byte("test")
	testTx := []byte("tx")

	stateStoreMock := &statemocks.Store{}
	stateStoreMock.On("Close").Return(nil)
	blockStoreMock := &statemocks.BlockStore{}
	blockStoreMock.On("Close").Return(nil)
	blkIdxMock := &indexermocks.BlockIndexer{}
	txIndexerMock := &txindexmocks.TxIndexer{}
	txIndexerMock.On("Get", testHash).Return(&abcitypes.TxResult{
		Tx: testTx,
	}, nil)

	rpcConfig := config.TestRPCConfig()
	d := inspect.New(rpcConfig, blockStoreMock, stateStoreMock, txIndexerMock, blkIdxMock)
	ctx, cancel := context.WithCancel(context.Background())
	wg := &sync.WaitGroup{}
	wg.Add(1)

	startedWG := &sync.WaitGroup{}
	startedWG.Add(1)
	go func() {
		startedWG.Done()
		defer wg.Done()
		require.NoError(t, d.Run(ctx))
	}()
	// FIXME: used to induce context switch.
	// Determine more deterministic method for prompting a context switch
	startedWG.Wait()
	requireConnect(t, rpcConfig.ListenAddress)
	cli, err := httpclient.New(rpcConfig.ListenAddress + "/v1")
	require.NoError(t, err)

	res, err := cli.Tx(context.Background(), testHash, false)
	require.NoError(t, err)
	require.Equal(t, types.Tx(testTx), res.Tx)

	cancel()
	wg.Wait()

	txIndexerMock.AssertExpectations(t)
	stateStoreMock.AssertExpectations(t)
	blockStoreMock.AssertExpectations(t)
}

func TestConsensusParams(t *testing.T) {
	testHeight := int64(1)
	testMaxGas := int64(55)
	stateStoreMock := &statemocks.Store{}
	stateStoreMock.On("Close").Return(nil)
	blockStoreMock := &statemocks.BlockStore{}
	blockStoreMock.On("Close").Return(nil)
	blockStoreMock.On("Height").Return(testHeight)
	blockStoreMock.On("Base").Return(int64(0))
	stateStoreMock.On("LoadConsensusParams", testHeight).Return(types.ConsensusParams{
		Block: types.BlockParams{
			MaxGas: testMaxGas,
		},
	}, nil)
	txIndexerMock := &txindexmocks.TxIndexer{}
	blkIdxMock := &indexermocks.BlockIndexer{}
	rpcConfig := config.TestRPCConfig()
	d := inspect.New(rpcConfig, blockStoreMock, stateStoreMock, txIndexerMock, blkIdxMock)

	ctx, cancel := context.WithCancel(context.Background())
	wg := &sync.WaitGroup{}
	wg.Add(1)

	startedWG := &sync.WaitGroup{}
	startedWG.Add(1)
	go func() {
		startedWG.Done()
		defer wg.Done()
		require.NoError(t, d.Run(ctx))
	}()
	// FIXME: used to induce context switch.
	// Determine more deterministic method for prompting a context switch
	startedWG.Wait()
	requireConnect(t, rpcConfig.ListenAddress)
	cli, err := httpclient.New(rpcConfig.ListenAddress + "/v1")
	require.NoError(t, err)
	params, err := cli.ConsensusParams(context.Background(), &testHeight)
	require.NoError(t, err)
	require.Equal(t, params.ConsensusParams.Block.MaxGas, testMaxGas)

	cancel()
	wg.Wait()

	blockStoreMock.AssertExpectations(t)
	stateStoreMock.AssertExpectations(t)
}

func TestBlockResults(t *testing.T) {
	testHeight := int64(1)
	testGasUsed := int64(100)
	stateStoreMock := &statemocks.Store{}
	stateStoreMock.On("Close").Return(nil)
	//	cmtstate "github.com/cometbft/cometbft/api/cometbft/state/v1"
	stateStoreMock.On("LoadFinalizeBlockResponse", testHeight).Return(&abcitypes.FinalizeBlockResponse{
		TxResults: []*abcitypes.ExecTxResult{
			{
				GasUsed: testGasUsed,
			},
		},
	}, nil)
	blockStoreMock := &statemocks.BlockStore{}
	blockStoreMock.On("Close").Return(nil)
	blockStoreMock.On("Base").Return(int64(0))
	blockStoreMock.On("Height").Return(testHeight)
	txIndexerMock := &txindexmocks.TxIndexer{}
	blkIdxMock := &indexermocks.BlockIndexer{}
	rpcConfig := config.TestRPCConfig()
	d := inspect.New(rpcConfig, blockStoreMock, stateStoreMock, txIndexerMock, blkIdxMock)

	ctx, cancel := context.WithCancel(context.Background())
	wg := &sync.WaitGroup{}
	wg.Add(1)

	startedWG := &sync.WaitGroup{}
	startedWG.Add(1)
	go func() {
		startedWG.Done()
		defer wg.Done()
		require.NoError(t, d.Run(ctx))
	}()
	// FIXME: used to induce context switch.
	// Determine more deterministic method for prompting a context switch
	startedWG.Wait()
	requireConnect(t, rpcConfig.ListenAddress)
	cli, err := httpclient.New(rpcConfig.ListenAddress + "/v1")
	require.NoError(t, err)
	res, err := cli.BlockResults(context.Background(), &testHeight)
	require.NoError(t, err)
	require.Equal(t, res.TxResults[0].GasUsed, testGasUsed)

	cancel()
	wg.Wait()

	blockStoreMock.AssertExpectations(t)
	stateStoreMock.AssertExpectations(t)
}

func TestCommit(t *testing.T) {
	testHeight := int64(1)
	testRound := int32(101)
	stateStoreMock := &statemocks.Store{}
	stateStoreMock.On("Close").Return(nil)
	blockStoreMock := &statemocks.BlockStore{}
	blockStoreMock.On("Close").Return(nil)
	blockStoreMock.On("Base").Return(int64(0))
	blockStoreMock.On("Height").Return(testHeight)
	blockStoreMock.On("LoadBlockMeta", testHeight).Return(&types.BlockMeta{}, nil)
	blockStoreMock.On("LoadSeenCommit", testHeight).Return(&types.Commit{
		Height: testHeight,
		Round:  testRound,
	}, nil)
	txIndexerMock := &txindexmocks.TxIndexer{}
	blkIdxMock := &indexermocks.BlockIndexer{}
	rpcConfig := config.TestRPCConfig()
	d := inspect.New(rpcConfig, blockStoreMock, stateStoreMock, txIndexerMock, blkIdxMock)

	ctx, cancel := context.WithCancel(context.Background())
	wg := &sync.WaitGroup{}
	wg.Add(1)

	startedWG := &sync.WaitGroup{}
	startedWG.Add(1)
	go func() {
		startedWG.Done()
		defer wg.Done()
		require.NoError(t, d.Run(ctx))
	}()
	// FIXME: used to induce context switch.
	// Determine more deterministic method for prompting a context switch
	startedWG.Wait()
	requireConnect(t, rpcConfig.ListenAddress)
	cli, err := httpclient.New(rpcConfig.ListenAddress + "/v1")
	require.NoError(t, err)
	res, err := cli.Commit(context.Background(), &testHeight)
	require.NoError(t, err)
	require.NotNil(t, res)
	require.Equal(t, res.SignedHeader.Commit.Round, testRound)

	cancel()
	wg.Wait()

	blockStoreMock.AssertExpectations(t)
	stateStoreMock.AssertExpectations(t)
}

func TestBlockByHash(t *testing.T) {
	testHeight := int64(1)
	testHash := []byte("test hash")
	testBlock := new(types.Block)
	testBlock.Header.Height = testHeight
	testBlock.Header.LastCommitHash = testHash
	stateStoreMock := &statemocks.Store{}
	stateStoreMock.On("Close").Return(nil)
	blockStoreMock := &statemocks.BlockStore{}
	blockStoreMock.On("Close").Return(nil)
	blockStoreMock.On("LoadBlockByHash", testHash).Return(testBlock, &types.BlockMeta{
		BlockID: types.BlockID{
			Hash: testHash,
		},
		Header: types.Header{
			Height: testHeight,
		},
	}, nil)
	txIndexerMock := &txindexmocks.TxIndexer{}
	blkIdxMock := &indexermocks.BlockIndexer{}
	rpcConfig := config.TestRPCConfig()
	d := inspect.New(rpcConfig, blockStoreMock, stateStoreMock, txIndexerMock, blkIdxMock)

	ctx, cancel := context.WithCancel(context.Background())
	wg := &sync.WaitGroup{}
	wg.Add(1)

	startedWG := &sync.WaitGroup{}
	startedWG.Add(1)
	go func() {
		startedWG.Done()
		defer wg.Done()
		require.NoError(t, d.Run(ctx))
	}()
	// FIXME: used to induce context switch.
	// Determine more deterministic method for prompting a context switch
	startedWG.Wait()
	requireConnect(t, rpcConfig.ListenAddress)
	cli, err := httpclient.New(rpcConfig.ListenAddress + "/v1")
	require.NoError(t, err)
	res, err := cli.BlockByHash(context.Background(), testHash)
	require.NoError(t, err)
	require.NotNil(t, res)
	require.Equal(t, []byte(res.BlockID.Hash), testHash)

	cancel()
	wg.Wait()

	blockStoreMock.AssertExpectations(t)
	stateStoreMock.AssertExpectations(t)
}

func TestBlockchain(t *testing.T) {
	testHeight := int64(1)
	testBlock := new(types.Block)
	testBlockHash := []byte("test hash")
	testBlock.Header.Height = testHeight
	testBlock.Header.LastCommitHash = testBlockHash
	stateStoreMock := &statemocks.Store{}
	stateStoreMock.On("Close").Return(nil)

	blockStoreMock := &statemocks.BlockStore{}
	blockStoreMock.On("Close").Return(nil)
	blockStoreMock.On("Height").Return(testHeight)
	blockStoreMock.On("Base").Return(int64(0))
	blockStoreMock.On("LoadBlockMeta", testHeight).Return(&types.BlockMeta{
		BlockID: types.BlockID{
			Hash: testBlockHash,
		},
	})
	txIndexerMock := &txindexmocks.TxIndexer{}
	blkIdxMock := &indexermocks.BlockIndexer{}
	rpcConfig := config.TestRPCConfig()
	d := inspect.New(rpcConfig, blockStoreMock, stateStoreMock, txIndexerMock, blkIdxMock)

	ctx, cancel := context.WithCancel(context.Background())
	wg := &sync.WaitGroup{}
	wg.Add(1)

	startedWG := &sync.WaitGroup{}
	startedWG.Add(1)
	go func() {
		startedWG.Done()
		defer wg.Done()
		require.NoError(t, d.Run(ctx))
	}()
	// FIXME: used to induce context switch.
	// Determine more deterministic method for prompting a context switch
	startedWG.Wait()
	requireConnect(t, rpcConfig.ListenAddress)
	cli, err := httpclient.New(rpcConfig.ListenAddress + "/v1")
	require.NoError(t, err)
	res, err := cli.BlockchainInfo(context.Background(), 0, 100)
	require.NoError(t, err)
	require.NotNil(t, res)
	require.Equal(t, testBlockHash, []byte(res.BlockMetas[0].BlockID.Hash))

	cancel()
	wg.Wait()

	blockStoreMock.AssertExpectations(t)
	stateStoreMock.AssertExpectations(t)
}

func TestValidators(t *testing.T) {
	testHeight := int64(1)
	testVotingPower := int64(100)
	testValidators := types.ValidatorSet{
		Validators: []*types.Validator{
			{
				VotingPower: testVotingPower,
			},
		},
	}
	stateStoreMock := &statemocks.Store{}
	stateStoreMock.On("Close").Return(nil)
	stateStoreMock.On("LoadValidators", testHeight).Return(&testValidators, nil)

	blockStoreMock := &statemocks.BlockStore{}
	blockStoreMock.On("Close").Return(nil)
	blockStoreMock.On("Height").Return(testHeight)
	blockStoreMock.On("Base").Return(int64(0))
	txIndexerMock := &txindexmocks.TxIndexer{}
	blkIdxMock := &indexermocks.BlockIndexer{}
	rpcConfig := config.TestRPCConfig()
	d := inspect.New(rpcConfig, blockStoreMock, stateStoreMock, txIndexerMock, blkIdxMock)

	ctx, cancel := context.WithCancel(context.Background())
	wg := &sync.WaitGroup{}
	wg.Add(1)

	startedWG := &sync.WaitGroup{}
	startedWG.Add(1)
	go func() {
		startedWG.Done()
		defer wg.Done()
		require.NoError(t, d.Run(ctx))
	}()
	// FIXME: used to induce context switch.
	// Determine more deterministic method for prompting a context switch
	startedWG.Wait()
	requireConnect(t, rpcConfig.ListenAddress)
	cli, err := httpclient.New(rpcConfig.ListenAddress + "/v1")
	require.NoError(t, err)

	testPage := 1
	testPerPage := 100
	res, err := cli.Validators(context.Background(), &testHeight, &testPage, &testPerPage)
	require.NoError(t, err)
	require.NotNil(t, res)
	require.Equal(t, testVotingPower, res.Validators[0].VotingPower)

	cancel()
	wg.Wait()

	blockStoreMock.AssertExpectations(t)
	stateStoreMock.AssertExpectations(t)
}

func TestBlockSearch(t *testing.T) {
	testHeight := int64(1)
	testBlockHash := []byte("test hash")
	testQuery := "block.height = 1"
	stateStoreMock := &statemocks.Store{}
	stateStoreMock.On("Close").Return(nil)

	blockStoreMock := &statemocks.BlockStore{}
	blockStoreMock.On("Close").Return(nil)

	txIndexerMock := &txindexmocks.TxIndexer{}
	blkIdxMock := &indexermocks.BlockIndexer{}
	blockStoreMock.On("LoadBlock", testHeight).Return(&types.Block{
		Header: types.Header{
			Height: testHeight,
		},
	}, &types.BlockMeta{
		BlockID: types.BlockID{
			Hash: testBlockHash,
		},
	}, nil)
	blkIdxMock.On("Search", mock.Anything,
		mock.MatchedBy(func(q *query.Query) bool { return testQuery == q.String() })).
		Return([]int64{testHeight}, nil)
	rpcConfig := config.TestRPCConfig()
	d := inspect.New(rpcConfig, blockStoreMock, stateStoreMock, txIndexerMock, blkIdxMock)

	ctx, cancel := context.WithCancel(context.Background())
	wg := &sync.WaitGroup{}
	wg.Add(1)

	startedWG := &sync.WaitGroup{}
	startedWG.Add(1)
	go func() {
		startedWG.Done()
		defer wg.Done()
		require.NoError(t, d.Run(ctx))
	}()
	// FIXME: used to induce context switch.
	// Determine more deterministic method for prompting a context switch
	startedWG.Wait()
	requireConnect(t, rpcConfig.ListenAddress)
	cli, err := httpclient.New(rpcConfig.ListenAddress + "/v1")
	require.NoError(t, err)

	testPage := 1
	testPerPage := 100
	testOrderBy := "desc"
	res, err := cli.BlockSearch(context.Background(), testQuery, &testPage, &testPerPage, testOrderBy)
	require.NoError(t, err)
	require.NotNil(t, res)
	require.Equal(t, testBlockHash, []byte(res.Blocks[0].BlockID.Hash))

	cancel()
	wg.Wait()

	blockStoreMock.AssertExpectations(t)
	stateStoreMock.AssertExpectations(t)
}

func requireConnect(tb testing.TB, addr string) {
	tb.Helper()
	retries := 20
	parts := strings.SplitN(addr, "://", 2)
	if len(parts) != 2 {
		tb.Fatalf("malformed address to dial: %s", addr)
	}
	var err error
	for i := 0; i < retries; i++ {
		var conn net.Conn
		conn, err = net.Dial(parts[0], parts[1])
		if err == nil {
			conn.Close()
			return
		}
		// FIXME attempt to yield and let the other goroutine continue execution.
		time.Sleep(time.Microsecond * 100)
	}
	tb.Fatalf("unable to connect to server %s after %d tries: %s", addr, retries, err)
}
