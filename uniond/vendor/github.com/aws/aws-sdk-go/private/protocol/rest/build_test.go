//go:build go1.7
// +build go1.7

package rest

import (
	"math"
	"net/http"
	"net/url"
	"reflect"
	"testing"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/request"
)

func TestCleanPath(t *testing.T) {
	uri := &url.URL{
		Path:   "//foo//bar",
		Scheme: "https",
		Host:   "host",
	}
	cleanPath(uri)

	expected := "https://host/foo/bar"
	if a, e := uri.String(), expected; a != e {
		t.Errorf("expect %q URI, got %q", e, a)
	}
}

func TestMarshalPath(t *testing.T) {
	in := struct {
		Bucket *string `location:"uri" locationName:"bucket"`
		Key    *string `location:"uri" locationName:"key"`
	}{
		Bucket: aws.String("mybucket"),
		Key:    aws.String("my/cool+thing space/object世界"),
	}

	expectURL := `/mybucket/my/cool+thing space/object世界`
	expectEscapedURL := `/mybucket/my/cool%2Bthing%20space/object%E4%B8%96%E7%95%8C`

	req := &request.Request{
		HTTPRequest: &http.Request{
			URL: &url.URL{Scheme: "https", Host: "example.com", Path: "/{bucket}/{key+}"},
		},
		Params: &in,
	}

	Build(req)

	if req.Error != nil {
		t.Fatalf("unexpected error, %v", req.Error)
	}

	if a, e := req.HTTPRequest.URL.Path, expectURL; a != e {
		t.Errorf("expect %q URI, got %q", e, a)
	}

	if a, e := req.HTTPRequest.URL.RawPath, expectEscapedURL; a != e {
		t.Errorf("expect %q escaped URI, got %q", e, a)
	}

	if a, e := req.HTTPRequest.URL.EscapedPath(), expectEscapedURL; a != e {
		t.Errorf("expect %q escaped URI, got %q", e, a)
	}

}

func TestListOfEnums(t *testing.T) {
	cases := []struct {
		Input    interface{}
		Expected http.Header
		WantErr  bool
	}{
		{
			Input: func() interface{} {
				type v struct {
					List []*string `type:"list" location:"header" locationName:"x-amz-test-header"`
				}
				return &v{
					List: []*string{aws.String("foo")},
				}
			}(),
			WantErr: true,
		},
		{
			Input: func() interface{} {
				type v struct {
					List []*string `type:"list" location:"uri" locationName:"someList"`
				}
				return &v{
					List: []*string{aws.String("foo")},
				}
			}(),
			WantErr: true,
		},
		{
			Input: func() interface{} {
				type v struct {
					List map[string][]*string `type:"map" location:"headers" locationName:"x-amz-metadata"`
				}
				return &v{
					List: map[string][]*string{
						"foo": {aws.String("foo")},
					},
				}
			}(),
			WantErr: true,
		},
		{
			Input: func() interface{} {
				type v struct {
					List []*string `type:"list" location:"header" locationName:"x-amz-test-header" enum:"FooBar"`
				}
				return &v{
					List: []*string{aws.String("foo")},
				}
			}(),
			Expected: http.Header{
				"X-Amz-Test-Header": {"foo"},
			},
		},
		{
			Input: func() interface{} {
				type v struct {
					List []*string `type:"list" location:"header" locationName:"x-amz-test-header" enum:"FooBar"`
				}
				return &v{
					List: []*string{aws.String("foo"), nil, aws.String(""), nil, aws.String("bar")},
				}
			}(),
			Expected: http.Header{
				"X-Amz-Test-Header": {"foo,bar"},
			},
		},
		{
			Input: func() interface{} {
				type v struct {
					List []*string `type:"list" location:"header" locationName:"x-amz-test-header" enum:"FooBar"`
				}
				return &v{
					List: []*string{aws.String("f,o,o"), nil, aws.String(""), nil, aws.String(`"bar"`)},
				}
			}(),
			Expected: http.Header{
				"X-Amz-Test-Header": {`"f,o,o","\"bar\""`},
			},
		},
		{
			Input: func() interface{} {
				type v struct {
					List []*string `type:"list" location:"header" locationName:"x-amz-test-header" enum:"FooBar"`
				}
				return &v{
					List: nil,
				}
			}(),
			Expected: http.Header{},
		},
		{
			Input: func() interface{} {
				type v struct {
					List []*string `type:"list" location:"header" locationName:"x-amz-test-header" enum:"FooBar"`
				}
				return &v{
					List: []*string{},
				}
			}(),
			Expected: http.Header{},
		},
	}

	for i, tt := range cases {
		req := &request.Request{
			HTTPRequest: &http.Request{
				URL: func() *url.URL {
					u, err := url.Parse("https://foo.amazonaws.com")
					if err != nil {
						panic(err)
					}
					return u
				}(),
				Header: map[string][]string{},
			},
			Params: tt.Input,
		}

		Build(req)

		if (req.Error != nil) != (tt.WantErr) {
			t.Fatalf("(%d) WantErr(%t) got %v", i, tt.WantErr, req.Error)
		}

		if tt.WantErr {
			continue
		}

		if e, a := tt.Expected, req.HTTPRequest.Header; !reflect.DeepEqual(e, a) {
			t.Errorf("(%d) expect %v, got %v", i, e, a)
		}
	}
}

