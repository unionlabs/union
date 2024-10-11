package validator

import (
	"fmt"
	"os"

	"github.com/logrusorgru/aurora"
	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/cmd/validator/accounts"
	"github.com/prysmaticlabs/prysm/v4/cmd/validator/flags"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	"github.com/prysmaticlabs/prysm/v4/runtime/tos"
	log "github.com/sirupsen/logrus"
	"github.com/urfave/cli/v2"
)

var (
	BeaconHostFlag = &cli.StringFlag{
		Name:  "beacon-node-host",
		Usage: "host:port for beacon node to query",
		Value: "127.0.0.1:3500",
	}

	PathFlag = &cli.StringFlag{
		Name:    "path",
		Aliases: []string{"p"},
		Usage:   "path to the signed withdrawal messages JSON",
	}

	ConfirmFlag = &cli.BoolFlag{
		Name:    "confirm",
		Aliases: []string{"c"},
		Usage: "WARNING: User confirms and accepts responsibility of all input data provided and actions for setting their withdrawal address for their validator key. " +
			"This action is not reversible and withdrawal addresses can not be changed once set.",
	}

	VerifyOnlyFlag = &cli.BoolFlag{
		Name:    "verify-only",
		Aliases: []string{"vo"},
		Usage:   "overrides withdrawal command to only verify whether requests are in the pool and does not submit withdrawal requests",
	}

	HostFlag = &cli.StringFlag{
		Name:    "validator-host",
		Aliases: []string{"vch"},
		Usage:   "host:port for validator client.",
		Value:   "http://127.0.0.1:7500",
	}

	ProposerSettingsOutputFlag = &cli.StringFlag{
		Name:    "output-proposer-settings-path",
		Aliases: []string{"settings-path"},
		Usage:   "path to outputting a proposer settings file ( i.e. ./path/to/proposer-settings.json), file does not include builder settings and will need to be added for advanced users using those features",
	}

	WithBuilderFlag = &cli.BoolFlag{
		Name:    "with-builder",
		Aliases: []string{"wb"},
		Usage:   "adds default builder options to proposer settings output, used for enabling mev-boost and relays",
	}

	DefaultFeeRecipientFlag = &cli.StringFlag{
		Name:    "default-fee-recipient",
		Aliases: []string{"dfr"},
		Usage:   "default fee recipient used for proposer-settings, only used with --output-proposer-settings-path",
	}

	TokenFlag = &cli.StringFlag{
		Name:    "token",
		Aliases: []string{"t"},
		Usage:   "keymanager API bearer token, note: currently required but may be removed in the future, this is the same token as the web ui token.",
	}
)

var Commands = []*cli.Command{
	{
		Name:    "validator",
		Aliases: []string{"v", "sign"}, // remove sign command should be depreciated but having as backwards compatibility.
		Usage:   "commands that affect the state of validators such as exiting or withdrawing",
		Subcommands: []*cli.Command{
			{
				Name:    "withdraw",
				Aliases: []string{"w"},
				Usage:   "Assign Ethereum withdrawal addresses to validator keys. WARNING: once set values are included they can no longer be updated.",
				Flags: []cli.Flag{
					BeaconHostFlag,
					PathFlag,
					ConfirmFlag,
					VerifyOnlyFlag,
					cmd.ConfigFileFlag,
					cmd.AcceptTosFlag,
				},
				Before: func(cliCtx *cli.Context) error {
					if err := cmd.LoadFlagsFromConfig(cliCtx, cliCtx.Command.Flags); err != nil {
						return err
					}
					au := aurora.NewAurora(true)
					if !cliCtx.Bool(cmd.AcceptTosFlag.Name) || !cliCtx.Bool(ConfirmFlag.Name) {
						fmt.Println(au.Red("===============IMPORTANT==============="))
						fmt.Println(au.Red("Please read the following carefully"))
						fmt.Print("This action will allow the partial withdrawal of amounts over the 32 staked ETH in your active validator balance. \n" +
							"You will also be entitled to the full withdrawal of the entire validator balance if your validator has exited. \n" +
							"Please navigate to our website (https://docs.prylabs.network/) and make sure you understand the full implications of setting your withdrawal address. \n")
						fmt.Println(au.Red("THIS ACTION WILL NOT BE REVERSIBLE ONCE INCLUDED. "))
						fmt.Println(au.Red("You will NOT be able to change the address again once changed. "))
						return fmt.Errorf("both the `--%s` and `--%s` flags are required to run this command. \n"+
							"By providing these flags the user has read and accepts the TERMS AND CONDITIONS: https://github.com/prysmaticlabs/prysm/blob/master/TERMS_OF_SERVICE.md "+
							"and confirms the action of setting withdrawals addresses", cmd.AcceptTosFlag.Name, ConfirmFlag.Name)
					} else {
						return nil
					}
				},
				Action: func(cliCtx *cli.Context) error {
					if cliCtx.Bool(VerifyOnlyFlag.Name) {
						if err := verifyWithdrawalsInPool(cliCtx); err != nil {
							log.WithError(err).Fatal("Could not verify withdrawal addresses")
						}
					} else {
						if err := setWithdrawalAddresses(cliCtx); err != nil {
							log.WithError(err).Fatal("Could not set withdrawal addresses")
						}
					}
					return nil
				},
			},
			{
				Name:    "proposer-settings",
				Aliases: []string{"ps"},
				Usage:   "Display or recreate currently used proposer settings.",
				Flags: []cli.Flag{
					cmd.ConfigFileFlag,
					DefaultFeeRecipientFlag,
					TokenFlag,
					HostFlag,
					ProposerSettingsOutputFlag,
				},
				Before: func(cliCtx *cli.Context) error {
					return cmd.LoadFlagsFromConfig(cliCtx, cliCtx.Command.Flags)
				},
				Action: func(cliCtx *cli.Context) error {
					if err := getProposerSettings(cliCtx, os.Stdin); err != nil {
						log.WithError(err).Fatal("Could not get proposer settings")
					}
					return nil
				},
			},
			{
				Name:    "exit",
				Aliases: []string{"e", "voluntary-exit"},
				Usage:   "Performs a voluntary exit on selected accounts",
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
					if err := accounts.Exit(cliCtx, os.Stdin); err != nil {
						log.WithError(err).Fatal("Could not perform voluntary exit")
					}
					return nil
				},
			},
		},
	},
}
