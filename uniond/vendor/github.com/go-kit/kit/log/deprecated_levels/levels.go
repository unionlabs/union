// Package levels implements leveled logging on top of Go kit's log package.
//
// Deprecated: Use github.com/go-kit/log/level instead.
package levels

import "github.com/go-kit/log"

// Levels provides a leveled logging wrapper around a logger. It has five
// levels: debug, info, warning (warn), error, and critical (crit). If you
// want a different set of levels, you can create your own levels type very
// easily, and you can elide the configuration.
type Levels struct {
	logger   log.Logger
	levelKey string

	// We have a choice between storing level values in string fields or
	// making a separate context for each level. When using string fields the
	// Log method must combine the base context, the level data, and the
	// logged keyvals; but the With method only requires updating one context.
	// If we instead keep a separate context for each level the Log method
	// must only append the new keyvals; but the With method would have to
	// update all five contexts.

	// Roughly speaking, storing multiple contexts breaks even if the ratio of
	// Log/With calls is more than the number of levels. We have chosen to
	// make the With method cheap and the Log method a bit more costly because
	// we do not expect most applications to Log more than five times for each
	// call to With.

	debugValue string
	infoValue  string
	warnValue  string
	errorValue string
	critValue  string
}

// New creates a new leveled logger, wrapping the passed logger.
func New(logger log.Logger, options ...Option) Levels {
	l := Levels{
		logger:   logger,
		levelKey: "level",

		debugValue: "debug",
		infoValue:  "info",
		warnValue:  "warn",
		errorValue: "error",
		critValue:  "crit",
	}
	for _, option := range options {
		option(&l)
	}
	return l
}

// With returns a new leveled logger that includes keyvals in all log events.
func (l Levels) With(keyvals ...interface{}) Levels {
	return Levels{
		logger:     log.With(l.logger, keyvals...),
		levelKey:   l.levelKey,
		debugValue: l.debugValue,
		infoValue:  l.infoValue,
		warnValue:  l.warnValue,
		errorValue: l.errorValue,
		critValue:  l.critValue,
	}
}

// Debug returns a debug level logger.
func (l Levels) Debug() log.Logger {
	return log.WithPrefix(l.logger, l.levelKey, l.debugValue)
}

// Info returns an info level logger.
func (l Levels) Info() log.Logger {
	return log.WithPrefix(l.logger, l.levelKey, l.infoValue)
}

// Warn returns a warning level logger.
func (l Levels) Warn() log.Logger {
	return log.WithPrefix(l.logger, l.levelKey, l.warnValue)
}

// Error returns an error level logger.
func (l Levels) Error() log.Logger {
	return log.WithPrefix(l.logger, l.levelKey, l.errorValue)
}

// Crit returns a critical level logger.
func (l Levels) Crit() log.Logger {
	return log.WithPrefix(l.logger, l.levelKey, l.critValue)
}

// Option sets a parameter for leveled loggers.
type Option func(*Levels)

// Key sets the key for the field used to indicate log level. By default,
// the key is "level".
func Key(key string) Option {
	return func(l *Levels) { l.levelKey = key }
}

// DebugValue sets the value for the field used to indicate the debug log
// level. By default, the value is "debug".
func DebugValue(value string) Option {
	return func(l *Levels) { l.debugValue = value }
}

// InfoValue sets the value for the field used to indicate the info log level.
// By default, the value is "info".
func InfoValue(value string) Option {
	return func(l *Levels) { l.infoValue = value }
}

// WarnValue sets the value for the field used to indicate the warning log
// level. By default, the value is "warn".
func WarnValue(value string) Option {
	return func(l *Levels) { l.warnValue = value }
}

// ErrorValue sets the value for the field used to indicate the error log
// level. By default, the value is "error".
func ErrorValue(value string) Option {
	return func(l *Levels) { l.errorValue = value }
}

// CritValue sets the value for the field used to indicate the critical log
// level. By default, the value is "crit".
func CritValue(value string) Option {
	return func(l *Levels) { l.critValue = value }
}
