/*
 * Copyright 2018 Dgraph Labs, Inc. and Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package badger

import (
	"log"
	"os"
)

// Logger is implemented by any logging system that is used for standard logs.
type Logger interface {
	Errorf(string, ...interface{})
	Warningf(string, ...interface{})
	Infof(string, ...interface{})
	Debugf(string, ...interface{})
}

// Errorf logs an ERROR log message to the logger specified in opts or to the
// global logger if no logger is specified in opts.
func (opt *Options) Errorf(format string, v ...interface{}) {
	if opt.Logger == nil {
		return
	}
	opt.Logger.Errorf(format, v...)
}

// Infof logs an INFO message to the logger specified in opts.
func (opt *Options) Infof(format string, v ...interface{}) {
	if opt.Logger == nil {
		return
	}
	opt.Logger.Infof(format, v...)
}

// Warningf logs a WARNING message to the logger specified in opts.
func (opt *Options) Warningf(format string, v ...interface{}) {
	if opt.Logger == nil {
		return
	}
	opt.Logger.Warningf(format, v...)
}

// Debugf logs a DEBUG message to the logger specified in opts.
func (opt *Options) Debugf(format string, v ...interface{}) {
	if opt.Logger == nil {
		return
	}
	opt.Logger.Debugf(format, v...)
}

type loggingLevel int

const (
	DEBUG loggingLevel = iota
	INFO
	WARNING
	ERROR
)

type defaultLog struct {
	*log.Logger
	level loggingLevel
}

func defaultLogger(level loggingLevel) *defaultLog {
	return &defaultLog{Logger: log.New(os.Stderr, "badger ", log.LstdFlags), level: level}
}

func (l *defaultLog) Errorf(f string, v ...interface{}) {
	if l.level <= ERROR {
		l.Printf("ERROR: "+f, v...)
	}
}

func (l *defaultLog) Warningf(f string, v ...interface{}) {
	if l.level <= WARNING {
		l.Printf("WARNING: "+f, v...)
	}
}

func (l *defaultLog) Infof(f string, v ...interface{}) {
	if l.level <= INFO {
		l.Printf("INFO: "+f, v...)
	}
}

func (l *defaultLog) Debugf(f string, v ...interface{}) {
	if l.level <= DEBUG {
		l.Printf("DEBUG: "+f, v...)
	}
}
