package db

import (
	"fmt"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/kv"
	"github.com/urfave/cli/v2"
)

var bucketsFlags = struct {
	Path string
}{}

var bucketsCmd = &cli.Command{
	Name:   "buckets",
	Usage:  "list db buckets",
	Action: bucketsAction,
	Flags: []cli.Flag{
		&cli.StringFlag{
			Name:        "path",
			Usage:       "path to directory containing beaconchain.db",
			Destination: &bucketsFlags.Path,
		},
	},
}

func bucketsAction(_ *cli.Context) error {
	for _, b := range kv.Buckets {
		fmt.Printf("%s\n", string(b))
	}
	return nil
}
