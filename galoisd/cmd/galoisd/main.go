package main

import (
	"github.com/spf13/cobra"
	"github.com/unionlabs/union/galoisd/cmd/galoisd/cmd"
)

func main() {
	var rootCmd = &cobra.Command{Use: "galoisd"}
	rootCmd.AddCommand(cmd.ServeCmd())
	rootCmd.AddCommand(cmd.GenContract())
	rootCmd.AddCommand(cmd.ExampleProveCmd())
	rootCmd.AddCommand(cmd.ExampleVerifyCmd())
	rootCmd.AddCommand(cmd.QueryStats())
	rootCmd.AddCommand(cmd.QueryStatsHealth())
	rootCmd.Execute()
}
