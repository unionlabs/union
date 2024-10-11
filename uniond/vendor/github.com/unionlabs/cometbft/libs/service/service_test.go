package service

import (
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

type testService struct {
	BaseService
}

func (testService) OnReset() error {
	return nil
}

func TestBaseServiceWait(t *testing.T) {
	ts := &testService{}
	ts.BaseService = *NewBaseService(nil, "TestService", ts)
	err := ts.Start()
	require.NoError(t, err)

	waitFinished := make(chan struct{})
	go func() {
		ts.Wait()
		waitFinished <- struct{}{}
	}()

	go ts.Stop() //nolint:errcheck // ignore for tests

	select {
	case <-waitFinished:
		// all good
	case <-time.After(100 * time.Millisecond):
		t.Fatal("expected Wait() to finish within 100 ms.")
	}
}

func TestBaseServiceReset(t *testing.T) {
	ts := &testService{}
	ts.BaseService = *NewBaseService(nil, "TestService", ts)
	err := ts.Start()
	require.NoError(t, err)

	err = ts.Reset()
	require.Error(t, err, "expected can't reset service error")

	err = ts.Stop()
	require.NoError(t, err)

	err = ts.Reset()
	require.NoError(t, err)

	err = ts.Start()
	require.NoError(t, err)
}
