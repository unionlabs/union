package client_test

import (
	"fmt"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/x/auth"
	authclient "github.com/cosmos/cosmos-sdk/x/auth/client"
)

func TestParseQueryResponse(t *testing.T) {
	simRes := &sdk.SimulationResponse{
		GasInfo: sdk.GasInfo{GasUsed: 10, GasWanted: 20},
		Result:  &sdk.Result{Data: []byte("tx data"), Log: "log"},
	}

	bz, err := codec.ProtoMarshalJSON(simRes, nil)
	require.NoError(t, err)

	res, err := authclient.ParseQueryResponse(bz)
	require.NoError(t, err)
	require.Equal(t, 10, int(res.GasInfo.GasUsed))
	require.NotNil(t, res.Result)

	res, err = authclient.ParseQueryResponse([]byte("fuzzy"))
	require.Error(t, err)
}

func TestReadTxFromFile(t *testing.T) {
	t.Parallel()

	encodingConfig := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{})
	interfaceRegistry := encodingConfig.InterfaceRegistry
	txConfig := encodingConfig.TxConfig

	clientCtx := client.Context{}
	clientCtx = clientCtx.WithInterfaceRegistry(interfaceRegistry)
	clientCtx = clientCtx.WithTxConfig(txConfig)

	feeAmount := sdk.Coins{sdk.NewInt64Coin("atom", 150)}
	gasLimit := uint64(50000)
	memo := "foomemo"

	txBuilder := txConfig.NewTxBuilder()
	txBuilder.SetFeeAmount(feeAmount)
	txBuilder.SetGasLimit(gasLimit)
	txBuilder.SetMemo(memo)

	// Write it to the file
	encodedTx, err := txConfig.TxJSONEncoder()(txBuilder.GetTx())
	require.NoError(t, err)

	jsonTxFile := testutil.WriteToNewTempFile(t, string(encodedTx))
	// Read it back
	decodedTx, err := authclient.ReadTxFromFile(clientCtx, jsonTxFile.Name())
	require.NoError(t, err)
	txBldr, err := txConfig.WrapTxBuilder(decodedTx)
	require.NoError(t, err)
	t.Log(txBuilder.GetTx())
	t.Log(txBldr.GetTx())
	require.Equal(t, txBuilder.GetTx().GetMemo(), txBldr.GetTx().GetMemo())
	require.Equal(t, txBuilder.GetTx().GetFee(), txBldr.GetTx().GetFee())
}

func TestReadTxsFromFile(t *testing.T) {
	t.Parallel()

	encodingConfig := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{})
	interfaceRegistry := encodingConfig.InterfaceRegistry
	txConfig := encodingConfig.TxConfig

	clientCtx := client.Context{}
	clientCtx = clientCtx.WithInterfaceRegistry(interfaceRegistry)
	clientCtx = clientCtx.WithTxConfig(txConfig)

	// Set up 2 txs
	txBuilders := make([]client.TxBuilder, 2)
	// Set up tx 1
	txBuilders[0] = txConfig.NewTxBuilder()
	txBuilders[0].SetFeeAmount(sdk.Coins{sdk.NewInt64Coin("atom", 150)})
	txBuilders[0].SetGasLimit(uint64(50000))
	txBuilders[0].SetMemo("foomemo")
	// Set up tx 2
	txBuilders[1] = txConfig.NewTxBuilder()
	txBuilders[1].SetFeeAmount(sdk.Coins{sdk.NewInt64Coin("atom", 200)})
	txBuilders[1].SetGasLimit(uint64(60000))
	txBuilders[1].SetMemo("foomemo2")

	// Write txs to the file
	encodedTx1, err := txConfig.TxJSONEncoder()(txBuilders[0].GetTx())
	require.NoError(t, err)
	encodedTx2, err := txConfig.TxJSONEncoder()(txBuilders[1].GetTx())
	require.NoError(t, err)

	tx1String := string(encodedTx1) + "\n"
	tx2String := string(encodedTx2) + "\n"
	jsonBatchTxsFile := testutil.WriteToNewTempFile(t, tx1String+tx2String)
	jsonSingleTxFile := testutil.WriteToNewTempFile(t, tx1String)

	// Read it back

	// 2 txs case
	decodedBatchTxs, err := authclient.ReadTxsFromFile(clientCtx, jsonBatchTxsFile.Name())
	require.NoError(t, err)
	require.Equal(t, len(decodedBatchTxs), 2)
	for i, decodedTx := range decodedBatchTxs {
		txBldr, err := txConfig.WrapTxBuilder(decodedTx)
		require.NoError(t, err)

		wantTx := txBuilders[i].GetTx()
		gotTx := txBldr.GetTx()
		require.Equal(t, wantTx.GetMemo(), gotTx.GetMemo())
		require.Equal(t, wantTx.GetFee(), gotTx.GetFee())
	}

	// single tx case
	decodedSingleTx, err := authclient.ReadTxsFromFile(clientCtx, jsonSingleTxFile.Name())
	require.NoError(t, err)
	require.Equal(t, len(decodedSingleTx), 1)
	txBldr, err := txConfig.WrapTxBuilder(decodedSingleTx[0])
	require.NoError(t, err)

	wantTx := txBuilders[0].GetTx()
	gotTx := txBldr.GetTx()
	require.Equal(t, wantTx.GetMemo(), gotTx.GetMemo())
	require.Equal(t, wantTx.GetFee(), gotTx.GetFee())
}

func TestBatchScanner_Scan(t *testing.T) {
	t.Parallel()

	encodingConfig := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, auth.AppModule{})
	txConfig := encodingConfig.TxConfig

	clientCtx := client.Context{}
	clientCtx = clientCtx.WithTxConfig(txConfig)

	// generate some tx JSON
	bldr := txConfig.NewTxBuilder()
	bldr.SetGasLimit(50000)
	bldr.SetFeeAmount(sdk.NewCoins(sdk.NewInt64Coin("atom", 150)))
	bldr.SetMemo("foomemo")
	txJSON, err := txConfig.TxJSONEncoder()(bldr.GetTx())
	require.NoError(t, err)

	// use the tx JSON to generate some tx batches (it doesn't matter that we use the same JSON because we don't care about the actual context)
	goodBatchOf3Txs := fmt.Sprintf("%s\n%s\n%s\n", txJSON, txJSON, txJSON)
	malformedBatch := fmt.Sprintf("%s\nmalformed\n%s\n", txJSON, txJSON)
	batchOf2TxsWithNoNewline := fmt.Sprintf("%s\n%s", txJSON, txJSON)
	batchWithEmptyLine := fmt.Sprintf("%s\n\n%s", txJSON, txJSON)

	tests := []struct {
		name               string
		batch              string
		wantScannerError   bool
		wantUnmarshalError bool
		numTxs             int
	}{
		{"good batch", goodBatchOf3Txs, false, false, 3},
		{"malformed", malformedBatch, false, true, 1},
		{"missing trailing newline", batchOf2TxsWithNoNewline, false, false, 2},
		{"empty line", batchWithEmptyLine, false, true, 1},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			scanner, i := authclient.NewBatchScanner(clientCtx.TxConfig, strings.NewReader(tt.batch)), 0
			for scanner.Scan() {
				_ = scanner.Tx()
				i++
			}
			require.Equal(t, tt.wantScannerError, scanner.Err() != nil)
			require.Equal(t, tt.wantUnmarshalError, scanner.UnmarshalErr() != nil)
			require.Equal(t, tt.numTxs, i)
		})
	}
}
