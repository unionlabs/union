//go:build go1.8
// +build go1.8

package request_test

import (
	"bytes"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/request"
	"github.com/aws/aws-sdk-go/awstesting"
	"github.com/aws/aws-sdk-go/awstesting/unit"
)

func TestResetBody_WithEmptyBody(t *testing.T) {
	r := request.Request{
		HTTPRequest: &http.Request{},
	}

	reader := strings.NewReader("")
	r.Body = reader

	r.ResetBody()

	if a, e := r.HTTPRequest.Body, http.NoBody; a != e {
		t.Errorf("expected request body to be set to reader, got %#v",
			r.HTTPRequest.Body)
	}
}

func TestRequest_FollowPUTRedirects(t *testing.T) {
	const bodySize = 1024

	redirectHit := 0
	endpointHit := 0

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		switch r.URL.Path {
		case "/redirect-me":
			u := *r.URL
			u.Path = "/endpoint"
			w.Header().Set("Location", u.String())
			w.WriteHeader(307)
			redirectHit++
		case "/endpoint":
			b := bytes.Buffer{}
			io.Copy(&b, r.Body)
			r.Body.Close()
			if e, a := bodySize, b.Len(); e != a {
				t.Fatalf("expect %d body size, got %d", e, a)
			}
			endpointHit++
		default:
			t.Fatalf("unexpected endpoint used, %q", r.URL.String())
		}
	}))
	defer server.Close()

	svc := awstesting.NewClient(&aws.Config{
		Region:     unit.Session.Config.Region,
		DisableSSL: aws.Bool(true),
		Endpoint:   aws.String(server.URL),
	})

	req := svc.NewRequest(&request.Operation{
		Name:       "Operation",
		HTTPMethod: "PUT",
		HTTPPath:   "/redirect-me",
	}, &struct{}{}, &struct{}{})
	req.SetReaderBody(bytes.NewReader(make([]byte, bodySize)))

	err := req.Send()
	if err != nil {
		t.Errorf("expect no error, got %v", err)
	}
	if e, a := 1, redirectHit; e != a {
		t.Errorf("expect %d redirect hits, got %d", e, a)
	}
	if e, a := 1, endpointHit; e != a {
		t.Errorf("expect %d endpoint hits, got %d", e, a)
	}
}

func TestNewRequest_JoinEndpointWithOperationPathQuery(t *testing.T) {
	cases := map[string]struct {
		HTTPPath    string
		Endpoint    *string
		ExpectQuery string
		ExpectPath  string
	}{
		"no op HTTP Path": {
			HTTPPath:    "",
			Endpoint:    aws.String("https://foo.bar.aws/foo?bar=Baz"),
			ExpectPath:  "/foo",
			ExpectQuery: "bar=Baz",
		},
		"no trailing slash": {
			HTTPPath:    "/",
			Endpoint:    aws.String("https://foo.bar.aws"),
			ExpectPath:  "/",
			ExpectQuery: "",
		},
		"set query": {
			HTTPPath:    "/?Foo=bar",
			Endpoint:    aws.String("https://foo.bar.aws"),
			ExpectPath:  "/",
			ExpectQuery: "Foo=bar",
		},
		"squash query": {
			HTTPPath:    "/?Foo=bar",
			Endpoint:    aws.String("https://foo.bar.aws/?bar=Foo"),
			ExpectPath:  "/",
			ExpectQuery: "Foo=bar",
		},
		"trailing slash": {
			HTTPPath:    "/",
			Endpoint:    aws.String("https://foo.bar.aws/"),
			ExpectPath:  "/",
			ExpectQuery: "",
		},
		"trailing slash set query": {
			HTTPPath:    "/?Foo=bar",
			Endpoint:    aws.String("https://foo.bar.aws/"),
			ExpectPath:  "/",
			ExpectQuery: "Foo=bar",
		},
	}

	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			client := awstesting.NewClient(&aws.Config{
				Endpoint: c.Endpoint,
			})

			client.Handlers.Clear()
			r := client.NewRequest(&request.Operation{
				Name:       "FooBar",
				HTTPMethod: "GET",
				HTTPPath:   c.HTTPPath,
			}, nil, nil)

			if e, a := c.ExpectPath, r.HTTPRequest.URL.Path; e != a {
				t.Errorf("expect %v path, got %v", e, a)
			}
			if e, a := c.ExpectQuery, r.HTTPRequest.URL.RawQuery; e != a {
				t.Errorf("expect %v query, got %v", e, a)
			}
		})
	}
}
