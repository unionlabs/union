package cli

import (
	"bytes"
	"fmt"
	"net/url"
	"strconv"
	"strings"

	wasmvm "github.com/CosmWasm/wasmvm"
	"github.com/distribution/reference"
	"github.com/pkg/errors"
	"github.com/spf13/cobra"
	flag "github.com/spf13/pflag"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/client/tx"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	"github.com/cosmos/cosmos-sdk/version"
	"github.com/cosmos/cosmos-sdk/x/gov/client/cli"
	v1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"

	"github.com/CosmWasm/wasmd/x/wasm/ioutils"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

// DefaultGovAuthority is set to the gov module address.
// Extension point for chains to overwrite the default
var DefaultGovAuthority = sdk.AccAddress(address.Module("gov"))

func SubmitProposalCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:          "submit-proposal",
		Short:        "Submit a wasm proposal.",
		SilenceUsage: true,
	}
	cmd.AddCommand(
		ProposalStoreCodeCmd(),
		ProposalInstantiateContractCmd(),
		ProposalInstantiateContract2Cmd(),
		ProposalStoreAndInstantiateContractCmd(),
		ProposalMigrateContractCmd(),
		ProposalExecuteContractCmd(),
		ProposalSudoContractCmd(),
		ProposalUpdateContractAdminCmd(),
		ProposalClearContractAdminCmd(),
		ProposalPinCodesCmd(),
		ProposalUnpinCodesCmd(),
		ProposalUpdateInstantiateConfigCmd(),
		ProposalAddCodeUploadParamsAddresses(),
		ProposalRemoveCodeUploadParamsAddresses(),
		ProposalStoreAndMigrateContractCmd(),
	)
	return cmd
}

func ProposalStoreCodeCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "wasm-store [wasm file] --title [text] --summary [text] --authority [address]",
		Short: "Submit a wasm binary proposal",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, proposalTitle, summary, deposit, expedite, err := getProposalInfo(cmd)
			if err != nil {
				return err
			}
			authority, err := cmd.Flags().GetString(flagAuthority)
			if err != nil {
				return fmt.Errorf("authority: %s", err)
			}

			if len(authority) == 0 {
				return errors.New("authority address is required")
			}

			src, err := parseStoreCodeArgs(args[0], authority, cmd.Flags())
			if err != nil {
				return err
			}

			proposalMsg, err := v1.NewMsgSubmitProposal([]sdk.Msg{&src}, deposit, clientCtx.GetFromAddress().String(), "", proposalTitle, summary, expedite)
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), proposalMsg)
		},
		SilenceUsage: true,
	}
	addInstantiatePermissionFlags(cmd)

	// proposal flags
	addCommonProposalFlags(cmd)
	return cmd
}

func parseVerificationFlags(gzippedWasm []byte, flags *flag.FlagSet) (string, string, []byte, error) {
	source, err := flags.GetString(flagSource)
	if err != nil {
		return "", "", nil, fmt.Errorf("source: %s", err)
	}
	builder, err := flags.GetString(flagBuilder)
	if err != nil {
		return "", "", nil, fmt.Errorf("builder: %s", err)
	}
	codeHash, err := flags.GetBytesHex(flagCodeHash)
	if err != nil {
		return "", "", nil, fmt.Errorf("codeHash: %s", err)
	}

	// if any set require others to be set
	if len(source) != 0 || len(builder) != 0 || len(codeHash) != 0 {
		if source == "" {
			return "", "", nil, fmt.Errorf("source is required")
		}
		if _, err = url.ParseRequestURI(source); err != nil {
			return "", "", nil, fmt.Errorf("source: %s", err)
		}
		if builder == "" {
			return "", "", nil, fmt.Errorf("builder is required")
		}
		if _, err := reference.ParseDockerRef(builder); err != nil {
			return "", "", nil, fmt.Errorf("builder: %s", err)
		}
		if len(codeHash) == 0 {
			return "", "", nil, fmt.Errorf("code hash is required")
		}
		// wasm is gzipped in parseStoreCodeArgs
		// checksum generation will be decoupled here
		// reference https://github.com/CosmWasm/wasmvm/issues/359
		raw, err := ioutils.Uncompress(gzippedWasm, int64(types.MaxWasmSize))
		if err != nil {
			return "", "", nil, fmt.Errorf("invalid zip: %w", err)
		}
		checksum, err := wasmvm.CreateChecksum(raw)
		if err != nil {
			return "", "", nil, fmt.Errorf("checksum: %s", err)
		}
		if !bytes.Equal(checksum[:], codeHash) {
			return "", "", nil, fmt.Errorf("code-hash mismatch: %X, checksum: %X", codeHash, checksum)
		}
	}
	return source, builder, codeHash, nil
}

func ProposalInstantiateContractCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "instantiate-contract [code_id_int64] [json_encoded_init_args] --authority [address] --label [text] --title [text] --summary [text] --admin [address,optional] --amount [coins,optional]",
		Short: "Submit an instantiate wasm contract proposal",
		Args:  cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, proposalTitle, summary, deposit, expedite, err := getProposalInfo(cmd)
			if err != nil {
				return err
			}

			authority, err := cmd.Flags().GetString(flagAuthority)
			if err != nil {
				return fmt.Errorf("authority: %s", err)
			}

			if len(authority) == 0 {
				return errors.New("authority address is required")
			}

			src, err := parseInstantiateArgs(args[0], args[1], clientCtx.Keyring, authority, cmd.Flags())
			if err != nil {
				return err
			}

			proposalMsg, err := v1.NewMsgSubmitProposal([]sdk.Msg{src}, deposit, clientCtx.GetFromAddress().String(), "", proposalTitle, summary, expedite)
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), proposalMsg)
		},
		SilenceUsage: true,
	}
	cmd.Flags().String(flagAmount, "", "Coins to send to the contract during instantiation")
	cmd.Flags().String(flagLabel, "", "A human-readable name for this contract in lists")
	cmd.Flags().String(flagAdmin, "", "Address or key name of an admin")
	cmd.Flags().Bool(flagNoAdmin, false, "You must set this explicitly if you don't want an admin")

	// proposal flags
	addCommonProposalFlags(cmd)
	return cmd
}

func ProposalInstantiateContract2Cmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "instantiate-contract-2 [code_id_int64] [json_encoded_init_args] --authority [address] --label [text] --title [text] --summary [text]  --admin [address,optional] --amount [coins,optional]",
		Short: "Submit an instantiate wasm contract proposal with predictable address",
		Args:  cobra.ExactArgs(3),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, proposalTitle, summary, deposit, expedite, err := getProposalInfo(cmd)
			if err != nil {
				return err
			}

			authority, err := cmd.Flags().GetString(flagAuthority)
			if err != nil {
				return fmt.Errorf("authority: %s", err)
			}

			if len(authority) == 0 {
				return errors.New("authority address is required")
			}

			src, err := parseInstantiateArgs(args[0], args[1], clientCtx.Keyring, authority, cmd.Flags())
			if err != nil {
				return err
			}

			proposalMsg, err := v1.NewMsgSubmitProposal([]sdk.Msg{src}, deposit, clientCtx.GetFromAddress().String(), "", proposalTitle, summary, expedite)
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), proposalMsg)
		},
		SilenceUsage: true,
	}

	cmd.Flags().String(flagAmount, "", "Coins to send to the contract during instantiation")
	cmd.Flags().String(flagLabel, "", "A human-readable name for this contract in lists")
	cmd.Flags().String(flagAdmin, "", "Address of an admin")
	cmd.Flags().Bool(flagNoAdmin, false, "You must set this explicitly if you don't want an admin")

	// proposal flags
	addCommonProposalFlags(cmd)
	return cmd
}

func ProposalStoreAndInstantiateContractCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use: "store-instantiate [wasm file] [json_encoded_init_args] --authority [address] --label [text] --title [text] --summary [text]" +
			"--unpin-code [unpin_code,optional] --source [source,optional] --builder [builder,optional] --code-hash [code_hash,optional] --admin [address,optional] --amount [coins,optional]",
		Short: "Submit a store and instantiate wasm contract proposal",
		Args:  cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, proposalTitle, summary, deposit, expedite, err := getProposalInfo(cmd)
			if err != nil {
				return err
			}

			authority, err := cmd.Flags().GetString(flagAuthority)
			if err != nil {
				return fmt.Errorf("authority: %s", err)
			}

			if len(authority) == 0 {
				return errors.New("authority address is required")
			}

			src, err := parseStoreCodeArgs(args[0], authority, cmd.Flags())
			if err != nil {
				return err
			}

			unpinCode, err := cmd.Flags().GetBool(flagUnpinCode)
			if err != nil {
				return err
			}

			source, builder, codeHash, err := parseVerificationFlags(src.WASMByteCode, cmd.Flags())
			if err != nil {
				return err
			}

			amountStr, err := cmd.Flags().GetString(flagAmount)
			if err != nil {
				return fmt.Errorf("amount: %s", err)
			}
			amount, err := sdk.ParseCoinsNormalized(amountStr)
			if err != nil {
				return fmt.Errorf("amount: %s", err)
			}
			label, err := cmd.Flags().GetString(flagLabel)
			if err != nil {
				return fmt.Errorf("label: %s", err)
			}
			if label == "" {
				return errors.New("label is required on all contracts")
			}
			adminStr, err := cmd.Flags().GetString(flagAdmin)
			if err != nil {
				return fmt.Errorf("admin: %s", err)
			}
			noAdmin, err := cmd.Flags().GetBool(flagNoAdmin)
			if err != nil {
				return fmt.Errorf("no-admin: %s", err)
			}

			// ensure sensible admin is set (or explicitly immutable)
			if adminStr == "" && !noAdmin {
				return fmt.Errorf("you must set an admin or explicitly pass --no-admin to make it immutible (wasmd issue #719)")
			}
			if adminStr != "" && noAdmin {
				return fmt.Errorf("you set an admin and passed --no-admin, those cannot both be true")
			}

			if adminStr != "" {
				addr, err := sdk.AccAddressFromBech32(adminStr)
				if err != nil {
					info, err := clientCtx.Keyring.Key(adminStr)
					if err != nil {
						return fmt.Errorf("admin %s", err)
					}
					admin, err := info.GetAddress()
					if err != nil {
						return err
					}
					adminStr = admin.String()
				} else {
					adminStr = addr.String()
				}
			}

			msg := types.MsgStoreAndInstantiateContract{
				Authority:             authority,
				WASMByteCode:          src.WASMByteCode,
				InstantiatePermission: src.InstantiatePermission,
				UnpinCode:             unpinCode,
				Source:                source,
				Builder:               builder,
				CodeHash:              codeHash,
				Admin:                 adminStr,
				Label:                 label,
				Msg:                   []byte(args[1]),
				Funds:                 amount,
			}
			if err = msg.ValidateBasic(); err != nil {
				return err
			}

			proposalMsg, err := v1.NewMsgSubmitProposal([]sdk.Msg{&msg}, deposit, clientCtx.GetFromAddress().String(), "", proposalTitle, summary, expedite)
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), proposalMsg)
		},
		SilenceUsage: true,
	}

	cmd.Flags().Bool(flagUnpinCode, false, "Unpin code on upload, optional")
	cmd.Flags().String(flagSource, "", "Code Source URL is a valid absolute HTTPS URI to the contract's source code,")
	cmd.Flags().String(flagBuilder, "", "Builder is a valid docker image name with tag, such as \"cosmwasm/workspace-optimizer:0.12.9\"")
	cmd.Flags().BytesHex(flagCodeHash, nil, "CodeHash is the sha256 hash of the wasm code")
	cmd.Flags().String(flagAmount, "", "Coins to send to the contract during instantiation")
	cmd.Flags().String(flagLabel, "", "A human-readable name for this contract in lists")
	cmd.Flags().String(flagAdmin, "", "Address or key name of an admin")
	cmd.Flags().Bool(flagNoAdmin, false, "You must set this explicitly if you don't want an admin")
	addInstantiatePermissionFlags(cmd)
	// proposal flags
	addCommonProposalFlags(cmd)
	return cmd
}

func ProposalMigrateContractCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "migrate-contract [contract_addr_bech32] [new_code_id_int64] [json_encoded_migration_args] --title [text] --summary [text] --authority [address]",
		Short: "Submit a migrate wasm contract to a new code version proposal",
		Args:  cobra.ExactArgs(3),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, proposalTitle, summary, deposit, expedite, err := getProposalInfo(cmd)
			if err != nil {
				return err
			}

			authority, err := cmd.Flags().GetString(flagAuthority)
			if err != nil {
				return fmt.Errorf("authority: %s", err)
			}

			if len(authority) == 0 {
				return errors.New("authority address is required")
			}

			src, err := parseMigrateContractArgs(args, authority)
			if err != nil {
				return err
			}

			proposalMsg, err := v1.NewMsgSubmitProposal([]sdk.Msg{&src}, deposit, clientCtx.GetFromAddress().String(), "", proposalTitle, summary, expedite)
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), proposalMsg)
		},
		SilenceUsage: true,
	}
	// proposal flags
	addCommonProposalFlags(cmd)
	return cmd
}

func ProposalExecuteContractCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "execute-contract [contract_addr_bech32] [json_encoded_execution_args] --title [text] --summary [text] --authority [address]",
		Short: "Submit a execute wasm contract proposal (run by any address)",
		Args:  cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, proposalTitle, summary, deposit, expedite, err := getProposalInfo(cmd)
			if err != nil {
				return err
			}

			authority, err := cmd.Flags().GetString(flagAuthority)
			if err != nil {
				return fmt.Errorf("authority: %s", err)
			}

			if len(authority) == 0 {
				return errors.New("authority address is required")
			}

			contract := args[0]
			execMsg := []byte(args[1])
			amountStr, err := cmd.Flags().GetString(flagAmount)
			if err != nil {
				return fmt.Errorf("amount: %s", err)
			}
			funds, err := sdk.ParseCoinsNormalized(amountStr)
			if err != nil {
				return fmt.Errorf("amount: %s", err)
			}

			msg := types.MsgExecuteContract{
				Sender:   authority,
				Contract: contract,
				Msg:      execMsg,
				Funds:    funds,
			}
			if err = msg.ValidateBasic(); err != nil {
				return err
			}

			proposalMsg, err := v1.NewMsgSubmitProposal([]sdk.Msg{&msg}, deposit, clientCtx.GetFromAddress().String(), "", proposalTitle, summary, expedite)
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), proposalMsg)
		},
		SilenceUsage: true,
	}
	cmd.Flags().String(flagAmount, "", "Coins to send to the contract during instantiation")

	// proposal flags
	addCommonProposalFlags(cmd)
	return cmd
}

func ProposalSudoContractCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "sudo-contract [contract_addr_bech32] [json_encoded_migration_args] --title [text] --summary [text] --authority [address]",
		Short: "Submit a sudo wasm contract proposal (to call privileged commands)",
		Args:  cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, proposalTitle, summary, deposit, expedite, err := getProposalInfo(cmd)
			if err != nil {
				return err
			}

			authority, err := cmd.Flags().GetString(flagAuthority)
			if err != nil {
				return fmt.Errorf("authority: %s", err)
			}

			if len(authority) == 0 {
				return errors.New("authority address is required")
			}

			msg := types.MsgSudoContract{
				Authority: authority,
				Contract:  args[0],
				Msg:       []byte(args[1]),
			}
			if err = msg.ValidateBasic(); err != nil {
				return err
			}

			proposalMsg, err := v1.NewMsgSubmitProposal([]sdk.Msg{&msg}, deposit, clientCtx.GetFromAddress().String(), "", proposalTitle, summary, expedite)
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), proposalMsg)
		},
		SilenceUsage: true,
	}
	// proposal flagsExecute
	addCommonProposalFlags(cmd)
	return cmd
}

func ProposalUpdateContractAdminCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "set-contract-admin [contract_addr_bech32] [new_admin_addr_bech32] --title [text] --summary [text] --authority [address]",
		Short: "Submit a new admin for a contract proposal",
		Args:  cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, proposalTitle, summary, deposit, expedite, err := getProposalInfo(cmd)
			if err != nil {
				return err
			}

			authority, err := cmd.Flags().GetString(flagAuthority)
			if err != nil {
				return fmt.Errorf("authority: %s", err)
			}

			if len(authority) == 0 {
				return errors.New("authority address is required")
			}

			src, err := parseUpdateContractAdminArgs(args, authority)
			if err != nil {
				return err
			}

			proposalMsg, err := v1.NewMsgSubmitProposal([]sdk.Msg{&src}, deposit, clientCtx.GetFromAddress().String(), "", proposalTitle, summary, expedite)
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), proposalMsg)
		},
		SilenceUsage: true,
	}
	// proposal flags
	addCommonProposalFlags(cmd)
	return cmd
}

func ProposalClearContractAdminCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "clear-contract-admin [contract_addr_bech32] --title [text] --summary [text] --authority [address]",
		Short: "Submit a clear admin for a contract to prevent further migrations proposal",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, proposalTitle, summary, deposit, expedite, err := getProposalInfo(cmd)
			if err != nil {
				return err
			}

			authority, err := cmd.Flags().GetString(flagAuthority)
			if err != nil {
				return fmt.Errorf("authority: %s", err)
			}

			if len(authority) == 0 {
				return errors.New("authority address is required")
			}

			msg := types.MsgClearAdmin{
				Sender:   authority,
				Contract: args[0],
			}
			if err = msg.ValidateBasic(); err != nil {
				return err
			}

			proposalMsg, err := v1.NewMsgSubmitProposal([]sdk.Msg{&msg}, deposit, clientCtx.GetFromAddress().String(), "", proposalTitle, summary, expedite)
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), proposalMsg)
		},
		SilenceUsage: true,
	}
	// proposal flags
	addCommonProposalFlags(cmd)
	return cmd
}

func ProposalPinCodesCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "pin-codes [code-ids] --title [text] --summary [text] --authority [address]",
		Short: "Submit a pin code proposal for pinning a code to cache",
		Args:  cobra.MinimumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, proposalTitle, summary, deposit, expedite, err := getProposalInfo(cmd)
			if err != nil {
				return err
			}

			authority, err := cmd.Flags().GetString(flagAuthority)
			if err != nil {
				return fmt.Errorf("authority: %s", err)
			}

			if len(authority) == 0 {
				return errors.New("authority address is required")
			}

			codeIds, err := parsePinCodesArgs(args)
			if err != nil {
				return err
			}

			msg := types.MsgPinCodes{
				Authority: authority,
				CodeIDs:   codeIds,
			}
			if err = msg.ValidateBasic(); err != nil {
				return err
			}

			proposalMsg, err := v1.NewMsgSubmitProposal([]sdk.Msg{&msg}, deposit, clientCtx.GetFromAddress().String(), "", proposalTitle, summary, expedite)
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), proposalMsg)
		},
		SilenceUsage: true,
	}
	// proposal flags
	addCommonProposalFlags(cmd)
	return cmd
}

func parsePinCodesArgs(args []string) ([]uint64, error) {
	codeIDs := make([]uint64, len(args))
	for i, c := range args {
		codeID, err := strconv.ParseUint(c, 10, 64)
		if err != nil {
			return codeIDs, fmt.Errorf("code IDs: %s", err)
		}
		codeIDs[i] = codeID
	}
	return codeIDs, nil
}

func ProposalUnpinCodesCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "unpin-codes [code-ids] --title [text] --summary [text] --authority [address]",
		Short: "Submit a unpin code proposal for unpinning a code to cache",
		Args:  cobra.MinimumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, proposalTitle, summary, deposit, expedite, err := getProposalInfo(cmd)
			if err != nil {
				return err
			}
			authority, err := cmd.Flags().GetString(flagAuthority)
			if err != nil {
				return fmt.Errorf("authority: %s", err)
			}

			if len(authority) == 0 {
				return errors.New("authority address is required")
			}

			codeIds, err := parsePinCodesArgs(args)
			if err != nil {
				return err
			}

			msg := types.MsgUnpinCodes{
				Authority: authority,
				CodeIDs:   codeIds,
			}
			if err = msg.ValidateBasic(); err != nil {
				return err
			}

			proposalMsg, err := v1.NewMsgSubmitProposal([]sdk.Msg{&msg}, deposit, clientCtx.GetFromAddress().String(), "", proposalTitle, summary, expedite)
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), proposalMsg)
		},
		SilenceUsage: true,
	}
	// proposal flags
	addCommonProposalFlags(cmd)
	return cmd
}

func parseAccessConfig(raw string) (c types.AccessConfig, err error) {
	switch raw {
	case "nobody":
		return types.AllowNobody, nil
	case "everybody":
		return types.AllowEverybody, nil
	default:
		parts := strings.Split(raw, ",")
		addrs := make([]sdk.AccAddress, len(parts))
		for i, v := range parts {
			addr, err := sdk.AccAddressFromBech32(v)
			if err != nil {
				return types.AccessConfig{}, fmt.Errorf("unable to parse address %q: %s", v, err)
			}
			addrs[i] = addr
		}
		defer func() { // convert panic in ".With" to error for better output
			if r := recover(); r != nil {
				err = r.(error)
			}
		}()
		cfg := types.AccessTypeAnyOfAddresses.With(addrs...)
		return cfg, cfg.ValidateBasic()
	}
}

