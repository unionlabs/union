package grocksdb

import (
	"math"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestBackupableDBOptions(t *testing.T) {
	t.Parallel()

	opts := NewBackupableDBOptions("/tmp/v1")
	defer opts.Destroy()

	env := NewDefaultEnv()
	defer env.Destroy()

	opts.SetEnv(env)
	opts.SetBackupDir("/tmp/v2")

	require.True(t, opts.IsShareTableFiles()) // check default value
	opts.ShareTableFiles(false)
	require.False(t, opts.IsShareTableFiles())

	require.True(t, opts.IsSync())
	opts.SetSync(false)
	require.False(t, opts.IsSync())

	require.False(t, opts.IsDestroyOldData())
	opts.DestroyOldData(true)
	require.True(t, opts.IsDestroyOldData())

	require.True(t, opts.IsBackupLogFiles())
	opts.BackupLogFiles(false)
	require.False(t, opts.IsBackupLogFiles())

	require.EqualValues(t, 0, opts.GetBackupRateLimit())
	opts.SetBackupRateLimit(531 << 10)
	require.EqualValues(t, 531<<10, opts.GetBackupRateLimit())

	require.EqualValues(t, 0, opts.GetRestoreRateLimit())
	opts.SetRestoreRateLimit(53 << 10)
	require.EqualValues(t, 53<<10, opts.GetRestoreRateLimit())

	require.EqualValues(t, 1, opts.GetMaxBackgroundOperations())
	opts.SetMaxBackgroundOperations(3)
	require.EqualValues(t, 3, opts.GetMaxBackgroundOperations())

	require.EqualValues(t, 4194304, opts.GetCallbackTriggerIntervalSize())
	opts.SetCallbackTriggerIntervalSize(800 << 10)
	require.EqualValues(t, 800<<10, opts.GetCallbackTriggerIntervalSize())

	require.EqualValues(t, math.MaxInt32, opts.GetMaxValidBackupsToOpen())
	opts.SetMaxValidBackupsToOpen(29)
	require.EqualValues(t, 29, opts.GetMaxValidBackupsToOpen())

	require.EqualValues(t, 0x80000002, opts.GetShareFilesWithChecksumNaming())
	opts.SetShareFilesWithChecksumNaming(UseDBSessionID | LegacyCrc32cAndFileSize)
	require.EqualValues(t, UseDBSessionID|LegacyCrc32cAndFileSize, opts.GetShareFilesWithChecksumNaming())
}
