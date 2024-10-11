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

package logr

// testLogSink is a trivial LogSink implementation, just for testing, which
// calls (optional) hooks on each method.
type testLogSink struct {
	fnInit       func(ri RuntimeInfo)
	fnEnabled    func(lvl int) bool
	fnInfo       func(lvl int, msg string, kv ...any)
	fnError      func(err error, msg string, kv ...any)
	fnWithValues func(kv ...any)
	fnWithName   func(name string)

	withValues []any
}

var _ LogSink = &testLogSink{}

func (ls *testLogSink) Init(ri RuntimeInfo) {
	if ls.fnInit != nil {
		ls.fnInit(ri)
	}
}

func (ls *testLogSink) Enabled(lvl int) bool {
	if ls.fnEnabled != nil {
		return ls.fnEnabled(lvl)
	}
	return false
}

func (ls *testLogSink) Info(lvl int, msg string, kv ...any) {
	if ls.fnInfo != nil {
		ls.fnInfo(lvl, msg, kv...)
	}
}

func (ls *testLogSink) Error(err error, msg string, kv ...any) {
	if ls.fnError != nil {
		ls.fnError(err, msg, kv...)
	}
}

func (ls *testLogSink) WithValues(kv ...any) LogSink {
	if ls.fnWithValues != nil {
		ls.fnWithValues(kv...)
	}
	out := *ls
	n := len(out.withValues)
	out.withValues = append(out.withValues[:n:n], kv...)
	return &out
}

func (ls *testLogSink) WithName(name string) LogSink {
	if ls.fnWithName != nil {
		ls.fnWithName(name)
	}
	out := *ls
	return &out
}

type testCallDepthLogSink struct {
	testLogSink
	callDepth       int
	fnWithCallDepth func(depth int)
}

var _ CallDepthLogSink = &testCallDepthLogSink{}

func (ls *testCallDepthLogSink) WithCallDepth(depth int) LogSink {
	if ls.fnWithCallDepth != nil {
		ls.fnWithCallDepth(depth)
	}
	out := *ls
	out.callDepth += depth
	return &out
}
