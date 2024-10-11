package log_test

import (
	"bytes"
	"errors"
	"io/ioutil"
	"testing"

	"github.com/go-kit/log"
)

func TestJSONLoggerCaller(t *testing.T) {
	t.Parallel()
	buf := &bytes.Buffer{}
	logger := log.NewJSONLogger(buf)
	logger = log.With(logger, "caller", log.DefaultCaller)

	if err := logger.Log(); err != nil {
		t.Fatal(err)
	}
	if want, have := `{"caller":"json_logger_test.go:18"}`+"\n", buf.String(); want != have {
		t.Errorf("\nwant %#v\nhave %#v", want, have)
	}
}

func TestJSONLogger(t *testing.T) {
	t.Parallel()
	buf := &bytes.Buffer{}
	logger := log.NewJSONLogger(buf)
	if err := logger.Log("err", errors.New("err"), "m", map[string]int{"0": 0}, "a", []int{1, 2, 3}); err != nil {
		t.Fatal(err)
	}
	if want, have := `{"a":[1,2,3],"err":"err","m":{"0":0}}`+"\n", buf.String(); want != have {
		t.Errorf("\nwant %#v\nhave %#v", want, have)
	}
}

func TestJSONLoggerMissingValue(t *testing.T) {
	t.Parallel()
	buf := &bytes.Buffer{}
	logger := log.NewJSONLogger(buf)
	if err := logger.Log("k"); err != nil {
		t.Fatal(err)
	}
	if want, have := `{"k":"(MISSING)"}`+"\n", buf.String(); want != have {
		t.Errorf("\nwant %#v\nhave %#v", want, have)
	}
}

func TestJSONLoggerNilStringerKey(t *testing.T) {
	t.Parallel()

	buf := &bytes.Buffer{}
	logger := log.NewJSONLogger(buf)
	if err := logger.Log((*stringer)(nil), "v"); err != nil {
		t.Fatal(err)
	}
	if want, have := `{"NULL":"v"}`+"\n", buf.String(); want != have {
		t.Errorf("\nwant %#v\nhave %#v", want, have)
	}
}

func TestJSONLoggerPanicStringerValue(t *testing.T) {
	t.Parallel()

	buf := &bytes.Buffer{}
	logger := log.NewJSONLogger(buf)
	if err := logger.Log("k", unsafeStringer{}); err != nil {
		t.Fatal(err)
	}
	if want, have := `{"k":"PANIC in String method: error"}`+"\n", buf.String(); want != have {
		t.Errorf("\nwant %#v\nhave %#v", want, have)
	}
}

func TestJSONLoggerNilErrorValue(t *testing.T) {
	t.Parallel()

	buf := &bytes.Buffer{}
	logger := log.NewJSONLogger(buf)
	if err := logger.Log("err", (*stringError)(nil)); err != nil {
		t.Fatal(err)
	}
	if want, have := `{"err":null}`+"\n", buf.String(); want != have {
		t.Errorf("\nwant %#v\nhave %#v", want, have)
	}
}

func TestJSONLoggerPanicErrorValue(t *testing.T) {
	t.Parallel()

	buf := &bytes.Buffer{}
	logger := log.NewJSONLogger(buf)
	if err := logger.Log("err", unsafeError{}); err != nil {
		t.Fatal(err)
	}
	if want, have := `{"err":"PANIC in Error method: error"}`+"\n", buf.String(); want != have {
		t.Errorf("\nwant %#v\nhave %#v", want, have)
	}
}

func TestJSONLoggerNoHTMLEscape(t *testing.T) {
	t.Parallel()
	buf := &bytes.Buffer{}
	logger := log.NewJSONLogger(buf)
	if err := logger.Log("k", "<&>"); err != nil {
		t.Fatal(err)
	}
	if want, have := `{"k":"<&>"}`+"\n", buf.String(); want != have {
		t.Errorf("\nwant %#v\nhave%#v", want, have)
	}
}

// aller implements json.Marshaler, encoding.TextMarshaler, and fmt.Stringer.
type aller struct{}

func (aller) MarshalJSON() ([]byte, error) {
	return []byte("\"json\""), nil
}

func (aller) MarshalText() ([]byte, error) {
	return []byte("text"), nil
}

func (aller) String() string {
	return "string"
}

func (aller) Error() string {
	return "error"
}

// textstringer implements encoding.TextMarshaler and fmt.Stringer.
type textstringer struct{}

func (textstringer) MarshalText() ([]byte, error) {
	return []byte("text"), nil
}

func (textstringer) String() string {
	return "string"
}

func TestJSONLoggerStringValue(t *testing.T) {
	t.Parallel()
	tests := []struct {
		v        interface{}
		expected string
	}{
		{
			v:        aller{},
			expected: `{"v":"json"}`,
		},
		{
			v:        textstringer{},
			expected: `{"v":"text"}`,
		},
		{
			v:        stringer("string"),
			expected: `{"v":"string"}`,
		},
	}

	for _, test := range tests {
		buf := &bytes.Buffer{}
		logger := log.NewJSONLogger(buf)
		if err := logger.Log("v", test.v); err != nil {
			t.Fatal(err)
		}

		if want, have := test.expected+"\n", buf.String(); want != have {
			t.Errorf("\nwant %#v\nhave %#v", want, have)
		}
	}
}

type stringer string

func (s stringer) String() string {
	return string(s)
}

type stringError string

func (s stringError) Error() string {
	return string(s)
}

type unsafeStringer struct{}

func (s unsafeStringer) String() string {
	panic("error")
}

type unsafeError struct{}

func (s unsafeError) Error() string {
	panic("error")
}

func BenchmarkJSONLoggerSimple(b *testing.B) {
	benchmarkRunner(b, log.NewJSONLogger(ioutil.Discard), baseMessage)
}

func BenchmarkJSONLoggerContextual(b *testing.B) {
	benchmarkRunner(b, log.NewJSONLogger(ioutil.Discard), withMessage)
}

func TestJSONLoggerConcurrency(t *testing.T) {
	t.Parallel()
	testConcurrency(t, log.NewJSONLogger(ioutil.Discard), 10000)
}
