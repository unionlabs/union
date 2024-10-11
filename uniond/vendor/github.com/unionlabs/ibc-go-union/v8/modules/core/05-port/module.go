package port

import (
	"github.com/spf13/cobra"

	"github.com/cosmos/ibc-go/v8/modules/core/05-port/types"
	"github.com/cosmos/ibc-go/v8/modules/core/client/cli"
)

// Name returns the IBC port ICS name.
func Name() string {
	return types.SubModuleName
}

// GetQueryCmd returns the root query command for IBC ports.
func GetQueryCmd() *cobra.Command {
	return cli.GetQueryCmd()
}
