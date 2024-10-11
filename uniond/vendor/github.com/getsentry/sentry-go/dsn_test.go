package sentry

import (
	"encoding/json"
	"regexp"
	"strings"
	"testing"

	"github.com/google/go-cmp/cmp"
)

type DsnTest struct {
	in     string
	dsn    *Dsn   // expected value after parsing
	url    string // expected Store API URL
	envURL string // expected Envelope API URL
}

var dsnTests = map[string]DsnTest{
	"AllFields": {
		in: "https://public:secret@domain:8888/foo/bar/42",
		dsn: &Dsn{
			scheme:    schemeHTTPS,
			publicKey: "public",
			secretKey: "secret",
			host:      "domain",
			port:      8888,
			path:      "/foo/bar",
			projectID: "42",
		},
		url:    "https://domain:8888/foo/bar/api/42/store/",
		envURL: "https://domain:8888/foo/bar/api/42/envelope/",
	},
	"MinimalSecure": {
		in: "https://public@domain/42",
		dsn: &Dsn{
			scheme:    schemeHTTPS,
			publicKey: "public",
			host:      "domain",
			port:      443,
			projectID: "42",
		},
		url:    "https://domain/api/42/store/",
		envURL: "https://domain/api/42/envelope/",
	},
	"MinimalInsecure": {
		in: "http://public@domain/42",
		dsn: &Dsn{
			scheme:    schemeHTTP,
			publicKey: "public",
			host:      "domain",
			port:      80,
			projectID: "42",
		},
		url:    "http://domain/api/42/store/",
		envURL: "http://domain/api/42/envelope/",
	},
}

// nolint: scopelint // false positive https://github.com/kyoh86/scopelint/issues/4
func TestNewDsn(t *testing.T) {
	for name, tt := range dsnTests {
		t.Run(name, func(t *testing.T) {
			dsn, err := NewDsn(tt.in)
			if err != nil {
				t.Fatalf("NewDsn() error: %q", err)
			}
			// Internal fields
			if diff := cmp.Diff(tt.dsn, dsn, cmp.AllowUnexported(Dsn{})); diff != "" {
				t.Errorf("NewDsn() mismatch (-want +got):\n%s", diff)
			}
			url := dsn.GetAPIURL().String()
			if diff := cmp.Diff(tt.envURL, url); diff != "" {
				t.Errorf("dsn.EnvelopeAPIURL() mismatch (-want +got):\n%s", diff)
			}
		})
	}
}

type invalidDsnTest struct {
	in  string
	err string // expected substring of the error
}

var invalidDsnTests = map[string]invalidDsnTest{
	"Empty":     {"", "invalid scheme"},
	"NoScheme1": {"public:secret@:8888/42", "invalid scheme"},
	// FIXME: NoScheme2's error message is inconsistent with NoScheme1; consider
	// avoiding leaking errors from url.Parse.
	"NoScheme2":     {"://public:secret@:8888/42", "missing protocol scheme"},
	"NoPublicKey":   {"https://:secret@domain:8888/42", "empty username"},
	"NoHost":        {"https://public:secret@:8888/42", "empty host"},
	"NoProjectID1":  {"https://public:secret@domain:8888/", "empty project id"},
	"NoProjectID2":  {"https://public:secret@domain:8888", "empty project id"},
	"BadURL":        {"!@#$%^&*()", "invalid url"},
	"BadScheme":     {"ftp://public:secret@domain:8888/1", "invalid scheme"},
	"BadPort":       {"https://public:secret@domain:wat/42", "invalid port"},
	"TrailingSlash": {"https://public:secret@domain:8888/42/", "empty project id"},
}

// nolint: scopelint // false positive https://github.com/kyoh86/scopelint/issues/4
func TestNewDsnInvalidInput(t *testing.T) {
	for name, tt := range invalidDsnTests {
		t.Run(name, func(t *testing.T) {
			_, err := NewDsn(tt.in)
			if err == nil {
				t.Fatalf("got nil, want error with %q", tt.err)
			}
			if _, ok := err.(*DsnParseError); !ok {
				t.Errorf("got %T, want %T", err, (*DsnParseError)(nil))
			}
			if !strings.Contains(err.Error(), tt.err) {
				t.Errorf("%q does not contain %q", err.Error(), tt.err)
			}
		})
	}
}

func TestDsnSerializeDeserialize(t *testing.T) {
	url := "https://public:secret@domain:8888/foo/bar/42"
	dsn, dsnErr := NewDsn(url)
	serialized, _ := json.Marshal(dsn)
	var deserialized Dsn
	unmarshalErr := json.Unmarshal(serialized, &deserialized)

	if unmarshalErr != nil {
		t.Error("expected dsn unmarshal to not return error")
	}
	if dsnErr != nil {
		t.Error("expected NewDsn to not return error")
	}
	assertEqual(t, `"https://public:secret@domain:8888/foo/bar/42"`, string(serialized))
	assertEqual(t, url, deserialized.String())
}

