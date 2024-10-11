// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package qlog

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"log/slog"
	"reflect"
	"testing"
	"time"
)

// QLog tests are mostly in the quic package, where we can test event generation
// and serialization together.

func TestQLogHandlerEvents(t *testing.T) {
	for _, test := range []struct {
		name string
		f    func(*slog.Logger)
		want []map[string]any // events, not counting the trace header
	}{{
		name: "various types",
		f: func(log *slog.Logger) {
			log.Info("message",
				"bool", true,
				"duration", time.Duration(1*time.Second),
				"float", 0.0,
				"int", 0,
				"string", "value",
				"uint", uint64(0),
				slog.Group("group",
					"a", 0,
				),
			)
		},
		want: []map[string]any{{
			"name": "message",
			"data": map[string]any{
				"bool":     true,
				"duration": float64(1000),
				"float":    float64(0.0),
				"int":      float64(0),
				"string":   "value",
				"uint":     float64(0),
				"group": map[string]any{
					"a": float64(0),
				},
			},
		}},
	}, {
		name: "WithAttrs",
		f: func(log *slog.Logger) {
			log = log.With(
				"with_a", "a",
				"with_b", "b",
			)
			log.Info("m1", "field", "1")
			log.Info("m2", "field", "2")
		},
		want: []map[string]any{{
			"name": "m1",
			"data": map[string]any{
				"with_a": "a",
				"with_b": "b",
				"field":  "1",
			},
		}, {
			"name": "m2",
			"data": map[string]any{
				"with_a": "a",
				"with_b": "b",
				"field":  "2",
			},
		}},
	}, {
		name: "WithGroup",
		f: func(log *slog.Logger) {
			log = log.With(
				"with_a", "a",
				"with_b", "b",
			)
			log.Info("m1", "field", "1")
			log.Info("m2", "field", "2")
		},
		want: []map[string]any{{
			"name": "m1",
			"data": map[string]any{
				"with_a": "a",
				"with_b": "b",
				"field":  "1",
			},
		}, {
			"name": "m2",
			"data": map[string]any{
				"with_a": "a",
				"with_b": "b",
				"field":  "2",
			},
		}},
	}} {
		var out bytes.Buffer
		opts := HandlerOptions{
			Level: slog.LevelDebug,
			NewTrace: func(TraceInfo) (io.WriteCloser, error) {
				return nopCloseWriter{&out}, nil
			},
		}
		h, err := newJSONTraceHandler(opts, []slog.Attr{
			slog.String("group_id", "group"),
			slog.Group("vantage_point",
				slog.String("type", "client"),
			),
		})
		if err != nil {
			t.Fatal(err)
		}
		log := slog.New(h)
		test.f(log)
		got := []map[string]any{}
		for i, e := range bytes.Split(out.Bytes(), []byte{0x1e}) {
			// i==0: empty string before the initial record separator
			// i==1: trace header; not part of this test
			if i < 2 {
				continue
			}
			var val map[string]any
			if err := json.Unmarshal(e, &val); err != nil {
				panic(fmt.Errorf("log unmarshal failure: %v\n%q", err, string(e)))
			}
			delete(val, "time")
			got = append(got, val)
		}
		if !reflect.DeepEqual(got, test.want) {
			t.Errorf("event mismatch\ngot:  %v\nwant: %v", got, test.want)
		}
	}

}

type nopCloseWriter struct {
	io.Writer
}

func (nopCloseWriter) Close() error { return nil }
