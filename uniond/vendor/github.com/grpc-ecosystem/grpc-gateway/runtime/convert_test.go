package runtime_test

import (
	"testing"

	"github.com/golang/protobuf/proto"
	"github.com/golang/protobuf/ptypes/duration"
	"github.com/golang/protobuf/ptypes/timestamp"
	"github.com/grpc-ecosystem/grpc-gateway/runtime"
)

func TestConvertTimestamp(t *testing.T) {
	specs := []struct {
		name    string
		input   string
		output  *timestamp.Timestamp
		wanterr bool
	}{
		{
			name:  "a valid RFC3339 timestamp",
			input: `"2016-05-10T10:19:13.123Z"`,
			output: &timestamp.Timestamp{
				Seconds: 1462875553,
				Nanos:   123000000,
			},
			wanterr: false,
		},
		{
			name:    "invalid timestamp",
			input:   `"05-10-2016T10:19:13.123Z"`,
			output:  nil,
			wanterr: true,
		},
		{
			name:    "JSON number",
			input:   "123",
			output:  nil,
			wanterr: true,
		},
		{
			name:    "JSON bool",
			input:   "true",
			output:  nil,
			wanterr: true,
		},
	}

	for _, spec := range specs {
		t.Run(spec.name, func(t *testing.T) {
			ts, err := runtime.Timestamp(spec.input)
			switch {
			case err != nil && !spec.wanterr:
				t.Errorf("got unexpected error\n%#v", err)
			case err == nil && spec.wanterr:
				t.Errorf("did not error when expecte")
			case !proto.Equal(ts, spec.output):
				t.Errorf(
					"when testing %s; got\n%#v\nexpected\n%#v",
					spec.name,
					ts,
					spec.output,
				)
			}
		})
	}
}

func TestConvertDuration(t *testing.T) {
	specs := []struct {
		name    string
		input   string
		output  *duration.Duration
		wanterr bool
	}{
		{
			name:  "a valid duration",
			input: `"123.456s"`,
			output: &duration.Duration{
				Seconds: 123,
				Nanos:   456000000,
			},
			wanterr: false,
		},
		{
			name:    "invalid duration",
			input:   `"123years"`,
			output:  nil,
			wanterr: true,
		},
		{
			name:    "JSON number",
			input:   "123",
			output:  nil,
			wanterr: true,
		},
		{
			name:    "JSON bool",
			input:   "true",
			output:  nil,
			wanterr: true,
		},
	}

	for _, spec := range specs {
		t.Run(spec.name, func(t *testing.T) {
			ts, err := runtime.Duration(spec.input)
			switch {
			case err != nil && !spec.wanterr:
				t.Errorf("got unexpected error\n%#v", err)
			case err == nil && spec.wanterr:
				t.Errorf("did not error when expecte")
			case !proto.Equal(ts, spec.output):
				t.Errorf(
					"when testing %s; got\n%#v\nexpected\n%#v",
					spec.name,
					ts,
					spec.output,
				)
			}
		})
	}
}
