package checkpoint

import (
	"fmt"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/node"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/sync/checkpoint"
	"github.com/urfave/cli/v2"
)

var (
	// StatePath defines a flag to start the beacon chain from a give genesis state file.
	StatePath = &cli.PathFlag{
		Name: "checkpoint-state",
		Usage: "Rather than syncing from genesis, you can start processing from a ssz-serialized BeaconState+Block." +
			" This flag allows you to specify a local file containing the checkpoint BeaconState to load.",
	}
	// BlockPath is required when using StatePath to also provide the latest integrated block.
	BlockPath = &cli.PathFlag{
		Name: "checkpoint-block",
		Usage: "Rather than syncing from genesis, you can start processing from a ssz-serialized BeaconState+Block." +
			" This flag allows you to specify a local file containing the checkpoint Block to load.",
	}
	RemoteURL = &cli.StringFlag{
		Name: "checkpoint-sync-url",
		Usage: "URL of a synced beacon node to trust in obtaining checkpoint sync data. " +
			"As an additional safety measure, it is strongly recommended to only use this option in conjunction with " +
			"--weak-subjectivity-checkpoint flag",
	}
)

// BeaconNodeOptions is responsible for determining if the checkpoint sync options have been used, and if so,
// reading the block and state ssz-serialized values from the filesystem locations specified and preparing a
// checkpoint.Initializer, which uses the provided io.ReadClosers to initialize the beacon node database.
func BeaconNodeOptions(c *cli.Context) ([]node.Option, error) {
	blockPath := c.Path(BlockPath.Name)
	statePath := c.Path(StatePath.Name)
	remoteURL := c.String(RemoteURL.Name)
	if remoteURL != "" {
		opt := func(node *node.BeaconNode) error {
			var err error
			node.CheckpointInitializer, err = checkpoint.NewAPIInitializer(remoteURL)
			if err != nil {
				return errors.Wrap(err, "error while constructing beacon node api client for checkpoint sync")
			}
			return nil
		}
		return []node.Option{opt}, nil
	}

	if blockPath == "" && statePath == "" {
		return nil, nil
	}
	if blockPath != "" && statePath == "" {
		return nil, fmt.Errorf("--checkpoint-block specified, but not --checkpoint-state. both are required")
	}
	if blockPath == "" && statePath != "" {
		return nil, fmt.Errorf("--checkpoint-state specified, but not --checkpoint-block. both are required")
	}

	opt := func(node *node.BeaconNode) (err error) {
		node.CheckpointInitializer, err = checkpoint.NewFileInitializer(blockPath, statePath)
		if err != nil {
			return errors.Wrap(err, "error preparing to initialize checkpoint from local ssz files")
		}
		return nil
	}
	return []node.Option{opt}, nil
}
