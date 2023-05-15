package main

import (
	"github.com/spf13/cobra"
	"unionp/cmd/unionpd/cmd"
)

func main() {
	var rootCmd = &cobra.Command{Use: "unionpd"}
	rootCmd.AddCommand(cmd.ServeCmd())
	rootCmd.AddCommand(cmd.ProveCmd)
	rootCmd.AddCommand(cmd.VerifyCmd)
	rootCmd.AddCommand(cmd.GenContract())
	rootCmd.Execute()
}
