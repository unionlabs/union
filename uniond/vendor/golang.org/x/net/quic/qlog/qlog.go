// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

// Package qlog serializes qlog events.
package qlog

import (
	"bytes"
	"context"
	"errors"
	"io"
	"log/slog"
	"os"
	"path/filepath"
	"sync"
	"time"
)

// Vantage is the vantage point of a trace.
type Vantage string

const (
	// VantageEndpoint traces contain events not specific to a single connection.
	VantageEndpoint = Vantage("endpoint")

	// VantageClient traces follow a connection from the client's perspective.
	VantageClient = Vantage("client")

	// VantageServer traces follow a connection from the server's perspective.
	VantageServer = Vantage("server")
)

// TraceInfo contains information about a trace.
type TraceInfo struct {
	// Vantage is the vantage point of the trace.
	Vantage Vantage

	// GroupID identifies the logical group the trace belongs to.
	// For a connection trace, the group will be the same for
	// both the client and server vantage points.
	GroupID string
}

// HandlerOptions are options for a JSONHandler.
type HandlerOptions struct {
	// Level reports the minimum record level that will be logged.
	// If Level is nil, the handler assumes QLogLevelEndpoint.
	Level slog.Leveler

	// Dir is the directory in which to create trace files.
	// The handler will create one file per connection.
	// If NewTrace is non-nil or Dir is "", the handler will not create files.
	Dir string

	// NewTrace is called to create a new trace.
	// If NewTrace is nil and Dir is set,
	// the handler will create a new file in Dir for each trace.
	NewTrace func(TraceInfo) (io.WriteCloser, error)
}

type endpointHandler struct {
	opts HandlerOptions

	traceOnce sync.Once
	trace     *jsonTraceHandler
}

// NewJSONHandler returns a handler which serializes qlog events to JSON.
//
// The handler will write an endpoint-wide trace,
// and a separate trace for each connection.
// The HandlerOptions control the location traces are written.
//
// It uses the streamable JSON Text Sequences mapping (JSON-SEQ)
// defined in draft-ietf-quic-qlog-main-schema-04, Section 6.2.
//
// A JSONHandler may be used as the handler for a quic.Config.QLogLogger.
// It is not a general-purpose slog handler,
// and may not properly handle events from other sources.
func NewJSONHandler(opts HandlerOptions) slog.Handler {
	if opts.Dir == "" && opts.NewTrace == nil {
		return slogDiscard{}
	}
	return &endpointHandler{
		opts: opts,
	}
}

func (h *endpointHandler) Enabled(ctx context.Context, level slog.Level) bool {
	return enabled(h.opts.Level, level)
}

func (h *endpointHandler) Handle(ctx context.Context, r slog.Record) error {
	h.traceOnce.Do(func() {
		h.trace, _ = newJSONTraceHandler(h.opts, nil)
	})
	if h.trace != nil {
		h.trace.Handle(ctx, r)
	}
	return nil
}

func (h *endpointHandler) WithAttrs(attrs []slog.Attr) slog.Handler {
	// Create a new trace output file for each top-level WithAttrs.
	tr, err := newJSONTraceHandler(h.opts, attrs)
	if err != nil {
		return withAttrs(h, attrs)
	}
	return tr
}

func (h *endpointHandler) WithGroup(name string) slog.Handler {
	return withGroup(h, name)
}

type jsonTraceHandler struct {
	level slog.Leveler
	w     jsonWriter
	start time.Time
	buf   bytes.Buffer
}

func newJSONTraceHandler(opts HandlerOptions, attrs []slog.Attr) (*jsonTraceHandler, error) {
	w, err := newTraceWriter(opts, traceInfoFromAttrs(attrs))
	if err != nil {
		return nil, err
	}

	// For testing, it might be nice to set the start time used for relative timestamps
	// to the time of the first event.
	//
	// At the expense of some additional complexity here, we could defer writing
	// the reference_time header field until the first event is processed.
	//
	// Just use the current time for now.
	start := time.Now()

	h := &jsonTraceHandler{
		w:     jsonWriter{w: w},
		level: opts.Level,
		start: start,
	}
	h.writeHeader(attrs)
	return h, nil
}

