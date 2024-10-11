// Copyright 2020 Google Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package gomock

import (
	"fmt"
	"reflect"
	"testing"
)

type foo struct{}

func (f foo) String() string {
	return "meow"
}

type a struct {
	name string
}

func (testObj a) Name() string {
	return testObj.name
}

type b struct {
	foo string
}

func (testObj b) Foo() string {
	return testObj.foo
}

type mockTestReporter struct {
	errorCalls int
	fatalCalls int
}

func (o *mockTestReporter) Errorf(format string, args ...interface{}) {
	o.errorCalls++
}

func (o *mockTestReporter) Fatalf(format string, args ...interface{}) {
	o.fatalCalls++
}

func (o *mockTestReporter) Helper() {}

func TestCall_After(t *testing.T) {
	t.Run("SelfPrereqCallsFatalf", func(t *testing.T) {
		tr1 := &mockTestReporter{}

		c := &Call{t: tr1}
		c.After(c)

		if tr1.fatalCalls != 1 {
			t.Errorf("number of fatal calls == %v, want 1", tr1.fatalCalls)
		}
	})

	t.Run("LoopInCallOrderCallsFatalf", func(t *testing.T) {
		tr1 := &mockTestReporter{}
		tr2 := &mockTestReporter{}

		c1 := &Call{t: tr1}
		c2 := &Call{t: tr2}
		c1.After(c2)
		c2.After(c1)

		if tr1.errorCalls != 0 || tr1.fatalCalls != 0 {
			t.Error("unexpected errors")
		}

		if tr2.fatalCalls != 1 {
			t.Errorf("number of fatal calls == %v, want 1", tr2.fatalCalls)
		}
	})
}

func prepareDoCall(doFunc, callFunc interface{}) *Call {
	tr := &mockTestReporter{}

	c := &Call{
		t:          tr,
		methodType: reflect.TypeOf(callFunc),
	}

	c.Do(doFunc)

	return c
}

func prepareDoAndReturnCall(doFunc, callFunc interface{}) *Call {
	tr := &mockTestReporter{}

	c := &Call{
		t:          tr,
		methodType: reflect.TypeOf(callFunc),
	}

	c.DoAndReturn(doFunc)

	return c
}

type testCase struct {
	description string
	doFunc      interface{}
	callFunc    interface{}
	args        []interface{}
	expectPanic bool
}

