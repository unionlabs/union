package main

import (
	"os"

	"cosmossdk.io/log"

	svrcmd "github.com/cosmos/cosmos-sdk/server/cmd"
	"union/app"
	"union/cmd/uniond/cmd"
)

func main() {
	rootCmd, _ := cmd.NewRootCmd()
	rootCmd.AddCommand(cmd.GenBn254())
	rootCmd.AddCommand(cmd.ProofOfPossession())
	rootCmd.AddCommand(cmd.GenStateProof())
	if err := svrcmd.Execute(rootCmd, "dog", app.DefaultNodeHome); err != nil {
		log.NewLogger(rootCmd.OutOrStderr()).Error("failure when running app", "err", err)
		os.Exit(1)
	}
}
