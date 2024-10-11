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

package logr

import (
	"log/slog"
	"os"
	"testing"

	"github.com/go-logr/logr"
)

//
// slogSink wrapper of discard
//

func BenchmarkSlogSinkLogInfoOneArg(b *testing.B) {
	var log logr.Logger = logr.FromSlogHandler(logr.ToSlogHandler(logr.Discard()))
	doInfoOneArg(b, log)
}

func BenchmarkSlogSinkLogInfoSeveralArgs(b *testing.B) {
	var log logr.Logger = logr.FromSlogHandler(logr.ToSlogHandler(logr.Discard()))
	doInfoSeveralArgs(b, log)
}

func BenchmarkSlogSinkLogInfoWithValues(b *testing.B) {
	var log logr.Logger = logr.FromSlogHandler(logr.ToSlogHandler(logr.Discard()))
	doInfoWithValues(b, log)
}

func BenchmarkSlogSinkLogV0Info(b *testing.B) {
	var log logr.Logger = logr.FromSlogHandler(logr.ToSlogHandler(logr.Discard()))
	doV0Info(b, log)
}

func BenchmarkSlogSinkLogV9Info(b *testing.B) {
	var log logr.Logger = logr.FromSlogHandler(logr.ToSlogHandler(logr.Discard()))
	doV9Info(b, log)
}

func BenchmarkSlogSinkLogError(b *testing.B) {
	var log logr.Logger = logr.FromSlogHandler(logr.ToSlogHandler(logr.Discard()))
	doError(b, log)
}

func BenchmarkSlogSinkWithValues(b *testing.B) {
	var log logr.Logger = logr.FromSlogHandler(logr.ToSlogHandler(logr.Discard()))
	doWithValues(b, log)
}

func BenchmarkSlogSinkWithName(b *testing.B) {
	var log logr.Logger = logr.FromSlogHandler(logr.ToSlogHandler(logr.Discard()))
	doWithName(b, log)
}

//
// slogSink wrapper of slog's JSONHandler, for comparison
//

func makeSlogJSONLogger() logr.Logger {
	devnull, _ := os.Open("/dev/null")
	handler := slog.NewJSONHandler(devnull, nil)
	return logr.FromSlogHandler(handler)
}

func BenchmarkSlogJSONLogInfoOneArg(b *testing.B) {
	var log logr.Logger = makeSlogJSONLogger()
	doInfoOneArg(b, log)
}

func BenchmarkSlogJSONLogInfoSeveralArgs(b *testing.B) {
	var log logr.Logger = makeSlogJSONLogger()
	doInfoSeveralArgs(b, log)
}

func BenchmarkSlogJSONLogInfoWithValues(b *testing.B) {
	var log logr.Logger = makeSlogJSONLogger()
	doInfoWithValues(b, log)
}

func BenchmarkSlogJSONLogV0Info(b *testing.B) {
	var log logr.Logger = makeSlogJSONLogger()
	doV0Info(b, log)
}

func BenchmarkSlogJSONLogV9Info(b *testing.B) {
	var log logr.Logger = makeSlogJSONLogger()
	doV9Info(b, log)
}

func BenchmarkSlogJSONLogError(b *testing.B) {
	var log logr.Logger = makeSlogJSONLogger()
	doError(b, log)
}

func BenchmarkSlogJSONLogWithValues(b *testing.B) {
	var log logr.Logger = makeSlogJSONLogger()
	doWithValues(b, log)
}

func BenchmarkSlogJSONWithName(b *testing.B) {
	var log logr.Logger = makeSlogJSONLogger()
	doWithName(b, log)
}

func BenchmarkSlogJSONWithCallDepth(b *testing.B) {
	var log logr.Logger = makeSlogJSONLogger()
	doWithCallDepth(b, log)
}

func BenchmarkSlogJSONLogInfoStringerValue(b *testing.B) {
	var log logr.Logger = makeSlogJSONLogger()
	doStringerValue(b, log)
}

func BenchmarkSlogJSONLogInfoErrorValue(b *testing.B) {
	var log logr.Logger = makeSlogJSONLogger()
	doErrorValue(b, log)
}

func BenchmarkSlogJSONLogInfoMarshalerValue(b *testing.B) {
	var log logr.Logger = makeSlogJSONLogger()
	doMarshalerValue(b, log)
}
