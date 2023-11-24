// Copyright 2022 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build !go1.21

package slog

import (
	"time"

	"golang.org/x/exp/slog"
)

// A Record holds information about a log event.
// Copies of a Record share state.
// Do not modify a Record after handing out a copy to it.
// Call [NewRecord] to create a new Record.
// Use [Record.Clone] to create a copy with no shared state.
type Record = slog.Record

// NewRecord creates a Record from the given arguments.
// Use [Record.AddAttrs] to add attributes to the Record.
//
// NewRecord is intended for logging APIs that want to support a [Handler] as
// a backend.
func NewRecord(t time.Time, level Level, msg string, pc uintptr) Record {
	return slog.NewRecord(t, level, msg, pc)
}

// Source describes the location of a line of source code.
type Source = slog.Source
