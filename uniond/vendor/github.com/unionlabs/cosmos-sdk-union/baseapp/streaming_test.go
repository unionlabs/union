package baseapp_test

import (
	"context"
	"fmt"
	"testing"

	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
	tmproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/schema/appdata"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/baseapp"
	baseapptestutil "github.com/cosmos/cosmos-sdk/baseapp/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var _ storetypes.ABCIListener = (*MockABCIListener)(nil)

type MockABCIListener struct {
	name      string
	ChangeSet []*storetypes.StoreKVPair
}

func NewMockABCIListener(name string) MockABCIListener {
	return MockABCIListener{
		name:      name,
		ChangeSet: make([]*storetypes.StoreKVPair, 0),
	}
}

func (m MockABCIListener) ListenFinalizeBlock(_ context.Context, _ abci.FinalizeBlockRequest, _ abci.FinalizeBlockResponse) error {
	return nil
}

func (m *MockABCIListener) ListenCommit(_ context.Context, _ abci.CommitResponse, cs []*storetypes.StoreKVPair) error {
	m.ChangeSet = cs
	return nil
}

var distKey1 = storetypes.NewKVStoreKey("distKey1")

func TestABCI_MultiListener_StateChanges(t *testing.T) {
	anteKey := []byte("ante-key")
	anteOpt := func(bapp *baseapp.BaseApp) { bapp.SetAnteHandler(anteHandlerTxTest(t, capKey1, anteKey)) }
	distOpt := func(bapp *baseapp.BaseApp) { bapp.MountStores(distKey1) }
	mockListener1 := NewMockABCIListener("lis_1")
	mockListener2 := NewMockABCIListener("lis_2")
	streamingManager := storetypes.StreamingManager{ABCIListeners: []storetypes.ABCIListener{&mockListener1, &mockListener2}}
	streamingManagerOpt := func(bapp *baseapp.BaseApp) { bapp.SetStreamingManager(streamingManager) }
	addListenerOpt := func(bapp *baseapp.BaseApp) { bapp.CommitMultiStore().AddListeners([]storetypes.StoreKey{distKey1}) }
	suite := NewBaseAppSuite(t, anteOpt, distOpt, streamingManagerOpt, addListenerOpt)

	_, err := suite.baseApp.InitChain(
		&abci.InitChainRequest{
			ConsensusParams: &tmproto.ConsensusParams{},
		},
	)
	require.NoError(t, err)
	deliverKey := []byte("deliver-key")
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), CounterServerImpl{t, capKey1, deliverKey})

	nBlocks := 3
	txPerHeight := 5

	for blockN := 0; blockN < nBlocks; blockN++ {
		txs := [][]byte{}

		var expectedChangeSet []*storetypes.StoreKVPair

		// create final block context state
		_, err := suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{Height: int64(blockN) + 1, Txs: txs})
		require.NoError(t, err)

		for i := 0; i < txPerHeight; i++ {
			counter := int64(blockN*txPerHeight + i)
			tx := newTxCounter(t, suite.txConfig, suite.ac, counter, counter)

			txBytes, err := suite.txConfig.TxEncoder()(tx)
			require.NoError(t, err)

			sKey := []byte(fmt.Sprintf("distKey%d", i))
			sVal := []byte(fmt.Sprintf("distVal%d", i))
			store := getFinalizeBlockStateCtx(suite.baseApp).KVStore(distKey1)
			store.Set(sKey, sVal)

			expectedChangeSet = append(expectedChangeSet, &storetypes.StoreKVPair{
				StoreKey: distKey1.Name(),
				Delete:   false,
				Key:      sKey,
				Value:    sVal,
			})

			txs = append(txs, txBytes)
		}

		res, err := suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{Height: int64(blockN) + 1, Txs: txs})
		require.NoError(t, err)
		for _, tx := range res.TxResults {
			events := tx.GetEvents()
			require.Len(t, events, 3, "should contain ante handler, message type and counter events respectively")
			// require.Equal(t, sdk.MarkEventsToIndex(counterEvent("ante_handler", counter).ToABCIEvents(), map[string]struct{}{})[0], events[0], "ante handler event")
			// require.Equal(t, sdk.MarkEventsToIndex(counterEvent(sdk.EventTypeMessage, counter).ToABCIEvents(), map[string]struct{}{})[0], events[2], "msg handler update counter event")
		}

		_, err = suite.baseApp.Commit()
		require.NoError(t, err)
		require.Equal(t, expectedChangeSet, mockListener1.ChangeSet, "should contain the same changeSet")
		require.Equal(t, expectedChangeSet, mockListener2.ChangeSet, "should contain the same changeSet")
	}
}

