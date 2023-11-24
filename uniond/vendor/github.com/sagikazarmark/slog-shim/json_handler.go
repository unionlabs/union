// Copyright 2022 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package slog

import (
	"io"
	"log/slog"
)

// JSONHandler is a Handler that writes Records to an io.Writer as
// line-delimited JSON objects.
type JSONHandler = slog.JSONHandler

// NewJSONHandler creates a JSONHandler that writes to w,
// using the given options.
// If opts is nil, the default options are used.
func NewJSONHandler(w io.Writer, opts *HandlerOptions) *JSONHandler {
	return slog.NewJSONHandler(w, opts)
}
