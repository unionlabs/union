package http_test

import (
	"context"
	"net/http/httptest"
	"testing"

	httptransport "github.com/go-kit/kit/transport/http"
)

func TestSetHeader(t *testing.T) {
	const (
		key = "X-Foo"
		val = "12345"
	)
	r := httptest.NewRecorder()
	httptransport.SetResponseHeader(key, val)(context.Background(), r)
	if want, have := val, r.Header().Get(key); want != have {
		t.Errorf("want %q, have %q", want, have)
	}
}

func TestSetContentType(t *testing.T) {
	const contentType = "application/json"
	r := httptest.NewRecorder()
	httptransport.SetContentType(contentType)(context.Background(), r)
	if want, have := contentType, r.Header().Get("Content-Type"); want != have {
		t.Errorf("want %q, have %q", want, have)
	}
}
