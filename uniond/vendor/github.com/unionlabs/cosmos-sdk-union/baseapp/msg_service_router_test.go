package baseapp_test

import (
	"context"
	"testing"

	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
	"github.com/stretchr/testify/require"

	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/depinject"
	"cosmossdk.io/log"

	"github.com/cosmos/cosmos-sdk/client/tx"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
	authsigning "github.com/cosmos/cosmos-sdk/x/auth/signing"
	authtx "github.com/cosmos/cosmos-sdk/x/auth/tx"
)

func TestRegisterMsgService(t *testing.T) {
	// Setup baseapp.
	var (
		appBuilder *runtime.AppBuilder
		registry   codectypes.InterfaceRegistry
	)
	err := depinject.Inject(
		depinject.Configs(
			makeMinimalConfig(),
			depinject.Supply(log.NewTestLogger(t)),
		), &appBuilder, &registry)
	require.NoError(t, err)
	app := appBuilder.Build(coretesting.NewMemDB(), nil)

	require.Panics(t, func() {
		testdata.RegisterMsgServer(
			app.MsgServiceRouter(),
			testdata.MsgServerImpl{},
		)
	})

	// Register testdata Msg services, and rerun `RegisterMsgService`.
	testdata.RegisterInterfaces(registry)

	require.NotPanics(t, func() {
		testdata.RegisterMsgServer(
			app.MsgServiceRouter(),
			testdata.MsgServerImpl{},
		)
	})
}

func TestRegisterMsgServiceTwice(t *testing.T) {
	// Setup baseapp.
	var (
		appBuilder *runtime.AppBuilder
		registry   codectypes.InterfaceRegistry
	)
	err := depinject.Inject(
		depinject.Configs(
			makeMinimalConfig(),
			depinject.Supply(log.NewTestLogger(t)),
		), &appBuilder, &registry)
	require.NoError(t, err)
	db := coretesting.NewMemDB()
	app := appBuilder.Build(db, nil)
	testdata.RegisterInterfaces(registry)

	// First time registering service shouldn't panic.
	require.NotPanics(t, func() {
		testdata.RegisterMsgServer(
			app.MsgServiceRouter(),
			testdata.MsgServerImpl{},
		)
	})

	// Second time should panic.
	require.Panics(t, func() {
		testdata.RegisterMsgServer(
			app.MsgServiceRouter(),
			testdata.MsgServerImpl{},
		)
	})
}

func TestHybridHandlerByMsgName(t *testing.T) {
	// Setup baseapp and router.
	var (
		appBuilder *runtime.AppBuilder
		registry   codectypes.InterfaceRegistry
	)
	err := depinject.Inject(
		depinject.Configs(
			makeMinimalConfig(),
			depinject.Supply(log.NewTestLogger(t)),
		), &appBuilder, &registry)
	require.NoError(t, err)
	db := coretesting.NewMemDB()
	app := appBuilder.Build(db, nil)
	testdata.RegisterInterfaces(registry)

	testdata.RegisterMsgServer(
		app.MsgServiceRouter(),
		testdata.MsgServerImpl{},
	)

	handler := app.MsgServiceRouter().HybridHandlerByMsgName("testpb.MsgCreateDog")

	require.NotNil(t, handler)
	require.NoError(t, app.Init())
	ctx := app.NewContext(true)
	resp := new(testdata.MsgCreateDogResponse)
	err = handler(ctx, &testdata.MsgCreateDog{
		Dog:   &testdata.Dog{Name: "Spot"},
		Owner: "me",
	}, resp)
	require.NoError(t, err)
	require.Equal(t, resp.Name, "Spot")
}

func TestMsgService(t *testing.T) {
	priv, _, _ := testdata.KeyTestPubAddr()

	var (
		appBuilder        *runtime.AppBuilder
		cdc               codec.Codec
		interfaceRegistry codectypes.InterfaceRegistry
	)
	err := depinject.Inject(
		depinject.Configs(
			makeMinimalConfig(),
			depinject.Supply(log.NewNopLogger()),
		), &appBuilder, &cdc, &interfaceRegistry)
	require.NoError(t, err)
	app := appBuilder.Build(coretesting.NewMemDB(), nil)
	signingCtx := interfaceRegistry.SigningContext()

	// patch in TxConfig instead of using an output from x/auth/tx
	txConfig := authtx.NewTxConfig(cdc, signingCtx.AddressCodec(), signingCtx.ValidatorAddressCodec(), authtx.DefaultSignModes)
	// set the TxDecoder in the BaseApp for minimal tx simulations
	app.SetTxDecoder(txConfig.TxDecoder())

	defaultSignMode, err := authsigning.APISignModeToInternal(txConfig.SignModeHandler().DefaultMode())
	require.NoError(t, err)

	testdata.RegisterInterfaces(interfaceRegistry)
	testdata.RegisterMsgServer(
		app.MsgServiceRouter(),
		testdata.MsgServerImpl{},
	)
	_, err = app.FinalizeBlock(&abci.FinalizeBlockRequest{Height: 1})
	require.NoError(t, err)

	_, _, addr := testdata.KeyTestPubAddr()
	addrStr, err := signingCtx.AddressCodec().BytesToString(addr)
	require.NoError(t, err)
	msg := testdata.MsgCreateDog{
		Dog:   &testdata.Dog{Name: "Spot"},
		Owner: addrStr,
	}

	txBuilder := txConfig.NewTxBuilder()
	txBuilder.SetFeeAmount(testdata.NewTestFeeAmount())
	txBuilder.SetGasLimit(testdata.NewTestGasLimit())
	err = txBuilder.SetMsgs(&msg)
	require.NoError(t, err)

	// First round: we gather all the signer infos. We use the "set empty
	// signature" hack to do that.
	sigV2 := signing.SignatureV2{
		PubKey: priv.PubKey(),
		Data: &signing.SingleSignatureData{
			SignMode:  defaultSignMode,
			Signature: nil,
		},
		Sequence: 0,
	}

	err = txBuilder.SetSignatures(sigV2)
	require.NoError(t, err)

	// Second round: all signer infos are set, so each signer can sign.
	signerData := authsigning.SignerData{
		ChainID:       "test",
		AccountNumber: 0,
		Sequence:      0,
		PubKey:        priv.PubKey(),
	}
	sigV2, err = tx.SignWithPrivKey(
		context.TODO(), defaultSignMode, signerData,
		txBuilder, priv, txConfig, 0)
	require.NoError(t, err)
	err = txBuilder.SetSignatures(sigV2)
	require.NoError(t, err)

	// Send the tx to the app
	txBytes, err := txConfig.TxEncoder()(txBuilder.GetTx())
	require.NoError(t, err)
	res, err := app.FinalizeBlock(&abci.FinalizeBlockRequest{Height: 1, Txs: [][]byte{txBytes}})
	require.NoError(t, err)
	require.Equal(t, abci.CodeTypeOK, res.TxResults[0].Code, "res=%+v", res)
}
