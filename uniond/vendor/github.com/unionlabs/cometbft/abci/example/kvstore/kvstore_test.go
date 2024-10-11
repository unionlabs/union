package kvstore

import (
	"bytes"
	"context"
	"fmt"
	"sort"
	"testing"

	"github.com/stretchr/testify/require"

	abcicli "github.com/cometbft/cometbft/abci/client"
	abciserver "github.com/cometbft/cometbft/abci/server"
	"github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/libs/log"
)

const (
	testKey   = "abc"
	testValue = "def"
)

func TestKVStoreKV(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	kvstore := NewInMemoryApplication()
	tx := []byte(testKey + ":" + testValue)
	testKVStore(ctx, t, kvstore, tx)
	tx = []byte(testKey + "=" + testValue)
	testKVStore(ctx, t, kvstore, tx)
}

func testKVStore(ctx context.Context, t *testing.T, app types.Application, tx []byte) {
	t.Helper()

	value := "def"
	key := "abc"
	checkTxResp, err := app.CheckTx(ctx, &types.CheckTxRequest{Tx: tx, Type: types.CHECK_TX_TYPE_CHECK})
	require.NoError(t, err)
	require.Equal(t, uint32(0), checkTxResp.Code)

	ppResp, err := app.PrepareProposal(ctx, &types.PrepareProposalRequest{Txs: [][]byte{tx}})
	require.NoError(t, err)
	require.Len(t, ppResp.Txs, 1)
	req := &types.FinalizeBlockRequest{Height: 1, Txs: ppResp.Txs}
	ar, err := app.FinalizeBlock(ctx, req)
	require.NoError(t, err)
	require.Len(t, ar.TxResults, 1)
	require.False(t, ar.TxResults[0].IsErr())
	// commit
	_, err = app.Commit(ctx, &types.CommitRequest{})
	require.NoError(t, err)

	info, err := app.Info(ctx, &types.InfoRequest{})
	require.NoError(t, err)
	require.NotZero(t, info.LastBlockHeight)

	// make sure query is fine
	resQuery, err := app.Query(ctx, &types.QueryRequest{
		Path: "/store",
		Data: []byte(key),
	})
	require.NoError(t, err)
	require.Equal(t, CodeTypeOK, resQuery.Code)
	require.Equal(t, key, string(resQuery.Key))
	require.Equal(t, value, string(resQuery.Value))
	require.EqualValues(t, info.LastBlockHeight, resQuery.Height)

	// make sure proof is fine
	resQuery, err = app.Query(ctx, &types.QueryRequest{
		Path:  "/store",
		Data:  []byte(key),
		Prove: true,
	})
	require.NoError(t, err)
	require.EqualValues(t, CodeTypeOK, resQuery.Code)
	require.Equal(t, key, string(resQuery.Key))
	require.Equal(t, value, string(resQuery.Value))
	require.EqualValues(t, info.LastBlockHeight, resQuery.Height)
}

func TestPersistentKVStoreEmptyTX(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	kvstore := NewPersistentApplication(t.TempDir())
	tx := []byte("")
	reqCheck := types.CheckTxRequest{Tx: tx, Type: types.CHECK_TX_TYPE_CHECK}
	resCheck, err := kvstore.CheckTx(ctx, &reqCheck)
	require.NoError(t, err)
	require.Equal(t, CodeTypeInvalidTxFormat, resCheck.Code)

	txs := make([][]byte, 0, 4)
	txs = append(txs, []byte("key=value"), []byte("key:val"), []byte(""), []byte("kee=value"))
	reqPrepare := types.PrepareProposalRequest{Txs: txs, MaxTxBytes: 10 * 1024}
	resPrepare, err := kvstore.PrepareProposal(ctx, &reqPrepare)
	require.NoError(t, err)
	require.Len(t, resPrepare.Txs, len(reqPrepare.Txs)-1, "Empty transaction not properly removed")
}

func TestPersistentKVStoreKV(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	kvstore := NewPersistentApplication(t.TempDir())
	key := testKey
	value := testValue
	testKVStore(ctx, t, kvstore, NewTx(key, value))
}

