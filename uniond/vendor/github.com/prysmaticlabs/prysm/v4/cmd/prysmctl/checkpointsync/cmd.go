package checkpointsync

import "github.com/urfave/cli/v2"

var Commands = []*cli.Command{
	{
		Name:    "checkpoint-sync",
		Aliases: []string{"cpt-sync"},
		Usage:   "commands for managing checkpoint sync",
		Subcommands: []*cli.Command{
			downloadCmd,
		},
	},
}