func TestDsnDeserializeInvalidJSON(t *testing.T) {
	var invalidJSON Dsn
	invalidJSONErr := json.Unmarshal([]byte(`"whoops`), &invalidJSON)
	var invalidDsn Dsn
	invalidDsnErr := json.Unmarshal([]byte(`"http://wat"`), &invalidDsn)

	if invalidJSONErr == nil {
		t.Error("expected dsn unmarshal to return error")
	}
	if invalidDsnErr == nil {
		t.Error("expected dsn unmarshal to return error")
	}
}

func TestRequestHeadersWithoutSecretKey(t *testing.T) {
	url := "https://public@domain/42"
	dsn, err := NewDsn(url)
	if err != nil {
		t.Fatal(err)
	}
	headers := dsn.RequestHeaders()
	authRegexp := regexp.MustCompile("^Sentry sentry_version=7, sentry_timestamp=\\d+, " +
		"sentry_client=sentry.go/.+, sentry_key=public$")

	if len(headers) != 2 {
		t.Error("expected request to have 2 headers")
	}
	assertEqual(t, "application/json", headers["Content-Type"])
	if authRegexp.FindStringIndex(headers["X-Sentry-Auth"]) == nil {
		t.Error("expected auth header to fulfill provided pattern")
	}
}

func TestRequestHeadersWithSecretKey(t *testing.T) {
	url := "https://public:secret@domain/42"
	dsn, err := NewDsn(url)
	if err != nil {
		t.Fatal(err)
	}
	headers := dsn.RequestHeaders()
	authRegexp := regexp.MustCompile("^Sentry sentry_version=7, sentry_timestamp=\\d+, " +
		"sentry_client=sentry.go/.+, sentry_key=public, sentry_secret=secret$")

	if len(headers) != 2 {
		t.Error("expected request to have 2 headers")
	}
	assertEqual(t, "application/json", headers["Content-Type"])
	if authRegexp.FindStringIndex(headers["X-Sentry-Auth"]) == nil {
		t.Error("expected auth header to fulfill provided pattern")
	}
}

func TestGetScheme(t *testing.T) {
	tests := []struct {
		dsn  string
		want string
	}{
		{"http://public:secret@domain/42", "http"},
		{"https://public:secret@domain/42", "https"},
	}
	for _, tt := range tests {
		dsn, err := NewDsn(tt.dsn)
		if err != nil {
			t.Fatal(err)
		}
		assertEqual(t, dsn.GetScheme(), tt.want)
	}
}

func TestGetPublicKey(t *testing.T) {
	tests := []struct {
		dsn  string
		want string
	}{
		{"https://public:secret@domain/42", "public"},
	}
	for _, tt := range tests {
		dsn, err := NewDsn(tt.dsn)
		if err != nil {
			t.Fatal(err)
		}
		assertEqual(t, dsn.GetPublicKey(), tt.want)
	}
}

func TestGetSecretKey(t *testing.T) {
	tests := []struct {
		dsn  string
		want string
	}{
		{"https://public:secret@domain/42", "secret"},
		{"https://public@domain/42", ""},
	}
	for _, tt := range tests {
		dsn, err := NewDsn(tt.dsn)
		if err != nil {
			t.Fatal(err)
		}
		assertEqual(t, dsn.GetSecretKey(), tt.want)
	}
}

func TestGetHost(t *testing.T) {
	tests := []struct {
		dsn  string
		want string
	}{
		{"http://public:secret@domain/42", "domain"},
	}
	for _, tt := range tests {
		dsn, err := NewDsn(tt.dsn)
		if err != nil {
			t.Fatal(err)
		}
		assertEqual(t, dsn.GetHost(), tt.want)
	}
}

func TestGetPort(t *testing.T) {
	tests := []struct {
		dsn  string
		want int
	}{
		{"https://public:secret@domain/42", 443},
		{"http://public:secret@domain/42", 80},
		{"https://public:secret@domain:3000/42", 3000},
	}
	for _, tt := range tests {
		dsn, err := NewDsn(tt.dsn)
		if err != nil {
			t.Fatal(err)
		}
		assertEqual(t, dsn.GetPort(), tt.want)
	}
}

func TestGetPath(t *testing.T) {
	tests := []struct {
		dsn  string
		want string
	}{
		{"https://public:secret@domain/42", ""},
		{"https://public:secret@domain/foo/bar/42", "/foo/bar"},
	}
	for _, tt := range tests {
		dsn, err := NewDsn(tt.dsn)
		if err != nil {
			t.Fatal(err)
		}
		assertEqual(t, dsn.GetPath(), tt.want)
	}
}

func TestGetProjectID(t *testing.T) {
	tests := []struct {
		dsn  string
		want string
	}{
		{"https://public:secret@domain/42", "42"},
	}
	for _, tt := range tests {
		dsn, err := NewDsn(tt.dsn)
		if err != nil {
			t.Fatal(err)
		}
		assertEqual(t, dsn.GetProjectID(), tt.want)
	}
}
