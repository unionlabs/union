package main

import (
	provercmd "cometbls-prover/cmd"
	"github.com/spf13/cobra"
)

func main() {
	var rootCmd = &cobra.Command{Use: "cometbls-prover"}
	rootCmd.AddCommand(provercmd.ServeCmd())
	rootCmd.Execute()
}
