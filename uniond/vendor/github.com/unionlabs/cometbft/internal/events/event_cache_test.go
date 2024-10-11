package events

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestEventCache_Flush(t *testing.T) {
	evsw := NewEventSwitch()
	err := evsw.Start()
	require.NoError(t, err)

	err = evsw.AddListenerForEvent("nothingness", "", func(_ EventData) {
		// Check we are not initializing an empty buffer full
		// of zeroed eventInfos in the EventCache
		require.FailNow(t, "We should never receive a message on this switch since none are fired")
	})
	require.NoError(t, err)

	evc := NewEventCache(evsw)
	evc.Flush()
	// Check after reset
	evc.Flush()
	fail := true
	pass := false
	err = evsw.AddListenerForEvent("somethingness", "something", func(_ EventData) {
		if fail {
			require.FailNow(t, "Shouldn't see a message until flushed")
		}
		pass = true
	})
	require.NoError(t, err)

	evc.FireEvent("something", struct{ int }{1})
	evc.FireEvent("something", struct{ int }{2})
	evc.FireEvent("something", struct{ int }{3})
	fail = false
	evc.Flush()
	assert.True(t, pass)
}
