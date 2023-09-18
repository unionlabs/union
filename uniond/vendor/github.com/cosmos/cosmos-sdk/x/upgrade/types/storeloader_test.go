package types

import (
	"encoding/json"
	"os"
	"path/filepath"
	"testing"

	dbm "github.com/cometbft/cometbft-db"
	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/libs/log"
	tmlog "github.com/cometbft/cometbft/libs/log"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/baseapp"
	pruningtypes "github.com/cosmos/cosmos-sdk/store/pruning/types"
	"github.com/cosmos/cosmos-sdk/store/rootmulti"
	storetypes "github.com/cosmos/cosmos-sdk/store/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func useUpgradeLoader(height int64, upgrades *storetypes.StoreUpgrades) func(*baseapp.BaseApp) {
	return func(app *baseapp.BaseApp) {
		app.SetStoreLoader(UpgradeStoreLoader(height, upgrades))
	}
}

func defaultLogger() log.Logger {
	return tmlog.NewTMLogger(tmlog.NewSyncWriter(os.Stdout))
}

func initStore(t *testing.T, db dbm.DB, storeKey string, k, v []byte) {
	rs := rootmulti.NewStore(db, log.NewNopLogger())
	rs.SetPruning(pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	key := sdk.NewKVStoreKey(storeKey)
	rs.MountStoreWithDB(key, storetypes.StoreTypeIAVL, nil)
	err := rs.LoadLatestVersion()
	require.Nil(t, err)
	require.Equal(t, int64(0), rs.LastCommitID().Version)

	// write some data in substore
	kv, _ := rs.GetStore(key).(storetypes.KVStore)
	require.NotNil(t, kv)
	kv.Set(k, v)
	commitID := rs.Commit()
	require.Equal(t, int64(1), commitID.Version)
}

func checkStore(t *testing.T, db dbm.DB, ver int64, storeKey string, k, v []byte) {
	rs := rootmulti.NewStore(db, log.NewNopLogger())
	rs.SetPruning(pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	key := sdk.NewKVStoreKey(storeKey)
	rs.MountStoreWithDB(key, storetypes.StoreTypeIAVL, nil)
	err := rs.LoadLatestVersion()
	require.Nil(t, err)
	require.Equal(t, ver, rs.LastCommitID().Version)

	// query data in substore
	kv, _ := rs.GetStore(key).(storetypes.KVStore)

	require.NotNil(t, kv)
	require.Equal(t, v, kv.Get(k))
}

// Test that we can make commits and then reload old versions.
// Test that LoadLatestVersion actually does.
func TestSetLoader(t *testing.T) {
	upgradeHeight := int64(5)

	// set a temporary home dir
	homeDir := t.TempDir()
	upgradeInfoFilePath := filepath.Join(homeDir, UpgradeInfoFilename)
	upgradeInfo := &Plan{
		Name: "test", Height: upgradeHeight,
	}

	data, err := json.Marshal(upgradeInfo)
	require.NoError(t, err)

	err = os.WriteFile(upgradeInfoFilePath, data, 0o644)
	require.NoError(t, err)

	// make sure it exists before running everything
	_, err = os.Stat(upgradeInfoFilePath)
	require.NoError(t, err)

	cases := map[string]struct {
		setLoader    func(*baseapp.BaseApp)
		origStoreKey string
		loadStoreKey string
	}{
		"don't set loader": {
			setLoader:    nil,
			origStoreKey: "foo",
			loadStoreKey: "foo",
		},
		"rename with inline opts": {
			setLoader: useUpgradeLoader(upgradeHeight, &storetypes.StoreUpgrades{
				Renamed: []storetypes.StoreRename{{
					OldKey: "foo",
					NewKey: "bar",
				}},
			}),
			origStoreKey: "foo",
			loadStoreKey: "bar",
		},
	}

	k := []byte("key")
	v := []byte("value")

	for name, tc := range cases {
		tc := tc
		t.Run(name, func(t *testing.T) {
			// prepare a db with some data
			db := dbm.NewMemDB()

			initStore(t, db, tc.origStoreKey, k, v)

			// load the app with the existing db
			opts := []func(*baseapp.BaseApp){baseapp.SetPruning(pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))}

			origapp := baseapp.NewBaseApp(t.Name(), defaultLogger(), db, nil, opts...)
			origapp.MountStores(sdk.NewKVStoreKey(tc.origStoreKey))
			err := origapp.LoadLatestVersion()
			require.Nil(t, err)

			for i := int64(2); i <= upgradeHeight-1; i++ {
				origapp.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{Height: i}})
				res := origapp.Commit()
				require.NotNil(t, res.Data)
			}

			if tc.setLoader != nil {
				opts = append(opts, tc.setLoader)
			}

			// load the new app with the original app db
			app := baseapp.NewBaseApp(t.Name(), defaultLogger(), db, nil, opts...)
			app.MountStores(sdk.NewKVStoreKey(tc.loadStoreKey))
			err = app.LoadLatestVersion()
			require.Nil(t, err)

			// "execute" one block
			app.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{Height: upgradeHeight}})
			res := app.Commit()
			require.NotNil(t, res.Data)

			// check db is properly updated
			checkStore(t, db, upgradeHeight, tc.loadStoreKey, k, v)
		})
	}
}
