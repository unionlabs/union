package sentry

import (
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
)

var (
	update   = flag.Bool("update", false, "update .golden files")
	generate = flag.Bool("gen", false, "generate missing .golden files")
)

func TestUserIsEmpty(t *testing.T) {
	tests := []struct {
		input User
		want  bool
	}{
		{input: User{}, want: true},
		{input: User{ID: "foo"}, want: false},
		{input: User{Email: "foo@example.com"}, want: false},
		{input: User{IPAddress: "127.0.0.1"}, want: false},
		{input: User{Username: "My Username"}, want: false},
		{input: User{Name: "My Name"}, want: false},
		{input: User{Segment: "My Segment"}, want: false},
		{input: User{Data: map[string]string{"foo": "bar"}}, want: false},
		{input: User{ID: "foo", Email: "foo@example.com", IPAddress: "127.0.0.1", Username: "My Username", Name: "My Name", Segment: "My Segment", Data: map[string]string{"foo": "bar"}}, want: false},
	}

	for _, test := range tests {
		assertEqual(t, test.input.IsEmpty(), test.want)
	}
}

func TestUserMarshalJson(t *testing.T) {
	tests := []struct {
		input User
		want  string
	}{
		{input: User{}, want: `{}`},
		{input: User{ID: "foo"}, want: `{"id":"foo"}`},
		{input: User{Email: "foo@example.com"}, want: `{"email":"foo@example.com"}`},
		{input: User{IPAddress: "127.0.0.1"}, want: `{"ip_address":"127.0.0.1"}`},
		{input: User{Username: "My Username"}, want: `{"username":"My Username"}`},
		{input: User{Name: "My Name"}, want: `{"name":"My Name"}`},
		{input: User{Segment: "My Segment"}, want: `{"segment":"My Segment"}`},
		{input: User{Data: map[string]string{"foo": "bar"}}, want: `{"data":{"foo":"bar"}}`},
	}

	for _, test := range tests {
		got, err := json.Marshal(test.input)
		if err != nil {
			t.Fatal(err)
		}

		assertEqual(t, string(got), test.want)
	}
}

func TestNewRequest(t *testing.T) {
	currentHub.BindClient(&Client{
		options: ClientOptions{
			SendDefaultPII: true,
		},
	})
	// Unbind the client afterwards, to not affect other tests
	defer currentHub.stackTop().SetClient(nil)

	const payload = `{"test_data": true}`
	r := httptest.NewRequest("POST", "/test/?q=sentry", strings.NewReader(payload))
	r.Header.Add("Authorization", "Bearer 1234567890")
	r.Header.Add("Proxy-Authorization", "Bearer 123")
	r.Header.Add("Cookie", "foo=bar")
	r.Header.Add("X-Forwarded-For", "127.0.0.1")
	r.Header.Add("X-Real-Ip", "127.0.0.1")
	r.Header.Add("Some-Header", "some-header value")

	got := NewRequest(r)
	want := &Request{
		URL:         "http://example.com/test/",
		Method:      "POST",
		Data:        "",
		QueryString: "q=sentry",
		Cookies:     "foo=bar",
		Headers: map[string]string{
			"Authorization":       "Bearer 1234567890",
			"Proxy-Authorization": "Bearer 123",
			"Cookie":              "foo=bar",
			"Host":                "example.com",
			"X-Forwarded-For":     "127.0.0.1",
			"X-Real-Ip":           "127.0.0.1",
			"Some-Header":         "some-header value",
		},
		Env: map[string]string{
			"REMOTE_ADDR": "192.0.2.1",
			"REMOTE_PORT": "1234",
		},
	}
	if diff := cmp.Diff(want, got); diff != "" {
		t.Errorf("Request mismatch (-want +got):\n%s", diff)
	}
}

func TestNewRequestWithNoPII(t *testing.T) {
	const payload = `{"test_data": true}`
	r := httptest.NewRequest("POST", "/test/?q=sentry", strings.NewReader(payload))
	r.Header.Add("Authorization", "Bearer 1234567890")
	r.Header.Add("Proxy-Authorization", "Bearer 123")
	r.Header.Add("Cookie", "foo=bar")
	r.Header.Add("X-Forwarded-For", "127.0.0.1")
	r.Header.Add("X-Real-Ip", "127.0.0.1")
	r.Header.Add("Some-Header", "some-header value")

	got := NewRequest(r)
	want := &Request{
		URL:         "http://example.com/test/",
		Method:      "POST",
		Data:        "",
		QueryString: "q=sentry",
		Cookies:     "",
		Headers: map[string]string{
			"Host":        "example.com",
			"Some-Header": "some-header value",
		},
		Env: nil,
	}
	if diff := cmp.Diff(want, got); diff != "" {
		t.Errorf("Request mismatch (-want +got):\n%s", diff)
	}
}

