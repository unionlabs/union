package handlers

import (
	"bufio"
	"bytes"
	"log"
	"net/http"
	"net/http/httptest"
	"net/url"
	"strings"
	"testing"
)

func TestCleanHost(t *testing.T) {
	tests := []struct {
		in, want string
	}{
		{"www.google.com", "www.google.com"},
		{"www.google.com foo", "www.google.com"},
		{"www.google.com/foo", "www.google.com"},
		{" first character is a space", ""},
	}
	for _, tt := range tests {
		got := cleanHost(tt.in)
		if tt.want != got {
			t.Errorf("cleanHost(%q) = %q, want %q", tt.in, got, tt.want)
		}
	}
}

func TestCanonicalHost(t *testing.T) {
	gorilla := "http://www.gorillatoolkit.org"

	rr := httptest.NewRecorder()
	r := newRequest(http.MethodGet, "http://www.example.com/")

	testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {})

	// Test a re-direct: should return a 302 Found.
	CanonicalHost(gorilla, http.StatusFound)(testHandler).ServeHTTP(rr, r)

	if rr.Code != http.StatusFound {
		t.Fatalf("bad status: got %v want %v", rr.Code, http.StatusFound)
	}

	if rr.Header().Get("Location") != gorilla+r.URL.Path {
		t.Fatalf("bad re-direct: got %q want %q", rr.Header().Get("Location"), gorilla+r.URL.Path)
	}
}

func TestKeepsQueryString(t *testing.T) {
	google := "https://www.google.com"

	rr := httptest.NewRecorder()
	querystring := url.Values{"q": {"golang"}, "format": {"json"}}.Encode()
	r := newRequest(http.MethodGet, "http://www.example.com/search?"+querystring)

	testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {})
	CanonicalHost(google, http.StatusFound)(testHandler).ServeHTTP(rr, r)

	want := google + r.URL.Path + "?" + querystring
	if rr.Header().Get("Location") != want {
		t.Fatalf("bad re-direct: got %q want %q", rr.Header().Get("Location"), want)
	}
}

func TestBadDomain(t *testing.T) {
	rr := httptest.NewRecorder()
	r := newRequest(http.MethodGet, "http://www.example.com/")

	testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {})

	// Test a bad domain - should return 200 OK.
	CanonicalHost("%", http.StatusFound)(testHandler).ServeHTTP(rr, r)

	if rr.Code != http.StatusOK {
		t.Fatalf("bad status: got %v want %v", rr.Code, http.StatusOK)
	}
}

func TestEmptyHost(t *testing.T) {
	rr := httptest.NewRecorder()
	r := newRequest(http.MethodGet, "http://www.example.com/")

	testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {})

	// Test a domain that returns an empty url.Host from url.Parse.
	CanonicalHost("hello.com", http.StatusFound)(testHandler).ServeHTTP(rr, r)

	if rr.Code != http.StatusOK {
		t.Fatalf("bad status: got %v want %v", rr.Code, http.StatusOK)
	}
}

func TestHeaderWrites(t *testing.T) {
	gorilla := "http://www.gorillatoolkit.org"

	testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	})

	// Catch the log output to ensure we don't write multiple headers.
	var b bytes.Buffer
	buf := bufio.NewWriter(&b)
	tl := log.New(buf, "test: ", log.Lshortfile)

	srv := httptest.NewServer(
		CanonicalHost(gorilla, http.StatusFound)(testHandler))
	defer srv.Close()
	srv.Config.ErrorLog = tl

	_, err := http.Get(srv.URL)
	if err != nil {
		t.Fatal(err)
	}

	err = buf.Flush()
	if err != nil {
		t.Fatal(err)
	}

	// We rely on the error not changing: net/http does not export it.
	if strings.Contains(b.String(), "multiple response.WriteHeader calls") {
		t.Fatalf("re-direct did not return early: multiple header writes")
	}
}
