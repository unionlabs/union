package rpc

import (
	"context"
	"encoding/hex"
	"fmt"
	"strings"
	"time"

	rpchttp "github.com/cometbft/cometbft/rpc/client/http"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	tmtypes "github.com/cometbft/cometbft/types"
	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/errors"
)

func newTxResponseCheckTx(res *coretypes.ResultBroadcastTxCommit) *sdk.TxResponse {
	if res == nil {
		return nil
	}

	var txHash string
	if res.Hash != nil {
		txHash = res.Hash.String()
	}

	parsedLogs, _ := sdk.ParseABCILogs(res.CheckTx.Log)

	return &sdk.TxResponse{
		Height:    res.Height,
		TxHash:    txHash,
		Codespace: res.CheckTx.Codespace,
		Code:      res.CheckTx.Code,
		Data:      strings.ToUpper(hex.EncodeToString(res.CheckTx.Data)),
		RawLog:    res.CheckTx.Log,
		Logs:      parsedLogs,
		Info:      res.CheckTx.Info,
		GasWanted: res.CheckTx.GasWanted,
		GasUsed:   res.CheckTx.GasUsed,
		Events:    res.CheckTx.Events,
	}
}

func newTxResponseDeliverTx(res *coretypes.ResultBroadcastTxCommit) *sdk.TxResponse {
	if res == nil {
		return nil
	}

	var txHash string
	if res.Hash != nil {
		txHash = res.Hash.String()
	}

	parsedLogs, _ := sdk.ParseABCILogs(res.TxResult.Log)

	return &sdk.TxResponse{
		Height:    res.Height,
		TxHash:    txHash,
		Codespace: res.TxResult.Codespace,
		Code:      res.TxResult.Code,
		Data:      strings.ToUpper(hex.EncodeToString(res.TxResult.Data)),
		RawLog:    res.TxResult.Log,
		Logs:      parsedLogs,
		Info:      res.TxResult.Info,
		GasWanted: res.TxResult.GasWanted,
		GasUsed:   res.TxResult.GasUsed,
		Events:    res.TxResult.Events,
	}
}

func newResponseFormatBroadcastTxCommit(res *coretypes.ResultBroadcastTxCommit) *sdk.TxResponse {
	if res == nil {
		return nil
	}

	if !res.CheckTx.IsOK() {
		return newTxResponseCheckTx(res)
	}

	return newTxResponseDeliverTx(res)
}

// QueryEventForTxCmd returns a CLI command that subscribes to a WebSocket connection and waits for a transaction event with the given hash.
func QueryEventForTxCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "event-query-tx-for [hash]",
		Short: "Query for a transaction by hash",
		Long:  `Subscribes to a CometBFT WebSocket connection and waits for a transaction event with the given hash.`,
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}
			c, err := rpchttp.New(clientCtx.NodeURI, "/websocket")
			if err != nil {
				return err
			}
			if err := c.Start(); err != nil {
				return err
			}
			defer c.Stop() //nolint:errcheck // ignore stop error

			ctx, cancel := context.WithTimeout(context.Background(), time.Second*15)
			defer cancel()

			hash := args[0]
			query := fmt.Sprintf("%s='%s' AND %s='%s'", tmtypes.EventTypeKey, tmtypes.EventTx, tmtypes.TxHashKey, hash)
			const subscriber = "subscriber"
			eventCh, err := c.Subscribe(ctx, subscriber, query)
			if err != nil {
				return fmt.Errorf("failed to subscribe to tx: %w", err)
			}
			defer c.UnsubscribeAll(context.Background(), subscriber) //nolint:errcheck // ignore unsubscribe error

			select {
			case evt := <-eventCh:
				if txe, ok := evt.Data.(tmtypes.EventDataTx); ok {
					res := &coretypes.ResultBroadcastTxCommit{
						TxResult: txe.Result,
						Hash:     tmtypes.Tx(txe.Tx).Hash(),
						Height:   txe.Height,
					}
					return clientCtx.PrintProto(newResponseFormatBroadcastTxCommit(res))
				}
			case <-ctx.Done():
				return errors.ErrLogic.Wrapf("timed out waiting for event, the transaction could have already been included or wasn't yet included")
			}
			return nil
		},
	}

	flags.AddTxFlagsToCmd(cmd)

	return cmd
}
