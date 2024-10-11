package cli

import (
	"os"
	"path/filepath"

	tmcmd "github.com/cometbft/cometbft/cmd/cometbft/commands"
	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/server"
)

// ExtendUnsafeResetAllCmd - also clear wasm dir
func ExtendUnsafeResetAllCmd(rootCmd *cobra.Command) {
	unsafeResetCmd := tmcmd.ResetAllCmd.Use
	for _, branchCmd := range rootCmd.Commands() {
		if branchCmd.Use != "comet" {
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
