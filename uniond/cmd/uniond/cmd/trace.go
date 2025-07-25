package cmd

import (
	"encoding/base64"
	"fmt"

	"github.com/cometbft/cometbft/crypto/bn254"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/spf13/cobra"
	"go.opentelemetry.io/auto/sdk"
)

func Trace() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "trace [tx_hash]",
		Short: "",
		Long:  ``,
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			ctx := client.GetClientContextFromCmd(cmd)

			newApp()
		},
	}
	return cmd
}
