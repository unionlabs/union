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

// Package main is an example of using funcr.
package main

import (
	"log/slog"

	"github.com/go-logr/logr"
)

func doSlog(log logr.Logger) {
	slogger := slog.New(logr.ToSlogHandler(log.WithName("slog").WithValues("mode", "slog")))
	slogExample(slogger)
}

func slogExample(log *slog.Logger) {
	log.Warn("hello", "val1", 1, "val2", map[string]int{"k": 1})
	log.Info("you should see this")
	log.Debug("you should NOT see this")
	log.Error("uh oh", "trouble", true, "reasons", []float64{0.1, 0.11, 3.14})
	log.With("attr1", 1, "attr2", 2).Info("with attrs")
	log.WithGroup("groupname").Info("with group", "slog2", false)
	log.WithGroup("group1").With("attr1", 1).WithGroup("group2").With("attr2", 2).Info("msg", "arg", "val")
}
