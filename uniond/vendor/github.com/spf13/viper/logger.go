package viper

import (
	"context"

	slog "github.com/sagikazarmark/slog-shim"
)

// Logger is a unified interface for various logging use cases and practices, including:
//   - leveled logging
//   - structured logging
//
// Deprecated: use `log/slog` instead.
type Logger interface {
	// Trace logs a Trace event.
	//
	// Even more fine-grained information than Debug events.
	// Loggers not supporting this level should fall back to Debug.
	Trace(msg string, keyvals ...any)

	// Debug logs a Debug event.
	//
	// A verbose series of information events.
	// They are useful when debugging the system.
	Debug(msg string, keyvals ...any)

	// Info logs an Info event.
	//
	// General information about what's happening inside the system.
	Info(msg string, keyvals ...any)

	// Warn logs a Warn(ing) event.
	//
	// Non-critical events that should be looked at.
	Warn(msg string, keyvals ...any)

	// Error logs an Error event.
	//
	// Critical events that require immediate attention.
	// Loggers commonly provide Fatal and Panic levels above Error level,
	// but exiting and panicking is out of scope for a logging library.
	Error(msg string, keyvals ...any)
}

// WithLogger sets a custom logger.
func WithLogger(l *slog.Logger) Option {
	return optionFunc(func(v *Viper) {
		v.logger = l
	})
}

type discardHandler struct{}

func (n *discardHandler) Enabled(_ context.Context, _ slog.Level) bool {
	return false
}

func (n *discardHandler) Handle(_ context.Context, _ slog.Record) error {
	return nil
}

func (n *discardHandler) WithAttrs(_ []slog.Attr) slog.Handler {
	return n
}

func (n *discardHandler) WithGroup(_ string) slog.Handler {
	return n
}
