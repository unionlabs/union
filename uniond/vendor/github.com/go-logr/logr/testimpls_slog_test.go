//go:build go1.21
// +build go1.21

/*
Copyright 2023 The logr Authors.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

package logr

import (
	"context"
	"log/slog"
	"time"
)

var _ SlogSink = &testSlogSink{}

// testSlogSink is a trivial SlogSink implementation, just for testing, which
// calls (optional) hooks on each method.
type testSlogSink struct {
	// embed a plain LogSink
	testLogSink

	attrs  []slog.Attr
	groups []string

	fnHandle    func(ss *testSlogSink, ctx context.Context, record slog.Record)
	fnWithAttrs func(ss *testSlogSink, attrs []slog.Attr)
	fnWithGroup func(ss *testSlogSink, name string)
}

func (ss *testSlogSink) Handle(ctx context.Context, record slog.Record) error {
	if ss.fnHandle != nil {
		ss.fnHandle(ss, ctx, record)
	}
	return nil
}

func (ss *testSlogSink) WithAttrs(attrs []slog.Attr) SlogSink {
	if ss.fnWithAttrs != nil {
		ss.fnWithAttrs(ss, attrs)
	}
	out := *ss
	n := len(out.attrs)
	out.attrs = append(out.attrs[:n:n], attrs...)
	return &out
}

func (ss *testSlogSink) WithGroup(name string) SlogSink {
	if ss.fnWithGroup != nil {
		ss.fnWithGroup(ss, name)
	}
	out := *ss
	n := len(out.groups)
	out.groups = append(out.groups[:n:n], name)
	return &out
}

// passthruLogSink is a trivial LogSink implementation, which implements the
// logr.LogSink methods in terms of a slog.Handler.
type passthruLogSink struct {
	handler slog.Handler
}

func (pl passthruLogSink) Init(RuntimeInfo) {}

func (pl passthruLogSink) Enabled(int) bool { return true }

func (pl passthruLogSink) Error(_ error, msg string, kvList ...interface{}) {
	var record slog.Record
	record.Message = msg
	record.Level = slog.LevelError
	record.Time = time.Now()
	record.Add(kvList...)
	_ = pl.handler.Handle(context.Background(), record)
}

func (pl passthruLogSink) Info(_ int, msg string, kvList ...interface{}) {
	var record slog.Record
	record.Message = msg
	record.Level = slog.LevelInfo
	record.Time = time.Now()
	record.Add(kvList...)
	_ = pl.handler.Handle(context.Background(), record)
}

func (pl passthruLogSink) WithName(string) LogSink { return &pl }

func (pl passthruLogSink) WithValues(kvList ...interface{}) LogSink {
	var values slog.Record
	values.Add(kvList...)
	var attrs []slog.Attr
	add := func(attr slog.Attr) bool {
		attrs = append(attrs, attr)
		return true
	}
	values.Attrs(add)

	pl.handler = pl.handler.WithAttrs(attrs)
	return &pl
}

// passthruSlogSink is a trivial SlogSink implementation, which stubs out the
// logr.LogSink methods and passes Logr.SlogSink thru to a slog.Handler.
type passthruSlogSink struct {
	handler slog.Handler
}

func (ps passthruSlogSink) Init(RuntimeInfo)                    {}
func (ps passthruSlogSink) Enabled(int) bool                    { return true }
func (ps passthruSlogSink) Error(error, string, ...interface{}) {}
func (ps passthruSlogSink) Info(int, string, ...interface{})    {}
func (ps passthruSlogSink) WithName(string) LogSink             { return &ps }
func (ps passthruSlogSink) WithValues(...interface{}) LogSink   { return &ps }

func (ps *passthruSlogSink) Handle(ctx context.Context, record slog.Record) error {
	return ps.handler.Handle(ctx, record)
}

func (ps passthruSlogSink) WithAttrs(attrs []slog.Attr) SlogSink {
	ps.handler = ps.handler.WithAttrs(attrs)
	return &ps
}

func (ps passthruSlogSink) WithGroup(name string) SlogSink {
	ps.handler = ps.handler.WithGroup(name)
	return &ps
}
