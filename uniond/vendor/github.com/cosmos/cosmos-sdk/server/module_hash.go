package server

import (
	"encoding/hex"
	"fmt"
	"sort"
	"strconv"
	"strings"

	"github.com/spf13/cobra"

	"cosmossdk.io/store/rootmulti"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
	"github.com/cosmos/cosmos-sdk/version"
)

// ModuleHashByHeightQuery retrieves the module hashes at a given height.
func ModuleHashByHeightQuery[T servertypes.Application](appCreator servertypes.AppCreator[T]) *cobra.Command {
	cmd := &cobra.Command{
		Use:     "module-hash-by-height <height>",
		Short:   "Get module hashes at a given height",
		Long:    "Get module hashes at a given height. This command is useful for debugging and verifying the state of the application at a given height. Daemon should not be running when calling this command.",
		Example: fmt.Sprintf("%s module-hash-by-height 16841115", version.AppName),
		Args:    cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			heightToRetrieveString := args[0]

			serverCtx := GetServerContextFromCmd(cmd)

			height, err := strconv.ParseInt(heightToRetrieveString, 10, 64)
			if err != nil {
				return fmt.Errorf("invalid height: %w", err)
			}

			commitInfoForHeight, err := getModuleHashesAtHeight(serverCtx, appCreator, height)
			if err != nil {
				return err
			}

			clientCtx := client.GetClientContextFromCmd(cmd)
			return clientCtx.PrintProto(commitInfoForHeight)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)

	return cmd
}

func getModuleHashesAtHeight[T servertypes.Application](svrCtx *Context, appCreator servertypes.AppCreator[T], height int64) (*storetypes.CommitInfo, error) {
	home := svrCtx.Config.RootDir
	db, err := OpenDB(home, GetAppDBBackend(svrCtx.Viper))
	if err != nil {
		return nil, fmt.Errorf("error opening DB, make sure daemon is not running when calling this query: %w", err)
	}
	app := appCreator(svrCtx.Logger, db, nil, svrCtx.Viper)
	rms, ok := app.CommitMultiStore().(*rootmulti.Store)
	if !ok {
		return nil, fmt.Errorf("expected rootmulti.Store, got %T", app.CommitMultiStore())
	}

	commitInfoForHeight, err := rms.GetCommitInfo(height)
	if err != nil {
		return nil, err
	}

	// Create a new slice of StoreInfos for storing the modified hashes.
	storeInfos := make([]storetypes.StoreInfo, len(commitInfoForHeight.StoreInfos))

	for i, storeInfo := range commitInfoForHeight.StoreInfos {
		// Convert the hash to a hexadecimal string.
		hash := strings.ToUpper(hex.EncodeToString(storeInfo.CommitId.Hash))

		// Create a new StoreInfo with the modified hash.
		storeInfos[i] = storetypes.StoreInfo{
			Name: storeInfo.Name,
			CommitId: storetypes.CommitID{
				Version: storeInfo.CommitId.Version,
				Hash:    []byte(hash),
			},
		}
	}

	// Sort the storeInfos slice based on the module name.
	sort.Slice(storeInfos, func(i, j int) bool {
		return storeInfos[i].Name < storeInfos[j].Name
	})

	// Create a new CommitInfo with the modified StoreInfos.
	commitInfoForHeight = &storetypes.CommitInfo{
		Version:    commitInfoForHeight.Version,
		StoreInfos: storeInfos,
		Timestamp:  commitInfoForHeight.Timestamp,
	}

	return commitInfoForHeight, nil
}
