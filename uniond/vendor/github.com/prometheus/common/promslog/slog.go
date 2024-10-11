// Copyright 2024 The Prometheus Authors
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

// Package promslog defines standardised ways to initialize the Go standard
// library's log/slog logger.
// It should typically only ever be imported by main packages.

//go:build go1.21

package promslog

import (
	"fmt"
	"io"
	"log/slog"
	"os"
	"path/filepath"
	"strconv"
	"strings"
)

type LogStyle string

const (
	SlogStyle  LogStyle = "slog"
	GoKitStyle LogStyle = "go-kit"
)

var (
	LevelFlagOptions  = []string{"debug", "info", "warn", "error"}
	FormatFlagOptions = []string{"logfmt", "json"}

	callerAddFunc             = false
	defaultWriter             = os.Stderr
	goKitStyleReplaceAttrFunc = func(groups []string, a slog.Attr) slog.Attr {
		key := a.Key
		switch key {
		case slog.TimeKey:
			a.Key = "ts"

			// This timestamp format differs from RFC3339Nano by using .000 instead
			// of .999999999 which changes the timestamp from 9 variable to 3 fixed
			// decimals (.130 instead of .130987456).
			t := a.Value.Time()
			a.Value = slog.StringValue(t.Format("2006-01-02T15:04:05.000Z07:00"))
		case slog.SourceKey:
			a.Key = "caller"
			src, _ := a.Value.Any().(*slog.Source)

			switch callerAddFunc {
			case true:
				a.Value = slog.StringValue(filepath.Base(src.File) + "(" + filepath.Base(src.Function) + "):" + strconv.Itoa(src.Line))
			default:
				a.Value = slog.StringValue(filepath.Base(src.File) + ":" + strconv.Itoa(src.Line))
			}
		case slog.LevelKey:
			a.Value = slog.StringValue(strings.ToLower(a.Value.String()))
		default:
		}

		return a
	}
)

// AllowedLevel is a settable identifier for the minimum level a log entry
// must be have.
type AllowedLevel struct {
	s   string
	lvl *slog.LevelVar
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
	if l.lvl == nil {
		l.lvl = &slog.LevelVar{}
	}

	switch s {
	case "debug":
		l.lvl.Set(slog.LevelDebug)
		callerAddFunc = true
	case "info":
		l.lvl.Set(slog.LevelInfo)
		callerAddFunc = false
	case "warn":
		l.lvl.Set(slog.LevelWarn)
		callerAddFunc = false
	case "error":
		l.lvl.Set(slog.LevelError)
		callerAddFunc = false
	default:
		return fmt.Errorf("unrecognized log level %s", s)
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
		return fmt.Errorf("unrecognized log format %s", s)
	}
	return nil
}

// Config is a struct containing configurable settings for the logger
type Config struct {
	Level    *AllowedLevel
	Format   *AllowedFormat
	Style    LogStyle
	ioWriter io.Writer
}

// New returns a new slog.Logger. Each logged line will be annotated
// with a timestamp. The output always goes to stderr.
func New(config *Config) *slog.Logger {
	if config.Level == nil {
		config.Level = &AllowedLevel{}
		_ = config.Level.Set("info")
	}

	if config.ioWriter == nil {
		config.ioWriter = defaultWriter
	}

	logHandlerOpts := &slog.HandlerOptions{
		Level:     config.Level.lvl,
		AddSource: true,
	}

	if config.Style == GoKitStyle {
		logHandlerOpts.ReplaceAttr = goKitStyleReplaceAttrFunc
	}

	if config.Format != nil && config.Format.s == "json" {
		return slog.New(slog.NewJSONHandler(config.ioWriter, logHandlerOpts))
	}
	return slog.New(slog.NewTextHandler(config.ioWriter, logHandlerOpts))
}
