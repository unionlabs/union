package client

import (
	"context"
	"testing"
	"time"

	"github.com/prysmaticlabs/prysm/v4/runtime"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	logTest "github.com/sirupsen/logrus/hooks/test"
	"google.golang.org/grpc/metadata"
)

var _ runtime.Service = (*ValidatorService)(nil)
var _ GenesisFetcher = (*ValidatorService)(nil)
var _ SyncChecker = (*ValidatorService)(nil)

func TestStop_CancelsContext(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	vs := &ValidatorService{
		ctx:    ctx,
		cancel: cancel,
	}

	assert.NoError(t, vs.Stop())

	select {
	case <-time.After(1 * time.Second):
		t.Error("Context not canceled within 1s")
	case <-vs.ctx.Done():
	}
}

func TestNew_Insecure(t *testing.T) {
	hook := logTest.NewGlobal()
	_, err := NewValidatorService(context.Background(), &Config{})
	require.NoError(t, err)
	require.LogsContain(t, hook, "You are using an insecure gRPC connection")
}

func TestStatus_NoConnectionError(t *testing.T) {
	validatorService := &ValidatorService{}
	assert.ErrorContains(t, "no connection", validatorService.Status())
}

func TestStart_GrpcHeaders(t *testing.T) {
	hook := logTest.NewGlobal()
	ctx := context.Background()
	for input, output := range map[string][]string{
		"should-break": {},
		"key=value":    {"key", "value"},
		"":             {},
		",":            {},
		"key=value,Authorization=Q=": {
			"key", "value", "Authorization", "Q=",
		},
		"Authorization=this is a valid value": {
			"Authorization", "this is a valid value",
		},
	} {
		cfg := &Config{GrpcHeadersFlag: input}
		validatorService, err := NewValidatorService(ctx, cfg)
		require.NoError(t, err)
		md, _ := metadata.FromOutgoingContext(validatorService.ctx)
		if input == "should-break" {
			require.LogsContain(t, hook, "Incorrect gRPC header flag format. Skipping should-break")
		} else if len(output) == 0 {
			require.DeepEqual(t, md, metadata.MD(nil))
		} else {
			require.DeepEqual(t, md, metadata.Pairs(output...))
		}
	}
}
