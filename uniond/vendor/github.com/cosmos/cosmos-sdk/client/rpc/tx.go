package rpc

import (
	"context"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"strings"
	"time"

	rpchttp "github.com/cometbft/cometbft/rpc/client/http"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"
	tmtypes "github.com/cometbft/cometbft/types"
	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/version"
)

const TimeoutFlag = "timeout"

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

// QueryEventForTxCmd is an alias for WaitTxCmd, kept for backwards compatibility.
func QueryEventForTxCmd() *cobra.Command {
	return WaitTxCmd()
}

// WaitTxCmd returns a CLI command that waits for a transaction with the given hash to be included in a block.
func WaitTxCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "wait-tx [hash]",
		Aliases: []string{"event-query-tx-for"},
		Short:   "Wait for a transaction to be included in a block",
		Long:    `Subscribes to a CometBFT WebSocket connection and waits for a transaction event with the given hash.`,
		Example: fmt.Sprintf(`By providing the transaction hash:
$ %[1]s q wait-tx [hash]

Or, by piping a "tx" command:
$ %[1]s tx [flags] | %[1]s q wait-tx
`, version.AppName),
		Args: cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			timeout, err := cmd.Flags().GetDuration(TimeoutFlag)
			if err != nil {
				return err
			}

			c, err := rpchttp.New(clientCtx.NodeURI)
			if err != nil {
				return err
			}
			if err := c.Start(); err != nil {
				return err
			}
			defer c.Stop() //nolint:errcheck // ignore stop error

			ctx, cancel := context.WithTimeout(context.Background(), timeout)
			defer cancel()

			var hash []byte
			if len(args) == 0 {
				// read hash from stdin
				in, err := io.ReadAll(cmd.InOrStdin())
				if err != nil {
					return err
				}
				hashByt, err := parseHashFromInput(in)
				if err != nil {
					return err
				}

				hash = hashByt
			} else {
				// read hash from args
				hashByt, err := hex.DecodeString(args[0])
				if err != nil {
					return err
				}

				hash = hashByt
			}

			// subscribe to websocket events
			query := fmt.Sprintf("%s='%s' AND %s='%X'", tmtypes.EventTypeKey, tmtypes.EventTx, tmtypes.TxHashKey, hash)
			const subscriber = "subscriber"
			eventCh, err := c.Subscribe(ctx, subscriber, query)
			if err != nil {
				return fmt.Errorf("failed to subscribe to tx: %w", err)
			}
			defer c.UnsubscribeAll(context.Background(), subscriber) //nolint:errcheck // ignore unsubscribe error

			// return immediately if tx is already included in a block
			res, err := c.Tx(ctx, hash, false)
			if err == nil {
				// tx already included in a block
				res := &coretypes.ResultBroadcastTxCommit{
					TxResult: res.TxResult,
					Hash:     res.Hash,
					Height:   res.Height,
				}
				return clientCtx.PrintProto(newResponseFormatBroadcastTxCommit(res))
			}

			// tx not yet included in a block, wait for event on websocket
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
				return sdkerrors.ErrLogic.Wrapf("timed out waiting for transaction %X to be included in a block", hash)
			}
			return nil
		},
	}

	cmd.Flags().Duration(TimeoutFlag, 15*time.Second, "The maximum time to wait for the transaction to be included in a block")
	flags.AddQueryFlagsToCmd(cmd)

	return cmd
}

func parseHashFromInput(in []byte) ([]byte, error) {
	// The content of in is expected to be the result of a tx command which should be using GenerateOrBroadcastTxCLI.
	// That outputs a sdk.TxResponse as either the json or yaml. As json, we can't unmarshal it back into that struct,
	// though, because the height field ends up quoted which confuses json.Unmarshal (because it's for an int64 field).

	// Try to find the txhash from json output.
	resultTx := make(map[string]json.RawMessage)
	if err := json.Unmarshal(in, &resultTx); err == nil && len(resultTx["txhash"]) > 0 {
		// input was JSON, return the hash
		hash := strings.Trim(strings.TrimSpace(string(resultTx["txhash"])), `"`)
		if len(hash) > 0 {
			return hex.DecodeString(hash)
		}
	}

	// Try to find the txhash from yaml output.
	lines := strings.Split(string(in), "\n")
	for _, line := range lines {
		if strings.HasPrefix(line, "txhash:") {
			hash := strings.TrimSpace(line[len("txhash:"):])
			return hex.DecodeString(hash)
		}
	}
	return nil, errors.New("txhash not found")
}