var testCases []testCase = []testCase{
	{
		description: "argument to Do is not a function",
		doFunc:      "meow",
		callFunc:    func(x int, y int) {},
		args:        []interface{}{0, 1},
		expectPanic: true,
	}, {
		description: "argument to Do is not a function",
		doFunc:      "meow",
		callFunc: func(x int, y int) bool {
			return true
		},
		args:        []interface{}{0, 1},
		expectPanic: true,
	}, {
		description: "number of args for Do func don't match Call func",
		doFunc:      func(x int) {},
		callFunc:    func(x int, y int) {},
		args:        []interface{}{0, 1},
		expectPanic: false,
	}, {
		description: "number of args for Do func don't match Call func",
		doFunc: func(x int) bool {
			return true
		},
		callFunc: func(x int, y int) bool {
			return true
		},
		args:        []interface{}{0, 1},
		expectPanic: false,
	}, {
		description: "arg type for Do func incompatible with Call func",
		doFunc:      func(x int) {},
		callFunc:    func(x string) {},
		args:        []interface{}{"meow"},
		expectPanic: true,
	}, {
		description: "arg type for Do func incompatible with Call func",
		doFunc: func(x int) bool {
			return true
		},
		callFunc: func(x string) bool {
			return true
		},
		args:        []interface{}{"meow"},
		expectPanic: true,
	}, {
		description: "Do func(int) Call func(int)",
		doFunc:      func(x int) {},
		callFunc:    func(x int) {},
		args:        []interface{}{0},
	}, {
		description: "Do func(int) Call func(interface{})",
		doFunc:      func(x int) {},
		callFunc:    func(x interface{}) {},
		args:        []interface{}{0},
	}, {
		description: "Do func(int) bool Call func(int) bool",
		doFunc: func(x int) bool {
			return true
		},
		callFunc: func(x int) bool {
			return true
		},
		args: []interface{}{0},
	}, {
		description: "Do func(int) bool Call func(interface{}) bool",
		doFunc: func(x int) bool {
			return true
		},
		callFunc: func(x interface{}) bool {
			return true
		},
		args: []interface{}{0},
	}, {
		description: "Do func(string) Call func([]byte)",
		doFunc:      func(x string) {},
		callFunc:    func(x []byte) {},
		args:        []interface{}{[]byte("meow")},
		expectPanic: true,
	}, {
		description: "Do func(string) bool Call func([]byte) bool",
		doFunc: func(x string) bool {
			return true
		},
		callFunc: func(x []byte) bool {
			return true
		},
		args:        []interface{}{[]byte("meow")},
		expectPanic: true,
	}, {
		description: "Do func(map[int]string) Call func(map[interface{}]int)",
		doFunc:      func(x map[int]string) {},
		callFunc:    func(x map[interface{}]int) {},
		args:        []interface{}{map[interface{}]int{"meow": 0}},
		expectPanic: true,
	}, {
		description: "Do func(map[int]string) Call func(map[interface{}]interface{})",
		doFunc:      func(x map[int]string) {},
		callFunc:    func(x map[interface{}]interface{}) {},
		args:        []interface{}{map[interface{}]interface{}{"meow": "meow"}},
		expectPanic: true,
	}, {
		description: "Do func(map[int]string) bool Call func(map[interface{}]int) bool",
		doFunc: func(x map[int]string) bool {
			return true
		},
		callFunc: func(x map[interface{}]int) bool {
			return true
		},
		args:        []interface{}{map[interface{}]int{"meow": 0}},
		expectPanic: true,
	}, {
		description: "Do func(map[int]string) bool Call func(map[interface{}]interface{}) bool",
		doFunc: func(x map[int]string) bool {
			return true
		},
		callFunc: func(x map[interface{}]interface{}) bool {
			return true
		},
		args:        []interface{}{map[interface{}]interface{}{"meow": "meow"}},
		expectPanic: true,
	}, {
		description: "Do func([]string) Call func([]interface{})",
		doFunc:      func(x []string) {},
		callFunc:    func(x []interface{}) {},
		args:        []interface{}{[]interface{}{0}},
		expectPanic: true,
	}, {
		description: "Do func([]string) Call func([]int)",
		doFunc:      func(x []string) {},
		callFunc:    func(x []int) {},
		args:        []interface{}{[]int{0, 1}},
		expectPanic: true,
	}, {
		description: "Do func([]int) Call func([]int)",
		doFunc:      func(x []int) {},
		callFunc:    func(x []int) {},
		args:        []interface{}{[]int{0, 1}},
	}, {
		description: "Do func([]int) Call func([]interface{})",
		doFunc:      func(x []int) {},
		callFunc:    func(x []interface{}) {},
		args:        []interface{}{[]interface{}{0}},
		expectPanic: true,
	}, {
		description: "Do func([]int) Call func(...interface{})",
		doFunc:      func(x []int) {},
		callFunc:    func(x ...interface{}) {},
		args:        []interface{}{0, 1},
		expectPanic: true,
	}, {
		description: "Do func([]int) Call func(...int)",
		doFunc:      func(x []int) {},
		callFunc:    func(x ...int) {},
		args:        []interface{}{0, 1},
		expectPanic: true,
	}, {
		description: "Do func([]string) bool Call func([]interface{}) bool",
		doFunc: func(x []string) bool {
			return true
		},
		callFunc: func(x []interface{}) bool {
			return true
		},
		args:        []interface{}{[]interface{}{0}},
		expectPanic: true,
	}, {
		description: "Do func([]string) bool Call func([]int) bool",
		doFunc: func(x []string) bool {
			return true
		},
		callFunc: func(x []int) bool {
			return true
		},
		args:        []interface{}{[]int{0, 1}},
		expectPanic: true,
	}, {
		description: "Do func([]int) bool Call func([]int) bool",
		doFunc: func(x []int) bool {
			return true
		},
		callFunc: func(x []int) bool {
			return true
		},
		args: []interface{}{[]int{0, 1}},
	}, {
		description: "Do func([]int) bool Call func([]interface{}) bool",
		doFunc: func(x []int) bool {
			return true
		},
		callFunc: func(x []interface{}) bool {
			return true
		},
		args:        []interface{}{[]interface{}{0}},
		expectPanic: true,
	}, {
		description: "Do func([]int) bool Call func(...interface{}) bool",
		doFunc: func(x []int) bool {
			return true
		},
		callFunc: func(x ...interface{}) bool {
			return true
		},
		args:        []interface{}{0, 1},
		expectPanic: true,
	}, {
		description: "Do func([]int) bool Call func(...int) bool",
		doFunc: func(x []int) bool {
			return true
		},
		callFunc: func(x ...int) bool {
			return true
		},
		args:        []interface{}{0, 1},
		expectPanic: true,
	}, {
		description: "Do func(...int) Call func([]int)",
		doFunc:      func(x ...int) {},
		callFunc:    func(x []int) {},
		args:        []interface{}{[]int{0, 1}},
		expectPanic: true,
	}, {
		description: "Do func(...int) Call func([]interface{})",
		doFunc:      func(x ...int) {},
		callFunc:    func(x []interface{}) {},
		args:        []interface{}{[]interface{}{0, 1}},
		expectPanic: true,
	}, {
		description: "Do func(...int) Call func(...interface{})",
		doFunc:      func(x ...int) {},
		callFunc:    func(x ...interface{}) {},
		args:        []interface{}{0, 1},
	}, {
		description: "Do func(...int) bool Call func(...int) bool",
		doFunc: func(x ...int) bool {
			return true
		},
		callFunc: func(x ...int) bool {
			return true
		},
		args: []interface{}{0, 1},
	}, {
		description: "Do func(...int) bool Call func([]int) bool",
		doFunc: func(x ...int) bool {
			return true
		},
		callFunc: func(x []int) bool {
			return true
		},
		args:        []interface{}{[]int{0, 1}},
		expectPanic: true,
	}, {
		description: "Do func(...int) bool Call func([]interface{}) bool",
		doFunc: func(x ...int) bool {
			return true
		},
		callFunc: func(x []interface{}) bool {
			return true
		},
		args:        []interface{}{[]interface{}{0, 1}},
		expectPanic: true,
	}, {
		description: "Do func(...int) bool Call func(...interface{}) bool",
		doFunc: func(x ...int) bool {
			return true
		},
		callFunc: func(x ...interface{}) bool {
			return true
		},
		args: []interface{}{0, 1},
	}, {
		description: "Do func(...int) Call func(...int)",
		doFunc:      func(x ...int) {},
		callFunc:    func(x ...int) {},
		args:        []interface{}{0, 1},
	}, {
		description: "Do func(foo); foo implements interface X Call func(interface X)",
		doFunc:      func(x foo) {},
		callFunc:    func(x fmt.Stringer) {},
		args:        []interface{}{foo{}},
	}, {
		description: "Do func(b); b does not implement interface X Call func(interface X)",
		doFunc:      func(x b) {},
		callFunc:    func(x fmt.Stringer) {},
		args:        []interface{}{foo{}},
		expectPanic: true,
	}, {
		description: "Do func(b) Call func(a); a and b are not aliases",
		doFunc:      func(x b) {},
		callFunc:    func(x a) {},
		args:        []interface{}{a{}},
		expectPanic: true,
	}, {
		description: "Do func(foo) bool; foo implements interface X Call func(interface X) bool",
		doFunc: func(x foo) bool {
			return true
		},
		callFunc: func(x fmt.Stringer) bool {
			return true
		},
		args: []interface{}{foo{}},
	}, {
		description: "Do func(b) bool; b does not implement interface X Call func(interface X) bool",
		doFunc: func(x b) bool {
			return true
		},
		callFunc: func(x fmt.Stringer) bool {
			return true
		},
		args:        []interface{}{foo{}},
		expectPanic: true,
	}, {
		description: "Do func(b) bool Call func(a) bool; a and b are not aliases",
		doFunc: func(x b) bool {
			return true
		},
		callFunc: func(x a) bool {
			return true
		},
		args:        []interface{}{a{}},
		expectPanic: true,
	}, {
		description: "Do func(bool) b Call func(bool) a; a and b are not aliases",
		doFunc: func(x bool) b {
			return b{}
		},
		callFunc: func(x bool) a {
			return a{}
		},
		args: []interface{}{true},
	},
}

