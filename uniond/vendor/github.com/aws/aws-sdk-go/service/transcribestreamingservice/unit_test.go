//go:build go1.10
// +build go1.10

package transcribestreamingservice

import (
	"bytes"
	"io/ioutil"
	"net/http"
	"strings"
	"testing"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/credentials"
	"github.com/aws/aws-sdk-go/aws/session"
)

type roundTripFunc func(req *http.Request) *http.Response

func (f roundTripFunc) RoundTrip(req *http.Request) (*http.Response, error) {
	return f(req), nil
}

func newTestClient(fn roundTripFunc) *http.Client {
	return &http.Client{
		Transport: fn,
	}
}

func TestStartStreamTranscription_Error(t *testing.T) {
	cfg := &aws.Config{
		Region:      aws.String("us-west-2"),
		Credentials: credentials.AnonymousCredentials,
		HTTPClient: newTestClient(func(req *http.Request) *http.Response {
			return &http.Response{
				StatusCode: http.StatusBadRequest,
				Body:       ioutil.NopCloser(bytes.NewReader([]byte("{ \"code\" : \"BadRequestException\" }"))),
				Header:     http.Header{},
			}
		}),
	}
	sess, err := session.NewSession(cfg)

	svc := New(sess)
	resp, err := svc.StartStreamTranscription(&StartStreamTranscriptionInput{
		LanguageCode:         aws.String(LanguageCodeEnUs),
		MediaEncoding:        aws.String(MediaEncodingPcm),
		MediaSampleRateHertz: aws.Int64(int64(16000)),
	})
	if err == nil {
		t.Fatalf("expect error, got none")
	} else {
		if e, a := "BadRequestException", err.Error(); !strings.Contains(a, e) {
			t.Fatalf("expected error to be %v, got %v", e, a)
		}
	}

	n, err := resp.GetStream().inputWriter.Write([]byte("text"))
	if err == nil {
		t.Fatalf("expected error stating write on closed pipe, got none")
	}

	if e, a := "write on closed pipe", err.Error(); !strings.Contains(a, e) {
		t.Fatalf("expected error to contain %v, got error as %v", e, a)
	}

	if e, a := 0, n; e != a {
		t.Fatalf("expected %d bytes to be written on inputWriter, but %v bytes were written", e, a)
	}
}
