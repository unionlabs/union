// Copyright 2022 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build !go1.21

package slog

import (
	"context"
	"log"

	"golang.org/x/exp/slog"
)

// Default returns the default Logger.
func Default() *Logger { return slog.Default() }

// SetDefault makes l the default Logger.
// After this call, output from the log package's default Logger
// (as with [log.Print], etc.) will be logged at LevelInfo using l's Handler.
func SetDefault(l *Logger) {
	slog.SetDefault(l)
}

// A Logger records structured information about each call to its
// Log, Debug, Info, Warn, and Error methods.
// For each call, it creates a Record and passes it to a Handler.
//
// To create a new Logger, call [New] or a Logger method
// that begins "With".
type Logger = slog.Logger

// New creates a new Logger with the given non-nil Handler.
func New(h Handler) *Logger {
	return slog.New(h)
}

// With calls Logger.With on the default logger.
func With(args ...any) *Logger {
	return slog.With(args...)
}

// NewLogLogger returns a new log.Logger such that each call to its Output method
// dispatches a Record to the specified handler. The logger acts as a bridge from
// the older log API to newer structured logging handlers.
func NewLogLogger(h Handler, level Level) *log.Logger {
	return slog.NewLogLogger(h, level)
}

// Debug calls Logger.Debug on the default logger.
func Debug(msg string, args ...any) {
	slog.Debug(msg, args...)
}

// DebugContext calls Logger.DebugContext on the default logger.
func DebugContext(ctx context.Context, msg string, args ...any) {
	slog.DebugContext(ctx, msg, args...)
}

// Info calls Logger.Info on the default logger.
func Info(msg string, args ...any) {
	slog.Info(msg, args...)
}

// InfoContext calls Logger.InfoContext on the default logger.
func InfoContext(ctx context.Context, msg string, args ...any) {
	slog.InfoContext(ctx, msg, args...)
}

// Warn calls Logger.Warn on the default logger.
func Warn(msg string, args ...any) {
	slog.Warn(msg, args...)
}

// WarnContext calls Logger.WarnContext on the default logger.
func WarnContext(ctx context.Context, msg string, args ...any) {
	slog.WarnContext(ctx, msg, args...)
}

// Error calls Logger.Error on the default logger.
func Error(msg string, args ...any) {
	slog.Error(msg, args...)
}

// ErrorContext calls Logger.ErrorContext on the default logger.
func ErrorContext(ctx context.Context, msg string, args ...any) {
	slog.ErrorContext(ctx, msg, args...)
}

// Log calls Logger.Log on the default logger.
func Log(ctx context.Context, level Level, msg string, args ...any) {
	slog.Log(ctx, level, msg, args...)
}

// LogAttrs calls Logger.LogAttrs on the default logger.
func LogAttrs(ctx context.Context, level Level, msg string, attrs ...Attr) {
	slog.LogAttrs(ctx, level, msg, attrs...)
}
