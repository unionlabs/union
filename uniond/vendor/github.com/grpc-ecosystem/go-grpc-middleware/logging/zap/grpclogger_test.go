package grpc_zap

import (
	"fmt"
	"runtime"
	"testing"

	"github.com/stretchr/testify/assert"
	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
	"go.uber.org/zap/zaptest"
	"go.uber.org/zap/zaptest/observer"
	"google.golang.org/grpc/grpclog"
)

func Test_zapGrpcLogger_V(t *testing.T) {
	const (
		// The default verbosity level.
		// See https://github.com/grpc/grpc-go/blob/8ab16ef276a33df4cdb106446eeff40ff56a6928/grpclog/loggerv2.go#L108.
		normal = 0

		// Currently the only level of "being verbose".
		// For example https://github.com/grpc/grpc-go/blob/8ab16ef276a33df4cdb106446eeff40ff56a6928/grpclog/grpclog.go#L21.
		verbose = 2

		// As is mentioned in https://github.com/grpc/grpc-go/blob/8ab16ef276a33df4cdb106446eeff40ff56a6928/README.md#how-to-turn-on-logging,
		// though currently not being used in the code.
		extremelyVerbose = 99
	)

	core, _ := observer.New(zapcore.DebugLevel)
	logger := zap.New(core)
	ReplaceGrpcLoggerV2WithVerbosity(logger, verbose)
	assert.True(t, grpclog.V(normal))
	assert.True(t, grpclog.V(verbose))
	assert.False(t, grpclog.V(extremelyVerbose))
}

func TestReplaceGrpcLoggerV2(t *testing.T) {
	defer ReplaceGrpcLoggerV2(zap.NewNop())

	args := []interface{}{"message", "param"}
	cases := []struct {
		name  string
		fn    func(...interface{})
		level zapcore.Level
	}{
		{name: "Info", fn: grpclog.Info, level: zap.InfoLevel},
		{name: "Infoln", fn: grpclog.Infoln, level: zap.InfoLevel},
		{name: "Warning", fn: grpclog.Warning, level: zap.WarnLevel},
		{name: "Warningln", fn: grpclog.Warningln, level: zap.WarnLevel},
		{name: "Error", fn: grpclog.Error, level: zap.ErrorLevel},
		{name: "Errorln", fn: grpclog.Errorln, level: zap.ErrorLevel},
		{name: "Fatal", fn: grpclog.Fatal, level: zap.FatalLevel},
		{name: "Fatalln", fn: grpclog.Fatalln, level: zap.FatalLevel},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			called := false
			ReplaceGrpcLoggerV2(zaptest.NewLogger(t, zaptest.WrapOptions(zap.Hooks(func(entry zapcore.Entry) error {
				called = true
				assert.Equal(t, c.level, entry.Level)
				assert.Equal(t, fmt.Sprint(args...), entry.Message)
				_, file, _, _ := runtime.Caller(0)
				assert.Equal(t, file, entry.Caller.File)
				return nil
			}), zap.AddCaller(), zap.OnFatal(zapcore.WriteThenPanic))))

			if c.level != zap.FatalLevel {
				c.fn(args...)
			} else {
				assert.Panics(t, func() {
					c.fn(args...)
				})
			}
			assert.True(t, called, "hook not called")
		})
	}
}
