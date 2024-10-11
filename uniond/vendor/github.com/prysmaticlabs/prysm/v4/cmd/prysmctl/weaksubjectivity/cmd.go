package weaksubjectivity

import "github.com/urfave/cli/v2"

var Commands = []*cli.Command{
	{
		Name:    "weak-subjectivity",
		Aliases: []string{"ws"},
		Usage:   "commands dealing with weak subjectivity",
		Subcommands: []*cli.Command{
			checkpointCmd,
		},
	},
}
