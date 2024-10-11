package testutil

import (
	"bytes"
	"io"
	"os"
	"strings"
	"testing"

	"github.com/spf13/cobra"
	"github.com/stretchr/testify/require"
)

// BufferReader is implemented by types that read from a string buffer.
type BufferReader interface {
	io.Reader
	Reset(string)
}

// BufferWriter is implemented by types that write to a buffer.
type BufferWriter interface {
	io.Writer
	Reset()
	Bytes() []byte
	String() string
}

// ApplyMockIO replaces stdin/out/err with buffers that can be used during testing.
// Returns an input BufferReader and an output BufferWriter.
func ApplyMockIO(c *cobra.Command) (BufferReader, BufferWriter) {
	mockIn := strings.NewReader("")
	mockOut := bytes.NewBufferString("")

	c.SetIn(mockIn)
	c.SetOut(mockOut)
	c.SetErr(mockOut)

	return mockIn, mockOut
}

// ApplyMockIODiscardOutErr replaces a cobra.Command output and error streams with a dummy io.Writer.
// Replaces and returns the io.Reader associated to the cobra.Command input stream.
func ApplyMockIODiscardOutErr(c *cobra.Command) BufferReader {
	mockIn := strings.NewReader("")

	c.SetIn(mockIn)
	c.SetOut(io.Discard)
	c.SetErr(io.Discard)

	return mockIn
}

// Write the given string to a new temporary file.
// Returns an open file for the test to use.
func WriteToNewTempFile(tb testing.TB, s string) *os.File {
	tb.Helper()

	fp := TempFile(tb)
	_, err := fp.WriteString(s)

	require.Nil(tb, err)

	return fp
}

// TempFile returns a writable temporary file for the test to use.
func TempFile(tb testing.TB) *os.File {
	tb.Helper()

	fp, err := os.CreateTemp(GetTempDir(tb), "")
	require.NoError(tb, err)

	return fp
}

// GetTempDir returns a writable temporary director for the test to use.
func GetTempDir(tb testing.TB) string {
	tb.Helper()
	// os.MkDir() is used instead of testing.T.TempDir()
	// see https://github.com/cosmos/cosmos-sdk/pull/8475 and
	// https://github.com/cosmos/cosmos-sdk/pull/10341 for
	// this change's rationale.
	tempdir, err := os.MkdirTemp("", "")
	require.NoError(tb, err)
	tb.Cleanup(func() { _ = os.RemoveAll(tempdir) })
	return tempdir
}
