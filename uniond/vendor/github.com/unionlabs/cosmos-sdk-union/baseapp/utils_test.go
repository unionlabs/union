package baseapp_test

import (
	"bytes"
	"context"
	"encoding/binary"
	"encoding/json"
	"errors"
	"fmt"
	"net/url"
	"reflect"
	"strconv"
	"testing"
	"unsafe"

	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	"github.com/stretchr/testify/require"

	runtimev1alpha1 "cosmossdk.io/api/cosmos/app/runtime/v1alpha1"
	appv1alpha1 "cosmossdk.io/api/cosmos/app/v1alpha1"
	"cosmossdk.io/core/address"
	"cosmossdk.io/core/server"
	corestore "cosmossdk.io/core/store"
	"cosmossdk.io/depinject"
	"cosmossdk.io/depinject/appconfig"
	errorsmod "cosmossdk.io/errors"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/baseapp"
	baseapptestutil "github.com/cosmos/cosmos-sdk/baseapp/testutil"
	"github.com/cosmos/cosmos-sdk/client"
	addresscodec "github.com/cosmos/cosmos-sdk/codec/address"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/mempool"
	signingtypes "github.com/cosmos/cosmos-sdk/types/tx/signing"
	_ "github.com/cosmos/cosmos-sdk/x/auth"
	"github.com/cosmos/cosmos-sdk/x/auth/signing"
	_ "github.com/cosmos/cosmos-sdk/x/auth/tx/config"
)

var ParamStoreKey = []byte("paramstore")

func makeMinimalConfig() depinject.Config {
	var (
		mempoolOpt            = baseapp.SetMempool(mempool.NewSenderNonceMempool())
		addressCodec          = func() address.Codec { return addresscodec.NewBech32Codec("cosmos") }
		validatorAddressCodec = func() address.ValidatorAddressCodec { return addresscodec.NewBech32Codec("cosmosvaloper") }
		consensusAddressCodec = func() address.ConsensusAddressCodec { return addresscodec.NewBech32Codec("cosmosvalcons") }
	)

	return depinject.Configs(
		depinject.Supply(mempoolOpt, addressCodec, validatorAddressCodec, consensusAddressCodec),
		appconfig.Compose(&appv1alpha1.Config{
			Modules: []*appv1alpha1.ModuleConfig{
				{
					Name: "runtime",
					Config: appconfig.WrapAny(&runtimev1alpha1.Module{
						AppName: "BaseAppApp",
					}),
				},
			},
		}))
}

type MsgKeyValueImpl struct{}

func (m MsgKeyValueImpl) Set(ctx context.Context, msg *baseapptestutil.MsgKeyValue) (*baseapptestutil.MsgCreateKeyValueResponse, error) {
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	sdkCtx.KVStore(capKey2).Set(msg.Key, msg.Value)
	return &baseapptestutil.MsgCreateKeyValueResponse{}, nil
}

type CounterServerImplGasMeterOnly struct {
	gas uint64
}

func (m CounterServerImplGasMeterOnly) IncrementCounter(ctx context.Context, msg *baseapptestutil.MsgCounter) (*baseapptestutil.MsgCreateCounterResponse, error) {
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	gas := m.gas

	// if no gas is provided, use the counter as gas. This is useful for testing
	if gas == 0 {
		gas = uint64(msg.Counter)
	}

	sdkCtx.GasMeter().ConsumeGas(gas, "test")
	return &baseapptestutil.MsgCreateCounterResponse{}, nil
}

type NoopCounterServerImpl struct{}

func (m NoopCounterServerImpl) IncrementCounter(
	_ context.Context,
	_ *baseapptestutil.MsgCounter,
) (*baseapptestutil.MsgCreateCounterResponse, error) {
	return &baseapptestutil.MsgCreateCounterResponse{}, nil
}

type CounterServerImpl struct {
	t          *testing.T
	capKey     storetypes.StoreKey
	deliverKey []byte
}

func (m CounterServerImpl) IncrementCounter(ctx context.Context, msg *baseapptestutil.MsgCounter) (*baseapptestutil.MsgCreateCounterResponse, error) {
	return incrementCounter(ctx, m.t, m.capKey, m.deliverKey, msg)
}

type Counter2ServerImpl struct {
	t          *testing.T
	capKey     storetypes.StoreKey
	deliverKey []byte
}

func (m Counter2ServerImpl) IncrementCounter(ctx context.Context, msg *baseapptestutil.MsgCounter2) (*baseapptestutil.MsgCreateCounterResponse, error) {
	return incrementCounter(ctx, m.t, m.capKey, m.deliverKey, msg)
}

