package ioutils

import (
	"os"
	"testing"

	"github.com/stretchr/testify/require"
)

func GetTestData() ([]byte, []byte, []byte, error) {
	wasmCode, err := os.ReadFile("../keeper/testdata/hackatom.wasm")
	if err != nil {
		return nil, nil, nil, err
	}

	gzipData, err := GzipIt(wasmCode)
	if err != nil {
		return nil, nil, nil, err
	}

	someRandomStr := []byte("hello world")

	return wasmCode, someRandomStr, gzipData, nil
}

func TestIsWasm(t *testing.T) {
	wasmCode, someRandomStr, gzipData, err := GetTestData()
	require.NoError(t, err)

	t.Log("should return false for some random string data")
	require.False(t, IsWasm(someRandomStr))
	t.Log("should return false for gzip data")
	require.False(t, IsWasm(gzipData))
	t.Log("should return true for exact wasm")
	require.True(t, IsWasm(wasmCode))
}

func TestIsGzip(t *testing.T) {
	wasmCode, someRandomStr, gzipData, err := GetTestData()
	require.NoError(t, err)

	require.False(t, IsGzip(wasmCode))
	require.False(t, IsGzip(someRandomStr))
	require.False(t, IsGzip(nil))
	require.True(t, IsGzip(gzipData[0:3]))
	require.True(t, IsGzip(gzipData))
}

func TestGzipIt(t *testing.T) {
	wasmCode, someRandomStr, _, err := GetTestData()
	originalGzipData := []byte{
		31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 202, 72, 205, 201, 201, 87, 40, 207, 47, 202, 73, 1,
		4, 0, 0, 255, 255, 133, 17, 74, 13, 11, 0, 0, 0,
	}

	require.NoError(t, err)

	t.Log("gzip wasm with no error")
	_, err = GzipIt(wasmCode)
	require.NoError(t, err)

	t.Log("gzip of a string should return exact gzip data")
	strToGzip, err := GzipIt(someRandomStr)

	require.True(t, IsGzip(strToGzip))
	require.NoError(t, err)
	require.Equal(t, originalGzipData, strToGzip)
}