func TestMarshalFloat64(t *testing.T) {
	cases := map[string]struct {
		Input          interface{}
		URL            string
		ExpectedHeader http.Header
		ExpectedURL    string
		WantErr        bool
	}{
		"header float values": {
			Input: &struct {
				Float       *float64 `location:"header" locationName:"x-amz-float"`
				FloatInf    *float64 `location:"header" locationName:"x-amz-float-inf"`
				FloatNegInf *float64 `location:"header" locationName:"x-amz-float-neg-inf"`
				FloatNaN    *float64 `location:"header" locationName:"x-amz-float-nan"`
			}{
				Float:       aws.Float64(123456789.123),
				FloatInf:    aws.Float64(math.Inf(1)),
				FloatNegInf: aws.Float64(math.Inf(-1)),
				FloatNaN:    aws.Float64(math.NaN()),
			},
			URL: "https://example.com/",
			ExpectedHeader: map[string][]string{
				"X-Amz-Float":         {"123456789.123"},
				"X-Amz-Float-Inf":     {"Infinity"},
				"X-Amz-Float-Neg-Inf": {"-Infinity"},
				"X-Amz-Float-Nan":     {"NaN"},
			},
			ExpectedURL: "https://example.com/",
		},
		"path float values": {
			Input: &struct {
				Float       *float64 `location:"uri" locationName:"float"`
				FloatInf    *float64 `location:"uri" locationName:"floatInf"`
				FloatNegInf *float64 `location:"uri" locationName:"floatNegInf"`
				FloatNaN    *float64 `location:"uri" locationName:"floatNaN"`
			}{
				Float:       aws.Float64(123456789.123),
				FloatInf:    aws.Float64(math.Inf(1)),
				FloatNegInf: aws.Float64(math.Inf(-1)),
				FloatNaN:    aws.Float64(math.NaN()),
			},
			URL:            "https://example.com/{float}/{floatInf}/{floatNegInf}/{floatNaN}",
			ExpectedHeader: map[string][]string{},
			ExpectedURL:    "https://example.com/123456789.123/Infinity/-Infinity/NaN",
		},
		"query float values": {
			Input: &struct {
				Float       *float64 `location:"querystring" locationName:"x-amz-float"`
				FloatInf    *float64 `location:"querystring" locationName:"x-amz-float-inf"`
				FloatNegInf *float64 `location:"querystring" locationName:"x-amz-float-neg-inf"`
				FloatNaN    *float64 `location:"querystring" locationName:"x-amz-float-nan"`
			}{
				Float:       aws.Float64(123456789.123),
				FloatInf:    aws.Float64(math.Inf(1)),
				FloatNegInf: aws.Float64(math.Inf(-1)),
				FloatNaN:    aws.Float64(math.NaN()),
			},
			URL:            "https://example.com/",
			ExpectedHeader: map[string][]string{},
			ExpectedURL:    "https://example.com/?x-amz-float=123456789.123&x-amz-float-inf=Infinity&x-amz-float-nan=NaN&x-amz-float-neg-inf=-Infinity",
		},
	}

	for name, tt := range cases {
		t.Run(name, func(t *testing.T) {
			req := &request.Request{
				HTTPRequest: &http.Request{
					URL: func() *url.URL {
						u, err := url.Parse(tt.URL)
						if err != nil {
							panic(err)
						}
						return u
					}(),
					Header: map[string][]string{},
				},
				Params: tt.Input,
			}

			Build(req)

			if (req.Error != nil) != (tt.WantErr) {
				t.Fatalf("WantErr(%t) got %v", tt.WantErr, req.Error)
			}

			if tt.WantErr {
				return
			}

			if e, a := tt.ExpectedHeader, req.HTTPRequest.Header; !reflect.DeepEqual(e, a) {
				t.Errorf("expect %v, got %v", e, a)
			}

			if e, a := tt.ExpectedURL, req.HTTPRequest.URL.String(); e != a {
				t.Errorf("expect %v, got %v", e, a)
			}
		})
	}
}
