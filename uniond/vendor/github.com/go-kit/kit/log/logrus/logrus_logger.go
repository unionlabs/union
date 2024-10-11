// Package logrus provides an adapter to the
// go-kit log.Logger interface.
package logrus

import (
	"errors"
	"fmt"

	"github.com/go-kit/log"
	"github.com/sirupsen/logrus"
)

type Logger struct {
	field logrus.FieldLogger
	level logrus.Level
}

type Option func(*Logger)

var errMissingValue = errors.New("(MISSING)")

// NewLogger returns a Go kit log.Logger that sends log events to a logrus.Logger.
func NewLogger(logger logrus.FieldLogger, options ...Option) log.Logger {
	l := &Logger{
		field: logger,
		level: logrus.InfoLevel,
	}

	for _, optFunc := range options {
		optFunc(l)
	}

	return l
}

// WithLevel configures a logrus logger to log at level for all events.
func WithLevel(level logrus.Level) Option {
	return func(c *Logger) {
		c.level = level
	}
}

func (l Logger) Log(keyvals ...interface{}) error {
	fields := logrus.Fields{}
	for i := 0; i < len(keyvals); i += 2 {
		if i+1 < len(keyvals) {
			fields[fmt.Sprint(keyvals[i])] = keyvals[i+1]
		} else {
			fields[fmt.Sprint(keyvals[i])] = errMissingValue
		}
	}

	switch l.level {
	case logrus.InfoLevel:
		l.field.WithFields(fields).Info()
	case logrus.ErrorLevel:
		l.field.WithFields(fields).Error()
	case logrus.DebugLevel:
		l.field.WithFields(fields).Debug()
	case logrus.WarnLevel:
		l.field.WithFields(fields).Warn()
	case logrus.TraceLevel:
		l.field.WithFields(fields).Trace()
	default:
		l.field.WithFields(fields).Print()
	}

	return nil
}