func TestEventMarshalJSON(t *testing.T) {
	event := NewEvent()
	event.Spans = []*Span{{
		TraceID:      TraceIDFromHex("d6c4f03650bd47699ec65c84352b6208"),
		SpanID:       SpanIDFromHex("1cc4b26ab9094ef0"),
		ParentSpanID: SpanIDFromHex("442bd97bbe564317"),
		StartTime:    time.Unix(8, 0).UTC(),
		EndTime:      time.Unix(10, 0).UTC(),
		Status:       SpanStatusOK,
	}}
	event.StartTime = time.Unix(7, 0).UTC()
	event.Timestamp = time.Unix(14, 0).UTC()

	got, err := json.Marshal(event)
	if err != nil {
		t.Fatal(err)
	}

	// Non-transaction event should not have fields Spans and StartTime
	want := `{"sdk":{},"user":{},"timestamp":"1970-01-01T00:00:14Z"}`

	if diff := cmp.Diff(want, string(got)); diff != "" {
		t.Errorf("Event mismatch (-want +got):\n%s", diff)
	}
}

func TestEventWithDebugMetaMarshalJSON(t *testing.T) {
	event := NewEvent()
	event.DebugMeta = &DebugMeta{
		SdkInfo: &DebugMetaSdkInfo{
			SdkName:           "test",
			VersionMajor:      1,
			VersionMinor:      2,
			VersionPatchlevel: 3,
		},
		Images: []DebugMetaImage{
			{
				Type:        "macho",
				ImageAddr:   "0xabcd0000",
				ImageSize:   32768,
				DebugID:     "42DB5B96-5144-4079-BE09-45E2142CA3E5",
				DebugFile:   "foo.dSYM",
				CodeID:      "A7AF6477-9130-4EB7-ADFE-AD0F57001DBD",
				CodeFile:    "foo.dylib",
				ImageVmaddr: "0x0",
				Arch:        "arm64",
			},
			{
				Type: "proguard",
				UUID: "982E62D4-6493-4E43-864B-6523C79C7064",
			},
		},
	}

	got, err := json.Marshal(event)
	if err != nil {
		t.Fatal(err)
	}

	want := `{"sdk":{},"user":{},` +
		`"debug_meta":{` +
		`"sdk_info":{"sdk_name":"test","version_major":1,"version_minor":2,"version_patchlevel":3},` +
		`"images":[` +
		`{"type":"macho",` +
		`"image_addr":"0xabcd0000",` +
		`"image_size":32768,` +
		`"debug_id":"42DB5B96-5144-4079-BE09-45E2142CA3E5",` +
		`"debug_file":"foo.dSYM",` +
		`"code_id":"A7AF6477-9130-4EB7-ADFE-AD0F57001DBD",` +
		`"code_file":"foo.dylib",` +
		`"image_vmaddr":"0x0",` +
		`"arch":"arm64"` +
		`},` +
		`{"type":"proguard","uuid":"982E62D4-6493-4E43-864B-6523C79C7064"}` +
		`]}}`

	if diff := cmp.Diff(want, string(got)); diff != "" {
		t.Errorf("Event mismatch (-want +got):\n%s", diff)
	}
}

type withCause struct {
	msg   string
	cause error
}

func (w *withCause) Error() string { return w.msg }
func (w *withCause) Cause() error  { return w.cause }

type customError struct {
	message string
}

func (e *customError) Error() string {
	return e.message
}