func traceInfoFromAttrs(attrs []slog.Attr) TraceInfo {
	info := TraceInfo{
		Vantage: VantageEndpoint, // default if not specified
	}
	for _, a := range attrs {
		if a.Key == "group_id" && a.Value.Kind() == slog.KindString {
			info.GroupID = a.Value.String()
		}
		if a.Key == "vantage_point" && a.Value.Kind() == slog.KindGroup {
			for _, aa := range a.Value.Group() {
				if aa.Key == "type" && aa.Value.Kind() == slog.KindString {
					info.Vantage = Vantage(aa.Value.String())
				}
			}
		}
	}
	return info
}

func newTraceWriter(opts HandlerOptions, info TraceInfo) (io.WriteCloser, error) {
	var w io.WriteCloser
	var err error
	if opts.NewTrace != nil {
		w, err = opts.NewTrace(info)
	} else if opts.Dir != "" {
		var filename string
		if info.GroupID != "" {
			filename = info.GroupID + "_"
		}
		filename += string(info.Vantage) + ".sqlog"
		if !filepath.IsLocal(filename) {
			return nil, errors.New("invalid trace filename")
		}
		w, err = os.OpenFile(filepath.Join(opts.Dir, filename), os.O_CREATE|os.O_EXCL|os.O_WRONLY, 0666)
	} else {
		err = errors.New("no log destination")
	}
	return w, err
}

func (h *jsonTraceHandler) writeHeader(attrs []slog.Attr) {
	h.w.writeRecordStart()
	defer h.w.writeRecordEnd()

	// At the time of writing this comment the most recent version is 0.4,
	// but qvis only supports up to 0.3.
	h.w.writeStringField("qlog_version", "0.3")
	h.w.writeStringField("qlog_format", "JSON-SEQ")

	// The attrs flatten both common trace event fields and Trace fields.
	// This identifies the fields that belong to the Trace.
	isTraceSeqField := func(s string) bool {
		switch s {
		case "title", "description", "configuration", "vantage_point":
			return true
		}
		return false
	}

	h.w.writeObjectField("trace", func() {
		h.w.writeObjectField("common_fields", func() {
			h.w.writeRawField("protocol_type", `["QUIC"]`)
			h.w.writeStringField("time_format", "relative")
			h.w.writeTimeField("reference_time", h.start)
			for _, a := range attrs {
				if !isTraceSeqField(a.Key) {
					h.w.writeAttr(a)
				}
			}
		})
		for _, a := range attrs {
			if isTraceSeqField(a.Key) {
				h.w.writeAttr(a)
			}
		}
	})
}

func (h *jsonTraceHandler) Enabled(ctx context.Context, level slog.Level) bool {
	return enabled(h.level, level)
}

func (h *jsonTraceHandler) Handle(ctx context.Context, r slog.Record) error {
	h.w.writeRecordStart()
	defer h.w.writeRecordEnd()
	h.w.writeDurationField("time", r.Time.Sub(h.start))
	h.w.writeStringField("name", r.Message)
	h.w.writeObjectField("data", func() {
		r.Attrs(func(a slog.Attr) bool {
			h.w.writeAttr(a)
			return true
		})
	})
	return nil
}

func (h *jsonTraceHandler) WithAttrs(attrs []slog.Attr) slog.Handler {
	return withAttrs(h, attrs)
}

func (h *jsonTraceHandler) WithGroup(name string) slog.Handler {
	return withGroup(h, name)
}

func enabled(leveler slog.Leveler, level slog.Level) bool {
	var minLevel slog.Level
	if leveler != nil {
		minLevel = leveler.Level()
	}
	return level >= minLevel
}

type slogDiscard struct{}

func (slogDiscard) Enabled(context.Context, slog.Level) bool        { return false }
func (slogDiscard) Handle(ctx context.Context, r slog.Record) error { return nil }
func (slogDiscard) WithAttrs(attrs []slog.Attr) slog.Handler        { return slogDiscard{} }
func (slogDiscard) WithGroup(name string) slog.Handler              { return slogDiscard{} }
