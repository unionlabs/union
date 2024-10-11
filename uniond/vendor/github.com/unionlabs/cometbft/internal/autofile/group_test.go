package autofile

import (
	"io"
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	cmtos "github.com/cometbft/cometbft/internal/os"
	cmtrand "github.com/cometbft/cometbft/internal/rand"
)

func createTestGroupWithHeadSizeLimit(t *testing.T, headSizeLimit int64) *Group {
	t.Helper()
	testID := cmtrand.Str(12)
	testDir := "_test_" + testID
	err := cmtos.EnsureDir(testDir, 0o700)
	require.NoError(t, err, "Error creating dir")

	headPath := testDir + "/myfile"
	g, err := OpenGroup(headPath, GroupHeadSizeLimit(headSizeLimit))
	require.NoError(t, err, "Error opening Group")
	require.NotEqual(t, nil, g, "Failed to create Group")

	return g
}

func destroyTestGroup(t *testing.T, g *Group) {
	t.Helper()
	g.Close()

	err := os.RemoveAll(g.Dir)
	require.NoError(t, err, "Error removing test Group directory")
}

func assertGroupInfo(t *testing.T, gInfo GroupInfo, maxIndex int, totalSize, headSize int64) {
	t.Helper()
	assert.Equal(t, 0, gInfo.MinIndex)
	assert.Equal(t, maxIndex, gInfo.MaxIndex)
	assert.Equal(t, totalSize, gInfo.TotalSize)
	assert.Equal(t, headSize, gInfo.HeadSize)
}

func TestCheckHeadSizeLimit(t *testing.T) {
	g := createTestGroupWithHeadSizeLimit(t, 1000*1000)

	// At first, there are no files.
	assertGroupInfo(t, g.ReadGroupInfo(), 0, 0, 0)

	// Write 1000 bytes 999 times.
	for i := 0; i < 999; i++ {
		err := g.WriteLine(cmtrand.Str(999))
		require.NoError(t, err, "Error appending to head")
	}
	err := g.FlushAndSync()
	require.NoError(t, err)
	assertGroupInfo(t, g.ReadGroupInfo(), 0, 999000, 999000)

	// Even calling checkHeadSizeLimit manually won't rotate it.
	g.checkHeadSizeLimit()
	assertGroupInfo(t, g.ReadGroupInfo(), 0, 999000, 999000)

	// Write 1000 more bytes.
	err = g.WriteLine(cmtrand.Str(999))
	require.NoError(t, err, "Error appending to head")
	err = g.FlushAndSync()
	require.NoError(t, err)

	// Calling checkHeadSizeLimit this time rolls it.
	g.checkHeadSizeLimit()
	assertGroupInfo(t, g.ReadGroupInfo(), 1, 1000000, 0)

	// Write 1000 more bytes.
	err = g.WriteLine(cmtrand.Str(999))
	require.NoError(t, err, "Error appending to head")
	err = g.FlushAndSync()
	require.NoError(t, err)

	// Calling checkHeadSizeLimit does nothing.
	g.checkHeadSizeLimit()
	assertGroupInfo(t, g.ReadGroupInfo(), 1, 1001000, 1000)

	// Write 1000 bytes 999 times.
	for i := 0; i < 999; i++ {
		err = g.WriteLine(cmtrand.Str(999))
		require.NoError(t, err, "Error appending to head")
	}
	err = g.FlushAndSync()
	require.NoError(t, err)
	assertGroupInfo(t, g.ReadGroupInfo(), 1, 2000000, 1000000)

	// Calling checkHeadSizeLimit rolls it again.
	g.checkHeadSizeLimit()
	assertGroupInfo(t, g.ReadGroupInfo(), 2, 2000000, 0)

	// Write 1000 more bytes.
	_, err = g.Head.Write([]byte(cmtrand.Str(999) + "\n"))
	require.NoError(t, err, "Error appending to head")
	err = g.FlushAndSync()
	require.NoError(t, err)
	assertGroupInfo(t, g.ReadGroupInfo(), 2, 2001000, 1000)

	// Calling checkHeadSizeLimit does nothing.
	g.checkHeadSizeLimit()
	assertGroupInfo(t, g.ReadGroupInfo(), 2, 2001000, 1000)

	// Cleanup
	destroyTestGroup(t, g)
}

