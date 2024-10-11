package cli

import (
	"encoding/json"

	"github.com/spf13/cobra"

	banktypes "cosmossdk.io/x/bank/types"

	"github.com/cosmos/cosmos-sdk/client"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
	"github.com/cosmos/cosmos-sdk/x/genutil"
	genutiltypes "github.com/cosmos/cosmos-sdk/x/genutil/types"
)

type genesisMM interface {
	DefaultGenesis() map[string]json.RawMessage
	ValidateGenesis(genesisData map[string]json.RawMessage) error
}

// Commands adds core sdk's sub-commands into genesis command.
func Commands(genutilModule genutil.AppModule, genMM genesisMM, appExport servertypes.AppExporter) *cobra.Command {
	return CommandsWithCustomMigrationMap(genutilModule, genMM, appExport, MigrationMap)
}

// CommandsWithCustomMigrationMap adds core sdk's sub-commands into genesis command with custom migration map.
// This custom migration map can be used by the application to add its own migration map.
func CommandsWithCustomMigrationMap(genutilModule genutil.AppModule, genMM genesisMM, appExport servertypes.AppExporter, migrationMap genutiltypes.MigrationMap) *cobra.Command {
	cmd := &cobra.Command{
		Use:                        "genesis",
		Short:                      "Application's genesis-related subcommands",
		DisableFlagParsing:         false,
		SuggestionsMinimumDistance: 2,
		RunE:                       client.ValidateCmd,
	}
	cmd.AddCommand(
		GenTxCmd(genMM, banktypes.GenesisBalancesIterator{}),
		MigrateGenesisCmd(migrationMap),
		CollectGenTxsCmd(genutilModule.GenTxValidator()),
		ValidateGenesisCmd(genMM),
		AddGenesisAccountCmd(),
		AddBulkGenesisAccountCmd(),
		ExportCmd(appExport),
	)

	return cmd
}
