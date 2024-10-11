package input

import (
	"bufio"
	"errors"
	"io"
	"testing"

	"github.com/stretchr/testify/require"
)

type fakeReader struct {
	fnc func(p []byte) (int, error)
}

func (f fakeReader) Read(p []byte) (int, error) {
	return f.fnc(p)
}

var _ io.Reader = fakeReader{}

func TestReadLineFromBuf(t *testing.T) {
	var fr fakeReader

	t.Run("it correctly returns the password when reader returns EOF", func(t *testing.T) {
		fr.fnc = func(p []byte) (int, error) {
			return copy(p, []byte("hello")), io.EOF
		}
		buf := bufio.NewReader(fr)

		pass, err := readLineFromBuf(buf)
		require.NoError(t, err)
		require.Equal(t, "hello", pass)
	})

	t.Run("it returns EOF if reader has been exhausted", func(t *testing.T) {
		fr.fnc = func(p []byte) (int, error) {
			return 0, io.EOF
		}
		buf := bufio.NewReader(fr)

		_, err := readLineFromBuf(buf)
		require.ErrorIs(t, err, io.EOF)
	})

	t.Run("it returns the error if it's not EOF regardless if it read something or not", func(t *testing.T) {
		expectedErr := errors.New("oh no")
		fr.fnc = func(p []byte) (int, error) {
			return copy(p, []byte("hello")), expectedErr
		}
		buf := bufio.NewReader(fr)

		_, err := readLineFromBuf(buf)
		require.ErrorIs(t, err, expectedErr)
	})
}
