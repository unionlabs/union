package commands

import (
	"encoding/json"
	"fmt"

	"github.com/spf13/cobra"

	"github.com/cometbft/cometbft/version"
)

// VersionCmd ...
var VersionCmd = &cobra.Command{
	Use:   "version",
	Short: "Show version info",
	Run: func(cmd *cobra.Command, args []string) {
		cmtVersion := version.TMCoreSemVer
		if version.TMGitCommitHash != "" {
			cmtVersion += "+" + version.TMGitCommitHash
		}

		if verbose {
			values, _ := json.MarshalIndent(struct {
				CometBFT      string `json:"cometbft"`
				ABCI          string `json:"abci"`
				BlockProtocol uint64 `json:"block_protocol"`
				P2PProtocol   uint64 `json:"p2p_protocol"`
			}{
				CometBFT:      cmtVersion,
				ABCI:          version.ABCISemVer,
				BlockProtocol: version.BlockProtocol,
				P2PProtocol:   version.P2PProtocol,
			}, "", "  ")
			fmt.Println(string(values))
		} else {
			fmt.Println(cmtVersion)
		}
	},
}

func init() {
	VersionCmd.Flags().BoolVarP(&verbose, "verbose", "v", false, "Show protocol and library versions")
}
