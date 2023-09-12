package cli

import (
	"encoding/json"
	"fmt"
	"sort"
	"time"

	tmjson "github.com/cometbft/cometbft/libs/json"
	"github.com/pkg/errors"
	"github.com/spf13/cobra"
	"golang.org/x/exp/maps"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/version"
	v043 "github.com/cosmos/cosmos-sdk/x/genutil/migrations/v043"
	v046 "github.com/cosmos/cosmos-sdk/x/genutil/migrations/v046"
	v047 "github.com/cosmos/cosmos-sdk/x/genutil/migrations/v047"
	"github.com/cosmos/cosmos-sdk/x/genutil/types"
)

const flagGenesisTime = "genesis-time"

// Allow applications to extend and modify the migration process.
//
// Ref: https://github.com/cosmos/cosmos-sdk/issues/5041
var migrationMap = types.MigrationMap{
	"v0.43": v043.Migrate, // NOTE: v0.43, v0.44 and v0.45 are genesis compatible.
	"v0.46": v046.Migrate,
	"v0.47": v047.Migrate,
}

// GetMigrationCallback returns a MigrationCallback for a given version.
func GetMigrationCallback(version string) types.MigrationCallback {
	return migrationMap[version]
}

// GetMigrationVersions get all migration version in a sorted slice.
func GetMigrationVersions() []string {
	versions := maps.Keys(migrationMap)
	sort.Strings(versions)

	return versions
}

// MigrateGenesisCmd returns a command to execute genesis state migration.
func MigrateGenesisCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "migrate [target-version] [genesis-file]",
		Short: "Migrate genesis to a specified target version",
		Long: fmt.Sprintf(`Migrate the source genesis into the target version and print to STDOUT.

Example:
$ %s migrate v0.36 /path/to/genesis.json --chain-id=cosmoshub-3 --genesis-time=2019-04-22T17:00:00Z
`, version.AppName),
		Args: cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			return MigrateHandler(cmd, args, migrationMap)
		},
	}

	cmd.Flags().String(flagGenesisTime, "", "override genesis_time with this flag")
	cmd.Flags().String(flags.FlagChainID, "", "override chain_id with this flag")

	return cmd
}

// MigrateHandler handles the migration command with a migration map as input,
// returning an error upon failure.
func MigrateHandler(cmd *cobra.Command, args []string, migrations types.MigrationMap) error {
	clientCtx := client.GetClientContextFromCmd(cmd)

	var err error

	target := args[0]
	importGenesis := args[1]

	genDoc, err := validateGenDoc(importGenesis)
	if err != nil {
		return err
	}

	// Since some default values are valid values, we just print to
	// make sure the user didn't forget to update these values.
	if genDoc.ConsensusParams.Evidence.MaxBytes == 0 {
		fmt.Printf("Warning: consensus_params.evidence.max_bytes is set to 0. If this is"+
			" deliberate, feel free to ignore this warning. If not, please have a look at the chain"+
			" upgrade guide at %s.\n", chainUpgradeGuide)
	}

	var initialState types.AppMap
	if err := json.Unmarshal(genDoc.AppState, &initialState); err != nil {
		return errors.Wrap(err, "failed to JSON unmarshal initial genesis state")
	}

	migrationFunc := migrations[target]
	if migrationFunc == nil {
		return fmt.Errorf("unknown migration function for version: %s", target)
	}

	// TODO: handler error from migrationFunc call
	newGenState := migrationFunc(initialState, clientCtx)

	genDoc.AppState, err = json.Marshal(newGenState)
	if err != nil {
		return errors.Wrap(err, "failed to JSON marshal migrated genesis state")
	}

	genesisTime, _ := cmd.Flags().GetString(flagGenesisTime)
	if genesisTime != "" {
		var t time.Time

		err := t.UnmarshalText([]byte(genesisTime))
		if err != nil {
			return errors.Wrap(err, "failed to unmarshal genesis time")
		}

		genDoc.GenesisTime = t
	}

	chainID, _ := cmd.Flags().GetString(flags.FlagChainID)
	if chainID != "" {
		genDoc.ChainID = chainID
	}

	bz, err := tmjson.Marshal(genDoc)
	if err != nil {
		return errors.Wrap(err, "failed to marshal genesis doc")
	}

	sortedBz, err := sdk.SortJSON(bz)
	if err != nil {
		return errors.Wrap(err, "failed to sort JSON genesis doc")
	}

	cmd.Println(string(sortedBz))
	return nil
}