func TestSetException(t *testing.T) {
	testCases := map[string]struct {
		exception     error
		maxErrorDepth int
		expected      []Exception
	}{
		"Single error without unwrap": {
			exception:     errors.New("simple error"),
			maxErrorDepth: 1,
			expected: []Exception{
				{
					Value:      "simple error",
					Type:       "*errors.errorString",
					Stacktrace: &Stacktrace{Frames: []Frame{}},
				},
			},
		},
		"Nested errors with Unwrap": {
			exception:     fmt.Errorf("level 2: %w", fmt.Errorf("level 1: %w", errors.New("base error"))),
			maxErrorDepth: 3,
			expected: []Exception{
				{
					Value: "base error",
					Type:  "*errors.errorString",
					Mechanism: &Mechanism{
						ExceptionID:      0,
						IsExceptionGroup: true,
					},
				},
				{
					Value: "level 1: base error",
					Type:  "*fmt.wrapError",
					Mechanism: &Mechanism{
						ExceptionID:      1,
						ParentID:         Pointer(0),
						IsExceptionGroup: true,
					},
				},
				{
					Value:      "level 2: level 1: base error",
					Type:       "*fmt.wrapError",
					Stacktrace: &Stacktrace{Frames: []Frame{}},
					Mechanism: &Mechanism{
						ExceptionID:      2,
						ParentID:         Pointer(1),
						IsExceptionGroup: true,
					},
				},
			},
		},
		"Custom error types": {
			exception: &customError{
				message: "custom error message",
			},
			maxErrorDepth: 1,
			expected: []Exception{
				{
					Value:      "custom error message",
					Type:       "*sentry.customError",
					Stacktrace: &Stacktrace{Frames: []Frame{}},
				},
			},
		},
		"Combination of Unwrap and Cause": {
			exception: fmt.Errorf("outer error: %w", &withCause{
				msg:   "error with cause",
				cause: errors.New("the cause"),
			}),
			maxErrorDepth: 3,
			expected: []Exception{
				{
					Value: "the cause",
					Type:  "*errors.errorString",
					Mechanism: &Mechanism{
						ExceptionID:      0,
						IsExceptionGroup: true,
					},
				},
				{
					Value: "error with cause",
					Type:  "*sentry.withCause",
					Mechanism: &Mechanism{
						ExceptionID:      1,
						ParentID:         Pointer(0),
						IsExceptionGroup: true,
					},
				},
				{
					Value:      "outer error: error with cause",
					Type:       "*fmt.wrapError",
					Stacktrace: &Stacktrace{Frames: []Frame{}},
					Mechanism: &Mechanism{
						ExceptionID:      2,
						ParentID:         Pointer(1),
						IsExceptionGroup: true,
					},
				},
			},
		},
	}

	for name, tc := range testCases {
		t.Run(name, func(t *testing.T) {
			e := &Event{}
			e.SetException(tc.exception, tc.maxErrorDepth)

			if len(e.Exception) != len(tc.expected) {
				t.Fatalf("Expected %d exceptions, got %d", len(tc.expected), len(e.Exception))
			}

			for i, exp := range tc.expected {
				if diff := cmp.Diff(exp, e.Exception[i]); diff != "" {
					t.Errorf("Event mismatch (-want +got):\n%s", diff)
				}
			}
		})
	}
}

func TestMechanismMarshalJSON(t *testing.T) {
	mechanism := &Mechanism{
		Type:        "some type",
		Description: "some description",
		HelpLink:    "some help link",
		Data: map[string]interface{}{
			"some data":         "some value",
			"some numeric data": 12345,
		},
	}

	got, err := json.Marshal(mechanism)
	if err != nil {
		t.Fatal(err)
	}

	want := `{"type":"some type","description":"some description","help_link":"some help link",` +
		`"exception_id":0,"data":{"some data":"some value","some numeric data":12345}}`

	if diff := cmp.Diff(want, string(got)); diff != "" {
		t.Errorf("Event mismatch (-want +got):\n%s", diff)
	}
}

func TestMechanismMarshalJSON_withHandled(t *testing.T) {
	mechanism := &Mechanism{
		Type:        "some type",
		Description: "some description",
		HelpLink:    "some help link",
		Data: map[string]interface{}{
			"some data":         "some value",
			"some numeric data": 12345,
		},
	}
	mechanism.SetUnhandled()

	got, err := json.Marshal(mechanism)
	if err != nil {
		t.Fatal(err)
	}

	want := `{"type":"some type","description":"some description","help_link":"some help link",` +
		`"handled":false,"exception_id":0,"data":{"some data":"some value","some numeric data":12345}}`

	if diff := cmp.Diff(want, string(got)); diff != "" {
		t.Errorf("Event mismatch (-want +got):\n%s", diff)
	}
}

