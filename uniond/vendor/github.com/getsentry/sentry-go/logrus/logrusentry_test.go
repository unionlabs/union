package sentrylogrus

import (
	"errors"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	pkgerr "github.com/pkg/errors"
	"github.com/sirupsen/logrus"

	"github.com/getsentry/sentry-go"
	"github.com/getsentry/sentry-go/internal/testutils"
)

func TestNew(t *testing.T) {
	t.Parallel()
	t.Run("invalid DSN", func(t *testing.T) {
		t.Parallel()
		_, err := New(nil, sentry.ClientOptions{Dsn: "%xxx"})
		if err == nil || !strings.Contains(err.Error(), "invalid URL escape") {
			t.Errorf("Unexpected error: %s", err)
		}
	})

	t.Run("success", func(t *testing.T) {
		t.Parallel()
		h, err := New(nil, sentry.ClientOptions{})
		if err != nil {
			t.Fatal(err)
		}
		if id := h.hub.CaptureEvent(&sentry.Event{}); id == nil {
			t.Error("CaptureEvent failed")
		}
		if !h.Flush(testutils.FlushTimeout()) {
			t.Error("flush failed")
		}
	})
}

func TestFire(t *testing.T) {
	t.Parallel()

	entry := &logrus.Entry{
		Level: logrus.ErrorLevel,
	}

	opts := sentry.ClientOptions{}
	hook, err := New([]logrus.Level{logrus.ErrorLevel}, opts)
	if err != nil {
		t.Fatal(err)
	}
	err = hook.Fire(entry)
	if err != nil {
		t.Fatal(err)
	}

	if !hook.Flush(testutils.FlushTimeout()) {
		t.Error("flush failed")
	}
}

func Test_entryToEvent(t *testing.T) {
	t.Parallel()
	tests := map[string]struct {
		entry *logrus.Entry
		want  *sentry.Event
	}{
		"empty entry": {
			entry: &logrus.Entry{},
			want: &sentry.Event{
				Level:  "fatal",
				Extra:  map[string]any{},
				Logger: "logrus",
			},
		},
		"data fields": {
			entry: &logrus.Entry{
				Data: map[string]any{
					"foo": 123.4,
					"bar": "oink",
				},
			},
			want: &sentry.Event{
				Level:  "fatal",
				Extra:  map[string]any{"bar": "oink", "foo": 123.4},
				Logger: "logrus",
			},
		},
		"info level": {
			entry: &logrus.Entry{
				Level: logrus.InfoLevel,
			},
			want: &sentry.Event{
				Level:  "info",
				Extra:  map[string]any{},
				Logger: "logrus",
			},
		},
		"message": {
			entry: &logrus.Entry{
				Message: "the only thing we have to fear is fear itself",
			},
			want: &sentry.Event{
				Level:   "fatal",
				Extra:   map[string]any{},
				Message: "the only thing we have to fear is fear itself",
				Logger:  "logrus",
			},
		},
		"timestamp": {
			entry: &logrus.Entry{
				Time: time.Unix(1, 2).UTC(),
			},
			want: &sentry.Event{
				Level:     "fatal",
				Extra:     map[string]any{},
				Timestamp: time.Unix(1, 2).UTC(),
				Logger:    "logrus",
			},
		},
		"http request": {
			entry: &logrus.Entry{
				Data: map[string]any{
					FieldRequest: httptest.NewRequest("GET", "/", nil),
				},
			},
			want: &sentry.Event{
				Level: "fatal",
				Extra: map[string]any{},
				Request: &sentry.Request{
					URL:     "http://example.com/",
					Method:  http.MethodGet,
					Headers: map[string]string{"Host": "example.com"},
				},
				Logger: "logrus",
			},
		},
		"error": {
			entry: &logrus.Entry{
				Data: map[string]any{
					logrus.ErrorKey: errors.New("things failed"),
				},
			},
			want: &sentry.Event{
				Level: "fatal",
				Extra: map[string]any{},
				Exception: []sentry.Exception{
					{Type: "*errors.errorString", Value: "things failed", Stacktrace: &sentry.Stacktrace{Frames: []sentry.Frame{}}},
				},
				Logger: "logrus",
			},
		},
		"non-error": {
			entry: &logrus.Entry{
				Data: map[string]any{
					logrus.ErrorKey: "this isn't really an error",
				},
			},
			want: &sentry.Event{
				Level: "fatal",
				Extra: map[string]any{
					"error": "this isn't really an error",
				},
				Logger: "logrus",
			},
		},
		"error with stack trace": {
			entry: &logrus.Entry{
				Data: map[string]any{
					logrus.ErrorKey: pkgerr.WithStack(errors.New("failure")),
				},
			},
			want: &sentry.Event{
				Level: "fatal",
				Extra: map[string]any{},
				Exception: []sentry.Exception{
					{
						Type:  "*errors.errorString",
						Value: "failure",
						Mechanism: &sentry.Mechanism{
							ExceptionID:      0,
							IsExceptionGroup: true,
						},
					},
					{
						Type:  "*errors.withStack",
						Value: "failure",
						Stacktrace: &sentry.Stacktrace{
							Frames: []sentry.Frame{},
						},
						Mechanism: &sentry.Mechanism{
							ExceptionID:      1,
							IsExceptionGroup: true,
							ParentID:         sentry.Pointer(0),
						},
					},
				},
				Logger: "logrus",
			},
		},
		"user": {
			entry: &logrus.Entry{
				Data: map[string]any{
					FieldUser: sentry.User{
						ID: "bob",
					},
				},
			},
			want: &sentry.Event{
				Level: "fatal",
				Extra: map[string]any{},
				User: sentry.User{
					ID: "bob",
				},
				Logger: "logrus",
			},
		},
		"user pointer": {
			entry: &logrus.Entry{
				Data: map[string]any{
					FieldUser: &sentry.User{
						ID: "alice",
					},
				},
			},
			want: &sentry.Event{
				Level: "fatal",
				Extra: map[string]any{},
				User: sentry.User{
					ID: "alice",
				},
				Logger: "logrus",
			},
		},
		"non-user": {
			entry: &logrus.Entry{
				Data: map[string]any{
					FieldUser: "just say no to drugs",
				},
			},
			want: &sentry.Event{
				Level: "fatal",
				Extra: map[string]any{
					"user": "just say no to drugs",
				},
				Logger: "logrus",
			},
		},
	}

	h, err := New(nil, sentry.ClientOptions{
		AttachStacktrace: true,
	})
	if err != nil {
		t.Fatal(err)
	}

	for name, tt := range tests {
		tt := tt
		t.Run(name, func(t *testing.T) {
			t.Parallel()
			got := h.entryToEvent(tt.entry)
			opts := cmp.Options{
				cmpopts.IgnoreFields(sentry.Event{}, "sdkMetaData"),
			}
			if d := cmp.Diff(tt.want, got, opts); d != "" {
				t.Error(d)
			}
		})
	}
}
