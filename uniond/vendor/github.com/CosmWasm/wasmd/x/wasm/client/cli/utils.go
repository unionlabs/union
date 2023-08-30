package cli

import (
	"os"
	"path/filepath"

	"github.com/cosmos/cosmos-sdk/server"
	"github.com/spf13/cobra"

	tmcmd "github.com/cometbft/cometbft/cmd/cometbft/commands"
)

// ExtendUnsafeResetAllCmd - also clear wasm dir
func ExtendUnsafeResetAllCmd(rootCmd *cobra.Command) {
	unsafeResetCmd := tmcmd.ResetAllCmd.Use
	for _, branchCmd := range rootCmd.Commands() {
		if branchCmd.Use != "tendermint" {
			continue
		}
		for _, cmd := range branchCmd.Commands() {
			if cmd.Use == unsafeResetCmd {
				serverRunE := cmd.RunE
				cmd.RunE = func(cmd *cobra.Command, args []string) error {
					if err := serverRunE(cmd, args); err != nil {
						return nil
					}
					serverCtx := server.GetServerContextFromCmd(cmd)
					return os.RemoveAll(filepath.Join(serverCtx.Config.RootDir, "wasm"))
				}
				return
			}
		}
	}
}
