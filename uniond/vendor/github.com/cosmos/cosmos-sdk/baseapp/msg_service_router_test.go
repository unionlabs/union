package baseapp_test

import (
	"os"
	"testing"

	dbm "github.com/cometbft/cometbft-db"
	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/libs/log"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/depinject"
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
	err := depinject.Inject(makeMinimalConfig(), &appBuilder, &registry)
	require.NoError(t, err)
	app := appBuilder.Build(log.NewTMLogger(log.NewSyncWriter(os.Stdout)), dbm.NewMemDB(), nil)

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
	err := depinject.Inject(makeMinimalConfig(), &appBuilder, &registry)
	require.NoError(t, err)
	db := dbm.NewMemDB()
	app := appBuilder.Build(log.NewTMLogger(log.NewSyncWriter(os.Stdout)), db, nil)
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

func TestMsgService(t *testing.T) {
	priv, _, _ := testdata.KeyTestPubAddr()

	var (
		appBuilder        *runtime.AppBuilder
		cdc               codec.ProtoCodecMarshaler
		interfaceRegistry codectypes.InterfaceRegistry
	)
	err := depinject.Inject(makeMinimalConfig(), &appBuilder, &cdc, &interfaceRegistry)
	require.NoError(t, err)
	app := appBuilder.Build(log.NewNopLogger(), dbm.NewMemDB(), nil)

	// patch in TxConfig instead of using an output from x/auth/tx
	txConfig := authtx.NewTxConfig(cdc, authtx.DefaultSignModes)
	// set the TxDecoder in the BaseApp for minimal tx simulations
	app.SetTxDecoder(txConfig.TxDecoder())

	testdata.RegisterInterfaces(interfaceRegistry)
	testdata.RegisterMsgServer(
		app.MsgServiceRouter(),
		testdata.MsgServerImpl{},
	)
	_ = app.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{Height: 1}})

	msg := testdata.MsgCreateDog{Dog: &testdata.Dog{Name: "Spot"}}

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
			SignMode:  txConfig.SignModeHandler().DefaultMode(),
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
	}
	sigV2, err = tx.SignWithPrivKey(
		txConfig.SignModeHandler().DefaultMode(), signerData,
		txBuilder, priv, txConfig, 0)
	require.NoError(t, err)
	err = txBuilder.SetSignatures(sigV2)
	require.NoError(t, err)

	// Send the tx to the app
	txBytes, err := txConfig.TxEncoder()(txBuilder.GetTx())
	require.NoError(t, err)
	res := app.DeliverTx(abci.RequestDeliverTx{Tx: txBytes})
	require.Equal(t, abci.CodeTypeOK, res.Code, "res=%+v", res)
}