func parseAccessConfigUpdates(args []string) ([]types.AccessConfigUpdate, error) {
	updates := make([]types.AccessConfigUpdate, len(args))
	for i, c := range args {
		// format: code_id:access_config
		// access_config: nobody|everybody|address(es)
		parts := strings.Split(c, ":")
		if len(parts) != 2 {
			return nil, fmt.Errorf("invalid format")
		}

		codeID, err := strconv.ParseUint(parts[0], 10, 64)
		if err != nil {
			return nil, fmt.Errorf("invalid code ID: %s", err)
		}

		accessConfig, err := parseAccessConfig(parts[1])
		if err != nil {
			return nil, err
		}
		updates[i] = types.AccessConfigUpdate{
			CodeID:                codeID,
			InstantiatePermission: accessConfig,
		}
	}
	return updates, nil
}

func ProposalUpdateInstantiateConfigCmd() *cobra.Command {
	bech32Prefix := sdk.GetConfig().GetBech32AccountAddrPrefix()
	cmd := &cobra.Command{
		Use:   "update-instantiate-config [code-id:permission] --title [text] --summary [text] --authority [address]",
		Short: "Submit an update instantiate config proposal.",
		Args:  cobra.MinimumNArgs(1),
		Long: strings.TrimSpace(
			fmt.Sprintf(`Submit an update instantiate config  proposal for multiple code ids.

Example: 
$ %s tx gov submit-proposal update-instantiate-config 1:nobody 2:everybody 3:%s1l2rsakp388kuv9k8qzq6lrm9taddae7fpx59wm,%s1vx8knpllrj7n963p9ttd80w47kpacrhuts497x
`, version.AppName, bech32Prefix, bech32Prefix)),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, proposalTitle, summary, deposit, expedite, err := getProposalInfo(cmd)
			if err != nil {
				return err
			}

			authority, err := cmd.Flags().GetString(flagAuthority)
			if err != nil {
				return fmt.Errorf("authority: %s", err)
			}

			if len(authority) == 0 {
				return errors.New("authority address is required")
			}

			updates, err := parseAccessConfigUpdates(args)
			if err != nil {
				return err
			}

			msgs := make([]sdk.Msg, len(updates))
			for i, update := range updates {
				permission := update.InstantiatePermission
				msg := &types.MsgUpdateInstantiateConfig{
					Sender:                   authority,
					CodeID:                   update.CodeID,
					NewInstantiatePermission: &permission,
				}
				if err = msg.ValidateBasic(); err != nil {
					return err
				}
				msgs[i] = msg
			}

			proposalMsg, err := v1.NewMsgSubmitProposal(msgs, deposit, clientCtx.GetFromAddress().String(), "", proposalTitle, summary, expedite)
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), proposalMsg)
		},
		SilenceUsage: true,
	}
	// proposal flags
	addCommonProposalFlags(cmd)
	return cmd
}

func ProposalAddCodeUploadParamsAddresses() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "add-code-upload-params-addresses [addresses] --title [text] --summary [text] --authority [address]",
		Short: "Submit an add code upload params addresses proposal to add addresses to code upload config params",
		Args:  cobra.MinimumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, proposalTitle, summary, deposit, expedite, err := getProposalInfo(cmd)
			if err != nil {
				return err
			}

			authority, err := cmd.Flags().GetString(flagAuthority)
			if err != nil {
				return fmt.Errorf("authority: %s", err)
			}

			if len(authority) == 0 {
				return errors.New("authority address is required")
			}

			msg := types.MsgAddCodeUploadParamsAddresses{
				Authority: authority,
				Addresses: args,
			}

			proposalMsg, err := v1.NewMsgSubmitProposal([]sdk.Msg{&msg}, deposit, clientCtx.GetFromAddress().String(), "", proposalTitle, summary, expedite)
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), proposalMsg)
		},
		SilenceUsage: true,
	}
	// proposal flags
	addCommonProposalFlags(cmd)
	return cmd
}

