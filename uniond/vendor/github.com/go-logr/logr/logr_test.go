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

import (
	"errors"
	"fmt"
	"reflect"
	"runtime"
	"testing"
)

func TestNew(t *testing.T) {
	calledInit := 0

	sink := &testLogSink{}
	sink.fnInit = func(ri RuntimeInfo) {
		if ri.CallDepth != 1 {
			t.Errorf("expected runtimeInfo.CallDepth = 1, got %d", ri.CallDepth)
		}
		calledInit++
	}
	logger := New(sink)

	if logger.sink == nil {
		t.Errorf("expected sink to be set, got %v", logger.sink)
	}
	if calledInit != 1 {
		t.Errorf("expected sink.Init() to be called once, got %d", calledInit)
	}
	if _, ok := logger.sink.(CallDepthLogSink); ok {
		t.Errorf("expected conversion to CallDepthLogSink to fail")
	}
}

func TestNewCachesCallDepthInterface(t *testing.T) {
	sink := &testCallDepthLogSink{}
	logger := New(sink)

	if _, ok := logger.sink.(CallDepthLogSink); !ok {
		t.Errorf("expected conversion to CallDepthLogSink to succeed")
	}
}

func TestEnabled(t *testing.T) {
	calledEnabled := 0

	sink := &testLogSink{}
	sink.fnEnabled = func(_ int) bool {
		calledEnabled++
		return true
	}
	logger := New(sink)

	if en := logger.Enabled(); en != true {
		t.Errorf("expected true")
	}
	if calledEnabled != 1 {
		t.Errorf("expected sink.Enabled() to be called once, got %d", calledEnabled)
	}
}

func TestError(t *testing.T) {
	calledError := 0
	errInput := fmt.Errorf("error")
	msgInput := "msg"
	kvInput := []any{0, 1, 2}

	sink := &testLogSink{}
	sink.fnError = func(err error, msg string, kv ...any) {
		calledError++
		if err != errInput {
			t.Errorf("unexpected err input, got %v", err)
		}
		if msg != msgInput {
			t.Errorf("unexpected msg input, got %q", msg)
		}
		if !reflect.DeepEqual(kv, kvInput) {
			t.Errorf("unexpected kv input, got %v", kv)
		}
	}
	logger := New(sink)

	logger.Error(errInput, msgInput, kvInput...)
	if calledError != 1 {
		t.Errorf("expected sink.Error() to be called once, got %d", calledError)
	}
}

func TestV(t *testing.T) {
	for name, logger := range map[string]Logger{
		"testLogSink": New(&testLogSink{}),
		"Discard":     Discard(),
		"Zero":        {},
	} {
		t.Run(name, func(t *testing.T) {
			adjust := func(level int) int {
				if logger.GetSink() == nil {
					// The Discard and the zero Logger short-cut the V call and don't
					// change the verbosity level.
					return 0
				}
				return level
			}
			inputs := []struct {
				name string
				fn   func() Logger
				exp  int
			}{{
				name: "V(0)",
				fn:   func() Logger { return logger.V(0) },
				exp:  0,
			}, {
				name: "V(93)",
				fn:   func() Logger { return logger.V(93) },
				exp:  adjust(93),
			}, {
				name: "V(70).V(6)",
				fn:   func() Logger { return logger.V(70).V(6) },
				exp:  adjust(76),
			}, {
				name: "V(-1)",
				fn:   func() Logger { return logger.V(-1) },
				exp:  0,
			}, {
				name: "V(1).V(-1)",
				fn:   func() Logger { return logger.V(1).V(-1) },
				exp:  adjust(1),
			}}
			for _, in := range inputs {
				t.Run(in.name, func(t *testing.T) {
					if want, got := in.exp, in.fn().GetV(); got != want {
						t.Errorf("expected %d, got %d", want, got)
					}
				})
			}
		})
	}
}

