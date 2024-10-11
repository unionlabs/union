//go:build go1.21
// +build go1.21

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

package funcr

import (
	"bytes"
	"fmt"
	"log/slog"
	"path/filepath"
	"runtime"
	"testing"

	"github.com/go-logr/logr"
	"github.com/go-logr/logr/internal/testhelp"
)

func TestSlogSink(t *testing.T) {
	testCases := []struct {
		name      string
		withAttrs []any
		withGroup string
		args      []any
		expect    string
	}{{
		name:   "just msg",
		args:   makeKV(),
		expect: `{"logger":"","level":0,"msg":"msg"}`,
	}, {
		name:   "primitives",
		args:   makeKV("int", 1, "str", "ABC", "bool", true),
		expect: `{"logger":"","level":0,"msg":"msg","int":1,"str":"ABC","bool":true}`,
	}, {
		name:      "with attrs",
		withAttrs: makeKV("attrInt", 1, "attrStr", "ABC", "attrBool", true),
		args:      makeKV("int", 2),
		expect:    `{"logger":"","level":0,"msg":"msg","attrInt":1,"attrStr":"ABC","attrBool":true,"int":2}`,
	}, {
		name:      "with group",
		withGroup: "groupname",
		args:      makeKV("int", 1, "str", "ABC", "bool", true),
		expect:    `{"logger":"","level":0,"msg":"msg","groupname":{"int":1,"str":"ABC","bool":true}}`,
	}, {
		name:      "with attrs and group",
		withAttrs: makeKV("attrInt", 1, "attrStr", "ABC"),
		withGroup: "groupname",
		args:      makeKV("int", 3, "bool", true),
		expect:    `{"logger":"","level":0,"msg":"msg","attrInt":1,"attrStr":"ABC","groupname":{"int":3,"bool":true}}`,
	}}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			capt := &capture{}
			logger := logr.New(newSink(capt.Func, NewFormatterJSON(Options{})))
			slogger := slog.New(logr.ToSlogHandler(logger))
			if len(tc.withAttrs) > 0 {
				slogger = slogger.With(tc.withAttrs...)
			}
			if tc.withGroup != "" {
				slogger = slogger.WithGroup(tc.withGroup)
			}
			slogger.Info("msg", tc.args...)
			if capt.log != tc.expect {
				t.Errorf("\nexpected %q\n     got %q", tc.expect, capt.log)
			}
		})
	}
}

func TestSlogSinkGroups(t *testing.T) {
	testCases := []struct {
		name   string
		fn     func(slogger *slog.Logger)
		expect string
	}{{
		name: "no group",
		fn: func(slogger *slog.Logger) {
			slogger.
				Info("msg", "k", "v")
		},
		expect: `{"logger":"","level":0,"msg":"msg","k":"v"}`,
	}, {
		name: "1 group with leaf args",
		fn: func(slogger *slog.Logger) {
			slogger.
				WithGroup("g1").
				Info("msg", "k", "v")
		},
		expect: `{"logger":"","level":0,"msg":"msg","g1":{"k":"v"}}`,
	}, {
		name: "1 group without leaf args",
		fn: func(slogger *slog.Logger) {
			slogger.
				WithGroup("g1").
				Info("msg")
		},
		expect: `{"logger":"","level":0,"msg":"msg"}`,
	}, {
		name: "1 group with value without leaf args",
		fn: func(slogger *slog.Logger) {
			slogger.
				WithGroup("g1").With("k1", 1).
				Info("msg")
		},
		expect: `{"logger":"","level":0,"msg":"msg","g1":{"k1":1}}`,
	}, {
		name: "2 groups with values no leaf args",
		fn: func(slogger *slog.Logger) {
			slogger.
				WithGroup("g1").With("k1", 1).
				WithGroup("g2").With("k2", 2).
				Info("msg")
		},
		expect: `{"logger":"","level":0,"msg":"msg","g1":{"k1":1,"g2":{"k2":2}}}`,
	}, {
		name: "3 empty groups with no values or leaf args",
		fn: func(slogger *slog.Logger) {
			slogger.
				WithGroup("g1").
				WithGroup("g2").
				WithGroup("g3").
				Info("msg")
		},
		expect: `{"logger":"","level":0,"msg":"msg"}`,
	}, {
		name: "3 empty groups with no values but with leaf args",
		fn: func(slogger *slog.Logger) {
			slogger.
				WithGroup("g1").
				WithGroup("g2").
				WithGroup("g3").
				Info("msg", "k", "v")
		},
		expect: `{"logger":"","level":0,"msg":"msg","g1":{"g2":{"g3":{"k":"v"}}}}`,
	}, {
		name: "multiple groups with and without values",
		fn: func(slogger *slog.Logger) {
			slogger.
				With("k0", 0).
				WithGroup("g1").
				WithGroup("g2").
				WithGroup("g3").With("k3", 3).
				WithGroup("g4").
				WithGroup("g5").
				WithGroup("g6").With("k6", 6).
				WithGroup("g7").
				WithGroup("g8").
				WithGroup("g9").
				Info("msg")
		},
		expect: `{"logger":"","level":0,"msg":"msg","k0":0,"g1":{"g2":{"g3":{"k3":3,"g4":{"g5":{"g6":{"k6":6}}}}}}}`,
	}}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			capt := &capture{}
			logger := logr.New(newSink(capt.Func, NewFormatterJSON(Options{})))
			slogger := slog.New(logr.ToSlogHandler(logger))
			tc.fn(slogger)
			if capt.log != tc.expect {
				t.Errorf("\nexpected: `%s`\n     got: `%s`", tc.expect, capt.log)
			}
		})
	}
}

func TestSlogSinkWithCaller(t *testing.T) {
	capt := &capture{}
	logger := logr.New(newSink(capt.Func, NewFormatterJSON(Options{LogCaller: All})))
	slogger := slog.New(logr.ToSlogHandler(logger))
	slogger.Error("msg", "int", 1)
	_, file, line, _ := runtime.Caller(0)
	expect := fmt.Sprintf(`{"logger":"","caller":{"file":%q,"line":%d},"msg":"msg","error":null,"int":1}`, filepath.Base(file), line-1)
	if capt.log != expect {
		t.Errorf("\nexpected %q\n     got %q", expect, capt.log)
	}
}

func TestRunSlogTests(t *testing.T) {
	fn := func(buffer *bytes.Buffer) slog.Handler {
		printfn := func(obj string) {
			fmt.Fprintln(buffer, obj)
		}
		opts := Options{
			LogTimestamp: true,
			Verbosity:    10,
			RenderBuiltinsHook: func(kvList []any) []any {
				mappedKVList := make([]any, len(kvList))
				for i := 0; i < len(kvList); i += 2 {
					key := kvList[i]
					switch key {
					case "ts":
						mappedKVList[i] = "time"
					default:
						mappedKVList[i] = key
					}
					mappedKVList[i+1] = kvList[i+1]
				}
				return mappedKVList
			},
		}
		logger := NewJSON(printfn, opts)
		return logr.ToSlogHandler(logger)
	}
	exceptions := []string{
		"a Handler should ignore a zero Record.Time", // Time is generated by sink.
	}
	testhelp.RunSlogTests(t, fn, exceptions...)
}

func TestLogrSlogConversion(t *testing.T) {
	f := New(func(_, _ string) {}, Options{})
	f2 := logr.FromSlogHandler(logr.ToSlogHandler(f))
	if want, got := f, f2; got != want {
		t.Helper()
		t.Errorf("Expected %T %+v, got instead: %T %+v", want, want, got, got)
	}
}
