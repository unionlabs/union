package server

import (
	"context"
	"encoding/json"
	"fmt"
	"strconv"
	"strings"

	cmtcfg "github.com/cometbft/cometbft/config"
	cmtjson "github.com/cometbft/cometbft/libs/json"
	"github.com/cometbft/cometbft/node"
	"github.com/cometbft/cometbft/p2p"
	pvm "github.com/cometbft/cometbft/privval"
	cmtversion "github.com/cometbft/cometbft/version"
	"github.com/spf13/cobra"
	"sigs.k8s.io/yaml"

	"cosmossdk.io/log"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/client/rpc"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	"github.com/cosmos/cosmos-sdk/server/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/query"
	"github.com/cosmos/cosmos-sdk/version"
	auth "github.com/cosmos/cosmos-sdk/x/auth/client/cli"
)

// StatusCommand returns the command to return the status of the network.
func StatusCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "status",
		Short: "Query remote node for status",
		RunE: func(cmd *cobra.Command, _ []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}
			node, err := clientCtx.GetNode()
			if err != nil {
				return err
			}
			status, err := node.Status(context.Background())
			if err != nil {
				return err
			}

			output, err := cmtjson.Marshal(status)
			if err != nil {
				return err
			}

			// In order to maintain backwards compatibility, the default json format output
			outputFormat, _ := cmd.Flags().GetString(flags.FlagOutput)
			if outputFormat == flags.OutputFormatJSON {
				clientCtx = clientCtx.WithOutputFormat(flags.OutputFormatJSON)
			}

			return clientCtx.PrintRaw(output)
		},
	}

	cmd.Flags().StringP(flags.FlagNode, "n", "tcp://localhost:26657", "Node to connect to")
	cmd.Flags().StringP(flags.FlagOutput, "o", "json", "Output format (text|json)")

	return cmd
}

// ShowNodeIDCmd - ported from CometBFT, dump node ID to stdout
func ShowNodeIDCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "show-node-id",
		Short: "Show this node's ID",
		RunE: func(cmd *cobra.Command, args []string) error {
			serverCtx := GetServerContextFromCmd(cmd)
			cfg := serverCtx.Config

			nodeKey, err := p2p.LoadNodeKey(cfg.NodeKeyFile())
			if err != nil {
				return err
			}

			cmd.Println(nodeKey.ID())
			return nil
		},
	}
}

// ShowValidatorCmd - ported from CometBFT, show this node's validator info
func ShowValidatorCmd() *cobra.Command {
	cmd := cobra.Command{
		Use:   "show-validator",
		Short: "Show this node's CometBFT validator info",
		RunE: func(cmd *cobra.Command, args []string) error {
			serverCtx := GetServerContextFromCmd(cmd)
			cfg := serverCtx.Config

			privValidator := pvm.LoadFilePV(cfg.PrivValidatorKeyFile(), cfg.PrivValidatorStateFile())
			pk, err := privValidator.GetPubKey()
			if err != nil {
				return err
			}

			sdkPK, err := cryptocodec.FromCmtPubKeyInterface(pk)
			if err != nil {
				return err
			}

			clientCtx := client.GetClientContextFromCmd(cmd)
			bz, err := clientCtx.Codec.MarshalInterfaceJSON(sdkPK)
			if err != nil {
				return err
			}

			cmd.Println(string(bz))
			return nil
		},
	}

	return &cmd
}

// ShowAddressCmd - show this node's validator address
func ShowAddressCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "show-address",
		Short: "Shows this node's CometBFT validator consensus address",
		RunE: func(cmd *cobra.Command, args []string) error {
			serverCtx := GetServerContextFromCmd(cmd)
			cfg := serverCtx.Config

			privValidator := pvm.LoadFilePV(cfg.PrivValidatorKeyFile(), cfg.PrivValidatorStateFile())

			valConsAddr := (sdk.ConsAddress)(privValidator.GetAddress())

			cmd.Println(valConsAddr.String())
			return nil
		},
	}

	return cmd
}

// VersionCmd prints CometBFT and ABCI version numbers.
func VersionCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "version",
		Short: "Print CometBFT libraries' version",
		Long:  "Print protocols' and libraries' version numbers against which this app has been compiled.",
		RunE: func(cmd *cobra.Command, args []string) error {
			bs, err := yaml.Marshal(&struct {
				CometBFT      string
				ABCI          string
				BlockProtocol uint64
				P2PProtocol   uint64
			}{
				CometBFT:      cmtversion.CMTSemVer,
				ABCI:          cmtversion.ABCIVersion,
				BlockProtocol: cmtversion.BlockProtocol,
				P2PProtocol:   cmtversion.P2PProtocol,
			})
			if err != nil {
				return err
			}

			cmd.Println(string(bs))
			return nil
		},
	}
}

// QueryBlocksCmd returns a command to search through blocks by events.
func QueryBlocksCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "blocks",
		Short: "Query for paginated blocks that match a set of events",
		Long: `Search for blocks that match the exact given events where results are paginated.
The events query is directly passed to CometBFT's RPC BlockSearch method and must
conform to CometBFT's query syntax.
Please refer to each module's documentation for the full set of events to query
for. Each module documents its respective events under 'xx_events.md'.
`,
		Example: fmt.Sprintf(
			"$ %s query blocks --query \"message.sender='cosmos1...' AND block.height > 7\" --page 1 --limit 30 --order_by asc",
			version.AppName,
		),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}
			query, _ := cmd.Flags().GetString(auth.FlagQuery)
			page, _ := cmd.Flags().GetInt(flags.FlagPage)
			limit, _ := cmd.Flags().GetInt(flags.FlagLimit)
			orderBy, _ := cmd.Flags().GetString(auth.FlagOrderBy)

			blocks, err := rpc.QueryBlocks(clientCtx, page, limit, query, orderBy)
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(blocks)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)
	cmd.Flags().Int(flags.FlagPage, query.DefaultPage, "Query a specific page of paginated results")
	cmd.Flags().Int(flags.FlagLimit, query.DefaultLimit, "Query number of transactions results per page returned")
	cmd.Flags().String(auth.FlagQuery, "", "The blocks events query per CometBFT's query semantics")
	cmd.Flags().String(auth.FlagOrderBy, "", "The ordering semantics (asc|dsc)")
	_ = cmd.MarkFlagRequired(auth.FlagQuery)

	return cmd
}

