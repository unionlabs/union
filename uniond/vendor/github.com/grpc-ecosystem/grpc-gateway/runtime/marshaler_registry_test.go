package runtime_test

import (
	"errors"
	"fmt"
	"io"
	"net/http"
	"testing"

	"github.com/grpc-ecosystem/grpc-gateway/runtime"
)

func TestMarshalerForRequest(t *testing.T) {
	r, err := http.NewRequest("GET", "http://example.com", nil)
	if err != nil {
		t.Fatalf(`http.NewRequest("GET", "http://example.com", nil) failed with %v; want success`, err)
	}

	mux := runtime.NewServeMux()

	r.Header.Set("Accept", "application/x-out")
	r.Header.Set("Content-Type", "application/x-in")
	in, out := runtime.MarshalerForRequest(mux, r)
	if _, ok := in.(*runtime.JSONPb); !ok {
		t.Errorf("in = %#v; want a runtime.JSONPb", in)
	}
	if _, ok := out.(*runtime.JSONPb); !ok {
		t.Errorf("out = %#v; want a runtime.JSONPb", in)
	}

	marshalers := []dummyMarshaler{0, 1, 2}
	specs := []struct {
		opt runtime.ServeMuxOption

		wantIn  runtime.Marshaler
		wantOut runtime.Marshaler
	}{
		// The option with wildcard overwrites the default configuration
		{
			opt:     runtime.WithMarshalerOption(runtime.MIMEWildcard, &marshalers[0]),
			wantIn:  &marshalers[0],
			wantOut: &marshalers[0],
		},
		// You can specify a marshaler for a specific MIME type.
		// The output marshaler follows the input one unless specified.
		{
			opt:     runtime.WithMarshalerOption("application/x-in", &marshalers[1]),
			wantIn:  &marshalers[1],
			wantOut: &marshalers[1],
		},
		// You can also separately specify an output marshaler
		{
			opt:     runtime.WithMarshalerOption("application/x-out", &marshalers[2]),
			wantIn:  &marshalers[1],
			wantOut: &marshalers[2],
		},
	}
	for i, spec := range specs {
		var opts []runtime.ServeMuxOption
		for _, s := range specs[:i+1] {
			opts = append(opts, s.opt)
		}
		mux = runtime.NewServeMux(opts...)

		in, out = runtime.MarshalerForRequest(mux, r)
		if got, want := in, spec.wantIn; got != want {
			t.Errorf("in = %#v; want %#v", got, want)
		}
		if got, want := out, spec.wantOut; got != want {
			t.Errorf("out = %#v; want %#v", got, want)
		}
	}

	r.Header.Set("Content-Type", "application/x-in; charset=UTF-8")
	in, out = runtime.MarshalerForRequest(mux, r)
	if got, want := in, &marshalers[1]; got != want {
		t.Errorf("in = %#v; want %#v", got, want)
	}
	if got, want := out, &marshalers[2]; got != want {
		t.Errorf("out = %#v; want %#v", got, want)
	}

	r.Header.Set("Content-Type", "application/x-another")
	r.Header.Set("Accept", "application/x-another")
	in, out = runtime.MarshalerForRequest(mux, r)
	if got, want := in, &marshalers[0]; got != want {
		t.Errorf("in = %#v; want %#v", got, want)
	}
	if got, want := out, &marshalers[0]; got != want {
		t.Errorf("out = %#v; want %#v", got, want)
	}
}

type dummyMarshaler int

func (dummyMarshaler) ContentType() string { return "" }
func (dummyMarshaler) Marshal(interface{}) ([]byte, error) {
	return nil, errors.New("not implemented")
}

func (dummyMarshaler) Unmarshal([]byte, interface{}) error {
	return errors.New("not implemented")
}

func (dummyMarshaler) NewDecoder(r io.Reader) runtime.Decoder {
	return dummyDecoder{}
}
func (dummyMarshaler) NewEncoder(w io.Writer) runtime.Encoder {
	return dummyEncoder{}
}

func (m dummyMarshaler) GoString() string {
	return fmt.Sprintf("dummyMarshaler(%d)", m)
}

type dummyDecoder struct{}

func (dummyDecoder) Decode(interface{}) error {
	return errors.New("not implemented")
}

type dummyEncoder struct{}

func (dummyEncoder) Encode(interface{}) error {
	return errors.New("not implemented")
}
