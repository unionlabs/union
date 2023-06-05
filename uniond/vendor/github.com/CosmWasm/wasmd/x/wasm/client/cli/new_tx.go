package cli

import (
	"strconv"

	errorsmod "cosmossdk.io/errors"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/client/tx"
	"github.com/spf13/cobra"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

// MigrateContractCmd will migrate a contract to a new code version
func MigrateContractCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "migrate [contract_addr_bech32] [new_code_id_int64] [json_encoded_migration_args]",
		Short:   "Migrate a wasm contract to a new code version",
		Aliases: []string{"update", "mig", "m"},
		Args:    cobra.ExactArgs(3),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			msg, err := parseMigrateContractArgs(args, clientCtx.GetFromAddress().String())
			if err != nil {
				return err
			}
			if err := msg.ValidateBasic(); err != nil {
				return nil
			}
			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), &msg)
		},
		SilenceUsage: true,
	}
	flags.AddTxFlagsToCmd(cmd)
	return cmd
}

func parseMigrateContractArgs(args []string, sender string) (types.MsgMigrateContract, error) {
	// get the id of the code to instantiate
	codeID, err := strconv.ParseUint(args[1], 10, 64)
	if err != nil {
		return types.MsgMigrateContract{}, errorsmod.Wrap(err, "code id")
	}

	migrateMsg := args[2]

	msg := types.MsgMigrateContract{
		Sender:   sender,
		Contract: args[0],
		CodeID:   codeID,
		Msg:      []byte(migrateMsg),
	}
	return msg, nil
}

// UpdateContractAdminCmd sets an new admin for a contract
func UpdateContractAdminCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "set-contract-admin [contract_addr_bech32] [new_admin_addr_bech32]",
		Short:   "Set new admin for a contract",
		Aliases: []string{"new-admin", "admin", "set-adm", "sa"},
		Args:    cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			msg := parseUpdateContractAdminArgs(args, clientCtx.GetFromAddress().String())
			if err := msg.ValidateBasic(); err != nil {
				return err
			}
			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), &msg)
		},
		SilenceUsage: true,
	}
	flags.AddTxFlagsToCmd(cmd)
	return cmd
}

func parseUpdateContractAdminArgs(args []string, sender string) types.MsgUpdateAdmin {
	msg := types.MsgUpdateAdmin{
		Sender:   sender,
		Contract: args[0],
		NewAdmin: args[1],
	}
	return msg
}

// ClearContractAdminCmd clears an admin for a contract
func ClearContractAdminCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "clear-contract-admin [contract_addr_bech32]",
		Short:   "Clears admin for a contract to prevent further migrations",
		Aliases: []string{"clear-admin", "clr-adm"},
		Args:    cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			msg := types.MsgClearAdmin{
				Sender:   clientCtx.GetFromAddress().String(),
				Contract: args[0],
			}
			if err := msg.ValidateBasic(); err != nil {
				return err
			}
			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), &msg)
		},
		SilenceUsage: true,
	}
	flags.AddTxFlagsToCmd(cmd)
	return cmd
}

// UpdateInstantiateConfigCmd updates instantiate config for a smart contract.
func UpdateInstantiateConfigCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "update-instantiate-config [code_id_int64]",
		Short:   "Update instantiate config for a codeID",
		Aliases: []string{"update-instantiate-config"},
		Args:    cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}
			codeID, err := strconv.ParseUint(args[0], 10, 64)
			if err != nil {
				return err
			}
			perm, err := parseAccessConfigFlags(cmd.Flags())
			if err != nil {
				return err
			}

			msg := types.MsgUpdateInstantiateConfig{
				Sender:                   clientCtx.GetFromAddress().String(),
				CodeID:                   codeID,
				NewInstantiatePermission: perm,
			}
			if err = msg.ValidateBasic(); err != nil {
				return err
			}
			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), &msg)
		},
		SilenceUsage: true,
	}

	addInstantiatePermissionFlags(cmd)
	flags.AddTxFlagsToCmd(cmd)
	return cmd
}
