package cli

import (
	"bytes"
	"context"

	"github.com/cosmos/gogoproto/proto"
	"github.com/spf13/cobra"

	sdkmath "cosmossdk.io/math"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/client/tx"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	txtypes "github.com/cosmos/cosmos-sdk/types/tx"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
)

// ExecTestCLICmd builds the client context, mocks the output and executes the command.
func ExecTestCLICmd(clientCtx client.Context, cmd *cobra.Command, extraArgs []string) (testutil.BufferWriter, error) {
	cmd.SetArgs(extraArgs)

	_, out := testutil.ApplyMockIO(cmd)
	clientCtx = clientCtx.WithOutput(out)

	ctx := context.Background()
	ctx = context.WithValue(ctx, client.ClientContextKey, &clientCtx)

	if err := cmd.ExecuteContext(ctx); err != nil {
		return out, err
	}

	return out, nil
}

type TestTxConfig struct {
	Simulate             bool
	GenOnly              bool
	Offline              bool
	Memo                 string
	Gas                  uint64
	AccNum               uint64
	Seq                  uint64
	Fee                  sdk.Coins
	IsAsyncBroadcastMode bool
}

func SubmitTestTx(clientCtx client.Context, msg proto.Message, from sdk.AccAddress, config TestTxConfig) (testutil.BufferWriter, error) {
	txBuilder := clientCtx.TxConfig.NewTxBuilder()

	err := txBuilder.SetMsgs(msg)
	if err != nil {
		return nil, err
	}

	if config.Fee != nil {
		txBuilder.SetFeeAmount(config.Fee)
	} else {
		txBuilder.SetFeeAmount(sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(10)))) // Arbitrary fee
	}

	if config.Gas != 0 {
		txBuilder.SetGasLimit(config.Gas)
	} else {
		txBuilder.SetGasLimit(flags.DefaultGasLimit) // Need at least 100386
	}

	if config.Memo != "" {
		txBuilder.SetMemo(config.Memo)
	}

	if config.GenOnly {
		txBz, err := clientCtx.TxConfig.TxJSONEncoder()(txBuilder.GetTx())
		if err != nil {
			return nil, err
		}

		out := bytes.NewBuffer(txBz)
		return out, nil
	}

	txFactory := tx.Factory{}
	txFactory = txFactory.
		WithChainID(clientCtx.ChainID).
		WithKeybase(clientCtx.Keyring).
		WithTxConfig(clientCtx.TxConfig).
		WithSignMode(signing.SignMode_SIGN_MODE_DIRECT)

	if config.Offline {
		txFactory = txFactory.
			WithAccountNumber(config.AccNum).
			WithSequence(config.Seq)
	} else {
		accNum, accSeq, err := clientCtx.AccountRetriever.GetAccountNumberSequence(clientCtx, from)
		if err != nil {
			return nil, err
		}

		txFactory = txFactory.
			WithAccountNumber(accNum).
			WithSequence(accSeq)
	}

	accBytes, err := clientCtx.AddressCodec.StringToBytes(from.String())
	if err != nil {
		return nil, err
	}

	keyRecord, err := clientCtx.Keyring.KeyByAddress(accBytes)
	if err != nil {
		return nil, err
	}

	err = tx.Sign(clientCtx, txFactory, keyRecord.Name, txBuilder, true)
	if err != nil {
		return nil, err
	}

	txBz, err := clientCtx.TxConfig.TxEncoder()(txBuilder.GetTx())
	if err != nil {
		return nil, err
	}

	clientCtx.BroadcastMode = flags.BroadcastSync
	if config.IsAsyncBroadcastMode {
		clientCtx.BroadcastMode = flags.BroadcastAsync
	}

	var res proto.Message

	if config.Simulate {
		txSvcClient := txtypes.NewServiceClient(clientCtx)
		res, err = txSvcClient.Simulate(context.Background(), &txtypes.SimulateRequest{
			TxBytes: txBz,
		})
		if err != nil {
			return nil, err
		}

	} else {
		res, err = clientCtx.BroadcastTxSync(txBz)
		if err != nil {
			return nil, err
		}
	}

	bz, err := clientCtx.Codec.MarshalJSON(res)
	if err != nil {
		return nil, err
	}

	out := bytes.NewBuffer(bz)

	return out, err
}