// QueryBlockCmd implements the default command for a Block query.
func QueryBlockCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "block --type={height|hash} [height|hash]",
		Short: "Query for a committed block by height, hash, or event(s)",
		Long:  "Query for a specific committed block using the CometBFT RPC `block` and `block_by_hash` method",
		Example: strings.TrimSpace(fmt.Sprintf(`
$ %s query block --%s=%s <height>
$ %s query block --%s=%s <hash>
`,
			version.AppName, auth.FlagType, auth.TypeHeight,
			version.AppName, auth.FlagType, auth.TypeHash)),
		Args: cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			typ, _ := cmd.Flags().GetString(auth.FlagType)
			if len(args) == 0 {
				// do not break default v0.50 behavior of block hash
				// if no args are provided, set the type to height
				typ = auth.TypeHeight
			}

			switch typ {
			case auth.TypeHeight:
				var (
					err    error
					height int64
				)
				heightStr := ""
				if len(args) > 0 {
					heightStr = args[0]
				}

				if heightStr == "" {
					cmd.Println("Falling back to latest block height:")
					height, err = rpc.GetChainHeight(clientCtx)
					if err != nil {
						return fmt.Errorf("failed to get chain height: %w", err)
					}
				} else {
					height, err = strconv.ParseInt(heightStr, 10, 64)
					if err != nil {
						return fmt.Errorf("failed to parse block height: %w", err)
					}
				}

				output, err := rpc.GetBlockByHeight(clientCtx, &height)
				if err != nil {
					return err
				}

				if output.Header.Height == 0 {
					return fmt.Errorf("no block found with height %s", args[0])
				}

				return clientCtx.PrintProto(output)

			case auth.TypeHash:

				if args[0] == "" {
					return fmt.Errorf("argument should be a tx hash")
				}

				// If hash is given, then query the tx by hash.
				output, err := rpc.GetBlockByHash(clientCtx, args[0])
				if err != nil {
					return err
				}

				if output.Header.AppHash == nil {
					return fmt.Errorf("no block found with hash %s", args[0])
				}

				return clientCtx.PrintProto(output)

			default:
				return fmt.Errorf("unknown --%s value %s", auth.FlagType, typ)
			}
		},
	}

	flags.AddQueryFlagsToCmd(cmd)
	cmd.Flags().String(auth.FlagType, auth.TypeHash, fmt.Sprintf("The type to be used when querying tx, can be one of \"%s\", \"%s\"", auth.TypeHeight, auth.TypeHash))

	return cmd
}

// QueryBlockResultsCmd implements the default command for a BlockResults query.
func QueryBlockResultsCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "block-results [height]",
		Short: "Query for a committed block's results by height",
		Long:  "Query for a specific committed block's results using the CometBFT RPC `block_results` method",
		Args:  cobra.RangeArgs(0, 1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			node, err := clientCtx.GetNode()
			if err != nil {
				return err
			}

			// optional height
			var height int64
			if len(args) > 0 {
				height, err = strconv.ParseInt(args[0], 10, 64)
				if err != nil {
					return err
				}
			} else {
				cmd.Println("Falling back to latest block height:")
				height, err = rpc.GetChainHeight(clientCtx)
				if err != nil {
					return fmt.Errorf("failed to get chain height: %w", err)
				}
			}

			blockRes, err := node.BlockResults(context.Background(), &height)
			if err != nil {
				return err
			}

			// coretypes.ResultBlockResults doesn't implement proto.Message interface
			// so we can't print it using clientCtx.PrintProto
			// we choose to serialize it to json and print the json instead
			blockResStr, err := json.Marshal(blockRes)
			if err != nil {
				return err
			}

			return clientCtx.PrintRaw(blockResStr)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)

	return cmd
}

func BootstrapStateCmd[T types.Application](appCreator types.AppCreator[T]) *cobra.Command {
	cmd := &cobra.Command{
		Use:     "bootstrap-state",
		Short:   "Bootstrap CometBFT state at an arbitrary block height using a light client",
		Args:    cobra.NoArgs,
		Example: "bootstrap-state --height 1000000",
		RunE: func(cmd *cobra.Command, args []string) error {
			serverCtx := GetServerContextFromCmd(cmd)
			logger := log.NewLogger(cmd.OutOrStdout())
			cfg := serverCtx.Config

			height, err := cmd.Flags().GetInt64("height")
			if err != nil {
				return err
			}
			if height == 0 {
				home := serverCtx.Viper.GetString(flags.FlagHome)
				db, err := OpenDB(home, GetAppDBBackend(serverCtx.Viper))
				if err != nil {
					return err
				}

				app := appCreator(logger, db, nil, serverCtx.Viper)
				height = app.CommitMultiStore().LastCommitID().Version
			}

			return node.BootstrapState(cmd.Context(), cfg, cmtcfg.DefaultDBProvider, getGenDocProvider(cfg), uint64(height), nil)
		},
	}

	cmd.Flags().Int64("height", 0, "Block height to bootstrap state at, if not provided it uses the latest block height in app state")

	return cmd
}
