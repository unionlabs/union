package panics

import (
	"errors"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestTry(t *testing.T) {
	t.Parallel()

	t.Run("panics", func(t *testing.T) {
		t.Parallel()

		err := errors.New("SOS")
		recovered := Try(func() { panic(err) })
		require.ErrorIs(t, recovered.AsError(), err)
		require.ErrorAs(t, recovered.AsError(), &err)
		// The exact contents aren't tested because the stacktrace contains local file paths
		// and even the structure of the stacktrace is bound to be unstable over time. Just
		// test a couple of basics.
		require.Contains(t, recovered.String(), "SOS", "formatted panic should contain the panic message")
		require.Contains(t, recovered.String(), "panics.(*Catcher).Try", recovered.String(), "formatted panic should contain the stack trace")
	})

	t.Run("no panic", func(t *testing.T) {
		t.Parallel()

		recovered := Try(func() {})
		require.Nil(t, recovered)
	})
}
