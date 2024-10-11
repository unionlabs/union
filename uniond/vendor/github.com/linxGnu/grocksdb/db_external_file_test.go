package grocksdb

import (
	"os"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestExternalFile(t *testing.T) {
	t.Parallel()

	db := newTestDB(t, nil)
	defer db.Close()

	envOpts := NewDefaultEnvOptions()
	opts := NewDefaultOptions()
	w := NewSSTFileWriter(envOpts, opts)
	defer w.Destroy()

	filePath, err := os.CreateTemp("", "sst-file-test")
	require.Nil(t, err)
	defer os.Remove(filePath.Name())

	err = w.Open(filePath.Name())
	require.Nil(t, err)

	err = w.Add([]byte("aaa"), []byte("aaaValue"))
	require.Nil(t, err)
	err = w.Add([]byte("bbb"), []byte("bbbValue"))
	require.Nil(t, err)
	err = w.Add([]byte("ccc"), []byte("cccValue"))
	require.Nil(t, err)
	err = w.Add([]byte("ddd"), []byte("dddValue"))
	require.Nil(t, err)

	err = w.Finish()
	require.Nil(t, err)

	ingestOpts := NewDefaultIngestExternalFileOptions()
	err = db.IngestExternalFile([]string{filePath.Name()}, ingestOpts)
	require.Nil(t, err)

	readOpts := NewDefaultReadOptions()

	v1, err := db.Get(readOpts, []byte("aaa"))
	require.Nil(t, err)
	require.EqualValues(t, v1.Data(), []byte("aaaValue"))
	v2, err := db.Get(readOpts, []byte("bbb"))
	require.Nil(t, err)
	require.EqualValues(t, v2.Data(), []byte("bbbValue"))
	v3, err := db.Get(readOpts, []byte("ccc"))
	require.Nil(t, err)
	require.EqualValues(t, v3.Data(), []byte("cccValue"))
	v4, err := db.Get(readOpts, []byte("ddd"))
	require.Nil(t, err)
	require.EqualValues(t, v4.Data(), []byte("dddValue"))
}

func TestExternalFileWithTS(t *testing.T) {
	t.Parallel()

	db := newTestDB(t, func(opts *Options) {
		opts.SetComparator(newDefaultComparatorWithTS())
	})
	defer db.Close()

	envOpts := NewDefaultEnvOptions()
	opts := NewDefaultOptions()
	opts.SetComparator(newDefaultComparatorWithTS())
	w := NewSSTFileWriter(envOpts, opts)
	defer w.Destroy()

	filePath, err := os.CreateTemp("", "sst-file-ts-test")
	require.Nil(t, err)
	defer os.Remove(filePath.Name())

	err = w.Open(filePath.Name())
	require.Nil(t, err)

	err = w.PutWithTS([]byte("aaa"), marshalTimestamp(1), []byte("aaaValue"))
	require.Nil(t, err)
	err = w.PutWithTS([]byte("bbb"), marshalTimestamp(2), []byte("bbbValue"))
	require.Nil(t, err)
	err = w.PutWithTS([]byte("ccc"), marshalTimestamp(3), []byte("cccValue"))
	require.Nil(t, err)
	err = w.PutWithTS([]byte("ddd"), marshalTimestamp(4), []byte("dddValue"))
	require.Nil(t, err)

	err = w.Finish()
	require.Nil(t, err)

	ingestOpts := NewDefaultIngestExternalFileOptions()
	err = db.IngestExternalFile([]string{filePath.Name()}, ingestOpts)
	require.Nil(t, err)

	readOpts := NewDefaultReadOptions()
	readOpts.SetTimestamp(marshalTimestamp(5))

	v1, t1, err := db.GetWithTS(readOpts, []byte("aaa"))
	require.Nil(t, err)
	require.EqualValues(t, v1.Data(), []byte("aaaValue"))
	require.EqualValues(t, t1.Data(), marshalTimestamp(1))

	v2, t2, err := db.GetWithTS(readOpts, []byte("bbb"))
	require.Nil(t, err)
	require.EqualValues(t, v2.Data(), []byte("bbbValue"))
	require.EqualValues(t, t2.Data(), marshalTimestamp(2))

	v3, t3, err := db.GetWithTS(readOpts, []byte("ccc"))
	require.Nil(t, err)
	require.EqualValues(t, v3.Data(), []byte("cccValue"))
	require.EqualValues(t, t3.Data(), marshalTimestamp(3))

	v4, t4, err := db.GetWithTS(readOpts, []byte("ddd"))
	require.Nil(t, err)
	require.EqualValues(t, v4.Data(), []byte("dddValue"))
	require.EqualValues(t, t4.Data(), marshalTimestamp(4))
}
