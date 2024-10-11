package require

import (
	"github.com/prysmaticlabs/prysm/v4/testing/assertions"
	"github.com/sirupsen/logrus/hooks/test"
)

// Equal compares values using comparison operator.
func Equal(tb assertions.AssertionTestingTB, expected, actual interface{}, msg ...interface{}) {
	assertions.Equal(tb.Fatalf, expected, actual, msg...)
}

// NotEqual compares values using comparison operator.
func NotEqual(tb assertions.AssertionTestingTB, expected, actual interface{}, msg ...interface{}) {
	assertions.NotEqual(tb.Fatalf, expected, actual, msg...)
}

// DeepEqual compares values using DeepEqual.
// NOTE: this function does not work for checking arrays/slices or maps of protobuf messages.
// For arrays/slices, please use DeepSSZEqual.
// For maps, please iterate through and compare the individual keys and values.
func DeepEqual(tb assertions.AssertionTestingTB, expected, actual interface{}, msg ...interface{}) {
	assertions.DeepEqual(tb.Fatalf, expected, actual, msg...)
}

// DeepNotEqual compares values using DeepEqual.
// NOTE: this function does not work for checking arrays/slices or maps of protobuf messages.
// For arrays/slices, please use DeepNotSSZEqual.
// For maps, please iterate through and compare the individual keys and values.
func DeepNotEqual(tb assertions.AssertionTestingTB, expected, actual interface{}, msg ...interface{}) {
	assertions.DeepNotEqual(tb.Fatalf, expected, actual, msg...)
}

// DeepSSZEqual compares values using DeepEqual.
func DeepSSZEqual(tb assertions.AssertionTestingTB, expected, actual interface{}, msg ...interface{}) {
	assertions.DeepSSZEqual(tb.Fatalf, expected, actual, msg...)
}

// DeepNotSSZEqual compares values using DeepEqual.
func DeepNotSSZEqual(tb assertions.AssertionTestingTB, expected, actual interface{}, msg ...interface{}) {
	assertions.DeepNotSSZEqual(tb.Fatalf, expected, actual, msg...)
}

// NoError asserts that error is nil.
func NoError(tb assertions.AssertionTestingTB, err error, msg ...interface{}) {
	assertions.NoError(tb.Fatalf, err, msg...)
}

// ErrorContains asserts that actual error contains wanted message.
func ErrorContains(tb assertions.AssertionTestingTB, want string, err error, msg ...interface{}) {
	assertions.ErrorContains(tb.Fatalf, want, err, msg...)
}

// IsNil asserts that the observed value is nil.
func IsNil(tb assertions.AssertionTestingTB, got interface{}, msg ...interface{}) {
	assertions.IsNil(tb.Fatalf, got, msg...)
}

// NotNil asserts that passed value is not nil.
func NotNil(tb assertions.AssertionTestingTB, obj interface{}, msg ...interface{}) {
	assertions.NotNil(tb.Fatalf, obj, msg...)
}

// LogsContain checks that the desired string is a subset of the current log output.
func LogsContain(tb assertions.AssertionTestingTB, hook *test.Hook, want string, msg ...interface{}) {
	assertions.LogsContain(tb.Fatalf, hook, want, true, msg...)
}

// LogsDoNotContain is the inverse check of LogsContain.
func LogsDoNotContain(tb assertions.AssertionTestingTB, hook *test.Hook, want string, msg ...interface{}) {
	assertions.LogsContain(tb.Fatalf, hook, want, false, msg...)
}

// NotEmpty checks that the object fields are not empty. This method also checks all of the
// pointer fields to ensure none of those fields are empty.
func NotEmpty(tb assertions.AssertionTestingTB, obj interface{}, msg ...interface{}) {
	assertions.NotEmpty(tb.Fatalf, obj, msg...)
}

// ErrorIs uses Errors.Is to recursively unwrap err looking for target in the chain.
// If any error in the chain matches target, the assertion will pass.
func ErrorIs(tb assertions.AssertionTestingTB, err, target error, msg ...interface{}) {
	assertions.ErrorIs(tb.Fatalf, err, target, msg)
}

// StringContains asserts that actual string contains expected message.
func StringContains(tb assertions.AssertionTestingTB, expected, actual string, msg ...interface{}) {
	assertions.StringContains(tb.Fatalf, expected, actual, true, msg)
}
