// Copyright 2022 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package slog

import (
	"log/slog"
)

// A Handler handles log records produced by a Logger..
//
// A typical handler may print log records to standard error,
// or write them to a file or database, or perhaps augment them
// with additional attributes and pass them on to another handler.
//
// Any of the Handler's methods may be called concurrently with itself
// or with other methods. It is the responsibility of the Handler to
// manage this concurrency.
//
// Users of the slog package should not invoke Handler methods directly.
// They should use the methods of [Logger] instead.
type Handler = slog.Handler

// HandlerOptions are options for a TextHandler or JSONHandler.
// A zero HandlerOptions consists entirely of default values.
type HandlerOptions = slog.HandlerOptions

// Keys for "built-in" attributes.
const (
	// TimeKey is the key used by the built-in handlers for the time
	// when the log method is called. The associated Value is a [time.Time].
	TimeKey = slog.TimeKey
	// LevelKey is the key used by the built-in handlers for the level
	// of the log call. The associated value is a [Level].
	LevelKey = slog.LevelKey
	// MessageKey is the key used by the built-in handlers for the
	// message of the log call. The associated value is a string.
	MessageKey = slog.MessageKey
	// SourceKey is the key used by the built-in handlers for the source file
	// and line of the log call. The associated value is a string.
	SourceKey = slog.SourceKey
)
