package main

import (
	"os"

	"github.com/cosmos/cosmos-sdk/server"
	svrcmd "github.com/cosmos/cosmos-sdk/server/cmd"
	"union/app"
	"union/cmd/uniond/cmd"
)

func main() {
	rootCmd, _ := cmd.NewRootCmd()
	rootCmd.AddCommand(cmd.GenBn254())
	rootCmd.AddCommand(cmd.GenStateProof())
	if err := svrcmd.Execute(rootCmd, "", app.DefaultNodeHome); err != nil {
		switch e := err.(type) {
		case server.ErrorCode:
			os.Exit(e.Code)

		default:
			os.Exit(1)
		}
	}
}
