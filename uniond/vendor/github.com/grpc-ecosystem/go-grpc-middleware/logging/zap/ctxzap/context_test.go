package ctxzap

import (
	"context"
	"runtime"
	"testing"

	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
	"go.uber.org/zap/zaptest"
)

func TestShorthands(t *testing.T) {
	cases := []struct {
		fn    func(ctx context.Context, msg string, fields ...zapcore.Field)
		level zapcore.Level
	}{
		{Debug, zap.DebugLevel},
		{Info, zap.InfoLevel},
		{Warn, zap.WarnLevel},
		{Error, zap.ErrorLevel},
	}
	const message = "omg!"
	for _, c := range cases {
		t.Run(c.level.String(), func(t *testing.T) {
			called := false
			logger := zaptest.NewLogger(t, zaptest.WrapOptions(zap.Hooks(func(e zapcore.Entry) error {
				called = true
				if e.Level != c.level {
					t.Fatalf("Expected %v, got %v", c.level, e.Level)
				}
				if e.Message != message {
					t.Fatalf("message: expected %v, got %v", message, e.Message)
				}
				if _, file, _, _ := runtime.Caller(0); e.Caller.File != file {
					t.Errorf("caller: expected %v, got %v", file, e.Caller.File)
				}
				return nil
			}))).WithOptions(zap.AddCaller())
			ctx := ToContext(context.Background(), logger)
			c.fn(ctx, message)
			if !called {
				t.Fatal("hook not called")
			}
		})
	}
}

func TestShorthandsNoop(t *testing.T) {
	// Just check we don't panic if there is no logger in the context.
	Debug(context.Background(), "no-op")
	Info(context.Background(), "no-op")
	Warn(context.Background(), "no-op")
	Error(context.Background(), "no-op")
}
