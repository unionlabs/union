package beacon_api

import (
	"context"
	"fmt"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	logtest "github.com/sirupsen/logrus/hooks/test"
)

func TestEventHandler(t *testing.T) {
	logHook := logtest.NewGlobal()

	mux := http.NewServeMux()
	mux.HandleFunc("/eth/v1/events", func(w http.ResponseWriter, r *http.Request) {
		flusher, ok := w.(http.Flusher)
		require.Equal(t, true, ok)
		_, err := fmt.Fprint(w, "head\ndata\n\n")
		require.NoError(t, err)
		flusher.Flush()
	})
	server := httptest.NewServer(mux)
	defer server.Close()

	handler := NewEventHandler(http.DefaultClient, server.URL)
	ch1 := make(chan event, 1)
	sub1 := eventSub{ch: ch1}
	ch2 := make(chan event, 1)
	sub2 := eventSub{ch: ch2}
	ch3 := make(chan event, 1)
	sub3 := eventSub{name: "sub3", ch: ch3}
	// fill up the channel so that it can't receive more events
	ch3 <- event{}
	handler.subscribe(sub1)
	handler.subscribe(sub2)
	handler.subscribe(sub3)

	require.NoError(t, handler.get(context.Background(), []string{"head"}))
	// make sure the goroutine inside handler.get is invoked
	time.Sleep(500 * time.Millisecond)

	e := <-ch1
	assert.Equal(t, "head", e.eventType)
	assert.Equal(t, "data", e.data)
	e = <-ch2
	assert.Equal(t, "head", e.eventType)
	assert.Equal(t, "data", e.data)

	assert.LogsContain(t, logHook, "Subscriber 'sub3' not ready to receive events")
}