func incrementCounter(ctx context.Context,
	t *testing.T,
	capKey storetypes.StoreKey,
	deliverKey []byte,
	msg sdk.Msg,
) (*baseapptestutil.MsgCreateCounterResponse, error) {
	t.Helper()
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	store := sdkCtx.KVStore(capKey)

	sdkCtx.GasMeter().ConsumeGas(5, "test")

	var msgCount int64

	switch m := msg.(type) {
	case *baseapptestutil.MsgCounter:
		if m.FailOnHandler {
			return nil, errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "message handler failure")
		}
		msgCount = m.Counter
	case *baseapptestutil.MsgCounter2:
		if m.FailOnHandler {
			return nil, errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "message handler failure")
		}
		msgCount = m.Counter
	}

	sdkCtx.EventManager().EmitEvents(
		counterEvent(sdk.EventTypeMessage, msgCount),
	)

	_, err := incrementingCounter(t, store, deliverKey, msgCount)
	if err != nil {
		return nil, err
	}

	return &baseapptestutil.MsgCreateCounterResponse{}, nil
}

func counterEvent(evType string, msgCount int64) sdk.Events {
	return sdk.Events{
		sdk.NewEvent(
			evType,
			sdk.NewAttribute("update_counter", fmt.Sprintf("%d", msgCount)),
		),
	}
}

func anteHandlerTxTest(t *testing.T, capKey storetypes.StoreKey, storeKey []byte) sdk.AnteHandler {
	t.Helper()
	return func(ctx sdk.Context, tx sdk.Tx, simulate bool) (sdk.Context, error) {
		store := ctx.KVStore(capKey)
		counter, failOnAnte := parseTxMemo(t, tx)

		if failOnAnte {
			return ctx, errorsmod.Wrap(sdkerrors.ErrUnauthorized, "ante handler failure")
		}

		_, err := incrementingCounter(t, store, storeKey, counter)
		if err != nil {
			return ctx, err
		}

		ctx.EventManager().EmitEvents(
			counterEvent("ante_handler", counter),
		)

		ctx = ctx.WithPriority(testTxPriority)
		return ctx, nil
	}
}

func incrementingCounter(t *testing.T, store storetypes.KVStore, counterKey []byte, counter int64) (*sdk.Result, error) {
	t.Helper()
	storedCounter := getIntFromStore(t, store, counterKey)
	require.Equal(t, storedCounter, counter)
	setIntOnStore(store, counterKey, counter+1)
	return &sdk.Result{}, nil
}

func setIntOnStore(store storetypes.KVStore, key []byte, i int64) {
	bz := make([]byte, 8)
	n := binary.PutVarint(bz, i)
	store.Set(key, bz[:n])
}

type paramStore struct {
	db corestore.KVStoreWithBatch
}

var _ baseapp.ParamStore = (*paramStore)(nil)

func (ps paramStore) Set(_ context.Context, value cmtproto.ConsensusParams) error {
	bz, err := json.Marshal(value)
	if err != nil {
		return err
	}

	return ps.db.Set(ParamStoreKey, bz)
}

func (ps paramStore) Has(_ context.Context) (bool, error) {
	return ps.db.Has(ParamStoreKey)
}

func (ps paramStore) Get(_ context.Context) (cmtproto.ConsensusParams, error) {
	bz, err := ps.db.Get(ParamStoreKey)
	if err != nil {
		return cmtproto.ConsensusParams{}, err
	}

	if len(bz) == 0 {
		return cmtproto.ConsensusParams{}, errors.New("params not found")
	}

	var params cmtproto.ConsensusParams
	if err := json.Unmarshal(bz, &params); err != nil {
		return cmtproto.ConsensusParams{}, err
	}

	return params, nil
}

func setTxSignature(t *testing.T, builder client.TxBuilder, nonce uint64) {
	t.Helper()
	privKey := secp256k1.GenPrivKeyFromSecret([]byte("test"))
	pubKey := privKey.PubKey()
	err := builder.SetSignatures(
		signingtypes.SignatureV2{
			PubKey:   pubKey,
			Sequence: nonce,
			Data:     &signingtypes.SingleSignatureData{},
		},
	)
	require.NoError(t, err)
}

func testLoadVersionHelper(t *testing.T, app *baseapp.BaseApp, expectedHeight int64, expectedID storetypes.CommitID) {
	t.Helper()
	lastHeight := app.LastBlockHeight()
	lastID := app.LastCommitID()
	require.Equal(t, expectedHeight, lastHeight)
	require.Equal(t, expectedID, lastID)
}

func getCheckStateCtx(app *baseapp.BaseApp) sdk.Context {
	v := reflect.ValueOf(app).Elem()
	f := v.FieldByName("checkState")
	rf := reflect.NewAt(f.Type(), unsafe.Pointer(f.UnsafeAddr())).Elem()
	return rf.MethodByName("Context").Call(nil)[0].Interface().(sdk.Context)
}

