package assert

import (
	"github.com/prysmaticlabs/prysm/v4/testing/assertions"
	"github.com/sirupsen/logrus/hooks/test"
)

// Equal compares values using comparison operator.
func Equal(tb assertions.AssertionTestingTB, expected, actual interface{}, msg ...interface{}) {
	assertions.Equal(tb.Errorf, expected, actual, msg...)
}

// NotEqual compares values using comparison operator.
func NotEqual(tb assertions.AssertionTestingTB, expected, actual interface{}, msg ...interface{}) {
	assertions.NotEqual(tb.Errorf, expected, actual, msg...)
}

// DeepEqual compares values using DeepEqual.
// NOTE: this function does not work for checking arrays/slices or maps of protobuf messages.
// For arrays/slices, please use DeepSSZEqual.
// For maps, please iterate through and compare the individual keys and values.
func DeepEqual(tb assertions.AssertionTestingTB, expected, actual interface{}, msg ...interface{}) {
	assertions.DeepEqual(tb.Errorf, expected, actual, msg...)
}

// DeepNotEqual compares values using DeepEqual.
// NOTE: this function does not work for checking arrays/slices or maps of protobuf messages.
// For arrays/slices, please use DeepNotSSZEqual.
// For maps, please iterate through and compare the individual keys and values.
func DeepNotEqual(tb assertions.AssertionTestingTB, expected, actual interface{}, msg ...interface{}) {
	assertions.DeepNotEqual(tb.Errorf, expected, actual, msg...)
}

// DeepSSZEqual compares values using ssz.DeepEqual.
func DeepSSZEqual(tb assertions.AssertionTestingTB, expected, actual interface{}, msg ...interface{}) {
	assertions.DeepSSZEqual(tb.Errorf, expected, actual, msg...)
}

// DeepNotSSZEqual compares values using ssz.DeepEqual.
func DeepNotSSZEqual(tb assertions.AssertionTestingTB, expected, actual interface{}, msg ...interface{}) {
	assertions.DeepNotSSZEqual(tb.Errorf, expected, actual, msg...)
}

// StringContains asserts a string contains specified substring.
func StringContains(tb assertions.AssertionTestingTB, expected, actual string, msg ...interface{}) {
	assertions.StringContains(tb.Errorf, expected, actual, true, msg...)
}

// StringNotContains asserts a string does not contain specified substring.
func StringNotContains(tb assertions.AssertionTestingTB, expected, actual string, msg ...interface{}) {
	assertions.StringContains(tb.Errorf, expected, actual, false, msg...)
}

// NoError asserts that error is nil.
func NoError(tb assertions.AssertionTestingTB, err error, msg ...interface{}) {
	assertions.NoError(tb.Errorf, err, msg...)
}

// ErrorContains asserts that actual error contains wanted message.
func ErrorContains(tb assertions.AssertionTestingTB, want string, err error, msg ...interface{}) {
	assertions.ErrorContains(tb.Errorf, want, err, msg...)
}

// NotNil asserts that passed value is not nil.
func NotNil(tb assertions.AssertionTestingTB, obj interface{}, msg ...interface{}) {
	assertions.NotNil(tb.Errorf, obj, msg...)
}

// LogsContain checks that the desired string is a subset of the current log output.
func LogsContain(tb assertions.AssertionTestingTB, hook *test.Hook, want string, msg ...interface{}) {
	assertions.LogsContain(tb.Errorf, hook, want, true, msg...)
}

// LogsDoNotContain is the inverse check of LogsContain.
func LogsDoNotContain(tb assertions.AssertionTestingTB, hook *test.Hook, want string, msg ...interface{}) {
	assertions.LogsContain(tb.Errorf, hook, want, false, msg...)
}

// NotEmpty checks that the object fields are not empty. This method also checks all of the
// pointer fields to ensure none of those fields are empty.
func NotEmpty(tb assertions.AssertionTestingTB, obj interface{}, msg ...interface{}) {
	assertions.NotEmpty(tb.Errorf, obj, msg...)
}