func TestPersistentKVStoreInfo(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	kvstore := NewPersistentApplication(t.TempDir())
	require.NoError(t, InitKVStore(ctx, kvstore))
	height := int64(0)

	resInfo, err := kvstore.Info(ctx, &types.InfoRequest{})
	require.NoError(t, err)
	if resInfo.LastBlockHeight != height {
		t.Fatalf("expected height of %d, got %d", height, resInfo.LastBlockHeight)
	}

	// make and apply block
	height = int64(1)
	hash := []byte("foo")
	if _, err := kvstore.FinalizeBlock(ctx, &types.FinalizeBlockRequest{Hash: hash, Height: height}); err != nil {
		t.Fatal(err)
	}

	_, err = kvstore.Commit(ctx, &types.CommitRequest{})
	require.NoError(t, err)

	resInfo, err = kvstore.Info(ctx, &types.InfoRequest{})
	require.NoError(t, err)
	require.Equal(t, height, resInfo.LastBlockHeight)
}

// add a validator, remove a validator, update a validator.
func TestValUpdates(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	kvstore := NewInMemoryApplication()

	// init with some validators
	total := 10
	nInit := 5
	vals := RandVals(total)
	// initialize with the first nInit
	_, err := kvstore.InitChain(ctx, &types.InitChainRequest{
		Validators: vals[:nInit],
	})
	require.NoError(t, err)

	vals1, vals2 := vals[:nInit], kvstore.getValidators()
	valsEqual(t, vals1, vals2)

	var v1, v2, v3 types.ValidatorUpdate

	// add some validators
	v1, v2 = vals[nInit], vals[nInit+1]
	diff := []types.ValidatorUpdate{v1, v2}
	tx1 := MakeValSetChangeTx(v1)
	tx2 := MakeValSetChangeTx(v2)

	makeApplyBlock(ctx, t, kvstore, 1, diff, tx1, tx2)

	vals1, vals2 = vals[:nInit+2], kvstore.getValidators()
	valsEqual(t, vals1, vals2)

	// remove some validators
	v1, v2, v3 = vals[nInit-2], vals[nInit-1], vals[nInit]
	v1.Power = 0
	v2.Power = 0
	v3.Power = 0
	diff = []types.ValidatorUpdate{v1, v2, v3}
	tx1 = MakeValSetChangeTx(v1)
	tx2 = MakeValSetChangeTx(v2)
	tx3 := MakeValSetChangeTx(v3)

	makeApplyBlock(ctx, t, kvstore, 2, diff, tx1, tx2, tx3)

	vals1 = append(vals[:nInit-2], vals[nInit+1])
	vals2 = kvstore.getValidators()
	valsEqual(t, vals1, vals2)

	// update some validators
	v1 = vals[0]
	if v1.Power == 5 {
		v1.Power = 6
	} else {
		v1.Power = 5
	}
	diff = []types.ValidatorUpdate{v1}
	tx1 = MakeValSetChangeTx(v1)

	makeApplyBlock(ctx, t, kvstore, 3, diff, tx1)

	vals1 = append([]types.ValidatorUpdate{v1}, vals1[1:]...)
	vals2 = kvstore.getValidators()
	valsEqual(t, vals1, vals2)
}

func TestCheckTx(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	kvstore := NewInMemoryApplication()

	val := RandVal()
	val.Power = 10

	testCases := []struct {
		expCode uint32
		tx      []byte
	}{
		{CodeTypeOK, NewTx("hello", "world")},
		{CodeTypeInvalidTxFormat, []byte("hello")},
		{CodeTypeOK, []byte("space:jam")},
		{CodeTypeInvalidTxFormat, []byte("=hello")},
		{CodeTypeInvalidTxFormat, []byte("hello=")},
		{CodeTypeOK, []byte("a=b")},
		{CodeTypeInvalidTxFormat, []byte("val=hello")},
		{CodeTypeInvalidTxFormat, []byte("val=hi!5")},
		{CodeTypeOK, MakeValSetChangeTx(val)},
	}

	for idx, tc := range testCases {
		resp, err := kvstore.CheckTx(ctx, &types.CheckTxRequest{
			Tx:   tc.tx,
			Type: types.CHECK_TX_TYPE_CHECK,
		})
		require.NoError(t, err, idx)
		fmt.Println(string(tc.tx))
		require.Equal(t, tc.expCode, resp.Code, idx)
	}
}

