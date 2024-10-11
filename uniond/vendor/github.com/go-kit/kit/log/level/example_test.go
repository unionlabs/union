package level_test

import (
	"errors"
	"os"

	"github.com/go-kit/kit/log"
	"github.com/go-kit/kit/log/level"
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
	// level=error caller=example_test.go:32 err="bad data"
	// level=info caller=example_test.go:33 event="data saved"
}
