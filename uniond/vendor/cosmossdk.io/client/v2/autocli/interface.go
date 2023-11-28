package autocli

import (
	"github.com/spf13/cobra"

	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	"cosmossdk.io/core/appmodule"
)

// HasAutoCLIConfig is an AppModule extension interface for declaring autocli module options.
type HasAutoCLIConfig interface {
	appmodule.AppModule

	// AutoCLIOptions are the autocli module options for this module.
	AutoCLIOptions() *autocliv1.ModuleOptions
}

// HasCustomQueryCommand is an AppModule extension interface for declaring a custom query command.
type HasCustomQueryCommand interface {
	appmodule.AppModule

	// GetQueryCmd returns a custom cobra query command for this module.
	GetQueryCmd() *cobra.Command
}

// HasCustomTxCommand is an AppModule extension interface for declaring a custom tx command.
type HasCustomTxCommand interface {
	appmodule.AppModule

	// GetTxCmd returns a custom cobra tx command for this module.
	GetTxCmd() *cobra.Command
}
