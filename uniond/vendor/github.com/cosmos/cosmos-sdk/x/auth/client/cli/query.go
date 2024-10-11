package cli

import (
	"errors"
	"fmt"
	"strings"

	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	querytypes "github.com/cosmos/cosmos-sdk/types/query"
	"github.com/cosmos/cosmos-sdk/version"
	authtx "github.com/cosmos/cosmos-sdk/x/auth/tx"
)

const (
	FlagQuery   = "query"
	FlagType    = "type"
	FlagOrderBy = "order_by"

	TypeHash   = "hash"
	TypeAccSeq = "acc_seq"
	TypeSig    = "signature"
	TypeHeight = "height"

	EventFormat = "{eventType}.{eventAttribute}={value}"
)

// QueryTxsByEventsCmd returns a command to search through transactions by events.
func QueryTxsByEventsCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "txs",
		Short: "Query for paginated transactions that match a set of events",
		Long: `Search for transactions that match the exact given events where results are paginated.
The events query is directly passed to Tendermint's RPC TxSearch method and must
conform to Tendermint's query syntax.

Please refer to each module's documentation for the full set of events to query
for. Each module documents its respective events under 'xx_events.md'.
`,
		Example: fmt.Sprintf(
			"$ %s query txs --query \"message.sender='cosmos1...' AND message.action='withdraw_delegator_reward' AND tx.height > 7\" --page 1 --limit 30",
			version.AppName,
		),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			query, _ := cmd.Flags().GetString(FlagQuery)
			page, _ := cmd.Flags().GetInt(flags.FlagPage)
			limit, _ := cmd.Flags().GetInt(flags.FlagLimit)
			orderBy, _ := cmd.Flags().GetString(FlagOrderBy)

			txs, err := authtx.QueryTxsByEvents(clientCtx, page, limit, query, orderBy)
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(txs)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)
	cmd.Flags().Int(flags.FlagPage, querytypes.DefaultPage, "Query a specific page of paginated results")
	cmd.Flags().Int(flags.FlagLimit, querytypes.DefaultLimit, "Query number of transactions results per page returned")
	cmd.Flags().String(FlagQuery, "", "The transactions events query per Tendermint's query semantics")
	cmd.Flags().String(FlagOrderBy, "", "The ordering semantics (asc|dsc)")
	_ = cmd.MarkFlagRequired(FlagQuery)

	return cmd
}

// QueryTxCmd implements the default command for a tx query.
func QueryTxCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "tx --type={hash|acc_seq|signature} <hash|acc_seq|signature>",
		Short: "Query for a transaction by hash, \"<addr>/<seq>\" combination or comma-separated signatures in a committed block",
		Long: strings.TrimSpace(fmt.Sprintf(`
Example:
$ %s query tx <hash>
$ %s query tx --%s=%s <addr>/<sequence>
$ %s query tx --%s=%s <sig1_base64>,<sig2_base64...>
`,
			version.AppName,
			version.AppName, FlagType, TypeAccSeq,
			version.AppName, FlagType, TypeSig)),
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			typ, _ := cmd.Flags().GetString(FlagType)

			switch typ {
			case TypeHash:
				if args[0] == "" {
					return errors.New("argument should be a tx hash")
				}

				// if hash is given, then query the tx by hash
				output, err := authtx.QueryTx(clientCtx, args[0])
				if err != nil {
					return err
				}

				if output.Empty() {
					return fmt.Errorf("no transaction found with hash %s", args[0])
				}

				return clientCtx.PrintProto(output)

			case TypeSig:
				sigParts, err := ParseSigArgs(args)
				if err != nil {
					return err
				}

				events := make([]string, len(sigParts))
				for i, sig := range sigParts {
					events[i] = fmt.Sprintf("%s.%s='%s'", sdk.EventTypeTx, sdk.AttributeKeySignature, sig)
				}

				query := strings.Join(events, " AND ")

				txs, err := authtx.QueryTxsByEvents(clientCtx, querytypes.DefaultPage, querytypes.DefaultLimit, query, "")
				if err != nil {
					return err
				}

				if len(txs.Txs) == 0 {
					return errors.New("found no txs matching given signatures")
				}
				if len(txs.Txs) > 1 {
					// This case means there's a bug somewhere else in the code as this
					// should not happen.
					return sdkerrors.ErrLogic.Wrapf("found %d txs matching given signatures", len(txs.Txs))
				}

				return clientCtx.PrintProto(txs.Txs[0])

			case TypeAccSeq:
				if args[0] == "" {
					return errors.New("`acc_seq` type takes an argument '<addr>/<seq>'")
				}

				query := fmt.Sprintf("%s.%s='%s'", sdk.EventTypeTx, sdk.AttributeKeyAccountSequence, args[0])

				txs, err := authtx.QueryTxsByEvents(clientCtx, querytypes.DefaultPage, querytypes.DefaultLimit, query, "")
				if err != nil {
					return err
				}

				if len(txs.Txs) == 0 {
					return errors.New("found no txs matching given address and sequence combination")
				}
				if len(txs.Txs) > 1 {
					// This case means there's a bug somewhere else in the code as this
					// should not happen.
					return fmt.Errorf("found %d txs matching given address and sequence combination", len(txs.Txs))
				}

				return clientCtx.PrintProto(txs.Txs[0])

			default:
				return fmt.Errorf("unknown --%s value %s", FlagType, typ)
			}
		},
	}

	flags.AddQueryFlagsToCmd(cmd)
	cmd.Flags().String(FlagType, TypeHash, fmt.Sprintf("The type to be used when querying tx, can be one of \"%s\", \"%s\", \"%s\"", TypeHash, TypeAccSeq, TypeSig))

	return cmd
}

// ParseSigArgs parses comma-separated signatures from the CLI arguments.
func ParseSigArgs(args []string) ([]string, error) {
	if len(args) != 1 || args[0] == "" {
		return nil, errors.New("argument should be comma-separated signatures")
	}

	return strings.Split(args[0], ","), nil
}