func TestInfo(t *testing.T) {
	calledEnabled := 0
	calledInfo := 0
	lvlInput := 0
	msgInput := "msg"
	kvInput := []any{0, 1, 2}

	sink := &testLogSink{}
	sink.fnEnabled = func(lvl int) bool {
		calledEnabled++
		return lvl < 100
	}
	sink.fnInfo = func(lvl int, msg string, kv ...any) {
		calledInfo++
		if lvl != lvlInput {
			t.Errorf("unexpected lvl input, got %v", lvl)
		}
		if msg != msgInput {
			t.Errorf("unexpected msg input, got %q", msg)
		}
		if !reflect.DeepEqual(kv, kvInput) {
			t.Errorf("unexpected kv input, got %v", kv)
		}
	}
	logger := New(sink)

	calledEnabled = 0
	calledInfo = 0
	lvlInput = 0
	logger.Info(msgInput, kvInput...)
	if calledEnabled != 1 {
		t.Errorf("expected sink.Enabled() to be called once, got %d", calledEnabled)
	}
	if calledInfo != 1 {
		t.Errorf("expected sink.Info() to be called once, got %d", calledInfo)
	}

	calledEnabled = 0
	calledInfo = 0
	lvlInput = 0
	logger.V(0).Info(msgInput, kvInput...)
	if calledEnabled != 1 {
		t.Errorf("expected sink.Enabled() to be called once, got %d", calledEnabled)
	}
	if calledInfo != 1 {
		t.Errorf("expected sink.Info() to be called once, got %d", calledInfo)
	}

	calledEnabled = 0
	calledInfo = 0
	lvlInput = 93
	logger.V(93).Info(msgInput, kvInput...)
	if calledEnabled != 1 {
		t.Errorf("expected sink.Enabled() to be called once, got %d", calledEnabled)
	}
	if calledInfo != 1 {
		t.Errorf("expected sink.Info() to be called once, got %d", calledInfo)
	}

	calledEnabled = 0
	calledInfo = 0
	lvlInput = 100
	logger.V(100).Info(msgInput, kvInput...)
	if calledEnabled != 1 {
		t.Errorf("expected sink.Enabled() to be called once, got %d", calledEnabled)
	}
	if calledInfo != 0 {
		t.Errorf("expected sink.Info() to not be called, got %d", calledInfo)
	}
}

func TestWithValues(t *testing.T) {
	calledWithValues := 0
	kvInput := []any{"zero", 0, "one", 1, "two", 2}

	sink := &testLogSink{}
	sink.fnWithValues = func(kv ...any) {
		calledWithValues++
		if !reflect.DeepEqual(kv, kvInput) {
			t.Errorf("unexpected kv input, got %v", kv)
		}
	}
	logger := New(sink)

	out := logger.WithValues(kvInput...)
	if calledWithValues != 1 {
		t.Errorf("expected sink.WithValues() to be called once, got %d", calledWithValues)
	}
	if p, _ := out.sink.(*testLogSink); p == sink {
		t.Errorf("expected output to be different from input, got in=%p, out=%p", sink, p)
	}
}

func TestWithName(t *testing.T) {
	calledWithName := 0
	nameInput := "name"

	sink := &testLogSink{}
	sink.fnWithName = func(name string) {
		calledWithName++
		if name != nameInput {
			t.Errorf("unexpected name input, got %q", name)
		}
	}
	logger := New(sink)

	out := logger.WithName(nameInput)
	if calledWithName != 1 {
		t.Errorf("expected sink.WithName() to be called once, got %d", calledWithName)
	}
	if p, _ := out.sink.(*testLogSink); p == sink {
		t.Errorf("expected output to be different from input, got in=%p, out=%p", sink, p)
	}
}

func TestWithCallDepthNotImplemented(t *testing.T) {
	depthInput := 7

	sink := &testLogSink{}
	logger := New(sink)

	out := logger.WithCallDepth(depthInput)
	if p, _ := out.sink.(*testLogSink); p != sink {
		t.Errorf("expected output to be the same as input, got in=%p, out=%p", sink, p)
	}
}

