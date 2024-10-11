package zerolog

import (
	"bytes"
	"context"
	"io"
	"reflect"
	"testing"

	"github.com/rs/zerolog/internal/cbor"
)

func TestCtx(t *testing.T) {
	log := New(io.Discard)
	ctx := log.WithContext(context.Background())
	log2 := Ctx(ctx)
	if !reflect.DeepEqual(log, *log2) {
		t.Error("Ctx did not return the expected logger")
	}

	// update
	log = log.Level(InfoLevel)
	ctx = log.WithContext(ctx)
	log2 = Ctx(ctx)
	if !reflect.DeepEqual(log, *log2) {
		t.Error("Ctx did not return the expected logger")
	}

	log2 = Ctx(context.Background())
	if log2 != disabledLogger {
		t.Error("Ctx did not return the expected logger")
	}

	DefaultContextLogger = &log
	t.Cleanup(func() { DefaultContextLogger = nil })
	log2 = Ctx(context.Background())
	if log2 != &log {
		t.Error("Ctx did not return the expected logger")
	}
}

func TestCtxDisabled(t *testing.T) {
	dl := New(io.Discard).Level(Disabled)
	ctx := dl.WithContext(context.Background())
	if ctx != context.Background() {
		t.Error("WithContext stored a disabled logger")
	}

	l := New(io.Discard).With().Str("foo", "bar").Logger()
	ctx = l.WithContext(ctx)
	if !reflect.DeepEqual(Ctx(ctx), &l) {
		t.Error("WithContext did not store logger")
	}

	l.UpdateContext(func(c Context) Context {
		return c.Str("bar", "baz")
	})
	ctx = l.WithContext(ctx)
	if !reflect.DeepEqual(Ctx(ctx), &l) {
		t.Error("WithContext did not store updated logger")
	}

	l = l.Level(DebugLevel)
	ctx = l.WithContext(ctx)
	if !reflect.DeepEqual(Ctx(ctx), &l) {
		t.Error("WithContext did not store copied logger")
	}

	ctx = dl.WithContext(ctx)
	if !reflect.DeepEqual(Ctx(ctx), &dl) {
		t.Error("WithContext did not override logger with a disabled logger")
	}
}

type logObjectMarshalerImpl struct {
	name string
	age  int
}

func (t logObjectMarshalerImpl) MarshalZerologObject(e *Event) {
	e.Str("name", "custom_value").Int("age", t.age)
}

func Test_InterfaceLogObjectMarshaler(t *testing.T) {
	var buf bytes.Buffer
	log := New(&buf)
	ctx := log.WithContext(context.Background())

	log2 := Ctx(ctx)

	withLog := log2.With().Interface("obj", &logObjectMarshalerImpl{
		name: "foo",
		age:  29,
	}).Logger()

	withLog.Info().Msg("test")

	if got, want := cbor.DecodeIfBinaryToString(buf.Bytes()), `{"level":"info","obj":{"name":"custom_value","age":29},"message":"test"}`+"\n"; got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}
