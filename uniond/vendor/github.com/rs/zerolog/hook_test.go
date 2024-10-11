package zerolog

import (
	"bytes"
	"context"
	"io"
	"testing"
)

type contextKeyType int

var contextKey contextKeyType

var (
	levelNameHook = HookFunc(func(e *Event, level Level, msg string) {
		levelName := level.String()
		if level == NoLevel {
			levelName = "nolevel"
		}
		e.Str("level_name", levelName)
	})
	simpleHook = HookFunc(func(e *Event, level Level, msg string) {
		e.Bool("has_level", level != NoLevel)
		e.Str("test", "logged")
	})
	copyHook = HookFunc(func(e *Event, level Level, msg string) {
		hasLevel := level != NoLevel
		e.Bool("copy_has_level", hasLevel)
		if hasLevel {
			e.Str("copy_level", level.String())
		}
		e.Str("copy_msg", msg)
	})
	nopHook = HookFunc(func(e *Event, level Level, message string) {
	})
	discardHook = HookFunc(func(e *Event, level Level, message string) {
		e.Discard()
	})
	contextHook = HookFunc(func(e *Event, level Level, message string) {
		contextData, ok := e.GetCtx().Value(contextKey).(string)
		if ok {
			e.Str("context-data", contextData)
		}
	})
)

