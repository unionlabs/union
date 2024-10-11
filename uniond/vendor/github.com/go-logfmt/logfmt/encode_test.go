package logfmt_test

import (
	"bytes"
	"errors"
	"fmt"
	"io/ioutil"
	"reflect"
	"testing"
	"time"

	"github.com/go-logfmt/logfmt"
)

func TestEncodeKeyval(t *testing.T) {
	data := []struct {
		key, value interface{}
		want       string
		err        error
	}{
		{key: "k", value: "v", want: "k=v"},
		{key: "k", value: nil, want: "k=null"},
		{key: `\`, value: "v", want: `\=v`},
		{key: "k", value: "", want: "k="},
		{key: "k", value: "null", want: `k="null"`},
		{key: "k", value: "<nil>", want: `k=<nil>`},
		{key: "k", value: true, want: "k=true"},
		{key: "k", value: 1, want: "k=1"},
		{key: "k", value: 1.025, want: "k=1.025"},
		{key: "k", value: 1e-3, want: "k=0.001"},
		{key: "k", value: 3.5 + 2i, want: "k=(3.5+2i)"},
		{key: "k", value: "v v", want: `k="v v"`},
		{key: "k", value: " ", want: `k=" "`},
		{key: "k", value: `"`, want: `k="\""`},
		{key: "k", value: `=`, want: `k="="`},
		{key: "k", value: `\`, want: `k=\`},
		{key: "k", value: `=\`, want: `k="=\\"`},
		{key: "k", value: `\"`, want: `k="\\\""`},
		{key: "k", value: [2]int{2, 19}, err: logfmt.ErrUnsupportedValueType},
		{key: "k", value: []string{"e1", "e 2"}, err: logfmt.ErrUnsupportedValueType},
		{key: "k", value: structData{"a a", 9}, err: logfmt.ErrUnsupportedValueType},
		{key: "k", value: decimalMarshaler{5, 9}, want: "k=5.9"},
		{key: "k", value: (*decimalMarshaler)(nil), want: "k=null"},
		{key: "k", value: decimalStringer{5, 9}, want: "k=5.9"},
		{key: "k", value: (*decimalStringer)(nil), want: "k=null"},
		{key: "k", value: marshalerStringer{5, 9}, want: "k=5.9"},
		{key: "k", value: (*marshalerStringer)(nil), want: "k=null"},
		{key: "k", value: new(nilMarshaler), want: "k=notnilmarshaler"},
		{key: "k", value: (*nilMarshaler)(nil), want: "k=nilmarshaler"},
		{key: (*marshalerStringer)(nil), value: "v", err: logfmt.ErrNilKey},
		{key: decimalMarshaler{5, 9}, value: "v", want: "5.9=v"},
		{key: (*decimalMarshaler)(nil), value: "v", err: logfmt.ErrNilKey},
		{key: decimalStringer{5, 9}, value: "v", want: "5.9=v"},
		{key: (*decimalStringer)(nil), value: "v", err: logfmt.ErrNilKey},
		{key: marshalerStringer{5, 9}, value: "v", want: "5.9=v"},
		{key: "k", value: "\xbd", want: `k="\ufffd"`},
		{key: "k", value: "\ufffd\x00", want: `k="\ufffd\u0000"`},
		{key: "k", value: "\ufffd", want: `k="\ufffd"`},
		{key: "k", value: []byte("\ufffd\x00"), want: `k="\ufffd\u0000"`},
		{key: "k", value: []byte("\ufffd"), want: `k="\ufffd"`},
	}

	for _, d := range data {
		w := &bytes.Buffer{}
		enc := logfmt.NewEncoder(w)
		err := enc.EncodeKeyval(d.key, d.value)
		if err != d.err {
			t.Errorf("%#v, %#v: got error: %v, want error: %v", d.key, d.value, err, d.err)
		}
		if got, want := w.String(), d.want; got != want {
			t.Errorf("%#v, %#v: got '%s', want '%s'", d.key, d.value, got, want)
		}
	}
}

func TestMarshalKeyvals(t *testing.T) {
	one := 1
	ptr := &one
	nilPtr := (*int)(nil)

	data := []struct {
		in   []interface{}
		want []byte
		err  error
	}{
		{in: nil, want: nil},
		{in: kv(), want: nil},
		{in: kv(nil, "v"), err: logfmt.ErrNilKey},
		{in: kv(nilPtr, "v"), err: logfmt.ErrNilKey},
		{in: kv("\ufffd"), err: logfmt.ErrInvalidKey},
		{in: kv("\xbd"), err: logfmt.ErrInvalidKey},
		{in: kv("k"), want: []byte("k=null")},
		{in: kv("k", nil), want: []byte("k=null")},
		{in: kv("k", ""), want: []byte("k=")},
		{in: kv("k", "null"), want: []byte(`k="null"`)},
		{in: kv("k", "v"), want: []byte("k=v")},
		{in: kv("k", true), want: []byte("k=true")},
		{in: kv("k", 1), want: []byte("k=1")},
		{in: kv("k", ptr), want: []byte("k=1")},
		{in: kv("k", nilPtr), want: []byte("k=null")},
		{in: kv("k", 1.025), want: []byte("k=1.025")},
		{in: kv("k", 1e-3), want: []byte("k=0.001")},
		{in: kv("k", "v v"), want: []byte(`k="v v"`)},
		{in: kv("k", `"`), want: []byte(`k="\""`)},
		{in: kv("k", `=`), want: []byte(`k="="`)},
		{in: kv("k", `\`), want: []byte(`k=\`)},
		{in: kv("k", `=\`), want: []byte(`k="=\\"`)},
		{in: kv("k", `\"`), want: []byte(`k="\\\""`)},
		{in: kv("k1", "v1", "k2", "v2"), want: []byte("k1=v1 k2=v2")},
		{in: kv("k1", "v1", "k2", [2]int{}), want: []byte("k1=v1 k2=\"unsupported value type\"")},
		{in: kv([2]int{}, "v1", "k2", "v2"), want: []byte("k2=v2")},
		{in: kv("k", time.Date(2009, time.November, 10, 23, 0, 0, 0, time.UTC)), want: []byte("k=2009-11-10T23:00:00Z")},
		{in: kv("k", errorMarshaler{}), want: []byte("k=\"error marshaling value of type logfmt_test.errorMarshaler: marshal error\"")},
		{in: kv("k", decimalMarshaler{5, 9}), want: []byte("k=5.9")},
		{in: kv("k", (*decimalMarshaler)(nil)), want: []byte("k=null")},
		{in: kv("k", decimalStringer{5, 9}), want: []byte("k=5.9")},
		{in: kv("k", (*decimalStringer)(nil)), want: []byte("k=null")},
		{in: kv("k", marshalerStringer{5, 9}), want: []byte("k=5.9")},
		{in: kv("k", (*marshalerStringer)(nil)), want: []byte("k=null")},
		{in: kv(one, "v"), want: []byte("1=v")},
		{in: kv(ptr, "v"), want: []byte("1=v")},
		{in: kv((*marshalerStringer)(nil), "v"), err: logfmt.ErrNilKey},
		{in: kv(decimalMarshaler{5, 9}, "v"), want: []byte("5.9=v")},
		{in: kv((*decimalMarshaler)(nil), "v"), err: logfmt.ErrNilKey},
		{in: kv(decimalStringer{5, 9}, "v"), want: []byte("5.9=v")},
		{in: kv((*decimalStringer)(nil), "v"), err: logfmt.ErrNilKey},
		{in: kv(marshalerStringer{5, 9}, "v"), want: []byte("5.9=v")},
		{in: kv("k", panicingStringer{0}), want: []byte("k=ok")},
		{in: kv("k", panicingStringer{1}), want: []byte("k=PANIC:panic1")},
		// Need extra mechanism to test panic-while-printing-panicVal
		//{in: kv("k", panicingStringer{2}), want: []byte("?")},
	}

	for _, d := range data {
		got, err := logfmt.MarshalKeyvals(d.in...)
		if err != d.err {
			t.Errorf("%#v: got error: %v, want error: %v", d.in, err, d.err)
		}
		if !reflect.DeepEqual(got, d.want) {
			t.Errorf("%#v: got '%s', want '%s'", d.in, got, d.want)
		}
	}
}

func kv(keyvals ...interface{}) []interface{} {
	return keyvals
}

type structData struct {
	A string `logfmt:"fieldA"`
	B int
}

type nilMarshaler int

func (m *nilMarshaler) MarshalText() ([]byte, error) {
	if m == nil {
		return []byte("nilmarshaler"), nil
	}
	return []byte("notnilmarshaler"), nil
}

type decimalMarshaler struct {
	a, b int
}

func (t decimalMarshaler) MarshalText() ([]byte, error) {
	buf := &bytes.Buffer{}
	fmt.Fprintf(buf, "%d.%d", t.a, t.b)
	return buf.Bytes(), nil
}

type decimalStringer struct {
	a, b int
}

func (s decimalStringer) String() string {
	return fmt.Sprintf("%d.%d", s.a, s.b)
}

type marshalerStringer struct {
	a, b int
}

func (t marshalerStringer) MarshalText() ([]byte, error) {
	buf := &bytes.Buffer{}
	fmt.Fprintf(buf, "%d.%d", t.a, t.b)
	return buf.Bytes(), nil
}

func (t marshalerStringer) String() string {
	return fmt.Sprint(t.a + t.b)
}

var errMarshal = errors.New("marshal error")

type errorMarshaler struct{}

func (errorMarshaler) MarshalText() ([]byte, error) {
	return nil, errMarshal
}

type panicingStringer struct {
	a int
}

func (p panicingStringer) String() string {
	switch p.a {
	case 1:
		panic("panic1")
	case 2:
		panic(panicingStringer{a: 2})
	}
	return "ok"
}

func BenchmarkEncodeKeyval(b *testing.B) {
	b.ReportAllocs()
	enc := logfmt.NewEncoder(ioutil.Discard)
	for i := 0; i < b.N; i++ {
		enc.EncodeKeyval("sk", "10")
		enc.EncodeKeyval("some-key", "a rather long string with spaces")
	}
}