func TestCall_Do(t *testing.T) {
	for _, tc := range testCases {
		t.Run(tc.description, func(t *testing.T) {
			c := prepareDoCall(tc.doFunc, tc.callFunc)

			if len(c.actions) != 1 {
				t.Errorf("expected %d actions but got %d", 1, len(c.actions))
			}

			action := c.actions[0]

			if tc.expectPanic {
				defer func() {
					if r := recover(); r == nil {
						t.Error("expected Do to panic")
					}
				}()
			}

			action(tc.args)
		})
	}
}

func TestCall_Do_NumArgValidation(t *testing.T) {
	tests := []struct {
		name       string
		methodType reflect.Type
		doFn       interface{}
		args       []interface{}
		wantErr    bool
	}{
		{
			name:       "too few",
			methodType: reflect.TypeOf(func(one, two string) {}),
			doFn:       func(one string) {},
			args:       []interface{}{"too", "few"},
			wantErr:    true,
		},
		{
			name:       "too many",
			methodType: reflect.TypeOf(func(one, two string) {}),
			doFn:       func(one, two, three string) {},
			args:       []interface{}{"too", "few"},
			wantErr:    true,
		},
		{
			name:       "just right",
			methodType: reflect.TypeOf(func(one, two string) {}),
			doFn:       func(one string, two string) {},
			args:       []interface{}{"just", "right"},
			wantErr:    false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tr := &mockTestReporter{}
			call := &Call{
				t:          tr,
				methodType: tt.methodType,
			}
			call.Do(tt.doFn)
			call.actions[0](tt.args)
			if tt.wantErr && tr.fatalCalls != 1 {
				t.Fatalf("expected call to fail")
			}
			if !tt.wantErr && tr.fatalCalls != 0 {
				t.Fatalf("expected call to pass")
			}
		})
	}
}

