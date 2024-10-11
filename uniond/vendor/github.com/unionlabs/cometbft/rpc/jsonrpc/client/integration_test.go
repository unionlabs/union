//go:build release
// +build release

// The code in here is comprehensive as an integration
// test and is long, hence is only run before releases.

package client

import (
	"bytes"
	"errors"
	"net"
	"regexp"
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/libs/log"
)

func TestWSClientReconnectWithJitter(t *testing.T) {
	n := 8
	maxReconnectAttempts := 3
	// Max wait time is ceil(1+0.999) + ceil(2+0.999) + ceil(4+0.999) + ceil(...) = 2 + 3 + 5 = 10s + ...
	maxSleepTime := time.Second * time.Duration(((1<<uint(maxReconnectAttempts))-1)+maxReconnectAttempts)

	errNotConnected := errors.New("not connected")
	clientMap := make(map[int]*WSClient)
	buf := new(bytes.Buffer)
	logger := log.NewTMLogger(buf)
	for i := 0; i < n; i++ {
		c, err := NewWS("tcp://foo", "/websocket")
		require.NoError(t, err)
		c.Dialer = func(string, string) (net.Conn, error) {
			return nil, errNotConnected
		}
		c.SetLogger(logger)
		c.maxReconnectAttempts = maxReconnectAttempts
		// Not invoking defer c.Stop() because
		// after all the reconnect attempts have been
		// exhausted, c.Stop is implicitly invoked.
		clientMap[i] = c
		// Trigger the reconnect routine that performs exponential backoff.
		go c.reconnect()
	}

	stopCount := 0
	time.Sleep(maxSleepTime)
	for key, c := range clientMap {
		if !c.IsActive() {
			delete(clientMap, key)
			stopCount++
		}
	}
	require.Equal(t, stopCount, n, "expecting all clients to have been stopped")

	// Next we have to examine the logs to ensure that no single time was repeated
	backoffDurRegexp := regexp.MustCompile(`backoff_duration=(.+)`)
	matches := backoffDurRegexp.FindAll(buf.Bytes(), -1)
	seenMap := make(map[string]int)
	for i, match := range matches {
		if origIndex, seen := seenMap[string(match)]; seen {
			t.Errorf("match #%d (%q) was seen originally at log entry #%d", i, match, origIndex)
		} else {
			seenMap[string(match)] = i
		}
	}
}
