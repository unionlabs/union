package db

import (
	beacondb "github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/runtime/tos"
	"github.com/sirupsen/logrus"
	"github.com/urfave/cli/v2"
)

var log = logrus.WithField("prefix", "db")

// Commands for interacting with a beacon chain database.
var Commands = &cli.Command{
	Name:     "db",
	Category: "db",
	Usage:    "Defines commands for interacting with the Ethereum Beacon Node database",
	Subcommands: []*cli.Command{
		{
			Name:        "restore",
			Description: `restores a database from a backup file`,
			Flags: cmd.WrapFlags([]cli.Flag{
				cmd.RestoreSourceFileFlag,
				cmd.RestoreTargetDirFlag,
			}),
			Before: tos.VerifyTosAcceptedOrPrompt,
			Action: func(cliCtx *cli.Context) error {
				if err := beacondb.Restore(cliCtx); err != nil {
					log.WithError(err).Fatal("Could not restore database")
				}
				return nil
			},
		},
	},
}
