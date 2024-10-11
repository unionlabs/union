package types

import (
	"encoding/json"
	"os"
	"path/filepath"
	"testing"

	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/stretchr/testify/require"

	corestore "cosmossdk.io/core/store"
	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/log"
	"cosmossdk.io/store/metrics"
	pruningtypes "cosmossdk.io/store/pruning/types"
	"cosmossdk.io/store/rootmulti"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/baseapp"
)

func initStore(t *testing.T, db corestore.KVStoreWithBatch, storeKey string, k, v []byte) {
	t.Helper()
	rs := rootmulti.NewStore(db, coretesting.NewNopLogger(), metrics.NewNoOpMetrics())
	rs.SetPruning(pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	key := storetypes.NewKVStoreKey(storeKey)
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

func checkStore(t *testing.T, db corestore.KVStoreWithBatch, ver int64, storeKey string, k, v []byte) {
	t.Helper()
	rs := rootmulti.NewStore(db, coretesting.NewNopLogger(), metrics.NewNoOpMetrics())
	rs.SetPruning(pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	key := storetypes.NewKVStoreKey(storeKey)
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

	err = os.WriteFile(upgradeInfoFilePath, data, 0o600)
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
	}

	k := []byte("key")
	v := []byte("value")

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			// prepare a db with some data
			db := coretesting.NewMemDB()

			initStore(t, db, tc.origStoreKey, k, v)

			// load the app with the existing db
			opts := []func(*baseapp.BaseApp){baseapp.SetPruning(pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))}

			logger := log.NewNopLogger()

			oldApp := baseapp.NewBaseApp(t.Name(), logger, db, nil, opts...)
			oldApp.MountStores(storetypes.NewKVStoreKey(tc.origStoreKey))

			err := oldApp.LoadLatestVersion()
			require.Nil(t, err)
			require.Equal(t, int64(1), oldApp.LastBlockHeight())

			for i := int64(2); i <= upgradeHeight-1; i++ {
				_, err := oldApp.FinalizeBlock(&abci.FinalizeBlockRequest{Height: i})
				require.NoError(t, err)
				_, err = oldApp.Commit()
				require.NoError(t, err)
			}

			require.Equal(t, upgradeHeight-1, oldApp.LastBlockHeight())

			if tc.setLoader != nil {
				opts = append(opts, tc.setLoader)
			}

			// load the new newApp with the original newApp db
			newApp := baseapp.NewBaseApp(t.Name(), logger.With("instance", "new"), db, nil, opts...)
			newApp.MountStores(storetypes.NewKVStoreKey(tc.loadStoreKey))

			err = newApp.LoadLatestVersion()
			require.Nil(t, err)

			require.Equal(t, upgradeHeight-1, newApp.LastBlockHeight())

			// "execute" one block
			_, err = newApp.FinalizeBlock(&abci.FinalizeBlockRequest{Height: upgradeHeight})
			require.NoError(t, err)
			_, err = newApp.Commit()
			require.NoError(t, err)
			require.Equal(t, upgradeHeight, newApp.LastBlockHeight())

			// check db is properly updated
			checkStore(t, db, upgradeHeight, tc.loadStoreKey, k, v)
		})
	}
}
