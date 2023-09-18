package baseapp_test

import (
	"bytes"
	"context"
	"encoding/binary"
	"encoding/json"
	"errors"
	"fmt"
	"net/url"
	"os"
	"reflect"
	"strconv"
	"testing"
	"unsafe"

	runtimev1alpha1 "cosmossdk.io/api/cosmos/app/runtime/v1alpha1"
	appv1alpha1 "cosmossdk.io/api/cosmos/app/v1alpha1"
	authmodulev1 "cosmossdk.io/api/cosmos/auth/module/v1"
	bankmodulev1 "cosmossdk.io/api/cosmos/bank/module/v1"
	consensusmodulev1 "cosmossdk.io/api/cosmos/consensus/module/v1"
	mintmodulev1 "cosmossdk.io/api/cosmos/mint/module/v1"
	paramsmodulev1 "cosmossdk.io/api/cosmos/params/module/v1"
	stakingmodulev1 "cosmossdk.io/api/cosmos/staking/module/v1"
	txconfigv1 "cosmossdk.io/api/cosmos/tx/config/v1"
	"cosmossdk.io/core/appconfig"
	"cosmossdk.io/depinject"
	dbm "github.com/cometbft/cometbft-db"
	"github.com/cometbft/cometbft/libs/log"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	tmtypes "github.com/cometbft/cometbft/types"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/baseapp"
	baseapptestutil "github.com/cosmos/cosmos-sdk/baseapp/testutil"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/runtime"
	storetypes "github.com/cosmos/cosmos-sdk/store/types"
	"github.com/cosmos/cosmos-sdk/testutil/mock"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/mempool"
	signingtypes "github.com/cosmos/cosmos-sdk/types/tx/signing"
	_ "github.com/cosmos/cosmos-sdk/x/auth"
	"github.com/cosmos/cosmos-sdk/x/auth/signing"
	_ "github.com/cosmos/cosmos-sdk/x/auth/tx/config"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	_ "github.com/cosmos/cosmos-sdk/x/bank"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	_ "github.com/cosmos/cosmos-sdk/x/consensus"
	_ "github.com/cosmos/cosmos-sdk/x/mint"
	minttypes "github.com/cosmos/cosmos-sdk/x/mint/types"
	_ "github.com/cosmos/cosmos-sdk/x/params"
	_ "github.com/cosmos/cosmos-sdk/x/staking"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
)

var ParamStoreKey = []byte("paramstore")

func defaultLogger() log.Logger {
	if testing.Verbose() {
		return log.NewTMLogger(log.NewSyncWriter(os.Stdout)).With("module", "baseapp/test")
	}

	return log.NewNopLogger()
}

// GenesisStateWithSingleValidator initializes GenesisState with a single validator and genesis accounts
// that also act as delegators.
func GenesisStateWithSingleValidator(t *testing.T, codec codec.Codec, builder *runtime.AppBuilder) map[string]json.RawMessage {
	t.Helper()

	privVal := mock.NewPV()
	pubKey, err := privVal.GetPubKey()
	require.NoError(t, err)

	// create validator set with single validator
	validator := tmtypes.NewValidator(pubKey, 1)
	valSet := tmtypes.NewValidatorSet([]*tmtypes.Validator{validator})

	// generate genesis account
	senderPrivKey := secp256k1.GenPrivKey()
	acc := authtypes.NewBaseAccount(senderPrivKey.PubKey().Address().Bytes(), senderPrivKey.PubKey(), 0, 0)
	balances := []banktypes.Balance{
		{
			Address: acc.GetAddress().String(),
			Coins:   sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdk.NewInt(100000000000000))),
		},
	}

	genesisState := builder.DefaultGenesis()
	// sus
	genesisState, err = simtestutil.GenesisStateWithValSet(codec, genesisState, valSet, []authtypes.GenesisAccount{acc}, balances...)
	require.NoError(t, err)

	return genesisState
}

func makeTestConfig() depinject.Config {
	return appconfig.Compose(&appv1alpha1.Config{
		Modules: []*appv1alpha1.ModuleConfig{
			{
				Name: "runtime",
				Config: appconfig.WrapAny(&runtimev1alpha1.Module{
					AppName: "BaseAppApp",
					BeginBlockers: []string{
						"mint",
						"staking",
						"auth",
						"bank",
						"params",
						"consensus",
					},
					EndBlockers: []string{
						"staking",
						"auth",
						"bank",
						"mint",
						"params",
						"consensus",
					},
					OverrideStoreKeys: []*runtimev1alpha1.StoreKeyConfig{
						{
							ModuleName: "auth",
							KvStoreKey: "acc",
						},
					},
					InitGenesis: []string{
						"auth",
						"bank",
						"staking",
						"mint",
						"params",
						"consensus",
					},
				}),
			},
			{
				Name: "auth",
				Config: appconfig.WrapAny(&authmodulev1.Module{
					Bech32Prefix: "cosmos",
					ModuleAccountPermissions: []*authmodulev1.ModuleAccountPermission{
						{Account: authtypes.FeeCollectorName},
						{Account: minttypes.ModuleName, Permissions: []string{authtypes.Minter}},
						{Account: stakingtypes.BondedPoolName, Permissions: []string{authtypes.Burner, stakingtypes.ModuleName}},
						{Account: stakingtypes.NotBondedPoolName, Permissions: []string{authtypes.Burner, stakingtypes.ModuleName}},
					},
				}),
			},
			{
				Name:   "bank",
				Config: appconfig.WrapAny(&bankmodulev1.Module{}),
			},
			{
				Name:   "params",
				Config: appconfig.WrapAny(&paramsmodulev1.Module{}),
			},
			{
				Name:   "staking",
				Config: appconfig.WrapAny(&stakingmodulev1.Module{}),
			},
			{
				Name:   "mint",
				Config: appconfig.WrapAny(&mintmodulev1.Module{}),
			},
			{
				Name:   "consensus",
				Config: appconfig.WrapAny(&consensusmodulev1.Module{}),
			},
			{
				Name:   "tx",
				Config: appconfig.WrapAny(&txconfigv1.Config{}),
			},
		},
	})
}

