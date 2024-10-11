package checkpoint

import "github.com/urfave/cli/v2"

var Commands = []*cli.Command{
	{
		Name:    "checkpoint",
		Aliases: []string{"cpt"},
		Usage:   "deprecated",
		Subcommands: []*cli.Command{
			checkpointCmd,
			saveCmd,
		},
	},
}
