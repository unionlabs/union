// Copyright 2020 Google LLC
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

// +build go1.14

package gomock_test

import (
	"testing"

	"github.com/golang/mock/gomock"
)

func (e *ErrorReporter) Cleanup(f func()) {
	e.t.Helper()
	e.t.Cleanup(f)
}

func TestMultipleDefers(t *testing.T) {
	reporter := NewErrorReporter(t)
	reporter.Cleanup(func() {
		reporter.assertPass("No errors for multiple calls to Finish")
	})
	ctrl := gomock.NewController(reporter)
	ctrl.Finish()
}

// Equivalent to the TestNoRecordedCallsForAReceiver, but without explicitly
// calling Finish.
func TestDeferNotNeededFail(t *testing.T) {
	reporter := NewErrorReporter(t)
	subject := new(Subject)
	var ctrl *gomock.Controller
	reporter.Cleanup(func() {
		reporter.assertFatal(func() {
			ctrl.Call(subject, "NotRecordedMethod", "argument")
		}, "Unexpected call to", "there are no expected calls of the method \"NotRecordedMethod\" for that receiver")
	})
	ctrl = gomock.NewController(reporter)
}

func TestDeferNotNeededPass(t *testing.T) {
	reporter := NewErrorReporter(t)
	subject := new(Subject)
	var ctrl *gomock.Controller
	reporter.Cleanup(func() {
		reporter.assertPass("Expected method call made.")
	})
	ctrl = gomock.NewController(reporter)
	ctrl.RecordCall(subject, "FooMethod", "argument")
	ctrl.Call(subject, "FooMethod", "argument")
}

func TestOrderedCallsInCorrect(t *testing.T) {
	reporter := NewErrorReporter(t)
	subjectOne := new(Subject)
	subjectTwo := new(Subject)
	var ctrl *gomock.Controller
	reporter.Cleanup(func() {
		reporter.assertFatal(func() {
			gomock.InOrder(
				ctrl.RecordCall(subjectOne, "FooMethod", "1").AnyTimes(),
				ctrl.RecordCall(subjectTwo, "FooMethod", "2"),
				ctrl.RecordCall(subjectTwo, "BarMethod", "3"),
			)
			ctrl.Call(subjectOne, "FooMethod", "1")
			// FooMethod(2) should be called before BarMethod(3)
			ctrl.Call(subjectTwo, "BarMethod", "3")
		}, "Unexpected call to", "Subject.BarMethod([3])", "doesn't have a prerequisite call satisfied")
	})
	ctrl = gomock.NewController(reporter)
}

// Test that calls that are prerequisites to other calls but have maxCalls >
// minCalls are removed from the expected call set.
func TestOrderedCallsWithPreReqMaxUnbounded(t *testing.T) {
	reporter := NewErrorReporter(t)
	subjectOne := new(Subject)
	subjectTwo := new(Subject)
	var ctrl *gomock.Controller
	reporter.Cleanup(func() {
		reporter.assertFatal(func() {
			// Initially we should be able to call FooMethod("1") as many times as we
			// want.
			ctrl.Call(subjectOne, "FooMethod", "1")
			ctrl.Call(subjectOne, "FooMethod", "1")

			// But calling something that has it as a prerequite should remove it from
			// the expected call set. This allows tests to ensure that FooMethod("1") is
			// *not* called after FooMethod("2").
			ctrl.Call(subjectTwo, "FooMethod", "2")

			ctrl.Call(subjectOne, "FooMethod", "1")
		})
	})
	ctrl = gomock.NewController(reporter)
}

func TestCallAfterLoopPanic(t *testing.T) {
	reporter := NewErrorReporter(t)
	subject := new(Subject)
	var ctrl *gomock.Controller
	reporter.Cleanup(func() {
		firstCall := ctrl.RecordCall(subject, "FooMethod", "1")
		secondCall := ctrl.RecordCall(subject, "FooMethod", "2")
		thirdCall := ctrl.RecordCall(subject, "FooMethod", "3")

		gomock.InOrder(firstCall, secondCall, thirdCall)

		defer func() {
			err := recover()
			if err == nil {
				t.Error("Call.After creation of dependency loop did not panic.")
			}
		}()

		// This should panic due to dependency loop.
		firstCall.After(thirdCall)
	})
	ctrl = gomock.NewController(reporter)
}
