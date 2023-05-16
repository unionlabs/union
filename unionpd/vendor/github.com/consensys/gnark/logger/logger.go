// Package logger provides a configurable logger accross gnark components
//
// The root logger defined by default uses github.com/rs/zerolog with a console writer
package logger

import (
	"io"
	"os"
	"strings"

	"github.com/consensys/gnark/debug"
	"github.com/rs/zerolog"
)

var logger zerolog.Logger

func init() {
	output := zerolog.ConsoleWriter{Out: os.Stdout, TimeFormat: "15:04:05"}
	logger = zerolog.New(output).With().Timestamp().Logger()

	if !debug.Debug && strings.HasSuffix(os.Args[0], ".test") {
		logger = zerolog.Nop()
	}

}

// SetOutput changes the output of the global logger
func SetOutput(w io.Writer) {
	logger = logger.Output(w)
}

// Set allow a gnark user to overhide the global logger
func Set(l zerolog.Logger) {
	logger = l
}

// Disable disables logging
func Disable() {
	logger = zerolog.Nop()
}

// Logger returns a sublogger for a component
func Logger() zerolog.Logger {
	return logger
}
