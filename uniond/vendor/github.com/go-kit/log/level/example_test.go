package level_test

import (
	"errors"
	"flag"
	"os"

	"github.com/go-kit/log"
	"github.com/go-kit/log/level"
)

func Example_basic() {
	logger := log.NewLogfmtLogger(os.Stdout)
	level.Debug(logger).Log("msg", "this message is at the debug level")
	level.Info(logger).Log("msg", "this message is at the info level")
	level.Warn(logger).Log("msg", "this message is at the warn level")
	level.Error(logger).Log("msg", "this message is at the error level")

	// Output:
	// level=debug msg="this message is at the debug level"
	// level=info msg="this message is at the info level"
	// level=warn msg="this message is at the warn level"
	// level=error msg="this message is at the error level"
}

func Example_filtered() {
	// Set up logger with level filter.
	logger := log.NewLogfmtLogger(os.Stdout)
	logger = level.NewFilter(logger, level.AllowInfo())
	logger = log.With(logger, "caller", log.DefaultCaller)

	// Use level helpers to log at different levels.
	level.Error(logger).Log("err", errors.New("bad data"))
	level.Info(logger).Log("event", "data saved")
	level.Debug(logger).Log("next item", 17) // filtered

	// Output:
	// level=error caller=example_test.go:33 err="bad data"
	// level=info caller=example_test.go:34 event="data saved"
}

func Example_parsed() {
	fs := flag.NewFlagSet("example", flag.ExitOnError)
	lvl := fs.String("log-level", "", `"debug", "info", "warn" or "error"`)
	fs.Parse([]string{"-log-level", "info"})

	// Set up logger with level filter.
	logger := log.NewLogfmtLogger(os.Stdout)
	logger = level.NewFilter(logger, level.Allow(level.ParseDefault(*lvl, level.DebugValue())))
	logger = log.With(logger, "caller", log.DefaultCaller)

	// Use level helpers to log at different levels.
	level.Error(logger).Log("err", errors.New("bad data"))
	level.Info(logger).Log("event", "data saved")
	level.Debug(logger).Log("next item", 17) // filtered

	// Output:
	// level=error caller=example_test.go:53 err="bad data"
	// level=info caller=example_test.go:54 event="data saved"
}
