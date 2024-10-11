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

	"github.com/go-logr/logr"
	"github.com/go-logr/logr/funcr"
)

func ExampleNew() {
	var log logr.Logger = funcr.New(func(prefix, args string) {
		fmt.Println(prefix, args)
	}, funcr.Options{})

	log = log.WithName("MyLogger")
	log = log.WithValues("savedKey", "savedValue")
	log.Info("the message", "key", "value")
	// Output: MyLogger "level"=0 "msg"="the message" "savedKey"="savedValue" "key"="value"
}

func ExampleNewJSON() {
	var log logr.Logger = funcr.NewJSON(func(obj string) {
		fmt.Println(obj)
	}, funcr.Options{})

	log = log.WithName("MyLogger")
	log = log.WithValues("savedKey", "savedValue")
	log.Info("the message", "key", "value")
	// Output: {"logger":"MyLogger","level":0,"msg":"the message","savedKey":"savedValue","key":"value"}
}

func ExampleUnderlier() {
	var log logr.Logger = funcr.New(func(prefix, args string) {
		fmt.Println(prefix, args)
	}, funcr.Options{})

	if underlier, ok := log.GetSink().(funcr.Underlier); ok {
		fn := underlier.GetUnderlying()
		fn("hello", "world")
	}
	// Output: hello world
}

func ExampleOptions() {
	var log logr.Logger = funcr.NewJSON(
		func(obj string) { fmt.Println(obj) },
		funcr.Options{
			LogCaller: funcr.All,
			Verbosity: 1, // V(2) and higher is ignored.
		})
	log.V(0).Info("V(0) message", "key", "value")
	log.V(1).Info("V(1) message", "key", "value")
	log.V(2).Info("V(2) message", "key", "value")
	// Output:
	// {"logger":"","caller":{"file":"example_test.go","line":67},"level":0,"msg":"V(0) message","key":"value"}
	// {"logger":"","caller":{"file":"example_test.go","line":68},"level":1,"msg":"V(1) message","key":"value"}
}

func ExampleOptions_renderHooks() {
	// prefix all builtin keys with "log:"
	prefixSpecialKeys := func(kvList []any) []any {
		for i := 0; i < len(kvList); i += 2 {
			k, _ := kvList[i].(string)
			kvList[i] = "log:" + k
		}
		return kvList
	}

	// present saved values as a single JSON object
	valuesAsObject := func(kvList []any) []any {
		return []any{"labels", funcr.PseudoStruct(kvList)}
	}

	var log logr.Logger = funcr.NewJSON(
		func(obj string) { fmt.Println(obj) },
		funcr.Options{
			RenderBuiltinsHook: prefixSpecialKeys,
			RenderValuesHook:   valuesAsObject,
		})
	log = log.WithName("MyLogger")
	log = log.WithValues("savedKey1", "savedVal1")
	log = log.WithValues("savedKey2", "savedVal2")
	log.Info("the message", "key", "value")
	// Output: {"log:logger":"MyLogger","log:level":0,"log:msg":"the message","labels":{"savedKey1":"savedVal1","savedKey2":"savedVal2"},"key":"value"}
}

func ExamplePseudoStruct() {
	var log logr.Logger = funcr.NewJSON(
		func(obj string) { fmt.Println(obj) },
		funcr.Options{})
	kv := []any{
		"field1", 12345,
		"field2", true,
	}
	log.Info("the message", "key", funcr.PseudoStruct(kv))
	// Output: {"logger":"","level":0,"msg":"the message","key":{"field1":12345,"field2":true}}
}

func ExampleOptions_maxLogDepth() {
	type List struct {
		Next *List
	}
	l := List{}
	l.Next = &l // recursive

	var log logr.Logger = funcr.NewJSON(
		func(obj string) { fmt.Println(obj) },
		funcr.Options{MaxLogDepth: 4})
	log.Info("recursive", "list", l)
	// Output: {"logger":"","level":0,"msg":"recursive","list":{"Next":{"Next":{"Next":{"Next":{"Next":"<max-log-depth-exceeded>"}}}}}}
}