func getFinalizeBlockStateCtx(app *baseapp.BaseApp) sdk.Context {
	v := reflect.ValueOf(app).Elem()
	f := v.FieldByName("finalizeBlockState")
	rf := reflect.NewAt(f.Type(), unsafe.Pointer(f.UnsafeAddr())).Elem()
	return rf.MethodByName("Context").Call(nil)[0].Interface().(sdk.Context)
}

func parseTxMemo(t *testing.T, tx sdk.Tx) (counter int64, failOnAnte bool) {
	t.Helper()
	txWithMemo, ok := tx.(sdk.TxWithMemo)
	require.True(t, ok)

	memo := txWithMemo.GetMemo()
	vals, err := url.ParseQuery(memo)
	require.NoError(t, err)

	counter, err = strconv.ParseInt(vals.Get("counter"), 10, 64)
	require.NoError(t, err)

	failOnAnte = vals.Get("failOnAnte") == "true"
	return counter, failOnAnte
}

func newTxCounter(t *testing.T, cfg client.TxConfig, ac address.Codec, counter int64, msgCounters ...int64) signing.Tx {
	t.Helper()
	_, _, addr := testdata.KeyTestPubAddr()
	addrStr, err := ac.BytesToString(addr)
	require.NoError(t, err)
	msgs := make([]sdk.Msg, 0, len(msgCounters))
	for _, c := range msgCounters {
		msg := &baseapptestutil.MsgCounter{Counter: c, FailOnHandler: false, Signer: addrStr}
		msgs = append(msgs, msg)
	}

	builder := cfg.NewTxBuilder()
	err = builder.SetMsgs(msgs...)
	require.NoError(t, err)
	builder.SetMemo("counter=" + strconv.FormatInt(counter, 10) + "&failOnAnte=false")
	setTxSignature(t, builder, uint64(counter))

	return builder.GetTx()
}

func getIntFromStore(t *testing.T, store storetypes.KVStore, key []byte) int64 {
	t.Helper()
	bz := store.Get(key)
	if len(bz) == 0 {
		return 0
	}

	i, err := binary.ReadVarint(bytes.NewBuffer(bz))
	require.NoError(t, err)

	return i
}

func setFailOnAnte(t *testing.T, cfg client.TxConfig, tx signing.Tx, failOnAnte bool) signing.Tx {
	t.Helper()
	builder := cfg.NewTxBuilder()
	err := builder.SetMsgs(tx.GetMsgs()...)
	require.NoError(t, err)
	memo := tx.GetMemo()
	vals, err := url.ParseQuery(memo)
	require.NoError(t, err)

	vals.Set("failOnAnte", strconv.FormatBool(failOnAnte))
	memo = vals.Encode()
	builder.SetMemo(memo)
	setTxSignature(t, builder, 1)

	return builder.GetTx()
}

func setFailOnHandler(t *testing.T, cfg client.TxConfig, ac address.Codec, tx signing.Tx, fail bool) signing.Tx {
	t.Helper()
	builder := cfg.NewTxBuilder()
	builder.SetMemo(tx.GetMemo())

	msgs := tx.GetMsgs()
	addr, err := ac.BytesToString(sdk.AccAddress("addr"))
	require.NoError(t, err)
	for i, msg := range msgs {
		msgs[i] = &baseapptestutil.MsgCounter{
			Counter:       msg.(*baseapptestutil.MsgCounter).Counter,
			FailOnHandler: fail,
			Signer:        addr,
		}
	}

	err = builder.SetMsgs(msgs...)
	require.NoError(t, err)
	return builder.GetTx()
}

// wonkyMsg is to be used to run a MsgCounter2 message when the MsgCounter2 handler is not registered.
func wonkyMsg(t *testing.T, cfg client.TxConfig, ac address.Codec, tx signing.Tx) signing.Tx {
	t.Helper()
	builder := cfg.NewTxBuilder()
	builder.SetMemo(tx.GetMemo())

	msgs := tx.GetMsgs()
	addr, err := ac.BytesToString(sdk.AccAddress("wonky"))
	require.NoError(t, err)
	msgs = append(msgs, &baseapptestutil.MsgCounter2{
		Signer: addr,
	})

	err = builder.SetMsgs(msgs...)
	require.NoError(t, err)
	return builder.GetTx()
}

func newMockedVersionModifier(startingVersion uint64) server.VersionModifier {
	return &mockedVersionModifier{version: startingVersion}
}

type mockedVersionModifier struct {
	version uint64
}

func (m *mockedVersionModifier) SetAppVersion(ctx context.Context, u uint64) error {
	m.version = u
	return nil
}

func (m *mockedVersionModifier) AppVersion(ctx context.Context) (uint64, error) {
	return m.version, nil
}