func TestStructSnapshots(t *testing.T) {
	testSpan := &Span{
		TraceID:      TraceIDFromHex("d6c4f03650bd47699ec65c84352b6208"),
		SpanID:       SpanIDFromHex("1cc4b26ab9094ef0"),
		ParentSpanID: SpanIDFromHex("442bd97bbe564317"),
		Description:  `SELECT * FROM user WHERE "user"."id" = {id}`,
		Op:           "db.sql",
		Tags: map[string]string{
			"function_name":  "get_users",
			"status_message": "MYSQL OK",
		},
		StartTime: time.Unix(0, 0).UTC(),
		EndTime:   time.Unix(5, 0).UTC(),
		Status:    SpanStatusOK,
		Data: map[string]interface{}{
			"related_ids":  []uint{12312342, 76572, 4123485},
			"aws_instance": "ca-central-1",
		},
	}

	testCases := []struct {
		testName     string
		sentryStruct interface{}
	}{
		{
			testName:     "span",
			sentryStruct: testSpan,
		},
		{
			testName: "error_event",
			sentryStruct: &Event{
				Message:     "event message",
				Environment: "production",
				EventID:     EventID("0123456789abcdef"),
				Fingerprint: []string{"abcd"},
				Level:       LevelError,
				Platform:    "myplatform",
				Release:     "myrelease",
				Sdk: SdkInfo{
					Name:         "sentry.go",
					Version:      "0.0.1",
					Integrations: []string{"gin", "iris"},
					Packages: []SdkPackage{{
						Name:    "sentry-go",
						Version: "0.0.1",
					}},
				},
				ServerName:  "myhost",
				Timestamp:   time.Unix(5, 0).UTC(),
				Transaction: "mytransaction",
				User:        User{ID: "foo"},
				Breadcrumbs: []*Breadcrumb{{
					Data: map[string]interface{}{
						"data_key": "data_val",
					},
				}},
				Extra: map[string]interface{}{
					"extra_key": "extra_val",
				},
				Contexts: map[string]Context{
					"context_key": {
						"context_key": "context_val",
					},
				},
			},
		},
		{
			testName: "transaction_event",
			sentryStruct: &Event{
				Type:      transactionType,
				Spans:     []*Span{testSpan},
				StartTime: time.Unix(3, 0).UTC(),
				Timestamp: time.Unix(5, 0).UTC(),
				Contexts: map[string]Context{
					"trace": TraceContext{
						TraceID:     TraceIDFromHex("90d57511038845dcb4164a70fc3a7fdb"),
						SpanID:      SpanIDFromHex("f7f3fd754a9040eb"),
						Op:          "http.GET",
						Description: "description",
						Status:      SpanStatusOK,
					}.Map(),
				},
			},
		},
	}

	for _, test := range testCases {
		test := test
		t.Run(test.testName, func(t *testing.T) {
			got, err := json.MarshalIndent(test.sentryStruct, "", "    ")
			if err != nil {
				t.Error(err)
			}

			golden := filepath.Join(".", "testdata", fmt.Sprintf("%s.golden", test.testName))
			if *update {
				err := os.WriteFile(golden, got, 0600)
				if err != nil {
					t.Fatal(err)
				}
			}

			want, err := os.ReadFile(golden)
			if err != nil {
				t.Fatal(err)
			}

			if diff := cmp.Diff(want, got); diff != "" {
				t.Errorf("struct %s mismatch (-want +got):\n%s", test.testName, diff)
			}
		})
	}
}

func TestMarshalMetrics(t *testing.T) {
	tests := []struct {
		name    string
		metrics []Metric
		want    string
	}{
		{
			name: "allowed characters",
			metrics: []Metric{
				NewCounterMetric("counter", Second(), map[string]string{"foo": "bar", "route": "GET /foo"}, 1597790835, 1.0),
				NewDistributionMetric("distribution", Second(), map[string]string{"$foo$": "%bar%"}, 1597790835, 1.0),
				NewGaugeMetric("gauge", Second(), map[string]string{"föö": "bär"}, 1597790835, 1.0),
				NewSetMetric[int]("set", Second(), map[string]string{"%{key}": "$value$"}, 1597790835, 1),
				NewCounterMetric("no_tags", Second(), nil, 1597790835, 1.0),
			},

			want: strings.Join([]string{
				"counter@second:1|c|#foo:bar,route:GET /foo|T1597790835",
				"distribution@second:1|d|#_foo_:bar|T1597790835",
				"gauge@second:1:1:1:1:1|g|#f_:br|T1597790835",
				"set@second:1|s|#_key_:$value$|T1597790835",
				"no_tags@second:1|c|T1597790835",
			}, "\n"),
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			serializedMetric := marshalMetrics(test.metrics)
			if diff := cmp.Diff(string(serializedMetric), test.want); diff != "" {
				t.Errorf("Context mismatch (-want +got):\n%s", diff)
			}
		})
	}
}