func TestCall_DoAndReturn_NumArgValidation(t *testing.T) {
	tests := []struct {
		name       string
		methodType reflect.Type
		doFn       interface{}
		args       []interface{}
		wantErr    bool
	}{
		{
			name:       "too few",
			methodType: reflect.TypeOf(func(one, two string) string { return "" }),
			doFn:       func(one string) {},
			args:       []interface{}{"too", "few"},
			wantErr:    true,
		},
		{
			name:       "too many",
			methodType: reflect.TypeOf(func(one, two string) string { return "" }),
			doFn:       func(one, two, three string) string { return "" },
			args:       []interface{}{"too", "few"},
			wantErr:    true,
		},
		{
			name:       "just right",
			methodType: reflect.TypeOf(func(one, two string) string { return "" }),
			doFn:       func(one string, two string) string { return "" },
			args:       []interface{}{"just", "right"},
			wantErr:    false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tr := &mockTestReporter{}
			call := &Call{
				t:          tr,
				methodType: tt.methodType,
			}
			call.DoAndReturn(tt.doFn)
			call.actions[0](tt.args)
			if tt.wantErr && tr.fatalCalls != 1 {
				t.Fatalf("expected call to fail")
			}
			if !tt.wantErr && tr.fatalCalls != 0 {
				t.Fatalf("expected call to pass")
			}
		})
	}
}

func TestCall_DoAndReturn(t *testing.T) {
	for _, tc := range testCases {
		t.Run(tc.description, func(t *testing.T) {
			c := prepareDoAndReturnCall(tc.doFunc, tc.callFunc)

			if len(c.actions) != 1 {
				t.Errorf("expected %d actions but got %d", 1, len(c.actions))
			}

			action := c.actions[0]

			if tc.expectPanic {
				defer func() {
					if r := recover(); r == nil {
						t.Error("expected DoAndReturn to panic")
					}
				}()
			}

			action(tc.args)
		})
	}
}