func TestWithCallDepthImplemented(t *testing.T) {
	calledWithCallDepth := 0
	depthInput := 7

	sink := &testCallDepthLogSink{}
	sink.fnWithCallDepth = func(depth int) {
		calledWithCallDepth++
		if depth != depthInput {
			t.Errorf("unexpected depth input, got %d", depth)
		}
	}
	logger := New(sink)

	out := logger.WithCallDepth(depthInput)
	if calledWithCallDepth != 1 {
		t.Errorf("expected sink.WithCallDepth() to be called once, got %d", calledWithCallDepth)
	}
	p, _ := out.sink.(*testCallDepthLogSink)
	if p == sink {
		t.Errorf("expected output to be different from input, got in=%p, out=%p", sink, p)
	}
	if p.callDepth != depthInput {
		t.Errorf("expected sink to have call depth %d, got %d", depthInput, p.callDepth)
	}
}

func TestWithCallDepthIncremental(t *testing.T) {
	calledWithCallDepth := 0
	depthInput := 7

	sink := &testCallDepthLogSink{}
	sink.fnWithCallDepth = func(depth int) {
		calledWithCallDepth++
		if depth != 1 {
			t.Errorf("unexpected depth input, got %d", depth)
		}
	}
	logger := New(sink)

	out := logger
	for i := 0; i < depthInput; i++ {
		out = out.WithCallDepth(1)
	}
	if calledWithCallDepth != depthInput {
		t.Errorf("expected sink.WithCallDepth() to be called %d times, got %d", depthInput, calledWithCallDepth)
	}
	p, _ := out.sink.(*testCallDepthLogSink)
	if p == sink {
		t.Errorf("expected output to be different from input, got in=%p, out=%p", sink, p)
	}
	if p.callDepth != depthInput {
		t.Errorf("expected sink to have call depth %d, got %d", depthInput, p.callDepth)
	}
}

func TestIsZero(t *testing.T) {
	var l Logger
	if !l.IsZero() {
		t.Errorf("expected IsZero")
	}
	sink := &testLogSink{}
	l = New(sink)
	if l.IsZero() {
		t.Errorf("expected not IsZero")
	}
	// Discard is the same as a nil sink.
	l = Discard()
	if !l.IsZero() {
		t.Errorf("expected IsZero")
	}
}

func TestZeroValue(t *testing.T) {
	// Make sure that the zero value is useful and equivalent to a Discard logger.
	var l Logger
	if l.Enabled() {
		t.Errorf("expected not Enabled")
	}
	if !l.IsZero() {
		t.Errorf("expected IsZero")
	}
	// Make sure that none of these methods cause a crash
	l.Info("foo")
	l.Error(errors.New("bar"), "some error")
	if l.GetSink() != nil {
		t.Errorf("expected nil from GetSink")
	}
	l2 := l.WithName("some-name").V(2).WithValues("foo", 1).WithCallDepth(1)
	l2.Info("foo")
	l2.Error(errors.New("bar"), "some error")
	_, _ = l.WithCallStackHelper()
}

func TestCallDepthConsistent(t *testing.T) {
	sink := &testLogSink{}

	depth := 0
	expect := "github.com/go-logr/logr.TestCallDepthConsistent"
	sink.fnInit = func(ri RuntimeInfo) {
		depth = ri.CallDepth + 1 // 1 for these function pointers
		if caller := getCaller(depth); caller != expect {
			t.Errorf("identified wrong caller %q", caller)
		}

	}
	sink.fnEnabled = func(_ int) bool {
		if caller := getCaller(depth); caller != expect {
			t.Errorf("identified wrong caller %q", caller)
		}
		return true
	}
	sink.fnError = func(_ error, _ string, _ ...any) {
		if caller := getCaller(depth); caller != expect {
			t.Errorf("identified wrong caller %q", caller)
		}
	}
	l := New(sink)

	l.Enabled()
	l.Info("msg")
	l.Error(nil, "msg")
}

func getCaller(depth int) string {
	// +1 for this frame, +1 for Info/Error/Enabled.
	pc, _, _, ok := runtime.Caller(depth + 2)
	if !ok {
		return "<runtime.Caller failed>"
	}
	fp := runtime.FuncForPC(pc)
	if fp == nil {
		return "<runtime.FuncForPC failed>"
	}
	return fp.Name()
}
