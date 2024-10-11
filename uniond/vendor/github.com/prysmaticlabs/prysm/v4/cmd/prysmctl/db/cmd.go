package db

import "github.com/urfave/cli/v2"

var Commands = []*cli.Command{
	{
		Name:  "db",
		Usage: "commands to work with the prysm beacon db",
		Subcommands: []*cli.Command{
			queryCmd,
			bucketsCmd,
		},
	},
}
