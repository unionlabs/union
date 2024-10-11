package historycmd

import (
	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/cmd/validator/flags"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	"github.com/prysmaticlabs/prysm/v4/runtime/tos"
	"github.com/sirupsen/logrus"
	"github.com/urfave/cli/v2"
)

// Commands for slashing protection.
var Commands = &cli.Command{
	Name:     "slashing-protection-history",
	Category: "slashing-protection-history",
	Usage:    "Defines commands for interacting your validator's slashing protection history.",
	Subcommands: []*cli.Command{
		{
			Name:        "export",
			Description: `exports your validator slashing protection history into an EIP-3076 compliant JSON`,
			Flags: cmd.WrapFlags([]cli.Flag{
				cmd.DataDirFlag,
				flags.SlashingProtectionExportDirFlag,
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
				return tos.VerifyTosAcceptedOrPrompt(cliCtx)
			},
			Action: func(cliCtx *cli.Context) error {
				if err := features.ConfigureValidator(cliCtx); err != nil {
					return err
				}
				if err := exportSlashingProtectionJSON(cliCtx); err != nil {
					logrus.Fatalf("Could not export slashing protection file: %v", err)
				}
				return nil
			},
		},
		{
			Name:        "import",
			Description: `imports a selected EIP-3076 compliant slashing protection JSON to the validator database`,
			Flags: cmd.WrapFlags([]cli.Flag{
				cmd.DataDirFlag,
				flags.SlashingProtectionJSONFileFlag,
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
				return tos.VerifyTosAcceptedOrPrompt(cliCtx)
			},
			Action: func(cliCtx *cli.Context) error {
				if err := features.ConfigureValidator(cliCtx); err != nil {
					return err
				}
				err := importSlashingProtectionJSON(cliCtx)
				if err != nil {
					logrus.Fatalf("Could not import slashing protection cli: %v", err)
				}
				return nil
			},
		},
	},
}
