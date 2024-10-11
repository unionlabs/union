// +build !binary_log

package log_test

import (
	"errors"
	"flag"
	"os"
	"time"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

// setup would normally be an init() function, however, there seems
// to be something awry with the testing framework when we set the
// global Logger from an init()
func setup() {
	// UNIX Time is faster and smaller than most timestamps
	// If you set zerolog.TimeFieldFormat to an empty string,
	// logs will write with UNIX time
	zerolog.TimeFieldFormat = ""
	// In order to always output a static time to stdout for these
	// examples to pass, we need to override zerolog.TimestampFunc
	// and log.Logger globals -- you would not normally need to do this
	zerolog.TimestampFunc = func() time.Time {
		return time.Date(2008, 1, 8, 17, 5, 05, 0, time.UTC)
	}
	log.Logger = zerolog.New(os.Stdout).With().Timestamp().Logger()
}

// Simple logging example using the Print function in the log package
// Note that both Print and Printf are at the debug log level by default
func ExamplePrint() {
	setup()

	log.Print("hello world")
	// Output: {"level":"debug","time":1199811905,"message":"hello world"}
}

// Simple logging example using the Printf function in the log package
func ExamplePrintf() {
	setup()

	log.Printf("hello %s", "world")
	// Output: {"level":"debug","time":1199811905,"message":"hello world"}
}

// Example of a log with no particular "level"
func ExampleLog() {
	setup()
	log.Log().Msg("hello world")

	// Output: {"time":1199811905,"message":"hello world"}
}

// Example of a conditional level based on the presence of an error.
func ExampleErr() {
	setup()
	err := errors.New("some error")
	log.Err(err).Msg("hello world")
	log.Err(nil).Msg("hello world")

	// Output: {"level":"error","error":"some error","time":1199811905,"message":"hello world"}
	// {"level":"info","time":1199811905,"message":"hello world"}
}

// Example of a log at a particular "level" (in this case, "trace")
func ExampleTrace() {
	setup()
	log.Trace().Msg("hello world")

	// Output: {"level":"trace","time":1199811905,"message":"hello world"}
}

// Example of a log at a particular "level" (in this case, "debug")
func ExampleDebug() {
	setup()
	log.Debug().Msg("hello world")

	// Output: {"level":"debug","time":1199811905,"message":"hello world"}
}

// Example of a log at a particular "level" (in this case, "info")
func ExampleInfo() {
	setup()
	log.Info().Msg("hello world")

	// Output: {"level":"info","time":1199811905,"message":"hello world"}
}

// Example of a log at a particular "level" (in this case, "warn")
func ExampleWarn() {
	setup()
	log.Warn().Msg("hello world")

	// Output: {"level":"warn","time":1199811905,"message":"hello world"}
}

// Example of a log at a particular "level" (in this case, "error")
func ExampleError() {
	setup()
	log.Error().Msg("hello world")

	// Output: {"level":"error","time":1199811905,"message":"hello world"}
}

// Example of a log at a particular "level" (in this case, "fatal")
func ExampleFatal() {
	setup()
	err := errors.New("A repo man spends his life getting into tense situations")
	service := "myservice"

	log.Fatal().
		Err(err).
		Str("service", service).
		Msgf("Cannot start %s", service)

	// Outputs: {"level":"fatal","time":1199811905,"error":"A repo man spends his life getting into tense situations","service":"myservice","message":"Cannot start myservice"}
}

// TODO: Panic

// This example uses command-line flags to demonstrate various outputs
// depending on the chosen log level.
func Example() {
	setup()
	debug := flag.Bool("debug", false, "sets log level to debug")

	flag.Parse()

	// Default level for this example is info, unless debug flag is present
	zerolog.SetGlobalLevel(zerolog.InfoLevel)
	if *debug {
		zerolog.SetGlobalLevel(zerolog.DebugLevel)
	}

	log.Debug().Msg("This message appears only when log level set to Debug")
	log.Info().Msg("This message appears when log level set to Debug or Info")

	if e := log.Debug(); e.Enabled() {
		// Compute log output only if enabled.
		value := "bar"
		e.Str("foo", value).Msg("some debug message")
	}

	// Output: {"level":"info","time":1199811905,"message":"This message appears when log level set to Debug or Info"}
}

// TODO: Output

// TODO: With

// TODO: Level

// TODO: Sample

// TODO: Hook

// TODO: WithLevel

// TODO: Ctx
