// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package qlog

import (
	"context"
	"log/slog"
)

type withAttrsHandler struct {
	attrs []slog.Attr
	h     slog.Handler
}

func withAttrs(h slog.Handler, attrs []slog.Attr) slog.Handler {
	if len(attrs) == 0 {
		return h
	}
	return &withAttrsHandler{attrs: attrs, h: h}
}

func (h *withAttrsHandler) Enabled(ctx context.Context, level slog.Level) bool {
	return h.h.Enabled(ctx, level)
}

func (h *withAttrsHandler) Handle(ctx context.Context, r slog.Record) error {
	r.AddAttrs(h.attrs...)
	return h.h.Handle(ctx, r)
}

func (h *withAttrsHandler) WithAttrs(attrs []slog.Attr) slog.Handler {
	return withAttrs(h, attrs)
}

func (h *withAttrsHandler) WithGroup(name string) slog.Handler {
	return withGroup(h, name)
}

type withGroupHandler struct {
	name string
	h    slog.Handler
}

func withGroup(h slog.Handler, name string) slog.Handler {
	if name == "" {
		return h
	}
	return &withGroupHandler{name: name, h: h}
}

func (h *withGroupHandler) Enabled(ctx context.Context, level slog.Level) bool {
	return h.h.Enabled(ctx, level)
}

func (h *withGroupHandler) Handle(ctx context.Context, r slog.Record) error {
	var attrs []slog.Attr
	r.Attrs(func(a slog.Attr) bool {
		attrs = append(attrs, a)
		return true
	})
	nr := slog.NewRecord(r.Time, r.Level, r.Message, r.PC)
	nr.Add(slog.Any(h.name, slog.GroupValue(attrs...)))
	return h.h.Handle(ctx, nr)
}

func (h *withGroupHandler) WithAttrs(attrs []slog.Attr) slog.Handler {
	return withAttrs(h, attrs)
}

func (h *withGroupHandler) WithGroup(name string) slog.Handler {
	return withGroup(h, name)
}
