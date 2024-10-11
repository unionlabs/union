//go:build go1.21
// +build go1.21

/*
Copyright 2023 The logr Authors.

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

// Package main is an example of using slogr.
package main

import (
	"context"
	"fmt"
	"log/slog"
	"os"

	"github.com/go-logr/logr"
	"github.com/go-logr/logr/funcr"
)

type e struct {
	str string
}

func (e e) Error() string {
	return e.str
}

func logrHelper(log logr.Logger, msg string) {
	logrHelper2(log, msg)
}

func logrHelper2(log logr.Logger, msg string) {
	log.WithCallDepth(2).Info(msg)
}

func slogHelper(log *slog.Logger, msg string) {
	slogHelper2(log, msg)
}

func slogHelper2(log *slog.Logger, msg string) {
	// slog.Logger has no API for skipping helper functions, so this gets logged as call location.
	log.Info(msg)
}

func main() {
	opts := slog.HandlerOptions{
		AddSource: true,
		Level:     slog.Level(-1),
	}
	handler := slog.NewJSONHandler(os.Stderr, &opts)
	logrLogger := logr.FromSlogHandler(handler)
	logrExample(logrLogger)

	logrLogger = funcr.NewJSON(
		func(obj string) { fmt.Println(obj) },
		funcr.Options{
			LogCaller:    funcr.All,
			LogTimestamp: true,
			Verbosity:    1,
		})
	slogLogger := slog.New(logr.ToSlogHandler(logrLogger))
	slogExample(slogLogger)
}

func logrExample(log logr.Logger) {
	log = log.WithName("my")
	log = log.WithName("logger")
	log = log.WithName("name")
	log = log.WithValues("saved", "value")
	log.Info("1) hello", "val1", 1, "val2", map[string]int{"k": 1})
	log.V(1).Info("2) you should see this")
	log.V(1).V(1).Info("you should NOT see this")
	log.Error(nil, "3) uh oh", "trouble", true, "reasons", []float64{0.1, 0.11, 3.14})
	log.Error(e{"an error occurred"}, "4) goodbye", "code", -1)
	logrHelper(log, "5) thru a helper")
}

func slogExample(log *slog.Logger) {
	// There's no guarantee that this logs the right source code location.
	// It works for Go 1.21.0 by compensating in logr.ToSlogHandler
	// for the additional callers, but those might change.
	log = log.With("saved", "value")
	log.Info("1) hello", "val1", 1, "val2", map[string]int{"k": 1})
	log.Log(context.TODO(), slog.Level(-1), "2) you should see this")
	log.Log(context.TODO(), slog.Level(-2), "you should NOT see this")
	log.Error("3) uh oh", "trouble", true, "reasons", []float64{0.1, 0.11, 3.14})
	log.Error("4) goodbye", "code", -1, "err", e{"an error occurred"})
	slogHelper(log, "5) thru a helper")
}
