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

package logr_test

import (
	"fmt"

	"github.com/go-logr/logr"
	"github.com/go-logr/logr/funcr"
)

// NewStdoutLogger returns a logr.Logger that prints to stdout.
func NewStdoutLogger() logr.Logger {
	return funcr.New(func(prefix, args string) {
		if prefix != "" {
			fmt.Printf("%s: %s\n", prefix, args)
		} else {
			fmt.Println(args)
		}
	}, funcr.Options{})
}

func Example() {
	l := NewStdoutLogger()
	l.Info("default info log", "stringVal", "value", "intVal", 12345)
	l.V(0).Info("V(0) info log", "stringVal", "value", "intVal", 12345)
	l.Error(fmt.Errorf("an error"), "error log", "stringVal", "value", "intVal", 12345)
	// Output:
	// "level"=0 "msg"="default info log" "stringVal"="value" "intVal"=12345
	// "level"=0 "msg"="V(0) info log" "stringVal"="value" "intVal"=12345
	// "msg"="error log" "error"="an error" "stringVal"="value" "intVal"=12345
}

func ExampleLogger_Info() {
	l := NewStdoutLogger()
	l.Info("this is a V(0)-equivalent info log", "stringVal", "value", "intVal", 12345)
	// Output:
	// "level"=0 "msg"="this is a V(0)-equivalent info log" "stringVal"="value" "intVal"=12345
}

func ExampleLogger_Error() {
	l := NewStdoutLogger()
	l.Error(fmt.Errorf("the error"), "this is an error log", "stringVal", "value", "intVal", 12345)
	l.Error(nil, "this is an error log with nil error", "stringVal", "value", "intVal", 12345)
	// Output:
	// "msg"="this is an error log" "error"="the error" "stringVal"="value" "intVal"=12345
	// "msg"="this is an error log with nil error" "error"=null "stringVal"="value" "intVal"=12345
}

func ExampleLogger_WithName() {
	l := NewStdoutLogger()
	l = l.WithName("name1")
	l.Info("this is an info log", "stringVal", "value", "intVal", 12345)
	l = l.WithName("name2")
	l.Info("this is an info log", "stringVal", "value", "intVal", 12345)
	// Output:
	// name1: "level"=0 "msg"="this is an info log" "stringVal"="value" "intVal"=12345
	// name1/name2: "level"=0 "msg"="this is an info log" "stringVal"="value" "intVal"=12345
}

func ExampleLogger_WithValues() {
	l := NewStdoutLogger()
	l = l.WithValues("stringVal", "value", "intVal", 12345)
	l = l.WithValues("boolVal", true)
	l.Info("this is an info log", "floatVal", 3.1415)
	// Output:
	// "level"=0 "msg"="this is an info log" "stringVal"="value" "intVal"=12345 "boolVal"=true "floatVal"=3.1415
}

func ExampleLogger_V() {
	l := NewStdoutLogger()
	l.V(0).Info("V(0) info log")
	l.V(1).Info("V(1) info log")
	l.V(2).Info("V(2) info log")
	// Output:
	// "level"=0 "msg"="V(0) info log"
}

func ExampleLogger_Enabled() {
	l := NewStdoutLogger()
	if loggerV := l.V(5); loggerV.Enabled() {
		// Do something expensive.
		loggerV.Info("this is an expensive log message")
	}
	// Output:
}