func makeMinimalConfig() depinject.Config {
	var mempoolOpt runtime.BaseAppOption = baseapp.SetMempool(mempool.NewSenderNonceMempool())
	return depinject.Configs(
		depinject.Supply(mempoolOpt),
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
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	store := sdkCtx.KVStore(capKey)

	sdkCtx.GasMeter().ConsumeGas(5, "test")

	var msgCount int64

	switch m := msg.(type) {
	case *baseapptestutil.MsgCounter:
		if m.FailOnHandler {
			return nil, sdkerrors.Wrap(sdkerrors.ErrInvalidRequest, "message handler failure")
		}
		msgCount = m.Counter
	case *baseapptestutil.MsgCounter2:
		if m.FailOnHandler {
			return nil, sdkerrors.Wrap(sdkerrors.ErrInvalidRequest, "message handler failure")
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
	return func(ctx sdk.Context, tx sdk.Tx, simulate bool) (sdk.Context, error) {
		store := ctx.KVStore(capKey)
		counter, failOnAnte := parseTxMemo(t, tx)

		if failOnAnte {
			return ctx, sdkerrors.Wrap(sdkerrors.ErrUnauthorized, "ante handler failure")
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

func incrementingCounter(t *testing.T, store sdk.KVStore, counterKey []byte, counter int64) (*sdk.Result, error) {
	storedCounter := getIntFromStore(t, store, counterKey)
	require.Equal(t, storedCounter, counter)
	setIntOnStore(store, counterKey, counter+1)
	return &sdk.Result{}, nil
}

func setIntOnStore(store sdk.KVStore, key []byte, i int64) {
	bz := make([]byte, 8)
	n := binary.PutVarint(bz, i)
	store.Set(key, bz[:n])
}

type paramStore struct {
	db *dbm.MemDB
}

func (ps *paramStore) Set(_ sdk.Context, value *tmproto.ConsensusParams) {
	bz, err := json.Marshal(value)
	if err != nil {
		panic(err)
	}

	ps.db.Set(ParamStoreKey, bz)
}

func (ps *paramStore) Has(_ sdk.Context) bool {
	ok, err := ps.db.Has(ParamStoreKey)
	if err != nil {
		panic(err)
	}

	return ok
}

func (ps paramStore) Get(ctx sdk.Context) (*tmproto.ConsensusParams, error) {
	bz, err := ps.db.Get(ParamStoreKey)
	if err != nil {
		panic(err)
	}

	if len(bz) == 0 {
		return nil, errors.New("params not found")
	}

	var params tmproto.ConsensusParams
	if err := json.Unmarshal(bz, &params); err != nil {
		panic(err)
	}

	return &params, nil
}

func setTxSignature(t *testing.T, builder client.TxBuilder, nonce uint64) {
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

func getDeliverStateCtx(app *baseapp.BaseApp) sdk.Context {
	v := reflect.ValueOf(app).Elem()
	f := v.FieldByName("deliverState")
	rf := reflect.NewAt(f.Type(), unsafe.Pointer(f.UnsafeAddr())).Elem()
	return rf.MethodByName("Context").Call(nil)[0].Interface().(sdk.Context)
}

func parseTxMemo(t *testing.T, tx sdk.Tx) (counter int64, failOnAnte bool) {
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

func newTxCounter(t *testing.T, cfg client.TxConfig, counter int64, msgCounters ...int64) signing.Tx {
	msgs := make([]sdk.Msg, 0, len(msgCounters))
	for _, c := range msgCounters {
		msg := &baseapptestutil.MsgCounter{Counter: c, FailOnHandler: false}
		msgs = append(msgs, msg)
	}

	builder := cfg.NewTxBuilder()
	builder.SetMsgs(msgs...)
	builder.SetMemo("counter=" + strconv.FormatInt(counter, 10) + "&failOnAnte=false")
	setTxSignature(t, builder, uint64(counter))

	return builder.GetTx()
}

func getIntFromStore(t *testing.T, store sdk.KVStore, key []byte) int64 {
	bz := store.Get(key)
	if len(bz) == 0 {
		return 0
	}

	i, err := binary.ReadVarint(bytes.NewBuffer(bz))
	require.NoError(t, err)

	return i
}

func setFailOnAnte(t *testing.T, cfg client.TxConfig, tx signing.Tx, failOnAnte bool) signing.Tx {
	builder := cfg.NewTxBuilder()
	builder.SetMsgs(tx.GetMsgs()...)

	memo := tx.GetMemo()
	vals, err := url.ParseQuery(memo)
	require.NoError(t, err)

	vals.Set("failOnAnte", strconv.FormatBool(failOnAnte))
	memo = vals.Encode()
	builder.SetMemo(memo)
	setTxSignature(t, builder, 1)

	return builder.GetTx()
}

func setFailOnHandler(cfg client.TxConfig, tx signing.Tx, fail bool) signing.Tx {
	builder := cfg.NewTxBuilder()
	builder.SetMemo(tx.GetMemo())

	msgs := tx.GetMsgs()
	for i, msg := range msgs {
		msgs[i] = &baseapptestutil.MsgCounter{
			Counter:       msg.(*baseapptestutil.MsgCounter).Counter,
			FailOnHandler: fail,
		}
	}

	builder.SetMsgs(msgs...)
	return builder.GetTx()
}
