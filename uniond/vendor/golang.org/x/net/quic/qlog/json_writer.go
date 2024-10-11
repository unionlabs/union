// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package qlog

import (
	"bytes"
	"fmt"
	"io"
	"log/slog"
	"strconv"
	"sync"
	"time"
)

// A jsonWriter writes JSON-SEQ (RFC 7464).
//
// A JSON-SEQ file consists of a series of JSON text records,
// each beginning with an RS (0x1e) character and ending with LF (0x0a).
type jsonWriter struct {
	mu  sync.Mutex
	w   io.WriteCloser
	buf bytes.Buffer
}

// writeRecordStart writes the start of a JSON-SEQ record.
func (w *jsonWriter) writeRecordStart() {
	w.mu.Lock()
	w.buf.WriteByte(0x1e)
	w.buf.WriteByte('{')
}

// writeRecordEnd finishes writing a JSON-SEQ record.
func (w *jsonWriter) writeRecordEnd() {
	w.buf.WriteByte('}')
	w.buf.WriteByte('\n')
	w.w.Write(w.buf.Bytes())
	w.buf.Reset()
	w.mu.Unlock()
}

func (w *jsonWriter) writeAttrs(attrs []slog.Attr) {
	w.buf.WriteByte('{')
	for _, a := range attrs {
		w.writeAttr(a)
	}
	w.buf.WriteByte('}')
}

func (w *jsonWriter) writeAttr(a slog.Attr) {
	if a.Key == "" {
		return
	}
	w.writeName(a.Key)
	w.writeValue(a.Value)
}

// writeAttr writes a []slog.Attr as an object field.
func (w *jsonWriter) writeAttrsField(name string, attrs []slog.Attr) {
	w.writeName(name)
	w.writeAttrs(attrs)
}

func (w *jsonWriter) writeValue(v slog.Value) {
	v = v.Resolve()
	switch v.Kind() {
	case slog.KindAny:
		switch v := v.Any().(type) {
		case []slog.Value:
			w.writeArray(v)
		case interface{ AppendJSON([]byte) []byte }:
			w.buf.Write(v.AppendJSON(w.buf.AvailableBuffer()))
		default:
			w.writeString(fmt.Sprint(v))
		}
	case slog.KindBool:
		w.writeBool(v.Bool())
	case slog.KindDuration:
		w.writeDuration(v.Duration())
	case slog.KindFloat64:
		w.writeFloat64(v.Float64())
	case slog.KindInt64:
		w.writeInt64(v.Int64())
	case slog.KindString:
		w.writeString(v.String())
	case slog.KindTime:
		w.writeTime(v.Time())
	case slog.KindUint64:
		w.writeUint64(v.Uint64())
	case slog.KindGroup:
		w.writeAttrs(v.Group())
	default:
		w.writeString("unhandled kind")
	}
}

// writeName writes an object field name followed by a colon.
func (w *jsonWriter) writeName(name string) {
	if b := w.buf.Bytes(); len(b) > 0 && b[len(b)-1] != '{' {
		// Add the comma separating this from the previous field.
		w.buf.WriteByte(',')
	}
	w.writeString(name)
	w.buf.WriteByte(':')
}

func (w *jsonWriter) writeObject(f func()) {
	w.buf.WriteByte('{')
	f()
	w.buf.WriteByte('}')
}

// writeObject writes an object-valued object field.
// The function f is called to write the contents.
func (w *jsonWriter) writeObjectField(name string, f func()) {
	w.writeName(name)
	w.writeObject(f)
}

func (w *jsonWriter) writeArray(vals []slog.Value) {
	w.buf.WriteByte('[')
	for i, v := range vals {
		if i != 0 {
			w.buf.WriteByte(',')
		}
		w.writeValue(v)
	}
	w.buf.WriteByte(']')
}

func (w *jsonWriter) writeRaw(v string) {
	w.buf.WriteString(v)
}

// writeRawField writes a field with a raw JSON value.
func (w *jsonWriter) writeRawField(name, v string) {
	w.writeName(name)
	w.writeRaw(v)
}