func TestClientServer(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	// set up socket app
	kvstore := NewInMemoryApplication()
	client, err := makeClientServer(t, kvstore, "kvstore-socket", "socket")
	require.NoError(t, err)
	runClientTests(ctx, t, client)

	// set up grpc app
	kvstore = NewInMemoryApplication()
	gclient, err := makeClientServer(t, kvstore, t.TempDir(), "grpc")
	require.NoError(t, err)
	runClientTests(ctx, t, gclient)
}

func makeApplyBlock(
	ctx context.Context,
	t *testing.T,
	kvstore types.Application,
	heightInt int,
	diff []types.ValidatorUpdate,
	txs ...[]byte,
) {
	t.Helper()
	// make and apply block
	height := int64(heightInt)
	hash := []byte("foo")
	resFinalizeBlock, err := kvstore.FinalizeBlock(ctx, &types.FinalizeBlockRequest{
		Hash:   hash,
		Height: height,
		Txs:    txs,
	})
	require.NoError(t, err)

	_, err = kvstore.Commit(ctx, &types.CommitRequest{})
	require.NoError(t, err)

	valsEqual(t, diff, resFinalizeBlock.ValidatorUpdates)
}

// order doesn't matter.
func valsEqual(t *testing.T, vals1, vals2 []types.ValidatorUpdate) {
	t.Helper()
	if len(vals1) != len(vals2) {
		t.Fatalf("vals dont match in len. got %d, expected %d", len(vals2), len(vals1))
	}
	sort.Sort(types.ValidatorUpdates(vals1))
	sort.Sort(types.ValidatorUpdates(vals2))
	for i, v1 := range vals1 {
		v2 := vals2[i]
		if v1.PubKeyType != v2.PubKeyType ||
			!bytes.Equal(v1.PubKeyBytes, v2.PubKeyBytes) ||
			v1.Power != v2.Power {
			t.Fatalf("vals dont match at index %d. got %s/%X/%d , expected %s/%X/%d", i,
				v2.PubKeyType, v2.PubKeyBytes, v2.Power, v1.PubKeyType, v1.PubKeyBytes, v1.Power)
		}
	}
}

func makeClientServer(t *testing.T, app types.Application, name, transport string) (abcicli.Client, error) {
	t.Helper()
	// Start the listener
	addr := fmt.Sprintf("unix://%s.sock", name)
	logger := log.TestingLogger()

	server, err := abciserver.NewServer(addr, transport, app)
	require.NoError(t, err)
	server.SetLogger(logger.With("module", "abci-server"))
	if err := server.Start(); err != nil {
		return nil, err
	}

	t.Cleanup(func() {
		if err := server.Stop(); err != nil {
			t.Error(err)
		}
	})

	// Connect to the client
	client, err := abcicli.NewClient(addr, transport, false)
	require.NoError(t, err)
	client.SetLogger(logger.With("module", "abci-client"))
	if err := client.Start(); err != nil {
		return nil, err
	}

	t.Cleanup(func() {
		if err := client.Stop(); err != nil {
			t.Error(err)
		}
	})

	return client, nil
}

func runClientTests(ctx context.Context, t *testing.T, client abcicli.Client) {
	t.Helper()
	// run some tests....
	tx := []byte(testKey + ":" + testValue)
	testKVStore(ctx, t, client, tx)
	tx = []byte(testKey + "=" + testValue)
	testKVStore(ctx, t, client, tx)
}

func TestTxGeneration(t *testing.T) {
	require.Len(t, NewRandomTx(20), 20)
	require.Len(t, NewRandomTxs(10), 10)
}
