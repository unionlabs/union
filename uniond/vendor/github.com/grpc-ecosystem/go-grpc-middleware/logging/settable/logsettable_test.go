package grpc_logsettable_test

import (
	"bytes"
	"io/ioutil"
	"os"
	"testing"

	grpc_logsettable "github.com/grpc-ecosystem/go-grpc-middleware/logging/settable"
	"github.com/stretchr/testify/assert"
	"google.golang.org/grpc/grpclog"
)

func ExampleSettableLoggerV2_init() {
	l1 := grpclog.NewLoggerV2(ioutil.Discard, ioutil.Discard, ioutil.Discard)
	l2 := grpclog.NewLoggerV2(os.Stdout, os.Stdout, os.Stdout)

	settableLogger := grpc_logsettable.ReplaceGrpcLoggerV2()
	grpclog.Info("Discarded by default")

	settableLogger.Set(l1)
	grpclog.Info("Discarded log by l1")

	settableLogger.Set(l2)
	grpclog.Info("Emitted log by l2")
	// Expected output: INFO: 2021/03/15 12:59:54 [Emitted log by l2]
}

func TestSettableLoggerV2_init(t *testing.T) {
	l1buffer := &bytes.Buffer{}
	l1 := grpclog.NewLoggerV2(l1buffer, l1buffer, l1buffer)

	l2buffer := &bytes.Buffer{}
	l2 := grpclog.NewLoggerV2(l2buffer, l2buffer, l2buffer)

	settableLogger := grpc_logsettable.ReplaceGrpcLoggerV2()
	grpclog.Info("Discarded by default")

	settableLogger.Set(l1)
	grpclog.SetLoggerV2(settableLogger)
	grpclog.Info("Emitted log by l1")

	settableLogger.Set(l2)
	grpclog.Info("Emitted log by l2")

	assert.Contains(t, l1buffer.String(), "Emitted log by l1")
	assert.Contains(t, l2buffer.String(), "Emitted log by l2")
}
