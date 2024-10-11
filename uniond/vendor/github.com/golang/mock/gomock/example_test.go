package gomock_test

//go:generate mockgen -destination mock_test.go -package gomock_test -source example_test.go

import (
	"fmt"
	"testing"
	"time"

	"github.com/golang/mock/gomock"
)

type Foo interface {
	Bar(string) string
}

func ExampleCall_DoAndReturn_latency() {
	t := &testing.T{} // provided by test
	ctrl := gomock.NewController(t)
	mockIndex := NewMockFoo(ctrl)

	mockIndex.EXPECT().Bar(gomock.Any()).DoAndReturn(
		// signature of anonymous function must have the same number of input and output arguments as the mocked method.
		func(arg string) string {
			time.Sleep(1 * time.Millisecond)
			return "I'm sleepy"
		},
	)

	r := mockIndex.Bar("foo")
	fmt.Println(r)
	// Output: I'm sleepy
}

func ExampleCall_DoAndReturn_captureArguments() {
	t := &testing.T{} // provided by test
	ctrl := gomock.NewController(t)
	mockIndex := NewMockFoo(ctrl)
	var s string

	mockIndex.EXPECT().Bar(gomock.AssignableToTypeOf(s)).DoAndReturn(
		// signature of anonymous function must have the same number of input and output arguments as the mocked method.
		func(arg string) interface{} {
			s = arg
			return "I'm sleepy"
		},
	)

	r := mockIndex.Bar("foo")
	fmt.Printf("%s %s", r, s)
	// Output: I'm sleepy foo
}