func ProposalRemoveCodeUploadParamsAddresses() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "remove-code-upload-params-addresses [addresses] --title [text] --summary [text] --authority [address]",
		Short: "Submit a remove code upload params addresses proposal to remove addresses from code upload config params",
		Args:  cobra.MinimumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, proposalTitle, summary, deposit, expedite, err := getProposalInfo(cmd)
			if err != nil {
				return err
			}

			authority, err := cmd.Flags().GetString(flagAuthority)
			if err != nil {
				return fmt.Errorf("authority: %s", err)
			}

			if len(authority) == 0 {
				return errors.New("authority address is required")
			}

			msg := types.MsgRemoveCodeUploadParamsAddresses{
				Authority: authority,
				Addresses: args,
			}

			proposalMsg, err := v1.NewMsgSubmitProposal([]sdk.Msg{&msg}, deposit, clientCtx.GetFromAddress().String(), "", proposalTitle, summary, expedite)
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), proposalMsg)
		},
		SilenceUsage: true,
	}
	// proposal flags
	addCommonProposalFlags(cmd)
	return cmd
}

func addCommonProposalFlags(cmd *cobra.Command) {
	flags.AddTxFlagsToCmd(cmd)
	cmd.Flags().String(cli.FlagTitle, "", "Title of proposal")
	cmd.Flags().String(cli.FlagSummary, "", "Summary of proposal")
	cmd.Flags().String(cli.FlagDeposit, "", "Deposit of proposal")
	cmd.Flags().String(flagAuthority, DefaultGovAuthority.String(), "The address of the governance account. Default is the sdk gov module account")
	cmd.Flags().Bool(flagExpedite, false, "Expedite proposals have shorter voting period but require higher voting threshold")
}

func getProposalInfo(cmd *cobra.Command) (client.Context, string, string, sdk.Coins, bool, error) {
	clientCtx, err := client.GetClientTxContext(cmd)
	if err != nil {
		return client.Context{}, "", "", nil, false, err
	}

	proposalTitle, err := cmd.Flags().GetString(cli.FlagTitle)
	if err != nil {
		return clientCtx, proposalTitle, "", nil, false, err
	}

	summary, err := cmd.Flags().GetString(cli.FlagSummary)
	if err != nil {
		return client.Context{}, proposalTitle, summary, nil, false, err
	}

	depositArg, err := cmd.Flags().GetString(cli.FlagDeposit)
	if err != nil {
		return client.Context{}, proposalTitle, summary, nil, false, err
	}

	deposit, err := sdk.ParseCoinsNormalized(depositArg)
	if err != nil {
		return client.Context{}, proposalTitle, summary, deposit, false, err
	}

	expedite, err := cmd.Flags().GetBool(flagExpedite)
	if err != nil {
		return client.Context{}, proposalTitle, summary, deposit, false, err
	}

	return clientCtx, proposalTitle, summary, deposit, expedite, nil
}

func ProposalStoreAndMigrateContractCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "store-migrate [wasm file] [contract_addr_bech32] [json_encoded_migration_args] --title [text] --summary [text] --authority [address]",
		Short: "Submit a store and migrate wasm contract proposal",
		Args:  cobra.ExactArgs(3),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, proposalTitle, summary, deposit, expedite, err := getProposalInfo(cmd)
			if err != nil {
				return err
			}

			authority, err := cmd.Flags().GetString(flagAuthority)
			if err != nil {
				return fmt.Errorf("authority: %s", err)
			}

			if len(authority) == 0 {
				return errors.New("authority address is required")
			}

			src, err := parseStoreCodeArgs(args[0], authority, cmd.Flags())
			if err != nil {
				return err
			}

			msg := types.MsgStoreAndMigrateContract{
				Authority:             authority,
				WASMByteCode:          src.WASMByteCode,
				InstantiatePermission: src.InstantiatePermission,
				Msg:                   []byte(args[2]),
				Contract:              args[1],
			}

			proposalMsg, err := v1.NewMsgSubmitProposal([]sdk.Msg{&msg}, deposit, clientCtx.GetFromAddress().String(), "", proposalTitle, summary, expedite)
			if err != nil {
				return err
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), proposalMsg)
		},
		SilenceUsage: true,
	}

	addInstantiatePermissionFlags(cmd)
	// proposal flags
	addCommonProposalFlags(cmd)
	return cmd
}
