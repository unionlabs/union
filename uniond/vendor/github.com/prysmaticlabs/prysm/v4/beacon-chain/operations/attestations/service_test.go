package attestations

import (
	"context"
	"errors"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestStop_OK(t *testing.T) {
	s, err := NewService(context.Background(), &Config{})
	require.NoError(t, err)
	require.NoError(t, s.Stop(), "Unable to stop attestation pool service")
	assert.ErrorContains(t, context.Canceled.Error(), s.ctx.Err(), "Context was not canceled")
}

func TestStatus_Error(t *testing.T) {
	err := errors.New("bad bad bad")
	s := &Service{err: err}
	assert.ErrorContains(t, s.err.Error(), s.Status())
}
