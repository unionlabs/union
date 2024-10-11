package zerolog

import (
	"context"
	"errors"
	"io"
	"net"
	"testing"
	"time"
)

var (
	errExample  = errors.New("fail")
	fakeMessage = "Test logging, but use a somewhat realistic message length."
)

func BenchmarkLogEmpty(b *testing.B) {
	logger := New(io.Discard)
	b.ResetTimer()
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			logger.Log().Msg("")
		}
	})
}

func BenchmarkDisabled(b *testing.B) {
	logger := New(io.Discard).Level(Disabled)
	b.ResetTimer()
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			logger.Info().Msg(fakeMessage)
		}
	})
}

func BenchmarkInfo(b *testing.B) {
	logger := New(io.Discard)
	b.ResetTimer()
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			logger.Info().Msg(fakeMessage)
		}
	})
}

func BenchmarkContextFields(b *testing.B) {
	logger := New(io.Discard).With().
		Str("string", "four!").
		Time("time", time.Time{}).
		Int("int", 123).
		Float32("float", -2.203230293249593).
		Logger()
	b.ResetTimer()
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			logger.Info().Msg(fakeMessage)
		}
	})
}

func BenchmarkContextAppend(b *testing.B) {
	logger := New(io.Discard).With().
		Str("foo", "bar").
		Logger()
	b.ResetTimer()
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			logger.With().Str("bar", "baz")
		}
	})
}

func BenchmarkLogFields(b *testing.B) {
	logger := New(io.Discard)
	b.ResetTimer()
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			logger.Info().
				Str("string", "four!").
				Time("time", time.Time{}).
				Int("int", 123).
				Float32("float", -2.203230293249593).
				Msg(fakeMessage)
		}
	})
}

type obj struct {
	Pub  string
	Tag  string `json:"tag"`
	priv int
}

func (o obj) MarshalZerologObject(e *Event) {
	e.Str("Pub", o.Pub).
		Str("Tag", o.Tag).
		Int("priv", o.priv)
}

func BenchmarkLogArrayObject(b *testing.B) {
	obj1 := obj{"a", "b", 2}
	obj2 := obj{"c", "d", 3}
	obj3 := obj{"e", "f", 4}
	logger := New(io.Discard)
	b.ResetTimer()
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		arr := Arr()
		arr.Object(&obj1)
		arr.Object(&obj2)
		arr.Object(&obj3)
		logger.Info().Array("objects", arr).Msg("test")
	}
}

func BenchmarkLogFieldType(b *testing.B) {
	bools := []bool{true, false, true, false, true, false, true, false, true, false}
	ints := []int{0, 1, 2, 3, 4, 5, 6, 7, 8, 9}
	floats := []float64{0, 1, 2, 3, 4, 5, 6, 7, 8, 9}
	strings := []string{"a", "b", "c", "d", "e", "f", "g", "h", "i", "j"}
	durations := []time.Duration{0, 1, 2, 3, 4, 5, 6, 7, 8, 9}
	times := []time.Time{
		time.Unix(0, 0),
		time.Unix(1, 0),
		time.Unix(2, 0),
		time.Unix(3, 0),
		time.Unix(4, 0),
		time.Unix(5, 0),
		time.Unix(6, 0),
		time.Unix(7, 0),
		time.Unix(8, 0),
		time.Unix(9, 0),
	}
	interfaces := []struct {
		Pub  string
		Tag  string `json:"tag"`
		priv int
	}{
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
	}
	objects := []obj{
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
	}
	errs := []error{errors.New("a"), errors.New("b"), errors.New("c"), errors.New("d"), errors.New("e")}
	ctx := context.Background()
	types := map[string]func(e *Event) *Event{
		"Bool": func(e *Event) *Event {
			return e.Bool("k", bools[0])
		},
		"Bools": func(e *Event) *Event {
			return e.Bools("k", bools)
		},
		"Int": func(e *Event) *Event {
			return e.Int("k", ints[0])
		},
		"Ints": func(e *Event) *Event {
			return e.Ints("k", ints)
		},
		"Float": func(e *Event) *Event {
			return e.Float64("k", floats[0])
		},
		"Floats": func(e *Event) *Event {
			return e.Floats64("k", floats)
		},
		"Str": func(e *Event) *Event {
			return e.Str("k", strings[0])
		},
		"Strs": func(e *Event) *Event {
			return e.Strs("k", strings)
		},
		"Err": func(e *Event) *Event {
			return e.Err(errs[0])
		},
		"Errs": func(e *Event) *Event {
			return e.Errs("k", errs)
		},
		"Ctx": func(e *Event) *Event {
			return e.Ctx(ctx)
		},
		"Time": func(e *Event) *Event {
			return e.Time("k", times[0])
		},
		"Times": func(e *Event) *Event {
			return e.Times("k", times)
		},
		"Dur": func(e *Event) *Event {
			return e.Dur("k", durations[0])
		},
		"Durs": func(e *Event) *Event {
			return e.Durs("k", durations)
		},
		"Interface": func(e *Event) *Event {
			return e.Interface("k", interfaces[0])
		},
		"Interfaces": func(e *Event) *Event {
			return e.Interface("k", interfaces)
		},
		"Interface(Object)": func(e *Event) *Event {
			return e.Interface("k", objects[0])
		},
		"Interface(Objects)": func(e *Event) *Event {
			return e.Interface("k", objects)
		},
		"Object": func(e *Event) *Event {
			return e.Object("k", objects[0])
		},
	}
	logger := New(io.Discard)
	b.ResetTimer()
	for name := range types {
		f := types[name]
		b.Run(name, func(b *testing.B) {
			b.RunParallel(func(pb *testing.PB) {
				for pb.Next() {
					f(logger.Info()).Msg("")
				}
			})
		})
	}
}

