package log_test

import (
	"bytes"
	"strings"
	"testing"

	"github.com/cometbft/cometbft/libs/log"
)

func TestVariousLevels(t *testing.T) {
	testCases := []struct {
		name    string
		allowed log.Option
		want    string
	}{
		{
			"AllowAll",
			log.AllowAll(),
			strings.Join([]string{
				`{"_msg":"here","level":"debug","this is":"debug log"}`,
				`{"_msg":"here","level":"info","this is":"info log"}`,
				`{"_msg":"here","level":"error","this is":"error log"}`,
			}, "\n"),
		},
		{
			"AllowDebug",
			log.AllowDebug(),
			strings.Join([]string{
				`{"_msg":"here","level":"debug","this is":"debug log"}`,
				`{"_msg":"here","level":"info","this is":"info log"}`,
				`{"_msg":"here","level":"error","this is":"error log"}`,
			}, "\n"),
		},
		{
			"AllowInfo",
			log.AllowInfo(),
			strings.Join([]string{
				`{"_msg":"here","level":"info","this is":"info log"}`,
				`{"_msg":"here","level":"error","this is":"error log"}`,
			}, "\n"),
		},
		{
			"AllowError",
			log.AllowError(),
			strings.Join([]string{
				`{"_msg":"here","level":"error","this is":"error log"}`,
			}, "\n"),
		},
		{
			"AllowNone",
			log.AllowNone(),
			``,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			var buf bytes.Buffer
			logger := log.NewFilter(log.NewTMJSONLoggerNoTS(&buf), tc.allowed)

			logger.Debug("here", "this is", "debug log")
			logger.Info("here", "this is", "info log")
			logger.Error("here", "this is", "error log")

			if want, have := tc.want, strings.TrimSpace(buf.String()); want != have {
				t.Errorf("\nwant:\n%s\nhave:\n%s", want, have)
			}
		})
	}
}

func TestLevelContext(t *testing.T) {
	var buf bytes.Buffer

	logger := log.NewTMJSONLoggerNoTS(&buf)
	logger = log.NewFilter(logger, log.AllowError())
	logger = logger.With("context", "value")

	logger.Error("foo", "bar", "baz")

	want := `{"_msg":"foo","bar":"baz","context":"value","level":"error"}`
	have := strings.TrimSpace(buf.String())
	if want != have {
		t.Errorf("\nwant '%s'\nhave '%s'", want, have)
	}

	buf.Reset()
	logger.Info("foo", "bar", "baz")
	if want, have := ``, strings.TrimSpace(buf.String()); want != have {
		t.Errorf("\nwant '%s'\nhave '%s'", want, have)
	}
}

func TestVariousAllowWith(t *testing.T) {
	var buf bytes.Buffer

	logger := log.NewTMJSONLoggerNoTS(&buf)

	logger1 := log.NewFilter(logger, log.AllowError(), log.AllowInfoWith("context", "value"))
	logger1.With("context", "value").Info("foo", "bar", "baz")

	want := `{"_msg":"foo","bar":"baz","context":"value","level":"info"}`
	have := strings.TrimSpace(buf.String())
	if want != have {
		t.Errorf("\nwant '%s'\nhave '%s'", want, have)
	}

	buf.Reset()

	logger2 := log.NewFilter(
		logger,
		log.AllowError(),
		log.AllowInfoWith("context", "value"),
		log.AllowNoneWith("user", "Sam"),
	)

	logger2.With("context", "value", "user", "Sam").Info("foo", "bar", "baz")
	if want, have := ``, strings.TrimSpace(buf.String()); want != have {
		t.Errorf("\nwant '%s'\nhave '%s'", want, have)
	}

	buf.Reset()

	logger3 := log.NewFilter(
		logger,
		log.AllowError(),
		log.AllowInfoWith("context", "value"),
		log.AllowNoneWith("user", "Sam"),
	)

	logger3.With("user", "Sam").With("context", "value").Info("foo", "bar", "baz")

	want = `{"_msg":"foo","bar":"baz","context":"value","level":"info","user":"Sam"}`
	have = strings.TrimSpace(buf.String())
	if want != have {
		t.Errorf("\nwant '%s'\nhave '%s'", want, have)
	}
}
