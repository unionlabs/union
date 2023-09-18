package cli_test

import (
	"context"
	"encoding/base64"
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/depinject"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/auth/client/cli"
	authtestutil "github.com/cosmos/cosmos-sdk/x/auth/testutil"
)

func TestGetCommandEncode(t *testing.T) {
	var (
		txCfg       client.TxConfig
		legacyAmino *codec.LegacyAmino
		codec       codec.Codec
	)

	err := depinject.Inject(
		authtestutil.AppConfig,
		&txCfg,
		&legacyAmino,
		&codec,
	)
	require.NoError(t, err)

	cmd := cli.GetEncodeCommand()
	_ = testutil.ApplyMockIODiscardOutErr(cmd)

	// Build a test transaction
	builder := txCfg.NewTxBuilder()
	builder.SetGasLimit(50000)
	builder.SetFeeAmount(sdk.Coins{sdk.NewInt64Coin("atom", 150)})
	builder.SetMemo("foomemo")
	jsonEncoded, err := txCfg.TxJSONEncoder()(builder.GetTx())
	require.NoError(t, err)

	txFile := testutil.WriteToNewTempFile(t, string(jsonEncoded))
	txFileName := txFile.Name()

	ctx := context.Background()
	clientCtx := client.Context{}.
		WithTxConfig(txCfg).
		WithCodec(codec)
	ctx = context.WithValue(ctx, client.ClientContextKey, &clientCtx)

	cmd.SetArgs([]string{txFileName})
	err = cmd.ExecuteContext(ctx)
	require.NoError(t, err)
}

func TestGetCommandDecode(t *testing.T) {
	var (
		txCfg       client.TxConfig
		legacyAmino *codec.LegacyAmino
		codec       codec.Codec
	)

	err := depinject.Inject(
		authtestutil.AppConfig,
		&txCfg,
		&legacyAmino,
		&codec,
	)
	require.NoError(t, err)

	clientCtx := client.Context{}.
		WithTxConfig(txCfg).
		WithCodec(codec)

	cmd := cli.GetDecodeCommand()
	_ = testutil.ApplyMockIODiscardOutErr(cmd)

	clientCtx = clientCtx.WithTxConfig(txCfg)

	// Build a test transaction
	builder := txCfg.NewTxBuilder()
	builder.SetGasLimit(50000)
	builder.SetFeeAmount(sdk.Coins{sdk.NewInt64Coin("atom", 150)})
	builder.SetMemo("foomemo")

	// Encode transaction
	txBytes, err := clientCtx.TxConfig.TxEncoder()(builder.GetTx())
	require.NoError(t, err)

	// Convert the transaction into base64 encoded string
	base64Encoded := base64.StdEncoding.EncodeToString(txBytes)

	ctx := context.Background()
	ctx = context.WithValue(ctx, client.ClientContextKey, &clientCtx)

	// Execute the command
	cmd.SetArgs([]string{base64Encoded})
	require.NoError(t, cmd.ExecuteContext(ctx))
}
