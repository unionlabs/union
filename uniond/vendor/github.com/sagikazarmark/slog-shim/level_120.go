// Copyright 2022 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build !go1.21

package slog

import (
	"golang.org/x/exp/slog"
)

// A Level is the importance or severity of a log event.
// The higher the level, the more important or severe the event.
type Level = slog.Level

// Level numbers are inherently arbitrary,
// but we picked them to satisfy three constraints.
// Any system can map them to another numbering scheme if it wishes.
//
// First, we wanted the default level to be Info, Since Levels are ints, Info is
// the default value for int, zero.
//
// Second, we wanted to make it easy to use levels to specify logger verbosity.
// Since a larger level means a more severe event, a logger that accepts events
// with smaller (or more negative) level means a more verbose logger. Logger
// verbosity is thus the negation of event severity, and the default verbosity
// of 0 accepts all events at least as severe as INFO.
//
// Third, we wanted some room between levels to accommodate schemes with named
// levels between ours. For example, Google Cloud Logging defines a Notice level
// between Info and Warn. Since there are only a few of these intermediate
// levels, the gap between the numbers need not be large. Our gap of 4 matches
// OpenTelemetry's mapping. Subtracting 9 from an OpenTelemetry level in the
// DEBUG, INFO, WARN and ERROR ranges converts it to the corresponding slog
// Level range. OpenTelemetry also has the names TRACE and FATAL, which slog
// does not. But those OpenTelemetry levels can still be represented as slog
// Levels by using the appropriate integers.
//
// Names for common levels.
const (
	LevelDebug Level = slog.LevelDebug
	LevelInfo  Level = slog.LevelInfo
	LevelWarn  Level = slog.LevelWarn
	LevelError Level = slog.LevelError
)

// A LevelVar is a Level variable, to allow a Handler level to change
// dynamically.
// It implements Leveler as well as a Set method,
// and it is safe for use by multiple goroutines.
// The zero LevelVar corresponds to LevelInfo.
type LevelVar = slog.LevelVar

// A Leveler provides a Level value.
//
// As Level itself implements Leveler, clients typically supply
// a Level value wherever a Leveler is needed, such as in HandlerOptions.
// Clients who need to vary the level dynamically can provide a more complex
// Leveler implementation such as *LevelVar.
type Leveler = slog.Leveler
