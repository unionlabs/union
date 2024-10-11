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

package stdr_test

import (
	"errors"
	"log"
	"os"

	"github.com/go-logr/stdr"
)

var errSome = errors.New("some error")

func newStdLogger(flags int) stdr.StdLogger {
	return log.New(os.Stdout, "", flags)
}

func ExampleNew() {
	log := stdr.New(newStdLogger(log.Lshortfile))
	log.Info("info message with default options")
	log.Error(errSome, "error message with default options")
	log.Info("invalid key", 42, "answer")
	log.Info("missing value", "answer")
	// Output:
	// example_test.go:35: "level"=0 "msg"="info message with default options"
	// example_test.go:36: "msg"="error message with default options" "error"="some error"
	// example_test.go:37: "level"=0 "msg"="invalid key" "<non-string-key: 42>"="answer"
	// example_test.go:38: "level"=0 "msg"="missing value" "answer"="<no-value>"
}

func ExampleNew_withName() {
	log := stdr.New(newStdLogger(0))
	log.WithName("hello").WithName("world").Info("thanks for the fish")
	// Output:
	// hello/world: "level"=0 "msg"="thanks for the fish"
}

func ExampleNewWithOptions() {
	log := stdr.NewWithOptions(newStdLogger(0), stdr.Options{LogCaller: stdr.All})
	log.Info("with LogCaller=All")
	// Output:
	// "caller"={"file":"example_test.go","line":55} "level"=0 "msg"="with LogCaller=All"
}