func Test_Ctx_with_StreamingManager(t *testing.T) {
	mockListener1 := NewMockABCIListener("lis_1")
	mockListener2 := NewMockABCIListener("lis_2")
	listeners := []storetypes.ABCIListener{&mockListener1, &mockListener2}
	streamingManager := storetypes.StreamingManager{ABCIListeners: listeners, StopNodeOnErr: true}
	streamingManagerOpt := func(bapp *baseapp.BaseApp) { bapp.SetStreamingManager(streamingManager) }
	addListenerOpt := func(bapp *baseapp.BaseApp) { bapp.CommitMultiStore().AddListeners([]storetypes.StoreKey{distKey1}) }
	suite := NewBaseAppSuite(t, streamingManagerOpt, addListenerOpt)

	_, err := suite.baseApp.InitChain(&abci.InitChainRequest{
		ConsensusParams: &tmproto.ConsensusParams{},
	})
	require.NoError(t, err)

	ctx := getFinalizeBlockStateCtx(suite.baseApp)
	sm := ctx.StreamingManager()
	require.NotNil(t, sm, fmt.Sprintf("nil StreamingManager: %v", sm))
	require.Equal(t, listeners, sm.ABCIListeners, fmt.Sprintf("should contain same listeners: %v", listeners))
	require.Equal(t, true, sm.StopNodeOnErr, "should contain StopNodeOnErr = true")

	nBlocks := 2

	for blockN := 0; blockN < nBlocks; blockN++ {

		_, err = suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{Height: int64(blockN) + 1})
		require.NoError(t, err)
		ctx := getFinalizeBlockStateCtx(suite.baseApp)
		sm := ctx.StreamingManager()
		require.NotNil(t, sm, fmt.Sprintf("nil StreamingManager: %v", sm))
		require.Equal(t, listeners, sm.ABCIListeners, fmt.Sprintf("should contain same listeners: %v", listeners))
		require.Equal(t, true, sm.StopNodeOnErr, "should contain StopNodeOnErr = true")

		_, err = suite.baseApp.Commit()
		require.NoError(t, err)
	}
}

type mockAppDataListener struct {
	appdata.Listener

	startBlockData []appdata.StartBlockData
	txData         []appdata.TxData
	eventData      []appdata.EventData
	kvPairData     []appdata.KVPairData
	commitData     []appdata.CommitData
}

func newMockAppDataListener() *mockAppDataListener {
	listener := &mockAppDataListener{}

	// Initialize the Listener with custom behavior to store data
	listener.Listener = appdata.Listener{
		StartBlock: func(data appdata.StartBlockData) error {
			listener.startBlockData = append(listener.startBlockData, data) // Store StartBlockData
			return nil
		},
		OnTx: func(data appdata.TxData) error {
			listener.txData = append(listener.txData, data) // Store TxData
			return nil
		},
		OnEvent: func(data appdata.EventData) error {
			listener.eventData = append(listener.eventData, data) // Store EventData
			return nil
		},
		OnKVPair: func(data appdata.KVPairData) error {
			listener.kvPairData = append(listener.kvPairData, data) // Store KVPairData
			return nil
		},
		Commit: func(data appdata.CommitData) (func() error, error) {
			listener.commitData = append(listener.commitData, data) // Store CommitData
			return nil, nil
		},
	}

	return listener
}

