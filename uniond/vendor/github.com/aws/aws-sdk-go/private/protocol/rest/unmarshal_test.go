//go:build go1.7
// +build go1.7

package rest

import (
	"bytes"
	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/request"
	"io/ioutil"
	"math"
	"net/http"
	"reflect"
	"testing"
)

func TestUnmarshalFloat64(t *testing.T) {
	cases := map[string]struct {
		Headers  http.Header
		OutputFn func() (interface{}, func(*testing.T, interface{}))
		WantErr  bool
	}{
		"header float values": {
			OutputFn: func() (interface{}, func(*testing.T, interface{})) {
				type output struct {
					Float       *float64 `location:"header" locationName:"x-amz-float"`
					FloatInf    *float64 `location:"header" locationName:"x-amz-float-inf"`
					FloatNegInf *float64 `location:"header" locationName:"x-amz-float-neg-inf"`
					FloatNaN    *float64 `location:"header" locationName:"x-amz-float-nan"`
				}

				return &output{}, func(t *testing.T, out interface{}) {
					o, ok := out.(*output)
					if !ok {
						t.Errorf("expect %T, got %T", (*output)(nil), out)
					}
					if e, a := aws.Float64(123456789.123), o.Float; !reflect.DeepEqual(e, a) {
						t.Errorf("expect %v, got %v", e, a)
					}
					if a := aws.Float64Value(o.FloatInf); !math.IsInf(a, 1) {
						t.Errorf("expect infinity, got %v", a)
					}
					if a := aws.Float64Value(o.FloatNegInf); !math.IsInf(a, -1) {
						t.Errorf("expect infinity, got %v", a)
					}
					if a := aws.Float64Value(o.FloatNaN); !math.IsNaN(a) {
						t.Errorf("expect infinity, got %v", a)
					}
				}
			},
			Headers: map[string][]string{
				"X-Amz-Float":         {"123456789.123"},
				"X-Amz-Float-Inf":     {"Infinity"},
				"X-Amz-Float-Neg-Inf": {"-Infinity"},
				"X-Amz-Float-Nan":     {"NaN"},
			},
		},
	}

	for name, tt := range cases {
		t.Run(name, func(t *testing.T) {
			output, expectFn := tt.OutputFn()

			req := &request.Request{
				Data: output,
				HTTPResponse: &http.Response{
					StatusCode: 200,
					Body:       ioutil.NopCloser(bytes.NewReader(nil)),
					Header:     tt.Headers,
				},
			}

			if (req.Error != nil) != tt.WantErr {
				t.Fatalf("WantErr(%v) != %v", tt.WantErr, req.Error)
			}

			if tt.WantErr {
				return
			}

			UnmarshalMeta(req)

			expectFn(t, output)
		})
	}
}
