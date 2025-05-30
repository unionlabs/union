//go:build binary
// +build binary

package main

import (
	"galois/cmd/galoisd/cmd"
	"github.com/spf13/cobra"
)

func main() {
	var rootCmd = &cobra.Command{Use: "galoisd"}
	rootCmd.AddCommand(cmd.ServeCmd())
	rootCmd.AddCommand(cmd.ServeBls12381Cmd())
	rootCmd.AddCommand(cmd.GenContract())
	rootCmd.AddCommand(cmd.ExampleProveCmd())
	rootCmd.AddCommand(cmd.ExampleVerifyCmd())
	rootCmd.AddCommand(cmd.QueryStats())
	rootCmd.AddCommand(cmd.QueryStatsHealth())
	rootCmd.AddCommand(
		cmd.Phase1InitCmd(),
		cmd.Phase2InitCmd(),
		cmd.Phase2ContributeCmd(),
		cmd.Phase2VerifyCmd(),
		cmd.Phase2ExtractCmd(),
	)
	rootCmd.Execute()
}