func TestAppDataListener(t *testing.T) {
	anteKey := []byte("ante-key")
	anteOpt := func(bapp *baseapp.BaseApp) { bapp.SetAnteHandler(anteHandlerTxTest(t, capKey1, anteKey)) }
	distOpt := func(bapp *baseapp.BaseApp) { bapp.MountStores(distKey1) }
	mockListener := newMockAppDataListener()
	streamingManager := storetypes.StreamingManager{ABCIListeners: []storetypes.ABCIListener{baseapp.NewListenerWrapper(mockListener.Listener)}}
	streamingManagerOpt := func(bapp *baseapp.BaseApp) { bapp.SetStreamingManager(streamingManager) }
	addListenerOpt := func(bapp *baseapp.BaseApp) { bapp.CommitMultiStore().AddListeners([]storetypes.StoreKey{distKey1}) }

	// for event tests
	baseappOpts := func(app *baseapp.BaseApp) {
		app.SetPreBlocker(func(ctx sdk.Context, req *abci.FinalizeBlockRequest) error {
			ctx.EventManager().EmitEvent(sdk.NewEvent("pre-block"))
			return nil
		})
		app.SetBeginBlocker(func(_ sdk.Context) (sdk.BeginBlock, error) {
			return sdk.BeginBlock{
				Events: []abci.Event{
					{Type: "begin-block"},
				},
			}, nil
		})
		app.SetEndBlocker(func(_ sdk.Context) (sdk.EndBlock, error) {
			return sdk.EndBlock{
				Events: []abci.Event{
					{Type: "end-block"},
				},
			}, nil
		})
	}

	suite := NewBaseAppSuite(t, anteOpt, distOpt, streamingManagerOpt, addListenerOpt, baseappOpts)

	_, err := suite.baseApp.InitChain(
		&abci.InitChainRequest{
			ConsensusParams: &tmproto.ConsensusParams{},
		},
	)
	require.NoError(t, err)
	deliverKey := []byte("deliver-key")
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), CounterServerImpl{t, capKey1, deliverKey})

	txCount := 5
	txs := make([][]byte, txCount)

	for i := 0; i < txCount; i++ {
		tx := newTxCounter(t, suite.txConfig, suite.ac, int64(i), int64(i))

		txBytes, err := suite.txConfig.TxEncoder()(tx)
		require.NoError(t, err)

		sKey := []byte(fmt.Sprintf("distKey%d", i))
		sVal := []byte(fmt.Sprintf("distVal%d", i))
		store := getFinalizeBlockStateCtx(suite.baseApp).KVStore(distKey1)
		store.Set(sKey, sVal)

		txs[i] = txBytes
	}

	_, err = suite.baseApp.FinalizeBlock(&abci.FinalizeBlockRequest{Height: 1, Txs: txs})
	require.NoError(t, err)
	_, err = suite.baseApp.Commit()
	require.NoError(t, err)

	// StartBlockData
	require.Len(t, mockListener.startBlockData, 1)
	require.Equal(t, uint64(1), mockListener.startBlockData[0].Height)
	// TxData
	txData := mockListener.txData
	require.Len(t, txData, len(txs))
	for i := 0; i < txCount; i++ {
		require.Equal(t, int32(i), txData[i].TxIndex)
		txBytes, err := txData[i].Bytes()
		require.NoError(t, err)
		require.Equal(t, txs[i], txBytes)
	}
	// KVPairData
	require.Len(t, mockListener.kvPairData, 1)
	updates := mockListener.kvPairData[0].Updates
	for i := 0; i < txCount; i++ {
		require.Equal(t, []byte(distKey1.Name()), updates[i].Actor)
		require.Len(t, updates[i].StateChanges, 1)
		sKey := []byte(fmt.Sprintf("distKey%d", i))
		sVal := []byte(fmt.Sprintf("distVal%d", i))
		require.Equal(t, sKey, updates[i].StateChanges[0].Key)
		require.Equal(t, sVal, updates[i].StateChanges[0].Value)
	}
	// CommitData
	require.Len(t, mockListener.commitData, 1)
	// EventData
	require.Len(t, mockListener.eventData, 1)
	events := mockListener.eventData[0].Events
	require.Len(t, events, 3+txCount*3)

	for i := 0; i < 3; i++ {
		require.Equal(t, int32(0), events[i].TxIndex)
		require.Equal(t, int32(0), events[i].MsgIndex)
		require.Equal(t, int32(1), events[i].EventIndex)
		attrs, err := events[i].Attributes()
		require.NoError(t, err)
		require.Len(t, attrs, 2)
		switch i {
		case 0:
			require.Equal(t, appdata.PreBlockStage, events[i].BlockStage)
			require.Equal(t, "pre-block", events[i].Type)
		case 1:
			require.Equal(t, appdata.BeginBlockStage, events[i].BlockStage)
			require.Equal(t, "begin-block", events[i].Type)
		case 2:
			require.Equal(t, appdata.EndBlockStage, events[i].BlockStage)
			require.Equal(t, "end-block", events[i].Type)
		}
	}

	for i := 3; i < 3+txCount*3; i++ {
		require.Equal(t, appdata.TxProcessingStage, events[i].BlockStage)
		require.Equal(t, int32(i/3), events[i].TxIndex)
		switch i % 3 {
		case 0:
			require.Equal(t, "ante_handler", events[i].Type)
			require.Equal(t, int32(0), events[i].MsgIndex)
			require.Equal(t, int32(0), events[i].EventIndex)
			attrs, err := events[i].Attributes()
			require.NoError(t, err)
			require.Len(t, attrs, 2)
		case 1:
			require.Equal(t, "message", events[i].Type)
			require.Equal(t, int32(1), events[i].MsgIndex)
			require.Equal(t, int32(1), events[i].EventIndex)
			attrs, err := events[i].Attributes()
			require.NoError(t, err)
			require.Len(t, attrs, 5)
		case 2:
			require.Equal(t, "message", events[i].Type)
			require.Equal(t, int32(1), events[i].MsgIndex)
			require.Equal(t, int32(2), events[i].EventIndex)
			attrs, err := events[i].Attributes()
			require.NoError(t, err)
			require.Len(t, attrs, 4)
		}
	}
}
