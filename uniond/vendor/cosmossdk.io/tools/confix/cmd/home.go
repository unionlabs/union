package cmd

import (
	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
)

func HomeCommand() *cobra.Command {
	return &cobra.Command{
		Use:   "home",
		Short: "Outputs the folder used as the binary home. No home directory is set when using the `confix` tool standalone.",
		Long:  `Outputs the folder used as the binary home. In order to change the home directory path, set the $APPD_HOME environment variable, or use the "--home" flag.`,
		Args:  cobra.NoArgs,
		Run: func(cmd *cobra.Command, args []string) {
			clientCtx := client.GetClientContextFromCmd(cmd)
			if clientCtx.HomeDir == "" {
				cmd.Println("No home directory set.")
			} else {
				cmd.Println(clientCtx.HomeDir)
			}
		},
	}
}
