package log

import (
	"io"

	"github.com/go-kit/log"
)

// StdlibWriter implements io.Writer by invoking the stdlib log.Print. It's
// designed to be passed to a Go kit logger as the writer, for cases where
// it's necessary to redirect all Go kit log output to the stdlib logger.
//
// If you have any choice in the matter, you shouldn't use this. Prefer to
// redirect the stdlib log to the Go kit logger via NewStdlibAdapter.
type StdlibWriter = log.StdlibWriter

// StdlibAdapter wraps a Logger and allows it to be passed to the stdlib
// logger's SetOutput. It will extract date/timestamps, filenames, and
// messages, and place them under relevant keys.
type StdlibAdapter = log.StdlibAdapter

// StdlibAdapterOption sets a parameter for the StdlibAdapter.
type StdlibAdapterOption = log.StdlibAdapterOption

// TimestampKey sets the key for the timestamp field. By default, it's "ts".
func TimestampKey(key string) StdlibAdapterOption {
	return log.TimestampKey(key)
}

// FileKey sets the key for the file and line field. By default, it's "caller".
func FileKey(key string) StdlibAdapterOption {
	return log.FileKey(key)
}

// MessageKey sets the key for the actual log message. By default, it's "msg".
func MessageKey(key string) StdlibAdapterOption {
	return log.MessageKey(key)
}

// Prefix configures the adapter to parse a prefix from stdlib log events. If
// you provide a non-empty prefix to the stdlib logger, then your should provide
// that same prefix to the adapter via this option.
//
// By default, the prefix isn't included in the msg key. Set joinPrefixToMsg to
// true if you want to include the parsed prefix in the msg.
func Prefix(prefix string, joinPrefixToMsg bool) StdlibAdapterOption {
	return log.Prefix(prefix, joinPrefixToMsg)
}

// NewStdlibAdapter returns a new StdlibAdapter wrapper around the passed
// logger. It's designed to be passed to log.SetOutput.
func NewStdlibAdapter(logger Logger, options ...StdlibAdapterOption) io.Writer {
	return log.NewStdlibAdapter(logger, options...)
}