func (w *jsonWriter) writeBool(v bool) {
	if v {
		w.buf.WriteString("true")
	} else {
		w.buf.WriteString("false")
	}
}

// writeBoolField writes a bool-valued object field.
func (w *jsonWriter) writeBoolField(name string, v bool) {
	w.writeName(name)
	w.writeBool(v)
}

// writeDuration writes a duration as milliseconds.
func (w *jsonWriter) writeDuration(v time.Duration) {
	if v < 0 {
		w.buf.WriteByte('-')
		v = -v
	}
	fmt.Fprintf(&w.buf, "%d.%06d", v.Milliseconds(), v%time.Millisecond)
}

// writeDurationField writes a millisecond duration-valued object field.
func (w *jsonWriter) writeDurationField(name string, v time.Duration) {
	w.writeName(name)
	w.writeDuration(v)
}

func (w *jsonWriter) writeFloat64(v float64) {
	w.buf.Write(strconv.AppendFloat(w.buf.AvailableBuffer(), v, 'f', -1, 64))
}

// writeFloat64Field writes an float64-valued object field.
func (w *jsonWriter) writeFloat64Field(name string, v float64) {
	w.writeName(name)
	w.writeFloat64(v)
}

func (w *jsonWriter) writeInt64(v int64) {
	w.buf.Write(strconv.AppendInt(w.buf.AvailableBuffer(), v, 10))
}

// writeInt64Field writes an int64-valued object field.
func (w *jsonWriter) writeInt64Field(name string, v int64) {
	w.writeName(name)
	w.writeInt64(v)
}

func (w *jsonWriter) writeUint64(v uint64) {
	w.buf.Write(strconv.AppendUint(w.buf.AvailableBuffer(), v, 10))
}

// writeUint64Field writes a uint64-valued object field.
func (w *jsonWriter) writeUint64Field(name string, v uint64) {
	w.writeName(name)
	w.writeUint64(v)
}

// writeTime writes a time as seconds since the Unix epoch.
func (w *jsonWriter) writeTime(v time.Time) {
	fmt.Fprintf(&w.buf, "%d.%06d", v.UnixMilli(), v.Nanosecond()%int(time.Millisecond))
}

// writeTimeField writes a time-valued object field.
func (w *jsonWriter) writeTimeField(name string, v time.Time) {
	w.writeName(name)
	w.writeTime(v)
}

func jsonSafeSet(c byte) bool {
	// mask is a 128-bit bitmap with 1s for allowed bytes,
	// so that the byte c can be tested with a shift and an and.
	// If c > 128, then 1<<c and 1<<(c-64) will both be zero,
	// and this function will return false.
	const mask = 0 |
		(1<<(0x22-0x20)-1)<<0x20 |
		(1<<(0x5c-0x23)-1)<<0x23 |
		(1<<(0x7f-0x5d)-1)<<0x5d
	return ((uint64(1)<<c)&(mask&(1<<64-1)) |
		(uint64(1)<<(c-64))&(mask>>64)) != 0
}

func jsonNeedsEscape(s string) bool {
	for i := range s {
		if !jsonSafeSet(s[i]) {
			return true
		}
	}
	return false
}

// writeString writes an ASCII string.
//
// qlog fields should never contain anything that isn't ASCII,
// so we do the bare minimum to avoid producing invalid output if we
// do write something unexpected.
func (w *jsonWriter) writeString(v string) {
	w.buf.WriteByte('"')
	if !jsonNeedsEscape(v) {
		w.buf.WriteString(v)
	} else {
		for i := range v {
			if jsonSafeSet(v[i]) {
				w.buf.WriteByte(v[i])
			} else {
				fmt.Fprintf(&w.buf, `\u%04x`, v[i])
			}
		}
	}
	w.buf.WriteByte('"')
}

// writeStringField writes a string-valued object field.
func (w *jsonWriter) writeStringField(name, v string) {
	w.writeName(name)
	w.writeString(v)
}
