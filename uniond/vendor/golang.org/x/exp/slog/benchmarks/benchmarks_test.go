// Copyright 2022 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package benchmarks

import (
	"context"
	"flag"
	"io"
	"testing"

	"golang.org/x/exp/slog"
	"golang.org/x/exp/slog/internal"
)

func init() {
	flag.BoolVar(&internal.IgnorePC, "nopc", false, "do not invoke runtime.Callers")
}

// We pass Attrs (or zap.Fields) inline because it affects allocations: building
// up a list outside of the benchmarked code and passing it in with "..."
// reduces measured allocations.

func BenchmarkAttrs(b *testing.B) {
	ctx := context.Background()
	for _, handler := range []struct {
		name string
		h    slog.Handler
	}{
		{"disabled", disabledHandler{}},
		{"async discard", newAsyncHandler()},
		{"fastText discard", newFastTextHandler(io.Discard)},
		{"Text discard", slog.NewTextHandler(io.Discard, nil)},
		{"JSON discard", slog.NewJSONHandler(io.Discard, nil)},
	} {
		logger := slog.New(handler.h)
		b.Run(handler.name, func(b *testing.B) {
			for _, call := range []struct {
				name string
				f    func()
			}{
				{
					// The number should match nAttrsInline in slog/record.go.
					// This should exercise the code path where no allocations
					// happen in Record or Attr. If there are allocations, they
					// should only be from Duration.String and Time.String.
					"5 args",
					func() {
						logger.LogAttrs(nil, slog.LevelInfo, TestMessage,
							slog.String("string", TestString),
							slog.Int("status", TestInt),
							slog.Duration("duration", TestDuration),
							slog.Time("time", TestTime),
							slog.Any("error", TestError),
						)
					},
				},
				{
					"5 args ctx",
					func() {
						logger.LogAttrs(ctx, slog.LevelInfo, TestMessage,
							slog.String("string", TestString),
							slog.Int("status", TestInt),
							slog.Duration("duration", TestDuration),
							slog.Time("time", TestTime),
							slog.Any("error", TestError),
						)
					},
				},
				{
					"10 args",
					func() {
						logger.LogAttrs(nil, slog.LevelInfo, TestMessage,
							slog.String("string", TestString),
							slog.Int("status", TestInt),
							slog.Duration("duration", TestDuration),
							slog.Time("time", TestTime),
							slog.Any("error", TestError),
							slog.String("string", TestString),
							slog.Int("status", TestInt),
							slog.Duration("duration", TestDuration),
							slog.Time("time", TestTime),
							slog.Any("error", TestError),
						)
					},
				},
				{
					"40 args",
					func() {
						logger.LogAttrs(nil, slog.LevelInfo, TestMessage,
							slog.String("string", TestString),
							slog.Int("status", TestInt),
							slog.Duration("duration", TestDuration),
							slog.Time("time", TestTime),
							slog.Any("error", TestError),
							slog.String("string", TestString),
							slog.Int("status", TestInt),
							slog.Duration("duration", TestDuration),
							slog.Time("time", TestTime),
							slog.Any("error", TestError),
							slog.String("string", TestString),
							slog.Int("status", TestInt),
							slog.Duration("duration", TestDuration),
							slog.Time("time", TestTime),
							slog.Any("error", TestError),
							slog.String("string", TestString),
							slog.Int("status", TestInt),
							slog.Duration("duration", TestDuration),
							slog.Time("time", TestTime),
							slog.Any("error", TestError),
							slog.String("string", TestString),
							slog.Int("status", TestInt),
							slog.Duration("duration", TestDuration),
							slog.Time("time", TestTime),
							slog.Any("error", TestError),
							slog.String("string", TestString),
							slog.Int("status", TestInt),
							slog.Duration("duration", TestDuration),
							slog.Time("time", TestTime),
							slog.Any("error", TestError),
							slog.String("string", TestString),
							slog.Int("status", TestInt),
							slog.Duration("duration", TestDuration),
							slog.Time("time", TestTime),
							slog.Any("error", TestError),
							slog.String("string", TestString),
							slog.Int("status", TestInt),
							slog.Duration("duration", TestDuration),
							slog.Time("time", TestTime),
							slog.Any("error", TestError),
						)
					},
				},
			} {
				b.Run(call.name, func(b *testing.B) {
					b.ReportAllocs()
					b.RunParallel(func(pb *testing.PB) {
						for pb.Next() {
							call.f()
						}
					})
				})
			}
		})
	}
}
