//go:build !windows && !plan9 && !nacl
// +build !windows,!plan9,!nacl

// Deprecated: Use github.com/go-kit/log/syslog instead.
package syslog

import (
	"io"

	"github.com/go-kit/log"
	"github.com/go-kit/log/syslog"
)

// SyslogWriter is an interface wrapping stdlib syslog Writer.
type SyslogWriter = syslog.SyslogWriter

// NewSyslogLogger returns a new Logger which writes to syslog in syslog format.
// The body of the log message is the formatted output from the Logger returned
// by newLogger.
func NewSyslogLogger(w SyslogWriter, newLogger func(io.Writer) log.Logger, options ...Option) log.Logger {
	return syslog.NewSyslogLogger(w, newLogger, options...)
}

// Option sets a parameter for syslog loggers.
type Option = syslog.Option

// PrioritySelector inspects the list of keyvals and selects a syslog priority.
type PrioritySelector = syslog.PrioritySelector

// PrioritySelectorOption sets priority selector function to choose syslog
// priority.
func PrioritySelectorOption(selector PrioritySelector) Option {
	return syslog.PrioritySelectorOption(selector)
}
