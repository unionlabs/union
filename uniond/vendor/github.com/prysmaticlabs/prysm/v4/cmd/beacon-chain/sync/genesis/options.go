package genesis

import (
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/node"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/sync/genesis"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/sync/checkpoint"
	log "github.com/sirupsen/logrus"
	"github.com/urfave/cli/v2"
)

var (
	// StatePath defines a flag to start the beacon chain from a give genesis state file.
	StatePath = &cli.PathFlag{
		Name: "genesis-state",
		Usage: "Load a genesis state from ssz file. Testnet genesis files can be found in the " +
			"eth2-clients/eth2-testnets repository on github.",
	}
	BeaconAPIURL = &cli.StringFlag{
		Name: "genesis-beacon-api-url",
		Usage: "URL of a synced beacon node to trust for obtaining genesis state. " +
			"As an additional safety measure, it is strongly recommended to only use this option in conjunction with " +
			"--weak-subjectivity-checkpoint flag",
	}
)

// BeaconNodeOptions is responsible for determining if the checkpoint sync options have been used, and if so,
// reading the block and state ssz-serialized values from the filesystem locations specified and preparing a
// checkpoint.Initializer, which uses the provided io.ReadClosers to initialize the beacon node database.
func BeaconNodeOptions(c *cli.Context) ([]node.Option, error) {
	statePath := c.Path(StatePath.Name)
	remoteURL := c.String(BeaconAPIURL.Name)
	if remoteURL == "" && c.String(checkpoint.RemoteURL.Name) != "" {
		log.Infof("using checkpoint sync url %s for value in --%s flag", c.String(checkpoint.RemoteURL.Name), BeaconAPIURL.Name)
		remoteURL = c.String(checkpoint.RemoteURL.Name)
	}
	if remoteURL != "" {
		opt := func(node *node.BeaconNode) error {
			var err error
			node.GenesisInitializer, err = genesis.NewAPIInitializer(remoteURL)
			if err != nil {
				return errors.Wrap(err, "error constructing beacon node api client for genesis state init")
			}
			return nil
		}
		return []node.Option{opt}, nil
	}

	if statePath == "" {
		return nil, nil
	}

	opt := func(node *node.BeaconNode) (err error) {
		node.GenesisInitializer, err = genesis.NewFileInitializer(statePath)
		if err != nil {
			return errors.Wrap(err, "error preparing to initialize genesis db state from local ssz files")
		}
		return nil
	}
	return []node.Option{opt}, nil
}
