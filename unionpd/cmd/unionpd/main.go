package main

import (
	"github.com/spf13/cobra"
	"unionp/cmd/unionpd/cmd"
)

func main() {


	var rootCmd = &cobra.Command{Use: "unionpd"}
	rootCmd.AddCommand(cmd.ServeCmd())
	rootCmd.AddCommand(cmd.GenContract())
	rootCmd.AddCommand(cmd.ExampleProveCmd())
	rootCmd.AddCommand(cmd.ExampleVerifyCmd())
	rootCmd.Execute()
}
