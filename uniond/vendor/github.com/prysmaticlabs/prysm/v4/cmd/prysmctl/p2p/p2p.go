package p2p

import "github.com/urfave/cli/v2"

var Commands = []*cli.Command{
	{
		Name:  "p2p",
		Usage: "commands for interacting with beacon nodes via p2p",
		Subcommands: []*cli.Command{
			{
				Name:        "send",
				Usage:       "commands for sending p2p rpc requests to beacon nodes",
				Subcommands: []*cli.Command{requestBlocksCmd, requestBlobsCmd},
			},
		},
	},
}
