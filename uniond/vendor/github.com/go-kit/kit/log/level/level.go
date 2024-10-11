package level

import (
	"github.com/go-kit/log"
	"github.com/go-kit/log/level"
)

// Error returns a logger that includes a Key/ErrorValue pair.
func Error(logger log.Logger) log.Logger {
	return level.Error(logger)
}

// Warn returns a logger that includes a Key/WarnValue pair.
func Warn(logger log.Logger) log.Logger {
	return level.Warn(logger)
}

// Info returns a logger that includes a Key/InfoValue pair.
func Info(logger log.Logger) log.Logger {
	return level.Info(logger)
}

// Debug returns a logger that includes a Key/DebugValue pair.
func Debug(logger log.Logger) log.Logger {
	return level.Debug(logger)
}

// NewFilter wraps next and implements level filtering. See the commentary on
// the Option functions for a detailed description of how to configure levels.
// If no options are provided, all leveled log events created with Debug,
// Info, Warn or Error helper methods are squelched and non-leveled log
// events are passed to next unmodified.
func NewFilter(next log.Logger, options ...Option) log.Logger {
	return level.NewFilter(next, options...)
}

// Option sets a parameter for the leveled logger.
type Option = level.Option

// AllowAll is an alias for AllowDebug.
func AllowAll() Option {
	return level.AllowAll()
}

// AllowDebug allows error, warn, info and debug level log events to pass.
func AllowDebug() Option {
	return level.AllowDebug()
}

// AllowInfo allows error, warn and info level log events to pass.
func AllowInfo() Option {
	return level.AllowInfo()
}

// AllowWarn allows error and warn level log events to pass.
func AllowWarn() Option {
	return level.AllowWarn()
}

// AllowError allows only error level log events to pass.
func AllowError() Option {
	return level.AllowError()
}

// AllowNone allows no leveled log events to pass.
func AllowNone() Option {
	return level.AllowNone()
}

// ErrNotAllowed sets the error to return from Log when it squelches a log
// event disallowed by the configured Allow[Level] option. By default,
// ErrNotAllowed is nil; in this case the log event is squelched with no
// error.
func ErrNotAllowed(err error) Option {
	return level.ErrNotAllowed(err)
}

// SquelchNoLevel instructs Log to squelch log events with no level, so that
// they don't proceed through to the wrapped logger. If SquelchNoLevel is set
// to true and a log event is squelched in this way, the error value
// configured with ErrNoLevel is returned to the caller.
func SquelchNoLevel(squelch bool) Option {
	return level.SquelchNoLevel(squelch)
}

// ErrNoLevel sets the error to return from Log when it squelches a log event
// with no level. By default, ErrNoLevel is nil; in this case the log event is
// squelched with no error.
func ErrNoLevel(err error) Option {
	return level.ErrNoLevel(err)
}

// NewInjector wraps next and returns a logger that adds a Key/level pair to
// the beginning of log events that don't already contain a level. In effect,
// this gives a default level to logs without a level.
func NewInjector(next log.Logger, lvl Value) log.Logger {
	return level.NewInjector(next, lvl)
}

// Value is the interface that each of the canonical level values implement.
// It contains unexported methods that prevent types from other packages from
// implementing it and guaranteeing that NewFilter can distinguish the levels
// defined in this package from all other values.
type Value = level.Value

// Key returns the unique key added to log events by the loggers in this
// package.
func Key() interface{} { return level.Key() }

// ErrorValue returns the unique value added to log events by Error.
func ErrorValue() Value { return level.ErrorValue() }

// WarnValue returns the unique value added to log events by Warn.
func WarnValue() Value { return level.WarnValue() }

// InfoValue returns the unique value added to log events by Info.
func InfoValue() Value { return level.InfoValue() }

// DebugValue returns the unique value added to log events by Debug.
func DebugValue() Value { return level.DebugValue() }
