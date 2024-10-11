package errutil

import (
	"errors"
	"reflect"
	"testing"
)

type errcloser struct {
	err error
}

func (e errcloser) Close() error { return e.err }

func TestCheckClose(t *testing.T) {
	a := errors.New("a")
	b := errors.New("b")
	cases := []struct {
		Previous   error
		CloseError error
		Expect     error
	}{
		{Previous: nil, CloseError: nil, Expect: nil},
		{Previous: a, CloseError: nil, Expect: a},
		{Previous: nil, CloseError: b, Expect: b},
		{Previous: a, CloseError: b, Expect: a},
	}
	for _, c := range cases {
		err := c.Previous
		closer := errcloser{err: c.CloseError}
		CheckClose(&err, closer)
		if !reflect.DeepEqual(err, c.Expect) {
			t.Fatalf("CheckErr() got %v expected %v", err, c.Expect)
		}
	}
}
