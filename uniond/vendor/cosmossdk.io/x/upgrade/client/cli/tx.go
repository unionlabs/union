package cli

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/spf13/cobra"

	"cosmossdk.io/x/gov/client/cli"
	"cosmossdk.io/x/upgrade/plan"
	"cosmossdk.io/x/upgrade/types"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/client/tx"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
)

const (
	FlagUpgradeHeight      = "upgrade-height"
	FlagUpgradeInfo        = "upgrade-info"
	FlagNoValidate         = "no-validate"
	FlagNoChecksumRequired = "no-checksum-required"
	FlagDaemonName         = "daemon-name"
	FlagAuthority          = "authority"
)

// GetTxCmd returns the transaction commands for this module
func GetTxCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   types.ModuleName,
		Short: "Upgrade transaction subcommands",
	}

	cmd.AddCommand(
		NewCmdSubmitUpgradeProposal(),
	)

	return cmd
}

// NewCmdSubmitUpgradeProposal implements a command handler for submitting a software upgrade proposal transaction.
// This commands is not migrated to autocli as it contains extra validation that is useful for submitting upgrade proposals.
func NewCmdSubmitUpgradeProposal() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "software-upgrade <name> [--upgrade-height <height>] [--upgrade-info <info>] [flags]",
		Args:  cobra.ExactArgs(1),
		Short: "Submit a software upgrade proposal",
		Long: "Submit a software upgrade along with an initial deposit.\n" +
			"Please specify a unique name and height for the upgrade to take effect.\n" +
			"You may include info to reference a binary download link, in a format compatible with: https://docs.cosmos.network/main/build/tooling/cosmovisor",
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			proposal, err := cli.ReadGovPropFlags(clientCtx, cmd.Flags())
			if err != nil {
				return err
			}

			name := args[0]
			p, err := parsePlan(cmd.Flags(), name)
			if err != nil {
				return err
			}

			noValidate, err := cmd.Flags().GetBool(FlagNoValidate)
			if err != nil {
				return err
			}

			if !noValidate {
				daemonName, err := cmd.Flags().GetString(FlagDaemonName)
				if err != nil {
					return err
				}

				noChecksum, err := cmd.Flags().GetBool(FlagNoChecksumRequired)
				if err != nil {
					return err
				}

				var planInfo *plan.Info
				if planInfo, err = plan.ParseInfo(p.Info, plan.ParseOptionEnforceChecksum(!noChecksum)); err != nil {
					return err
				}

				if err = planInfo.ValidateFull(daemonName); err != nil {
					return err
				}
			}

			authority, _ := cmd.Flags().GetString(FlagAuthority)
			if authority != "" {
				if _, err = clientCtx.AddressCodec.StringToBytes(authority); err != nil {
					return fmt.Errorf("invalid authority address: %w", err)
				}
			} else {
				if authority, err = clientCtx.AddressCodec.BytesToString(address.Module("gov")); err != nil {
					return fmt.Errorf("failed to convert authority address to string: %w", err)
				}
			}

			if err := proposal.SetMsgs([]sdk.Msg{
				&types.MsgSoftwareUpgrade{
					Authority: authority,
					Plan:      p,
				},
			}); err != nil {
				return fmt.Errorf("failed to create submit upgrade proposal message: %w", err)
			}

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), proposal)
		},
	}

	cmd.Flags().Int64(FlagUpgradeHeight, 0, "The height at which the upgrade must happen")
	cmd.Flags().String(FlagUpgradeInfo, "", "Info for the upgrade plan such as new version download urls, etc.")
	cmd.Flags().Bool(FlagNoValidate, false, "Skip validation of the upgrade info (dangerous!)")
	cmd.Flags().Bool(FlagNoChecksumRequired, false, "Skip requirement of checksums for binaries in the upgrade info")
	cmd.Flags().String(FlagDaemonName, getDefaultDaemonName(), "The name of the executable being upgraded (for upgrade-info validation). Default is the DAEMON_NAME env var if set, or else this executable")
	cmd.Flags().String(FlagAuthority, "", "The address of the upgrade module authority (defaults to gov)")

	// add common proposal flags
	flags.AddTxFlagsToCmd(cmd)
	cli.AddGovPropFlagsToCmd(cmd)
	err := cmd.MarkFlagRequired(cli.FlagTitle)
	if err != nil {
		panic(err)
	}

	return cmd
}

// getDefaultDaemonName gets the default name to use for the daemon.
// If a DAEMON_NAME env var is set, that is used.
// Otherwise, the last part of the currently running executable is used.
func getDefaultDaemonName() string {
	// DAEMON_NAME is specifically used here to correspond with the Cosmovisor setup env vars.
	name := os.Getenv("DAEMON_NAME")
	if len(name) == 0 {
		_, name = filepath.Split(os.Args[0])
	}
	return name
}
