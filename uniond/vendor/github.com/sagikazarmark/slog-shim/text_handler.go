// Copyright 2022 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package slog

import (
	"io"
	"log/slog"
)

// TextHandler is a Handler that writes Records to an io.Writer as a
// sequence of key=value pairs separated by spaces and followed by a newline.
type TextHandler = slog.TextHandler

// NewTextHandler creates a TextHandler that writes to w,
// using the given options.
// If opts is nil, the default options are used.
func NewTextHandler(w io.Writer, opts *HandlerOptions) *TextHandler {
	return slog.NewTextHandler(w, opts)
}
