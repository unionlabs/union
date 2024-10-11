package db

import (
	"context"
	"flag"
	"os"
	"path"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/db/kv"
	logTest "github.com/sirupsen/logrus/hooks/test"
	"github.com/urfave/cli/v2"
)

func TestRestore(t *testing.T) {
	logHook := logTest.NewGlobal()
	ctx := context.Background()

	backupDb, err := kv.NewKVStore(ctx, t.TempDir(), &kv.Config{})
	defer func() {
		require.NoError(t, backupDb.Close())
	}()
	require.NoError(t, err)
	root := [32]byte{1}
	require.NoError(t, backupDb.SaveGenesisValidatorsRoot(ctx, root[:]))
	require.NoError(t, backupDb.Close())
	// We rename the backup file so that we can later verify
	// whether the restored db has been renamed correctly.
	require.NoError(t, os.Rename(
		path.Join(backupDb.DatabasePath(), kv.ProtectionDbFileName),
		path.Join(backupDb.DatabasePath(), "backup.db")))

	restoreDir := t.TempDir()
	require.NoError(t, os.Chmod(restoreDir, params.BeaconIoConfig().ReadWriteExecutePermissions))
	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String(cmd.RestoreSourceFileFlag.Name, "", "")
	set.String(cmd.RestoreTargetDirFlag.Name, "", "")
	require.NoError(t, set.Set(cmd.RestoreSourceFileFlag.Name, path.Join(backupDb.DatabasePath(), "backup.db")))
	require.NoError(t, set.Set(cmd.RestoreTargetDirFlag.Name, restoreDir))
	cliCtx := cli.NewContext(&app, set, nil)

	assert.NoError(t, Restore(cliCtx))

	files, err := os.ReadDir(restoreDir)
	require.NoError(t, err)
	assert.Equal(t, 1, len(files))
	assert.Equal(t, kv.ProtectionDbFileName, files[0].Name())
	restoredDb, err := kv.NewKVStore(ctx, restoreDir, &kv.Config{})
	defer func() {
		require.NoError(t, restoredDb.Close())
	}()
	require.NoError(t, err)
	genesisRoot, err := restoredDb.GenesisValidatorsRoot(ctx)
	require.NoError(t, err)
	require.DeepEqual(t, root[:], genesisRoot, "Restored database has incorrect data")
	assert.LogsContain(t, logHook, "Restore completed successfully")
}
