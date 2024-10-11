package log

import (
	"encoding"
	"encoding/json"
	"fmt"
	"io"

	"github.com/pkg/errors"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/pkgerrors"
)

func init() {
	zerolog.InterfaceMarshalFunc = func(i any) ([]byte, error) {
		switch v := i.(type) {
		case json.Marshaler:
			return json.Marshal(i)
		case encoding.TextMarshaler:
			return json.Marshal(i)
		case fmt.Stringer:
			return json.Marshal(v.String())
		default:
			return json.Marshal(i)
		}
	}
}

// ModuleKey defines a module logging key.
const ModuleKey = "module"

// ContextKey is used to store the logger in the context.
var ContextKey contextKey

type contextKey struct{}

// Logger is the Cosmos SDK logger interface.
// It extends cosmossdk.io/core/log.Logger to return a child logger.
// Use cosmossdk.io/core/log.Logger instead in modules.
type Logger interface {
	// Info takes a message and a set of key/value pairs and logs with level INFO.
	// The key of the tuple must be a string.
	Info(msg string, keyVals ...any)

	// Warn takes a message and a set of key/value pairs and logs with level WARN.
	// The key of the tuple must be a string.
	Warn(msg string, keyVals ...any)

	// Error takes a message and a set of key/value pairs and logs with level ERR.
	// The key of the tuple must be a string.
	Error(msg string, keyVals ...any)

	// Debug takes a message and a set of key/value pairs and logs with level DEBUG.
	// The key of the tuple must be a string.
	Debug(msg string, keyVals ...any)

	// With returns a new wrapped logger with additional context provided by a set.
	With(keyVals ...any) Logger

	// Impl returns the underlying logger implementation.
	// It is used to access the full functionalities of the underlying logger.
	// Advanced users can type cast the returned value to the actual logger.
	Impl() any
}

// WithJSONMarshal configures zerolog global json encoding.
func WithJSONMarshal(marshaler func(v any) ([]byte, error)) {
	zerolog.InterfaceMarshalFunc = func(i any) ([]byte, error) {
		switch v := i.(type) {
		case json.Marshaler:
			return marshaler(i)
		case encoding.TextMarshaler:
			return marshaler(i)
		case fmt.Stringer:
			return marshaler(v.String())
		default:
			return marshaler(i)
		}
	}
}

type zeroLogWrapper struct {
	*zerolog.Logger
}

// NewLogger returns a new logger that writes to the given destination.
//
// Typical usage from a main function is:
//
//	logger := log.NewLogger(os.Stderr)
//
// Stderr is the typical destination for logs,
// so that any output from your application can still be piped to other processes.
// The returned value can be safely cast to cosmossdk.io/core/log.Logger.
func NewLogger(dst io.Writer, options ...Option) Logger {
	logCfg := defaultConfig
	for _, opt := range options {
		opt(&logCfg)
	}

	output := dst
	if !logCfg.OutputJSON {
		output = zerolog.ConsoleWriter{
			Out:        dst,
			NoColor:    !logCfg.Color,
			TimeFormat: logCfg.TimeFormat,
		}
	}

	if logCfg.Filter != nil {
		output = NewFilterWriter(output, logCfg.Filter)
	}

	logger := zerolog.New(output)
	if logCfg.StackTrace {
		zerolog.ErrorStackMarshaler = func(err error) interface{} {
			return pkgerrors.MarshalStack(errors.WithStack(err))
		}

		logger = logger.With().Stack().Logger()
	}

	if logCfg.TimeFormat != "" {
		logger = logger.With().Timestamp().Logger()
	}

	if logCfg.Level != zerolog.NoLevel {
		logger = logger.Level(logCfg.Level)
	}

	logger = logger.Hook(logCfg.Hooks...)

	return zeroLogWrapper{&logger}
}

// NewCustomLogger returns a new logger with the given zerolog logger.
func NewCustomLogger(logger zerolog.Logger) Logger {
	return zeroLogWrapper{&logger}
}

// Info takes a message and a set of key/value pairs and logs with level INFO.
// The key of the tuple must be a string.
func (l zeroLogWrapper) Info(msg string, keyVals ...interface{}) {
	l.Logger.Info().Fields(keyVals).Msg(msg)
}

// Warn takes a message and a set of key/value pairs and logs with level WARN.
// The key of the tuple must be a string.
func (l zeroLogWrapper) Warn(msg string, keyVals ...interface{}) {
	l.Logger.Warn().Fields(keyVals).Msg(msg)
}

// Error takes a message and a set of key/value pairs and logs with level ERROR.
// The key of the tuple must be a string.
func (l zeroLogWrapper) Error(msg string, keyVals ...interface{}) {
	l.Logger.Error().Fields(keyVals).Msg(msg)
}

// Debug takes a message and a set of key/value pairs and logs with level DEBUG.
// The key of the tuple must be a string.
func (l zeroLogWrapper) Debug(msg string, keyVals ...interface{}) {
	l.Logger.Debug().Fields(keyVals).Msg(msg)
}

// With returns a new wrapped logger with additional context provided by a set.
func (l zeroLogWrapper) With(keyVals ...interface{}) Logger {
	logger := l.Logger.With().Fields(keyVals).Logger()
	return zeroLogWrapper{&logger}
}

// WithContext returns a new wrapped logger with additional context provided by a set.
func (l zeroLogWrapper) WithContext(keyVals ...interface{}) any {
	logger := l.Logger.With().Fields(keyVals).Logger()
	return zeroLogWrapper{&logger}
}

// Impl returns the underlying zerolog logger.
// It can be used to used zerolog structured API directly instead of the wrapper.
func (l zeroLogWrapper) Impl() interface{} {
	return l.Logger
}

// NewNopLogger returns a new logger that does nothing.
func NewNopLogger() Logger {
	// The custom nopLogger is about 3x faster than a zeroLogWrapper with zerolog.Nop().
	return nopLogger{}
}

// nopLogger is a Logger that does nothing when called.
// See the "specialized nop logger" benchmark and compare with the "zerolog nop logger" benchmark.
// The custom implementation is about 3x faster.
type nopLogger struct{}

func (nopLogger) Info(string, ...any)    {}
func (nopLogger) Warn(string, ...any)    {}
func (nopLogger) Error(string, ...any)   {}
func (nopLogger) Debug(string, ...any)   {}
func (nopLogger) With(...any) Logger     { return nopLogger{} }
func (nopLogger) WithContext(...any) any { return nopLogger{} }
func (nopLogger) Impl() any              { return nopLogger{} }
