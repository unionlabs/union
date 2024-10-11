/*
Copyright 2019 The logr Authors.

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

package main

import (
	"fmt"
	"os"
	"text/tabwriter"

	"github.com/go-logr/logr"
)

// tabLogSink is a sample logr.LogSink that logs to stderr.
// It's terribly inefficient, and is only a basic example.
type tabLogSink struct {
	name      string
	keyValues map[string]any
	writer    *tabwriter.Writer
}

var _ logr.LogSink = &tabLogSink{}

// Note that Init usually takes a pointer so it can modify the receiver to save
// runtime info.
func (*tabLogSink) Init(info logr.RuntimeInfo) {
}

func (tabLogSink) Enabled(level int) bool {
	return true
}

func (l tabLogSink) Info(level int, msg string, kvs ...any) {
	fmt.Fprintf(l.writer, "%s\t%s\t", l.name, msg)
	for k, v := range l.keyValues {
		fmt.Fprintf(l.writer, "%s: %+v  ", k, v)
	}
	for i := 0; i < len(kvs); i += 2 {
		fmt.Fprintf(l.writer, "%s: %+v  ", kvs[i], kvs[i+1])
	}
	fmt.Fprintf(l.writer, "\n")
	l.writer.Flush()
}

func (l tabLogSink) Error(err error, msg string, kvs ...any) {
	kvs = append(kvs, "error", err)
	l.Info(0, msg, kvs...)
}

func (l tabLogSink) WithName(name string) logr.LogSink {
	return &tabLogSink{
		name:      l.name + "." + name,
		keyValues: l.keyValues,
		writer:    l.writer,
	}
}

func (l tabLogSink) WithValues(kvs ...any) logr.LogSink {
	newMap := make(map[string]any, len(l.keyValues)+len(kvs)/2)
	for k, v := range l.keyValues {
		newMap[k] = v
	}
	for i := 0; i < len(kvs); i += 2 {
		newMap[kvs[i].(string)] = kvs[i+1]
	}
	return &tabLogSink{
		name:      l.name,
		keyValues: newMap,
		writer:    l.writer,
	}
}

// NewTabLogger is the main entry-point to this implementation.  App developers
// call this somewhere near main() and thenceforth only deal with logr.Logger.
func NewTabLogger() logr.Logger {
	sink := &tabLogSink{
		writer: tabwriter.NewWriter(os.Stderr, 40, 8, 2, '\t', 0),
	}
	return logr.New(sink)
}
