package snapshots_test

import (
	"bytes"
	"errors"
	"io"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/store/snapshots"
)

func TestChunkWriter(t *testing.T) {
	ch := make(chan io.ReadCloser, 100)
	go func() {
		chunkWriter := snapshots.NewChunkWriter(ch, 2)

		n, err := chunkWriter.Write([]byte{1, 2, 3})
		require.NoError(t, err)
		assert.Equal(t, 3, n)

		n, err = chunkWriter.Write([]byte{4, 5, 6})
		require.NoError(t, err)
		assert.Equal(t, 3, n)

		n, err = chunkWriter.Write([]byte{7, 8, 9})
		require.NoError(t, err)
		assert.Equal(t, 3, n)

		err = chunkWriter.Close()
		require.NoError(t, err)

		// closed writer should error
		_, err = chunkWriter.Write([]byte{10})
		require.Error(t, err)

		// closing again should be fine
		err = chunkWriter.Close()
		require.NoError(t, err)
	}()

	assert.Equal(t, [][]byte{{1, 2}, {3, 4}, {5, 6}, {7, 8}, {9}}, readChunks(ch))

	// 0-sized chunks should return the whole body as one chunk
	ch = make(chan io.ReadCloser, 100)
	go func() {
		chunkWriter := snapshots.NewChunkWriter(ch, 0)
		_, err := chunkWriter.Write([]byte{1, 2, 3})
		require.NoError(t, err)
		_, err = chunkWriter.Write([]byte{4, 5, 6})
		require.NoError(t, err)
		err = chunkWriter.Close()
		require.NoError(t, err)
	}()
	assert.Equal(t, [][]byte{{1, 2, 3, 4, 5, 6}}, readChunks(ch))

	// closing with error should return the error
	theErr := errors.New("boom")
	ch = make(chan io.ReadCloser, 100)
	go func() {
		chunkWriter := snapshots.NewChunkWriter(ch, 2)
		_, err := chunkWriter.Write([]byte{1, 2, 3})
		require.NoError(t, err)
		chunkWriter.CloseWithError(theErr)
	}()
	chunk, err := io.ReadAll(<-ch)
	require.NoError(t, err)
	assert.Equal(t, []byte{1, 2}, chunk)
	_, err = io.ReadAll(<-ch)
	require.Error(t, err)
	assert.Equal(t, theErr, err)
	assert.Empty(t, ch)

	// closing immediately should return no chunks
	ch = make(chan io.ReadCloser, 100)
	chunkWriter := snapshots.NewChunkWriter(ch, 2)
	err = chunkWriter.Close()
	require.NoError(t, err)
	assert.Empty(t, ch)
}

func TestChunkReader(t *testing.T) {
	ch := makeChunks([][]byte{
		{1, 2, 3},
		{4},
		{},
		{5, 6},
	})
	chunkReader := snapshots.NewChunkReader(ch)

	buf := []byte{0, 0, 0, 0}
	n, err := chunkReader.Read(buf)
	require.NoError(t, err)
	assert.Equal(t, 3, n)
	assert.Equal(t, []byte{1, 2, 3, 0}, buf)

	buf = []byte{0, 0, 0, 0}
	n, err = chunkReader.Read(buf)
	require.NoError(t, err)
	assert.Equal(t, 1, n)
	assert.Equal(t, []byte{4, 0, 0, 0}, buf)

	buf = []byte{0, 0, 0, 0}
	n, err = chunkReader.Read(buf)
	require.NoError(t, err)
	assert.Equal(t, 2, n)
	assert.Equal(t, []byte{5, 6, 0, 0}, buf)

	buf = []byte{0, 0, 0, 0}
	_, err = chunkReader.Read(buf)
	require.Error(t, err)
	assert.Equal(t, io.EOF, err)

	err = chunkReader.Close()
	require.NoError(t, err)

	err = chunkReader.Close() // closing twice should be fine
	require.NoError(t, err)

	// Empty channel should be fine
	ch = makeChunks(nil)
	chunkReader = snapshots.NewChunkReader(ch)
	buf = make([]byte, 4)
	_, err = chunkReader.Read(buf)
	require.Error(t, err)
	assert.Equal(t, io.EOF, err)

	// Using a pipe that closes with an error should return the error
	theErr := errors.New("boom")
	pr, pw := io.Pipe()
	pch := make(chan io.ReadCloser, 1)
	pch <- pr
	_ = pw.CloseWithError(theErr)

	chunkReader = snapshots.NewChunkReader(pch)
	buf = make([]byte, 4)
	_, err = chunkReader.Read(buf)
	require.Error(t, err)
	assert.Equal(t, theErr, err)

	// Closing the reader should close the writer
	pr, pw = io.Pipe()
	pch = make(chan io.ReadCloser, 2)
	pch <- io.NopCloser(bytes.NewBuffer([]byte{1, 2, 3}))
	pch <- pr
	close(pch)

	go func() {
		chunkReader := snapshots.NewChunkReader(pch)
		buf := []byte{0, 0, 0, 0}
		_, err := chunkReader.Read(buf)
		require.NoError(t, err)
		assert.Equal(t, []byte{1, 2, 3, 0}, buf)

		err = chunkReader.Close()
		require.NoError(t, err)
	}()

	_, err = pw.Write([]byte{9, 9, 9})
	require.Error(t, err)
	assert.Equal(t, err, io.ErrClosedPipe)
}
