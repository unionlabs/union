// Copyright 2017 The Prometheus Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Package promlog defines standardised ways to initialize Go kit loggers
// across Prometheus components.
// It should typically only ever be imported by main packages.
package promlog

import (
	"fmt"
	"os"
	"sync"
	"time"

	"github.com/go-kit/log"
	"github.com/go-kit/log/level"
)

var (
	// This timestamp format differs from RFC3339Nano by using .000 instead
	// of .999999999 which changes the timestamp from 9 variable to 3 fixed
	// decimals (.130 instead of .130987456).
	timestampFormat = log.TimestampFormat(
		func() time.Time { return time.Now().UTC() },
		"2006-01-02T15:04:05.000Z07:00",
	)

	LevelFlagOptions  = []string{"debug", "info", "warn", "error"}
	FormatFlagOptions = []string{"logfmt", "json"}
)

// AllowedLevel is a settable identifier for the minimum level a log entry
// must be have.
type AllowedLevel struct {
	s string
	o level.Option
}

func (l *AllowedLevel) UnmarshalYAML(unmarshal func(interface{}) error) error {
	var s string
	type plain string
	if err := unmarshal((*plain)(&s)); err != nil {
		return err
	}
	if s == "" {
		return nil
	}
	lo := &AllowedLevel{}
	if err := lo.Set(s); err != nil {
		return err
	}
	*l = *lo
	return nil
}

func (l *AllowedLevel) String() string {
	return l.s
}

// Set updates the value of the allowed level.
func (l *AllowedLevel) Set(s string) error {
	switch s {
	case "debug":
		l.o = level.AllowDebug()
	case "info":
		l.o = level.AllowInfo()
	case "warn":
		l.o = level.AllowWarn()
	case "error":
		l.o = level.AllowError()
	default:
		return fmt.Errorf("unrecognized log level %q", s)
	}
	l.s = s
	return nil
}

// AllowedFormat is a settable identifier for the output format that the logger can have.
type AllowedFormat struct {
	s string
}

func (f *AllowedFormat) String() string {
	return f.s
}

// Set updates the value of the allowed format.
func (f *AllowedFormat) Set(s string) error {
	switch s {
	case "logfmt", "json":
		f.s = s
	default:
		return fmt.Errorf("unrecognized log format %q", s)
	}
	return nil
}

// Config is a struct containing configurable settings for the logger
type Config struct {
	Level  *AllowedLevel
	Format *AllowedFormat
}

// New returns a new leveled oklog logger. Each logged line will be annotated
// with a timestamp. The output always goes to stderr.
func New(config *Config) log.Logger {
	if config.Format != nil && config.Format.s == "json" {
		return NewWithLogger(log.NewJSONLogger(log.NewSyncWriter(os.Stderr)), config)
	}

	return NewWithLogger(log.NewLogfmtLogger(log.NewSyncWriter(os.Stderr)), config)
}

// NewWithLogger returns a new leveled oklog logger with a custom log.Logger.
// Each logged line will be annotated with a timestamp.
func NewWithLogger(l log.Logger, config *Config) log.Logger {
	if config.Level != nil {
		l = log.With(l, "ts", timestampFormat, "caller", log.Caller(5))
		l = level.NewFilter(l, config.Level.o)
	} else {
		l = log.With(l, "ts", timestampFormat, "caller", log.DefaultCaller)
	}
	return l
}

// NewDynamic returns a new leveled logger. Each logged line will be annotated
// with a timestamp. The output always goes to stderr. Some properties can be
// changed, like the level.
func NewDynamic(config *Config) *logger {
	if config.Format != nil && config.Format.s == "json" {
		return NewDynamicWithLogger(log.NewJSONLogger(log.NewSyncWriter(os.Stderr)), config)
	}

	return NewDynamicWithLogger(log.NewLogfmtLogger(log.NewSyncWriter(os.Stderr)), config)
}

// NewDynamicWithLogger returns a new leveled logger with a custom io.Writer.
// Each logged line will be annotated with a timestamp.
// Some properties can be changed, like the level.
func NewDynamicWithLogger(l log.Logger, config *Config) *logger {
	lo := &logger{
		base:    l,
		leveled: l,
	}

	if config.Level != nil {
		lo.SetLevel(config.Level)
	}

	return lo
}

type logger struct {
	base         log.Logger
	leveled      log.Logger
	currentLevel *AllowedLevel
	mtx          sync.Mutex
}

// Log implements logger.Log.
func (l *logger) Log(keyvals ...interface{}) error {
	l.mtx.Lock()
	defer l.mtx.Unlock()
	return l.leveled.Log(keyvals...)
}

// SetLevel changes the log level.
func (l *logger) SetLevel(lvl *AllowedLevel) {
	l.mtx.Lock()
	defer l.mtx.Unlock()
	if lvl == nil {
		l.leveled = log.With(l.base, "ts", timestampFormat, "caller", log.DefaultCaller)
		l.currentLevel = nil
		return
	}

	if l.currentLevel != nil && l.currentLevel.s != lvl.s {
		_ = l.base.Log("msg", "Log level changed", "prev", l.currentLevel, "current", lvl)
	}
	l.currentLevel = lvl
	l.leveled = level.NewFilter(log.With(l.base, "ts", timestampFormat, "caller", log.Caller(5)), lvl.o)
}
