package main_test

import (
	"os"
	"testing"

	"github.com/stretchr/testify/require"

	"go.etcd.io/bbolt/internal/common"
	"go.etcd.io/bbolt/internal/guts_cli"
)

func loadMetaPage(t *testing.T, dbPath string, pageID uint64) *common.Meta {
	_, buf, err := guts_cli.ReadPage(dbPath, pageID)
	require.NoError(t, err)
	return common.LoadPageMeta(buf)
}

func readMetaPage(t *testing.T, path string) *common.Meta {
	_, activeMetaPageId, err := guts_cli.GetRootPage(path)
	require.NoError(t, err)
	_, buf, err := guts_cli.ReadPage(path, uint64(activeMetaPageId))
	require.NoError(t, err)
	return common.LoadPageMeta(buf)
}

func readPage(t *testing.T, path string, pageId int, pageSize int) []byte {
	dbFile, err := os.Open(path)
	require.NoError(t, err)
	defer dbFile.Close()

	fi, err := dbFile.Stat()
	require.NoError(t, err)
	require.GreaterOrEqual(t, fi.Size(), int64((pageId+1)*pageSize))

	buf := make([]byte, pageSize)
	byteRead, err := dbFile.ReadAt(buf, int64(pageId*pageSize))
	require.NoError(t, err)
	require.Equal(t, pageSize, byteRead)

	return buf
}

func pageDataWithoutPageId(buf []byte) []byte {
	return buf[8:]
}
