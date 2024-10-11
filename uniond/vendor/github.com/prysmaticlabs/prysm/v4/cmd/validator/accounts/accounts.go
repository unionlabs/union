package accounts

import (
	"os"

	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/cmd/validator/flags"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	"github.com/prysmaticlabs/prysm/v4/runtime/tos"
	"github.com/sirupsen/logrus"
	"github.com/urfave/cli/v2"
)

var log = logrus.WithField("prefix", "accounts")

// Commands for managing Prysm validator accounts.
var Commands = &cli.Command{
	Name:     "accounts",
	Category: "accounts",
	Usage:    "Defines commands for interacting with Ethereum validator accounts.",
	Subcommands: []*cli.Command{
		{
			Name:        "delete",
			Description: `deletes the selected accounts from a users wallet.`,
			Flags: cmd.WrapFlags([]cli.Flag{
				flags.WalletDirFlag,
				flags.WalletPasswordFileFlag,
				flags.DeletePublicKeysFlag,
				features.Mainnet,
				features.PraterTestnet,
				features.SepoliaTestnet,
				features.HoleskyTestnet,
				cmd.AcceptTosFlag,
			}),
			Before: func(cliCtx *cli.Context) error {
				if err := cmd.LoadFlagsFromConfig(cliCtx, cliCtx.Command.Flags); err != nil {
					return err
				}
				if err := tos.VerifyTosAcceptedOrPrompt(cliCtx); err != nil {
					return err
				}
				return features.ConfigureValidator(cliCtx)
			},
			Action: func(cliCtx *cli.Context) error {
				if err := accountsDelete(cliCtx); err != nil {
					log.WithError(err).Fatal("Could not delete account")
				}
				return nil
			},
		},
		{
			Name:        "list",
			Description: "Lists all validator accounts in a user's wallet directory",
			Flags: cmd.WrapFlags([]cli.Flag{
				flags.WalletDirFlag,
				flags.WalletPasswordFileFlag,
				flags.ShowDepositDataFlag,
				flags.ShowPrivateKeysFlag,
				flags.ListValidatorIndices,
				flags.BeaconRPCProviderFlag,
				cmd.GrpcMaxCallRecvMsgSizeFlag,
				flags.CertFlag,
				flags.GrpcHeadersFlag,
				flags.GrpcRetriesFlag,
				flags.GrpcRetryDelayFlag,
				features.Mainnet,
				features.PraterTestnet,
				features.SepoliaTestnet,
				features.HoleskyTestnet,
				cmd.AcceptTosFlag,
			}),
			Before: func(cliCtx *cli.Context) error {
				if err := cmd.LoadFlagsFromConfig(cliCtx, cliCtx.Command.Flags); err != nil {
					return err
				}
				if err := tos.VerifyTosAcceptedOrPrompt(cliCtx); err != nil {
					return err
				}
				return features.ConfigureValidator(cliCtx)
			},
			Action: func(cliCtx *cli.Context) error {
				if err := accountsList(cliCtx); err != nil {
					log.WithError(err).Fatal("Could not list accounts")
				}
				return nil
			},
		},
		{
			Name: "backup",
			Description: "backup accounts into EIP-2335 compliant keystore.json files zipped into a backup.zip file " +
				"at a desired output directory. Accounts to backup can also " +
				"be specified programmatically via a --backup-for-public-keys flag which specifies a comma-separated " +
				"list of hex string public keys",
			Flags: cmd.WrapFlags([]cli.Flag{
				flags.WalletDirFlag,
				flags.WalletPasswordFileFlag,
				flags.BackupDirFlag,
				flags.BackupPublicKeysFlag,
				flags.BackupPasswordFile,
				features.Mainnet,
				features.PraterTestnet,
				features.SepoliaTestnet,
				features.HoleskyTestnet,
				cmd.AcceptTosFlag,
			}),
			Before: func(cliCtx *cli.Context) error {
				if err := cmd.LoadFlagsFromConfig(cliCtx, cliCtx.Command.Flags); err != nil {
					return err
				}
				if err := tos.VerifyTosAcceptedOrPrompt(cliCtx); err != nil {
					return err
				}
				return features.ConfigureValidator(cliCtx)
			},
			Action: func(cliCtx *cli.Context) error {
				if err := accountsBackup(cliCtx); err != nil {
					log.WithError(err).Fatal("Could not backup accounts")
				}
				return nil
			},
		},
		{
			Name:        "import",
			Description: `imports Ethereum validator accounts stored in EIP-2335 keystore.json files from an external directory`,
			Flags: cmd.WrapFlags([]cli.Flag{
				flags.WalletDirFlag,
				flags.KeysDirFlag,
				flags.WalletPasswordFileFlag,
				flags.AccountPasswordFileFlag,
				flags.ImportPrivateKeyFileFlag,
				features.Mainnet,
				features.PraterTestnet,
				features.SepoliaTestnet,
				features.HoleskyTestnet,
				cmd.AcceptTosFlag,
			}),
			Before: func(cliCtx *cli.Context) error {
				if err := cmd.LoadFlagsFromConfig(cliCtx, cliCtx.Command.Flags); err != nil {
					return err
				}
				if err := tos.VerifyTosAcceptedOrPrompt(cliCtx); err != nil {
					return err
				}
				return features.ConfigureValidator(cliCtx)
			},
			Action: func(cliCtx *cli.Context) error {
				if err := accountsImport(cliCtx); err != nil {
					log.WithError(err).Fatal("Could not import accounts")
				}
				return nil
			},
		},
		{
			Name:        "voluntary-exit",
			Description: "Performs a voluntary exit on selected accounts",
			Flags: cmd.WrapFlags([]cli.Flag{
				flags.WalletDirFlag,
				flags.WalletPasswordFileFlag,
				flags.AccountPasswordFileFlag,
				flags.VoluntaryExitPublicKeysFlag,
				flags.BeaconRPCProviderFlag,
				flags.Web3SignerURLFlag,
				flags.Web3SignerPublicValidatorKeysFlag,
				flags.InteropNumValidators,
				flags.InteropStartIndex,
				cmd.GrpcMaxCallRecvMsgSizeFlag,
				flags.CertFlag,
				flags.GrpcHeadersFlag,
				flags.GrpcRetriesFlag,
				flags.GrpcRetryDelayFlag,
				flags.ExitAllFlag,
				flags.ForceExitFlag,
				flags.VoluntaryExitJSONOutputPath,
				features.Mainnet,
				features.PraterTestnet,
				features.SepoliaTestnet,
				features.HoleskyTestnet,
				cmd.AcceptTosFlag,
			}),
			Before: func(cliCtx *cli.Context) error {
				if err := cmd.LoadFlagsFromConfig(cliCtx, cliCtx.Command.Flags); err != nil {
					return err
				}
				if err := tos.VerifyTosAcceptedOrPrompt(cliCtx); err != nil {
					return err
				}
				return features.ConfigureValidator(cliCtx)
			},
			Action: func(cliCtx *cli.Context) error {
				log.Info("This command will be deprecated in the future in favor of `prysmctl validator exit`")
				if err := Exit(cliCtx, os.Stdin); err != nil {
					log.WithError(err).Fatal("Could not perform voluntary exit")
				}
				return nil
			},
		},
	},
}
