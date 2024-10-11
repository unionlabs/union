package grpc_zap_test

import (
	"testing"

	grpc_logsettable "github.com/grpc-ecosystem/go-grpc-middleware/logging/settable"
	grpc_zap "github.com/grpc-ecosystem/go-grpc-middleware/logging/zap"
	"go.uber.org/zap/zaptest"
)

var grpc_logger grpc_logsettable.SettableLoggerV2

func init() {
	grpc_logger = grpc_logsettable.ReplaceGrpcLoggerV2()
}

func beforeTest(t testing.TB) {
	grpc_zap.SetGrpcLoggerV2(grpc_logger, zaptest.NewLogger(t))

	// Starting from go-1.15+ automated 'reset' can also be set:
	// t.Cleanup(func() {
	//     grpc_logger.Reset()
	// })
}

// This test illustrates setting up a testing harness that attributes
// all grpc logs emitted during the test to the test-specific log.
//
// In case of test failure, only logs emitted by this testcase will be printed.
func TestSpecificLogging(t *testing.T) {
	beforeTest(t)
	grpc_logger.Info("Test specific log-line")
}
