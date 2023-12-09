package commands

import (
	"fmt"

	"github.com/spf13/cobra"

	cmtos "github.com/cometbft/cometbft/libs/os"
	"github.com/cometbft/cometbft/p2p"
)

// GenNodeKeyCmd allows the generation of a node key. It prints node's ID to
// the standard output.
var GenNodeKeyCmd = &cobra.Command{
	Use:     "gen-node-key",
	Aliases: []string{"gen_node_key"},
	Short:   "Generate a node key for this node and print its ID",
	RunE:    genNodeKey,
}

func genNodeKey(*cobra.Command, []string) error {
	nodeKeyFile := config.NodeKeyFile()
	if cmtos.FileExists(nodeKeyFile) {
		return fmt.Errorf("node key at %s already exists", nodeKeyFile)
	}

	nodeKey, err := p2p.LoadOrGenNodeKey(nodeKeyFile)
	if err != nil {
		return err
	}
	fmt.Println(nodeKey.ID())
	return nil
}