func TestHook(t *testing.T) {
	tests := []struct {
		name string
		want string
		test func(log Logger)
	}{
		{"Message", `{"level_name":"nolevel","message":"test message"}` + "\n", func(log Logger) {
			log = log.Hook(levelNameHook)
			log.Log().Msg("test message")
		}},
		{"NoLevel", `{"level_name":"nolevel"}` + "\n", func(log Logger) {
			log = log.Hook(levelNameHook)
			log.Log().Msg("")
		}},
		{"Print", `{"level":"debug","level_name":"debug"}` + "\n", func(log Logger) {
			log = log.Hook(levelNameHook)
			log.Print("")
		}},
		{"Error", `{"level":"error","level_name":"error"}` + "\n", func(log Logger) {
			log = log.Hook(levelNameHook)
			log.Error().Msg("")
		}},
		{"Copy/1", `{"copy_has_level":false,"copy_msg":""}` + "\n", func(log Logger) {
			log = log.Hook(copyHook)
			log.Log().Msg("")
		}},
		{"Copy/2", `{"level":"info","copy_has_level":true,"copy_level":"info","copy_msg":"a message","message":"a message"}` + "\n", func(log Logger) {
			log = log.Hook(copyHook)
			log.Info().Msg("a message")
		}},
		{"Multi", `{"level":"error","level_name":"error","has_level":true,"test":"logged"}` + "\n", func(log Logger) {
			log = log.Hook(levelNameHook).Hook(simpleHook)
			log.Error().Msg("")
		}},
		{"Multi/Message", `{"level":"error","level_name":"error","has_level":true,"test":"logged","message":"a message"}` + "\n", func(log Logger) {
			log = log.Hook(levelNameHook).Hook(simpleHook)
			log.Error().Msg("a message")
		}},
		{"Output/single/pre", `{"level":"error","level_name":"error"}` + "\n", func(log Logger) {
			ignored := &bytes.Buffer{}
			log = New(ignored).Hook(levelNameHook).Output(log.w)
			log.Error().Msg("")
		}},
		{"Output/single/post", `{"level":"error","level_name":"error"}` + "\n", func(log Logger) {
			ignored := &bytes.Buffer{}
			log = New(ignored).Output(log.w).Hook(levelNameHook)
			log.Error().Msg("")
		}},
		{"Output/multi/pre", `{"level":"error","level_name":"error","has_level":true,"test":"logged"}` + "\n", func(log Logger) {
			ignored := &bytes.Buffer{}
			log = New(ignored).Hook(levelNameHook).Hook(simpleHook).Output(log.w)
			log.Error().Msg("")
		}},
		{"Output/multi/post", `{"level":"error","level_name":"error","has_level":true,"test":"logged"}` + "\n", func(log Logger) {
			ignored := &bytes.Buffer{}
			log = New(ignored).Output(log.w).Hook(levelNameHook).Hook(simpleHook)
			log.Error().Msg("")
		}},
		{"Output/mixed", `{"level":"error","level_name":"error","has_level":true,"test":"logged"}` + "\n", func(log Logger) {
			ignored := &bytes.Buffer{}
			log = New(ignored).Hook(levelNameHook).Output(log.w).Hook(simpleHook)
			log.Error().Msg("")
		}},
		{"With/single/pre", `{"level":"error","with":"pre","level_name":"error"}` + "\n", func(log Logger) {
			log = log.Hook(levelNameHook).With().Str("with", "pre").Logger()
			log.Error().Msg("")
		}},
		{"With/single/post", `{"level":"error","with":"post","level_name":"error"}` + "\n", func(log Logger) {
			log = log.With().Str("with", "post").Logger().Hook(levelNameHook)
			log.Error().Msg("")
		}},
		{"With/multi/pre", `{"level":"error","with":"pre","level_name":"error","has_level":true,"test":"logged"}` + "\n", func(log Logger) {
			log = log.Hook(levelNameHook).Hook(simpleHook).With().Str("with", "pre").Logger()
			log.Error().Msg("")
		}},
		{"With/multi/post", `{"level":"error","with":"post","level_name":"error","has_level":true,"test":"logged"}` + "\n", func(log Logger) {
			log = log.With().Str("with", "post").Logger().Hook(levelNameHook).Hook(simpleHook)
			log.Error().Msg("")
		}},
		{"With/mixed", `{"level":"error","with":"mixed","level_name":"error","has_level":true,"test":"logged"}` + "\n", func(log Logger) {
			log = log.Hook(levelNameHook).With().Str("with", "mixed").Logger().Hook(simpleHook)
			log.Error().Msg("")
		}},
		{"Discard", "", func(log Logger) {
			log = log.Hook(discardHook)
			log.Log().Msg("test message")
		}},
		{"Context/Background", `{"level":"info","message":"test message"}` + "\n", func(log Logger) {
			log = log.Hook(contextHook)
			log.Info().Ctx(context.Background()).Msg("test message")
		}},
		{"Context/nil", `{"level":"info","message":"test message"}` + "\n", func(log Logger) {
			// passing `nil` where a context is wanted is against
			// the rules, but people still do it.
			log = log.Hook(contextHook)
			log.Info().Ctx(nil).Msg("test message") // nolint
		}},
		{"Context/valid", `{"level":"info","context-data":"12345abcdef","message":"test message"}` + "\n", func(log Logger) {
			ctx := context.Background()
			ctx = context.WithValue(ctx, contextKey, "12345abcdef")
			log = log.Hook(contextHook)
			log.Info().Ctx(ctx).Msg("test message")
		}},
		{"Context/With/valid", `{"level":"info","context-data":"12345abcdef","message":"test message"}` + "\n", func(log Logger) {
			ctx := context.Background()
			ctx = context.WithValue(ctx, contextKey, "12345abcdef")
			log = log.Hook(contextHook)
			log = log.With().Ctx(ctx).Logger()
			log.Info().Msg("test message")
		}},
		{"None", `{"level":"error"}` + "\n", func(log Logger) {
			log.Error().Msg("")
		}},
	}
	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			out := &bytes.Buffer{}
			log := New(out)
			tt.test(log)
			if got, want := decodeIfBinaryToString(out.Bytes()), tt.want; got != want {
				t.Errorf("invalid log output:\ngot:  %v\nwant: %v", got, want)
			}
		})
	}
}

func BenchmarkHooks(b *testing.B) {
	logger := New(io.Discard)
	b.ResetTimer()
	b.Run("Nop/Single", func(b *testing.B) {
		log := logger.Hook(nopHook)
		b.RunParallel(func(pb *testing.PB) {
			for pb.Next() {
				log.Log().Msg("")
			}
		})
	})
	b.Run("Nop/Multi", func(b *testing.B) {
		log := logger.Hook(nopHook).Hook(nopHook)
		b.RunParallel(func(pb *testing.PB) {
			for pb.Next() {
				log.Log().Msg("")
			}
		})
	})
	b.Run("Simple", func(b *testing.B) {
		log := logger.Hook(simpleHook)
		b.RunParallel(func(pb *testing.PB) {
			for pb.Next() {
				log.Log().Msg("")
			}
		})
	})
}