func TestRotateFile(t *testing.T) {
	g := createTestGroupWithHeadSizeLimit(t, 0)

	// Create a different temporary directory and move into it, to make sure
	// relative paths are resolved at Group creation
	origDir, err := os.Getwd()
	require.NoError(t, err)
	defer func() {
		if err := os.Chdir(origDir); err != nil {
			t.Error(err)
		}
	}()

	dir, err := os.MkdirTemp("", "rotate_test")
	require.NoError(t, err)
	defer os.RemoveAll(dir)
	err = os.Chdir(dir)
	require.NoError(t, err)

	require.True(t, filepath.IsAbs(g.Head.Path))
	require.True(t, filepath.IsAbs(g.Dir))

	// Create and rotate files
	err = g.WriteLine("Line 1")
	require.NoError(t, err)
	err = g.WriteLine("Line 2")
	require.NoError(t, err)
	err = g.WriteLine("Line 3")
	require.NoError(t, err)
	err = g.FlushAndSync()
	require.NoError(t, err)
	g.RotateFile()
	err = g.WriteLine("Line 4")
	require.NoError(t, err)
	err = g.WriteLine("Line 5")
	require.NoError(t, err)
	err = g.WriteLine("Line 6")
	require.NoError(t, err)
	err = g.FlushAndSync()
	require.NoError(t, err)

	// Read g.Head.Path+"000"
	body1, err := os.ReadFile(g.Head.Path + ".000")
	require.NoError(t, err, "Failed to read first rolled file: %v", err)
	if string(body1) != "Line 1\nLine 2\nLine 3\n" {
		t.Errorf("got unexpected contents: [%v]", string(body1))
	}

	// Read g.Head.Path
	body2, err := os.ReadFile(g.Head.Path)
	require.NoError(t, err, "Failed to read first rolled file: %v", err)
	if string(body2) != "Line 4\nLine 5\nLine 6\n" {
		t.Errorf("got unexpected contents: [%v]", string(body2))
	}

	// Make sure there are no files in the current, temporary directory
	files, err := os.ReadDir(".")
	require.NoError(t, err)
	assert.Empty(t, files)

	// Cleanup
	destroyTestGroup(t, g)
}

func TestWrite(t *testing.T) {
	g := createTestGroupWithHeadSizeLimit(t, 0)

	written := []byte("Medusa")
	_, err := g.Write(written)
	require.NoError(t, err)
	err = g.FlushAndSync()
	require.NoError(t, err)

	read := make([]byte, len(written))
	gr, err := g.NewReader(0)
	require.NoError(t, err, "failed to create reader")

	_, err = gr.Read(read)
	require.NoError(t, err, "failed to read data: %v", err)
	assert.Equal(t, written, read)

	// Cleanup
	destroyTestGroup(t, g)
}

// test that Read reads the required amount of bytes from all the files in the
// group and returns no error if n == size of the given slice.
func TestGroupReaderRead(t *testing.T) {
	g := createTestGroupWithHeadSizeLimit(t, 0)

	professor := []byte("Professor Monster")
	_, err := g.Write(professor)
	require.NoError(t, err)
	err = g.FlushAndSync()
	require.NoError(t, err)
	g.RotateFile()
	frankenstein := []byte("Frankenstein's Monster")
	_, err = g.Write(frankenstein)
	require.NoError(t, err)
	err = g.FlushAndSync()
	require.NoError(t, err)

	totalWrittenLength := len(professor) + len(frankenstein)
	read := make([]byte, totalWrittenLength)
	gr, err := g.NewReader(0)
	require.NoError(t, err, "failed to create reader")

	n, err := gr.Read(read)
	require.NoError(t, err, "failed to read data: %v", err)
	assert.Equal(t, totalWrittenLength, n, "not enough bytes read")
	professorPlusFrankenstein := professor
	professorPlusFrankenstein = append(professorPlusFrankenstein, frankenstein...)
	assert.Equal(t, professorPlusFrankenstein, read)

	// Cleanup
	destroyTestGroup(t, g)
}

// test that Read returns an error if number of bytes read < size of
// the given slice. Subsequent call should return 0, io.EOF.
func TestGroupReaderRead2(t *testing.T) {
	g := createTestGroupWithHeadSizeLimit(t, 0)

	professor := []byte("Professor Monster")
	_, err := g.Write(professor)
	require.NoError(t, err)
	err = g.FlushAndSync()
	require.NoError(t, err)
	g.RotateFile()
	frankenstein := []byte("Frankenstein's Monster")
	frankensteinPart := []byte("Frankenstein")
	_, err = g.Write(frankensteinPart) // note writing only a part
	require.NoError(t, err)
	err = g.FlushAndSync()
	require.NoError(t, err)

	totalLength := len(professor) + len(frankenstein)
	read := make([]byte, totalLength)
	gr, err := g.NewReader(0)
	require.NoError(t, err, "failed to create reader")

	// 1) n < (size of the given slice), io.EOF
	n, err := gr.Read(read)
	assert.Equal(t, io.EOF, err)
	assert.Equal(t, len(professor)+len(frankensteinPart), n, "Read more/less bytes than it is in the group")

	// 2) 0, io.EOF
	n, err = gr.Read([]byte("0"))
	assert.Equal(t, io.EOF, err)
	assert.Equal(t, 0, n)

	// Cleanup
	destroyTestGroup(t, g)
}

func TestMinIndex(t *testing.T) {
	g := createTestGroupWithHeadSizeLimit(t, 0)

	assert.Zero(t, g.MinIndex(), "MinIndex should be zero at the beginning")

	// Cleanup
	destroyTestGroup(t, g)
}

func TestMaxIndex(t *testing.T) {
	g := createTestGroupWithHeadSizeLimit(t, 0)

	assert.Zero(t, g.MaxIndex(), "MaxIndex should be zero at the beginning")

	err := g.WriteLine("Line 1")
	require.NoError(t, err)
	err = g.FlushAndSync()
	require.NoError(t, err)
	g.RotateFile()

	assert.Equal(t, 1, g.MaxIndex(), "MaxIndex should point to the last file")

	// Cleanup
	destroyTestGroup(t, g)
}
