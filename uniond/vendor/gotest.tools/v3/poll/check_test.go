package poll

import (
	"fmt"
	"os"
	"testing"

	"gotest.tools/v3/assert"
)

func TestWaitOnFile(t *testing.T) {
	fakeFilePath := "./fakefile"

	check := FileExists(fakeFilePath)

	t.Run("file does not exist", func(t *testing.T) {
		r := check(t)
		assert.Assert(t, !r.Done())
		assert.Equal(t, r.Message(), fmt.Sprintf("file %s does not exist", fakeFilePath))
	})

	os.Create(fakeFilePath)
	defer os.Remove(fakeFilePath)

	t.Run("file exists", func(t *testing.T) {
		assert.Assert(t, check(t).Done())
	})
}

func TestWaitOnSocketWithTimeout(t *testing.T) {
	t.Run("connection to unavailable address", func(t *testing.T) {
		check := Connection("tcp", "foo.bar:55555")
		r := check(t)
		assert.Assert(t, !r.Done())
		assert.Equal(t, r.Message(), "socket tcp://foo.bar:55555 not available")
	})

	t.Run("connection to ", func(t *testing.T) {
		check := Connection("tcp", "google.com:80")
		assert.Assert(t, check(t).Done())
	})
}
