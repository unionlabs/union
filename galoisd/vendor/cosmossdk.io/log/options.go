package log

import (
	"time"

	"github.com/rs/zerolog"
)

// defaultConfig has all the options disabled, except Color and TimeFormat
var defaultConfig = Config{
	Level:      zerolog.NoLevel,
	Filter:     nil,
	OutputJSON: false,
	Color:      true,
	StackTrace: false,
	TimeFormat: time.Kitchen,
}

// Config defines configuration for the logger.
type Config struct {
	Level      zerolog.Level
	Filter     FilterFunc
	OutputJSON bool
	Color      bool
	StackTrace bool
	TimeFormat string
}

type Option func(*Config)

// FilterOption sets the filter for the Logger.
func FilterOption(filter FilterFunc) Option {
	return func(cfg *Config) {
		cfg.Filter = filter
	}
}

// LevelOption sets the level for the Logger.
// Messages with a lower level will be discarded.
func LevelOption(level zerolog.Level) Option {
	return func(cfg *Config) {
		cfg.Level = level
	}
}

// OutputJSONOption sets the output of the logger to JSON.
// By default, the logger outputs to a human-readable format.
func OutputJSONOption() Option {
	return func(cfg *Config) {
		cfg.OutputJSON = true
	}
}

// ColorOption add option to enable/disable coloring
// of the logs when console writer is in use
func ColorOption(val bool) Option {
	return func(cfg *Config) {
		cfg.Color = val
	}
}

// TimeFormatOption configures timestamp format of the logger
// timestamps disabled if empty.
// it is responsibility of the caller to provider correct values
// Supported formats:
//   - time.Layout
//   - time.ANSIC
//   - time.UnixDate
//   - time.RubyDate
//   - time.RFC822
//   - time.RFC822Z
//   - time.RFC850
//   - time.RFC1123
//   - time.RFC1123Z
//   - time.RFC3339
//   - time.RFC3339Nano
//   - time.Kitchen
func TimeFormatOption(format string) Option {
	return func(cfg *Config) {
		cfg.TimeFormat = format
	}
}

// TraceOption add option to enable/disable print of stacktrace on error log
func TraceOption(val bool) Option {
	return func(cfg *Config) {
		cfg.StackTrace = val
	}
}
