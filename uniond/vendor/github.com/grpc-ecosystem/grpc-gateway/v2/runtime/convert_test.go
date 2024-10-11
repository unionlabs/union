package runtime_test

import (
	"testing"

	"github.com/grpc-ecosystem/grpc-gateway/v2/runtime"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/durationpb"
	"google.golang.org/protobuf/types/known/timestamppb"
)

func TestConvertTimestamp(t *testing.T) {
	specs := []struct {
		name    string
		input   string
		output  *timestamppb.Timestamp
		wanterr bool
	}{
		{
			name:  "a valid RFC3339 timestamp",
			input: `"2016-05-10T10:19:13.123Z"`,
			output: &timestamppb.Timestamp{
				Seconds: 1462875553,
				Nanos:   123000000,
			},
			wanterr: false,
		},
		{
			name:  "a valid RFC3339 timestamp without double quotation",
			input: "2016-05-10T10:19:13.123Z",
			output: &timestamppb.Timestamp{
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
		output  *durationpb.Duration
		wanterr bool
	}{
		{
			name:  "a valid duration",
			input: `"123.456s"`,
			output: &durationpb.Duration{
				Seconds: 123,
				Nanos:   456000000,
			},
			wanterr: false,
		},
		{
			name:  "a valid duration without double quotation",
			input: "123.456s",
			output: &durationpb.Duration{
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