func BenchmarkContextFieldType(b *testing.B) {
	oldFormat := TimeFieldFormat
	TimeFieldFormat = TimeFormatUnix
	defer func() { TimeFieldFormat = oldFormat }()
	bools := []bool{true, false, true, false, true, false, true, false, true, false}
	ints := []int{0, 1, 2, 3, 4, 5, 6, 7, 8, 9}
	floats := []float64{0, 1, 2, 3, 4, 5, 6, 7, 8, 9}
	strings := []string{"a", "b", "c", "d", "e", "f", "g", "h", "i", "j"}
	stringer := net.IP{127, 0, 0, 1}
	durations := []time.Duration{0, 1, 2, 3, 4, 5, 6, 7, 8, 9}
	times := []time.Time{
		time.Unix(0, 0),
		time.Unix(1, 0),
		time.Unix(2, 0),
		time.Unix(3, 0),
		time.Unix(4, 0),
		time.Unix(5, 0),
		time.Unix(6, 0),
		time.Unix(7, 0),
		time.Unix(8, 0),
		time.Unix(9, 0),
	}
	interfaces := []struct {
		Pub  string
		Tag  string `json:"tag"`
		priv int
	}{
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
	}
	objects := []obj{
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
		{"a", "a", 0},
	}
	errs := []error{errors.New("a"), errors.New("b"), errors.New("c"), errors.New("d"), errors.New("e")}
	ctx := context.Background()
	types := map[string]func(c Context) Context{
		"Bool": func(c Context) Context {
			return c.Bool("k", bools[0])
		},
		"Bools": func(c Context) Context {
			return c.Bools("k", bools)
		},
		"Int": func(c Context) Context {
			return c.Int("k", ints[0])
		},
		"Ints": func(c Context) Context {
			return c.Ints("k", ints)
		},
		"Float": func(c Context) Context {
			return c.Float64("k", floats[0])
		},
		"Floats": func(c Context) Context {
			return c.Floats64("k", floats)
		},
		"Str": func(c Context) Context {
			return c.Str("k", strings[0])
		},
		"Strs": func(c Context) Context {
			return c.Strs("k", strings)
		},
		"Stringer": func(c Context) Context {
			return c.Stringer("k", stringer)
		},
		"Err": func(c Context) Context {
			return c.Err(errs[0])
		},
		"Errs": func(c Context) Context {
			return c.Errs("k", errs)
		},
		"Ctx": func(c Context) Context {
			return c.Ctx(ctx)
		},
		"Time": func(c Context) Context {
			return c.Time("k", times[0])
		},
		"Times": func(c Context) Context {
			return c.Times("k", times)
		},
		"Dur": func(c Context) Context {
			return c.Dur("k", durations[0])
		},
		"Durs": func(c Context) Context {
			return c.Durs("k", durations)
		},
		"Interface": func(c Context) Context {
			return c.Interface("k", interfaces[0])
		},
		"Interfaces": func(c Context) Context {
			return c.Interface("k", interfaces)
		},
		"Interface(Object)": func(c Context) Context {
			return c.Interface("k", objects[0])
		},
		"Interface(Objects)": func(c Context) Context {
			return c.Interface("k", objects)
		},
		"Object": func(c Context) Context {
			return c.Object("k", objects[0])
		},
		"Timestamp": func(c Context) Context {
			return c.Timestamp()
		},
	}
	logger := New(io.Discard)
	b.ResetTimer()
	for name := range types {
		f := types[name]
		b.Run(name, func(b *testing.B) {
			b.RunParallel(func(pb *testing.PB) {
				for pb.Next() {
					l := f(logger.With()).Logger()
					l.Info().Msg("")
				}
			})
		})
	}
}
