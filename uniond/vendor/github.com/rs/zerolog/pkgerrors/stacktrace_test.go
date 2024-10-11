// +build !binary_log

package pkgerrors

import (
	"bytes"
	"fmt"
	"regexp"
	"testing"

	"github.com/pkg/errors"
	"github.com/rs/zerolog"
)

func TestLogStack(t *testing.T) {
	zerolog.ErrorStackMarshaler = MarshalStack

	out := &bytes.Buffer{}
	log := zerolog.New(out)

	err := fmt.Errorf("from error: %w", errors.New("error message"))
	log.Log().Stack().Err(err).Msg("")

	got := out.String()
	want := `\{"stack":\[\{"func":"TestLogStack","line":"21","source":"stacktrace_test.go"\},.*\],"error":"from error: error message"\}\n`
	if ok, _ := regexp.MatchString(want, got); !ok {
		t.Errorf("invalid log output:\ngot:  %v\nwant: %v", got, want)
	}
}

func TestLogStackFields(t *testing.T) {
	zerolog.ErrorStackMarshaler = MarshalStack

	out := &bytes.Buffer{}
	log := zerolog.New(out)

	err := fmt.Errorf("from error: %w", errors.New("error message"))
	log.Log().Stack().Fields([]interface{}{"error", err}).Msg("")

	got := out.String()
	want := `\{"error":"from error: error message","stack":\[\{"func":"TestLogStackFields","line":"37","source":"stacktrace_test.go"\},.*\]\}\n`
	if ok, _ := regexp.MatchString(want, got); !ok {
		t.Errorf("invalid log output:\ngot:  %v\nwant: %v", got, want)
	}
}

func TestLogStackFromContext(t *testing.T) {
	zerolog.ErrorStackMarshaler = MarshalStack

	out := &bytes.Buffer{}
	log := zerolog.New(out).With().Stack().Logger() // calling Stack() on log context instead of event

	err := fmt.Errorf("from error: %w", errors.New("error message"))
	log.Log().Err(err).Msg("") // not explicitly calling Stack()

	got := out.String()
	want := `\{"stack":\[\{"func":"TestLogStackFromContext","line":"53","source":"stacktrace_test.go"\},.*\],"error":"from error: error message"\}\n`
	if ok, _ := regexp.MatchString(want, got); !ok {
		t.Errorf("invalid log output:\ngot:  %v\nwant: %v", got, want)
	}
}

func TestLogStackFromContextWith(t *testing.T) {
	zerolog.ErrorStackMarshaler = MarshalStack

	err := fmt.Errorf("from error: %w", errors.New("error message"))
	out := &bytes.Buffer{}
	log := zerolog.New(out).With().Stack().Err(err).Logger() // calling Stack() on log context instead of event

	log.Error().Msg("")

	got := out.String()
	want := `\{"level":"error","stack":\[\{"func":"TestLogStackFromContextWith","line":"66","source":"stacktrace_test.go"\},.*\],"error":"from error: error message"\}\n`
	if ok, _ := regexp.MatchString(want, got); !ok {
		t.Errorf("invalid log output:\ngot:  %v\nwant: %v", got, want)
	}
}

func BenchmarkLogStack(b *testing.B) {
	zerolog.ErrorStackMarshaler = MarshalStack
	out := &bytes.Buffer{}
	log := zerolog.New(out)
	err := errors.Wrap(errors.New("error message"), "from error")
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		log.Log().Stack().Err(err).Msg("")
		out.Reset()
	}
}
