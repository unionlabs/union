package weaksubjectivity

import (
	"context"
	"fmt"
	"time"

	"github.com/prysmaticlabs/prysm/v4/api/client"
	"github.com/prysmaticlabs/prysm/v4/api/client/beacon"
	log "github.com/sirupsen/logrus"
	"github.com/urfave/cli/v2"
)

var checkpointFlags = struct {
	BeaconNodeHost string
	Timeout        time.Duration
}{}

var checkpointCmd = &cli.Command{
	Name:    "checkpoint",
	Aliases: []string{"cpt"},
	Usage:   "Compute the latest weak subjectivity checkpoint (block_root:epoch) using trusted server data.",
	Action: func(cliCtx *cli.Context) error {
		if err := cliActionCheckpoint(cliCtx); err != nil {
			log.WithError(err).Fatal("Could not perform checkpoint-sync")
		}
		return nil
	},
	Flags: []cli.Flag{
		&cli.StringFlag{
			Name:        "beacon-node-host",
			Usage:       "host:port for beacon node to query",
			Destination: &checkpointFlags.BeaconNodeHost,
			Value:       "http://localhost:3500",
		},
		&cli.DurationFlag{
			Name:        "http-timeout",
			Usage:       "timeout for http requests made to beacon-node-url (uses duration format, ex: 2m31s). default: 2m",
			Destination: &checkpointFlags.Timeout,
			Value:       time.Minute * 2,
		},
	},
}

func cliActionCheckpoint(_ *cli.Context) error {
	ctx := context.Background()
	f := checkpointFlags

	opts := []client.ClientOpt{client.WithTimeout(f.Timeout)}
	client, err := beacon.NewClient(checkpointFlags.BeaconNodeHost, opts...)
	if err != nil {
		return err
	}

	ws, err := beacon.ComputeWeakSubjectivityCheckpoint(ctx, client)
	if err != nil {
		return err
	}
	fmt.Println("\nUse the following flag when starting a prysm Beacon Node to ensure the chain history " +
		"includes the Weak Subjectivity Checkpoint: ")
	fmt.Printf("--weak-subjectivity-checkpoint=%s\n\n", ws.CheckpointString())
	return nil
}
