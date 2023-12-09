package cli

import (
	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/types/module"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	"github.com/cosmos/cosmos-sdk/x/genutil"
	genutiltypes "github.com/cosmos/cosmos-sdk/x/genutil/types"
)

// GenesisCoreCommand adds core sdk's sub-commands into genesis command.
// Deprecated: use Commands instead.
func GenesisCoreCommand(txConfig client.TxConfig, moduleBasics module.BasicManager, defaultNodeHome string) *cobra.Command {
	return Commands(txConfig, moduleBasics, defaultNodeHome)
}

// Commands adds core sdk's sub-commands into genesis command.
func Commands(txConfig client.TxConfig, moduleBasics module.BasicManager, defaultNodeHome string) *cobra.Command {
	return CommandsWithCustomMigrationMap(txConfig, moduleBasics, defaultNodeHome, MigrationMap)
}

// CommandsWithCustomMigrationMap adds core sdk's sub-commands into genesis command with custom migration map.
// This custom migration map can be used by the application to add its own migration map.
func CommandsWithCustomMigrationMap(txConfig client.TxConfig, moduleBasics module.BasicManager, defaultNodeHome string, migrationMap genutiltypes.MigrationMap) *cobra.Command {
	cmd := &cobra.Command{
		Use:                        "genesis",
		Short:                      "Application's genesis-related subcommands",
		DisableFlagParsing:         false,
		SuggestionsMinimumDistance: 2,
		RunE:                       client.ValidateCmd,
	}
	gentxModule := moduleBasics[genutiltypes.ModuleName].(genutil.AppModuleBasic)

	cmd.AddCommand(
		GenTxCmd(moduleBasics, txConfig, banktypes.GenesisBalancesIterator{}, defaultNodeHome, txConfig.SigningContext().ValidatorAddressCodec()),
		MigrateGenesisCmd(migrationMap),
		CollectGenTxsCmd(banktypes.GenesisBalancesIterator{}, defaultNodeHome, gentxModule.GenTxValidator, txConfig.SigningContext().ValidatorAddressCodec()),
		ValidateGenesisCmd(moduleBasics),
		AddGenesisAccountCmd(defaultNodeHome, txConfig.SigningContext().AddressCodec()),
	)

	return cmd
}
