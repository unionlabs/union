package commands

import (
	"github.com/spf13/cobra"

	"github.com/cometbft/cometbft/consensus"
)

// ReplayCmd allows replaying of messages from the WAL.
var ReplayCmd = &cobra.Command{
	Use:   "replay",
	Short: "Replay messages from WAL",
	Run: func(cmd *cobra.Command, args []string) {
		consensus.RunReplayFile(config.BaseConfig, config.Consensus, false)
	},
}

// ReplayConsoleCmd allows replaying of messages from the WAL in a
// console.
var ReplayConsoleCmd = &cobra.Command{
	Use:     "replay-console",
	Aliases: []string{"replay_console"},
	Short:   "Replay messages from WAL in a console",
	Run: func(cmd *cobra.Command, args []string) {
		consensus.RunReplayFile(config.BaseConfig, config.Consensus, true)
	},
}
