package logfmt

import (
	"bytes"
	"errors"
	"fmt"
	"io/ioutil"
	"reflect"
	"testing"
)

func TestSafeString(t *testing.T) {
	_, ok := safeString((*stringStringer)(nil))
	if got, want := ok, false; got != want {
		t.Errorf(" got %v, want %v", got, want)
	}
}

func TestSafeMarshal(t *testing.T) {
	kb, err := safeMarshal((*stringMarshaler)(nil))
	if got := kb; got != nil {
		t.Errorf(" got %v, want nil", got)
	}
	if got, want := err, error(nil); got != want {
		t.Errorf(" got %v, want %v", got, want)
	}
}

func TestWriteKeyStrings(t *testing.T) {
	keygen := []struct {
		name string
		fn   func(string) interface{}
	}{
		{
			name: "string",
			fn:   func(s string) interface{} { return s },
		},
		{
			name: "named-string",
			fn:   func(s string) interface{} { return stringData(s) },
		},
		{
			name: "Stringer",
			fn:   func(s string) interface{} { return stringStringer(s) },
		},
		{
			name: "TextMarshaler",
			fn:   func(s string) interface{} { return stringMarshaler(s) },
		},
	}

	data := []struct {
		key  string
		want string
		err  error
	}{
		{key: "k", want: "k"},
		{key: `\`, want: `\`},
		{key: "\n", err: ErrInvalidKey},
		{key: "\x00", err: ErrInvalidKey},
		{key: "\x10", err: ErrInvalidKey},
		{key: "\x1F", err: ErrInvalidKey},
		{key: "", err: ErrInvalidKey},
		{key: " ", err: ErrInvalidKey},
		{key: "=", err: ErrInvalidKey},
		{key: `"`, err: ErrInvalidKey},
		{key: "k\n", want: "k"},
		{key: "k\nk", want: "kk"},
		{key: "k\tk", want: "kk"},
		{key: "k=k", want: "kk"},
		{key: `"kk"`, want: "kk"},
	}

	for _, g := range keygen {
		t.Run(g.name, func(t *testing.T) {
			for _, d := range data {
				w := &bytes.Buffer{}
				key := g.fn(d.key)
				err := writeKey(w, key)
				if err != d.err {
					t.Errorf("%#v: got error: %v, want error: %v", key, err, d.err)
				}
				if err != nil {
					continue
				}
				if got, want := w.String(), d.want; got != want {
					t.Errorf("%#v: got '%s', want '%s'", key, got, want)
				}
			}
		})
	}
}

func TestWriteKey(t *testing.T) {
	var (
		nilPtr *int
		one    = 1
		ptr    = &one
	)

	data := []struct {
		key  interface{}
		want string
		err  error
	}{
		{key: nil, err: ErrNilKey},
		{key: nilPtr, err: ErrNilKey},
		{key: (*stringStringer)(nil), err: ErrNilKey},
		{key: (*stringMarshaler)(nil), err: ErrNilKey},
		{key: (*stringerMarshaler)(nil), err: ErrNilKey},
		{key: ptr, want: "1"},

		{key: errorMarshaler{}, err: &MarshalerError{Type: reflect.TypeOf(errorMarshaler{}), Err: errMarshaling}},
		{key: make(chan int), err: ErrUnsupportedKeyType},
		{key: []int{}, err: ErrUnsupportedKeyType},
		{key: map[int]int{}, err: ErrUnsupportedKeyType},
		{key: [2]int{}, err: ErrUnsupportedKeyType},
		{key: struct{}{}, err: ErrUnsupportedKeyType},
		{key: fmt.Sprint, err: ErrUnsupportedKeyType},
	}

	for _, d := range data {
		w := &bytes.Buffer{}
		err := writeKey(w, d.key)
		if !reflect.DeepEqual(err, d.err) {
			t.Errorf("%#v: got error: %v, want error: %v", d.key, err, d.err)
		}
		if err != nil {
			continue
		}
		if got, want := w.String(), d.want; got != want {
			t.Errorf("%#v: got '%s', want '%s'", d.key, got, want)
		}
	}
}

func TestWriteValueStrings(t *testing.T) {
	keygen := []func(string) interface{}{
		func(s string) interface{} { return s },
		func(s string) interface{} { return errors.New(s) },
		func(s string) interface{} { return stringData(s) },
		func(s string) interface{} { return stringStringer(s) },
		func(s string) interface{} { return stringMarshaler(s) },
	}

	data := []struct {
		value string
		want  string
		err   error
	}{
		{value: "", want: ""},
		{value: "v", want: "v"},
		{value: " ", want: `" "`},
		{value: "=", want: `"="`},
		{value: `\`, want: `\`},
		{value: `"`, want: `"\""`},
		{value: `\"`, want: `"\\\""`},
		{value: "\n", want: `"\n"`},
		{value: "\x00", want: `"\u0000"`},
		{value: "\x10", want: `"\u0010"`},
		{value: "\x1F", want: `"\u001f"`},
		{value: "µ", want: `µ`},
	}

	for _, g := range keygen {
		for _, d := range data {
			w := &bytes.Buffer{}
			value := g(d.value)
			err := writeValue(w, value)
			if err != d.err {
				t.Errorf("%#v (%[1]T): got error: %v, want error: %v", value, err, d.err)
			}
			if err != nil {
				continue
			}
			if got, want := w.String(), d.want; got != want {
				t.Errorf("%#v (%[1]T): got '%s', want '%s'", value, got, want)
			}
		}
	}
}

func TestWriteValue(t *testing.T) {
	var (
		nilPtr *int
		one    = 1
		ptr    = &one
	)

	data := []struct {
		value interface{}
		want  string
		err   error
	}{
		{value: nil, want: "null"},
		{value: nilPtr, want: "null"},
		{value: (*stringStringer)(nil), want: "null"},
		{value: (*stringMarshaler)(nil), want: "null"},
		{value: (*stringerMarshaler)(nil), want: "null"},
		{value: ptr, want: "1"},

		{value: errorMarshaler{}, err: &MarshalerError{Type: reflect.TypeOf(errorMarshaler{}), Err: errMarshaling}},
		{value: make(chan int), err: ErrUnsupportedValueType},
		{value: []int{}, err: ErrUnsupportedValueType},
		{value: map[int]int{}, err: ErrUnsupportedValueType},
		{value: [2]int{}, err: ErrUnsupportedValueType},
		{value: struct{}{}, err: ErrUnsupportedValueType},
		{value: fmt.Sprint, err: ErrUnsupportedValueType},
	}

	for _, d := range data {
		w := &bytes.Buffer{}
		err := writeValue(w, d.value)
		if !reflect.DeepEqual(err, d.err) {
			t.Errorf("%#v: got error: %v, want error: %v", d.value, err, d.err)
		}
		if err != nil {
			continue
		}
		if got, want := w.String(), d.want; got != want {
			t.Errorf("%#v: got '%s', want '%s'", d.value, got, want)
		}
	}
}

type stringData string

type stringStringer string

func (s stringStringer) String() string {
	return string(s)
}

type stringMarshaler string

func (s stringMarshaler) MarshalText() ([]byte, error) {
	return []byte(s), nil
}

type stringerMarshaler string

func (s stringerMarshaler) String() string {
	return "String() called"
}

func (s stringerMarshaler) MarshalText() ([]byte, error) {
	return []byte(s), nil
}

var errMarshaling = errors.New("marshal error")

type errorMarshaler struct{}

func (errorMarshaler) MarshalText() ([]byte, error) {
	return nil, errMarshaling
}

func BenchmarkWriteStringKey(b *testing.B) {
	keys := []string{
		"k",
		"caller",
		"has space",
		`"quoted"`,
	}

	for _, k := range keys {
		b.Run(k, func(b *testing.B) {
			for i := 0; i < b.N; i++ {
				writeStringKey(ioutil.Discard, k)
			}
		})
	}
}
