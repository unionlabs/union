package logrus_test

import (
	"bytes"
	"encoding/json"
	"errors"
	"strings"
	"testing"

	log "github.com/go-kit/kit/log/logrus"
	"github.com/sirupsen/logrus"
)

func TestLogrusLogger(t *testing.T) {
	t.Parallel()
	buf := &bytes.Buffer{}
	logrusLogger := logrus.New()
	logrusLogger.Out = buf
	logrusLogger.Formatter = &logrus.TextFormatter{TimestampFormat: "02-01-2006 15:04:05", FullTimestamp: true}
	logger := log.NewLogger(logrusLogger)

	if err := logger.Log("hello", "world"); err != nil {
		t.Fatal(err)
	}
	if want, have := "hello=world\n", strings.Split(buf.String(), " ")[3]; want != have {
		t.Errorf("want %#v, have %#v", want, have)
	}

	buf.Reset()
	if err := logger.Log("a", 1, "err", errors.New("error")); err != nil {
		t.Fatal(err)
	}
	if want, have := "a=1 err=error", strings.TrimSpace(strings.SplitAfterN(buf.String(), " ", 4)[3]); want != have {
		t.Errorf("want %#v, have %#v", want, have)
	}

	buf.Reset()
	if err := logger.Log("a", 1, "b"); err != nil {
		t.Fatal(err)
	}
	if want, have := "a=1 b=\"(MISSING)\"", strings.TrimSpace(strings.SplitAfterN(buf.String(), " ", 4)[3]); want != have {
		t.Errorf("want %#v, have %#v", want, have)
	}

	buf.Reset()
	if err := logger.Log("my_map", mymap{0: 0}); err != nil {
		t.Fatal(err)
	}
	if want, have := "my_map=special_behavior", strings.TrimSpace(strings.Split(buf.String(), " ")[3]); want != have {
		t.Errorf("want %#v, have %#v", want, have)
	}
}

type mymap map[int]int

func (m mymap) String() string { return "special_behavior" }

func TestWithLevel(t *testing.T) {
	tests := []struct {
		name          string
		level         logrus.Level
		expectedLevel logrus.Level
	}{
		{
			name:          "Test Debug level",
			level:         logrus.DebugLevel,
			expectedLevel: logrus.DebugLevel,
		},
		{
			name:          "Test Error level",
			level:         logrus.ErrorLevel,
			expectedLevel: logrus.ErrorLevel,
		},
		{
			name:          "Test Warn level",
			level:         logrus.WarnLevel,
			expectedLevel: logrus.WarnLevel,
		},
		{
			name:          "Test Info level",
			level:         logrus.InfoLevel,
			expectedLevel: logrus.InfoLevel,
		},
		{
			name:          "Test Trace level",
			level:         logrus.TraceLevel,
			expectedLevel: logrus.TraceLevel,
		},
		{
			name:          "Test not existing level",
			level:         999,
			expectedLevel: logrus.InfoLevel,
		},
	}
	for _, tt := range tests {
		buf := &bytes.Buffer{}
		logrusLogger := logrus.New()
		logrusLogger.Out = buf
		logrusLogger.Level = tt.level
		logrusLogger.Formatter = &logrus.JSONFormatter{}
		logger := log.NewLogger(logrusLogger, log.WithLevel(tt.level))

		t.Run(tt.name, func(t *testing.T) {
			if err := logger.Log(); err != nil {
				t.Fatal(err)
			}

			l := map[string]interface{}{}
			if err := json.Unmarshal(buf.Bytes(), &l); err != nil {
				t.Fatal(err)
			}

			if v, ok := l["level"].(string); !ok || v != tt.expectedLevel.String() {
				t.Fatalf("Logging levels doesn't match. Expected: %s, got: %s", tt.level, v)
			}

		})
	}
}
