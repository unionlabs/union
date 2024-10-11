// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package qlog

import (
	"bytes"
	"errors"
	"fmt"
	"log/slog"
	"strings"
	"sync"
	"testing"
	"time"
)

type testJSONOut struct {
	bytes.Buffer
}

func (o *testJSONOut) Close() error { return nil }

func newTestJSONWriter() *jsonWriter {
	return &jsonWriter{w: &testJSONOut{}}
}

func wantJSONRecord(t *testing.T, w *jsonWriter, want string) {
	t.Helper()
	want = "\x1e" + want + "\n"
	got := w.w.(*testJSONOut).String()
	if got != want {
		t.Errorf("jsonWriter contains unexpected output\ngot:  %q\nwant: %q", got, want)
	}
}

func TestJSONWriterWriteConcurrentRecords(t *testing.T) {
	w := newTestJSONWriter()
	var wg sync.WaitGroup
	for i := 0; i < 3; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			w.writeRecordStart()
			w.writeInt64Field("field", 0)
			w.writeRecordEnd()
		}()
	}
	wg.Wait()
	wantJSONRecord(t, w, strings.Join([]string{
		`{"field":0}`,
		`{"field":0}`,
		`{"field":0}`,
	}, "\n\x1e"))
}

func TestJSONWriterAttrs(t *testing.T) {
	w := newTestJSONWriter()
	w.writeRecordStart()
	w.writeAttrsField("field", []slog.Attr{
		slog.Any("any", errors.New("value")),
		slog.Bool("bool", true),
		slog.Duration("duration", 1*time.Second),
		slog.Float64("float64", 1),
		slog.Int64("int64", 1),
		slog.String("string", "value"),
		slog.Time("time", time.Date(2000, 1, 1, 0, 0, 0, 0, time.UTC)),
		slog.Uint64("uint64", 1),
		slog.Group("group", "a", 1),
	})
	w.writeRecordEnd()
	wantJSONRecord(t, w,
		`{"field":{`+
			`"any":"value",`+
			`"bool":true,`+
			`"duration":1000.000000,`+
			`"float64":1,`+
			`"int64":1,`+
			`"string":"value",`+
			`"time":946684800000.000000,`+
			`"uint64":1,`+
			`"group":{"a":1}`+
			`}}`)
}

func TestJSONWriterAttrEmpty(t *testing.T) {
	w := newTestJSONWriter()
	w.writeRecordStart()
	var a slog.Attr
	w.writeAttr(a)
	w.writeRecordEnd()
	wantJSONRecord(t, w, `{}`)
}

func TestJSONWriterObjectEmpty(t *testing.T) {
	w := newTestJSONWriter()
	w.writeRecordStart()
	w.writeObjectField("field", func() {})
	w.writeRecordEnd()
	wantJSONRecord(t, w, `{"field":{}}`)
}

func TestJSONWriterObjectFields(t *testing.T) {
	w := newTestJSONWriter()
	w.writeRecordStart()
	w.writeObjectField("field", func() {
		w.writeStringField("a", "value")
		w.writeInt64Field("b", 10)
	})
	w.writeRecordEnd()
	wantJSONRecord(t, w, `{"field":{"a":"value","b":10}}`)
}

func TestJSONWriterRawField(t *testing.T) {
	w := newTestJSONWriter()
	w.writeRecordStart()
	w.writeRawField("field", `[1]`)
	w.writeRecordEnd()
	wantJSONRecord(t, w, `{"field":[1]}`)
}

func TestJSONWriterBoolField(t *testing.T) {
	w := newTestJSONWriter()
	w.writeRecordStart()
	w.writeBoolField("true", true)
	w.writeBoolField("false", false)
	w.writeRecordEnd()
	wantJSONRecord(t, w, `{"true":true,"false":false}`)
}

func TestJSONWriterDurationField(t *testing.T) {
	w := newTestJSONWriter()
	w.writeRecordStart()
	w.writeDurationField("field1", (10*time.Millisecond)+(2*time.Nanosecond))
	w.writeDurationField("field2", -((10 * time.Millisecond) + (2 * time.Nanosecond)))
	w.writeRecordEnd()
	wantJSONRecord(t, w, `{"field1":10.000002,"field2":-10.000002}`)
}

func TestJSONWriterFloat64Field(t *testing.T) {
	w := newTestJSONWriter()
	w.writeRecordStart()
	w.writeFloat64Field("field", 1.1)
	w.writeRecordEnd()
	wantJSONRecord(t, w, `{"field":1.1}`)
}

func TestJSONWriterInt64Field(t *testing.T) {
	w := newTestJSONWriter()
	w.writeRecordStart()
	w.writeInt64Field("field", 1234)
	w.writeRecordEnd()
	wantJSONRecord(t, w, `{"field":1234}`)
}

func TestJSONWriterUint64Field(t *testing.T) {
	w := newTestJSONWriter()
	w.writeRecordStart()
	w.writeUint64Field("field", 1234)
	w.writeRecordEnd()
	wantJSONRecord(t, w, `{"field":1234}`)
}

func TestJSONWriterStringField(t *testing.T) {
	w := newTestJSONWriter()
	w.writeRecordStart()
	w.writeStringField("field", "value")
	w.writeRecordEnd()
	wantJSONRecord(t, w, `{"field":"value"}`)
}

func TestJSONWriterStringFieldEscaped(t *testing.T) {
	w := newTestJSONWriter()
	w.writeRecordStart()
	w.writeStringField("field", "va\x00ue")
	w.writeRecordEnd()
	wantJSONRecord(t, w, `{"field":"va\u0000ue"}`)
}

func TestJSONWriterStringEscaping(t *testing.T) {
	for c := 0; c <= 0xff; c++ {
		w := newTestJSONWriter()
		w.writeRecordStart()
		w.writeStringField("field", string([]byte{byte(c)}))
		w.writeRecordEnd()
		var want string
		if (c >= 0x20 && c <= 0x21) || (c >= 0x23 && c <= 0x5b) || (c >= 0x5d && c <= 0x7e) {
			want = fmt.Sprintf(`%c`, c)
		} else {
			want = fmt.Sprintf(`\u%04x`, c)
		}
		wantJSONRecord(t, w, `{"field":"`+want+`"}`)
	}
}
