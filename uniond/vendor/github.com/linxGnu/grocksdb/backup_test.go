package grocksdb

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestBackupEngine(t *testing.T) {
	t.Parallel()

	db := newTestDB(t, nil)
	defer db.Close()

	var (
		givenKey  = []byte("hello")
		givenVal1 = []byte("")
		givenVal2 = []byte("world1")
		wo        = NewDefaultWriteOptions()
		ro        = NewDefaultReadOptions()
	)
	defer wo.Destroy()
	defer ro.Destroy()

	// create
	require.Nil(t, db.Put(wo, givenKey, givenVal1))

	// retrieve
	v1, err := db.Get(ro, givenKey)
	defer v1.Free()
	require.Nil(t, err)
	require.EqualValues(t, v1.Data(), givenVal1)

	// retrieve bytes
	_v1, err := db.GetBytes(ro, givenKey)
	require.Nil(t, err)
	require.EqualValues(t, _v1, givenVal1)

	// update
	require.Nil(t, db.Put(wo, givenKey, givenVal2))
	v2, err := db.Get(ro, givenKey)
	defer v2.Free()
	require.Nil(t, err)
	require.EqualValues(t, v2.Data(), givenVal2)

	// retrieve pinned
	v3, err := db.GetPinned(ro, givenKey)
	defer v3.Destroy()
	require.Nil(t, err)
	require.EqualValues(t, v3.Data(), givenVal2)

	engine, err := CreateBackupEngine(db)
	require.Nil(t, err)
	defer func() {
		engine.Close()

		// re-open with opts
		opts := NewBackupableDBOptions(db.name)
		env := NewDefaultEnv()

		_, err = OpenBackupEngineWithOpt(opts, env)
		require.Nil(t, err)

		env.Destroy()
		opts.Destroy()
	}()

	{
		infos := engine.GetInfo()
		require.Empty(t, infos)

		// create first backup
		require.Nil(t, engine.CreateNewBackup())

		// create second backup
		require.Nil(t, engine.CreateNewBackupFlush(true))

		infos = engine.GetInfo()
		require.Equal(t, 2, len(infos))
		for i := range infos {
			require.Nil(t, engine.VerifyBackup(infos[i].ID))
			require.True(t, infos[i].Size > 0)
			require.True(t, infos[i].NumFiles > 0)
		}
	}

	{
		require.Nil(t, engine.PurgeOldBackups(1))

		infos := engine.GetInfo()
		require.Equal(t, 1, len(infos))
	}

	{
		dir := t.TempDir()

		ro := NewRestoreOptions()
		defer ro.Destroy()
		require.Nil(t, engine.RestoreDBFromLatestBackup(dir, dir, ro))
		require.Nil(t, engine.RestoreDBFromLatestBackup(dir, dir, ro))
	}

	{
		infos := engine.GetInfo()
		require.Equal(t, 1, len(infos))

		dir := t.TempDir()

		ro := NewRestoreOptions()
		defer ro.Destroy()
		require.Nil(t, engine.RestoreDBFromBackup(dir, dir, ro, infos[0].ID))

		// try to reopen restored db
		backupDB, err := OpenDb(db.opts, dir)
		require.Nil(t, err)
		defer backupDB.Close()

		r := NewDefaultReadOptions()
		defer r.Destroy()

		for i := 0; i < 1000; i++ {
			v3, err := backupDB.GetPinned(r, givenKey)
			require.Nil(t, err)
			require.EqualValues(t, v3.Data(), givenVal2)
			v3.Destroy()

			v4, err := backupDB.GetPinned(r, []byte("justFake"))
			require.Nil(t, err)
			require.False(t, v4.Exists())
			v4.Destroy()
		}
	}
}
