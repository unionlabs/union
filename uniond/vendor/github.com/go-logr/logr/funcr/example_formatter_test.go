/*
Copyright 2021 The logr Authors.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

package funcr_test

import (
	"fmt"
	"strings"

	"github.com/go-logr/logr"
	"github.com/go-logr/logr/funcr"
)

// NewStdoutLogger returns a logr.Logger that prints to stdout.
// It demonstrates how to implement a custom With* function which
// controls whether INFO or ERROR are printed in front of the log
// message.
func NewStdoutLogger() logr.Logger {
	l := &stdoutlogger{
		Formatter: funcr.NewFormatter(funcr.Options{}),
	}
	return logr.New(l)
}

type stdoutlogger struct {
	funcr.Formatter
	logMsgType bool
}

func (l stdoutlogger) WithName(name string) logr.LogSink {
	l.Formatter.AddName(name)
	return &l
}

func (l stdoutlogger) WithValues(kvList ...any) logr.LogSink {
	l.Formatter.AddValues(kvList)
	return &l
}

func (l stdoutlogger) WithCallDepth(depth int) logr.LogSink {
	l.Formatter.AddCallDepth(depth)
	return &l
}

func (l stdoutlogger) Info(level int, msg string, kvList ...any) {
	prefix, args := l.FormatInfo(level, msg, kvList)
	l.write("INFO", prefix, args)
}

func (l stdoutlogger) Error(err error, msg string, kvList ...any) {
	prefix, args := l.FormatError(err, msg, kvList)
	l.write("ERROR", prefix, args)
}

func (l stdoutlogger) write(msgType, prefix, args string) {
	var parts []string
	if l.logMsgType {
		parts = append(parts, msgType)
	}
	if prefix != "" {
		parts = append(parts, prefix)
	}
	parts = append(parts, args)
	fmt.Println(strings.Join(parts, ": "))
}

// WithLogMsgType returns a copy of the logger with new settings for
// logging the message type. It returns the original logger if the
// underlying LogSink is not a stdoutlogger.
func WithLogMsgType(log logr.Logger, logMsgType bool) logr.Logger {
	if l, ok := log.GetSink().(*stdoutlogger); ok {
		clone := *l
		clone.logMsgType = logMsgType
		log = log.WithSink(&clone)
	}
	return log
}

// Assert conformance to the interfaces.
var _ logr.LogSink = &stdoutlogger{}
var _ logr.CallDepthLogSink = &stdoutlogger{}

func ExampleFormatter() {
	l := NewStdoutLogger()
	l.Info("no message type")
	WithLogMsgType(l, true).Info("with message type")
	// Output:
	// "level"=0 "msg"="no message type"
	// INFO: "level"=0 "msg"="with message type"
}
