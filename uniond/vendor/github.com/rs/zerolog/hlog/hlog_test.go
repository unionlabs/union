//go:build go1.7
// +build go1.7

package hlog

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"net/url"
	"reflect"
	"testing"

	"github.com/rs/xid"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/internal/cbor"
)

func decodeIfBinary(out *bytes.Buffer) string {
	p := out.Bytes()
	if len(p) == 0 || p[0] < 0x7F {
		return out.String()
	}
	return cbor.DecodeObjectToStr(p) + "\n"
}

func TestNewHandler(t *testing.T) {
	log := zerolog.New(nil).With().
		Str("foo", "bar").
		Logger()
	lh := NewHandler(log)
	h := lh(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		l := FromRequest(r)
		if !reflect.DeepEqual(*l, log) {
			t.Fail()
		}
	}))
	h.ServeHTTP(nil, &http.Request{})
}

func TestURLHandler(t *testing.T) {
	out := &bytes.Buffer{}
	r := &http.Request{
		URL: &url.URL{Path: "/path", RawQuery: "foo=bar"},
	}
	h := URLHandler("url")(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		l := FromRequest(r)
		l.Log().Msg("")
	}))
	h = NewHandler(zerolog.New(out))(h)
	h.ServeHTTP(nil, r)
	if want, got := `{"url":"/path?foo=bar"}`+"\n", decodeIfBinary(out); want != got {
		t.Errorf("Invalid log output, got: %s, want: %s", got, want)
	}
}

func TestMethodHandler(t *testing.T) {
	out := &bytes.Buffer{}
	r := &http.Request{
		Method: "POST",
	}
	h := MethodHandler("method")(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		l := FromRequest(r)
		l.Log().Msg("")
	}))
	h = NewHandler(zerolog.New(out))(h)
	h.ServeHTTP(nil, r)
	if want, got := `{"method":"POST"}`+"\n", decodeIfBinary(out); want != got {
		t.Errorf("Invalid log output, got: %s, want: %s", got, want)
	}
}

func TestRequestHandler(t *testing.T) {
	out := &bytes.Buffer{}
	r := &http.Request{
		Method: "POST",
		URL:    &url.URL{Path: "/path", RawQuery: "foo=bar"},
	}
	h := RequestHandler("request")(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		l := FromRequest(r)
		l.Log().Msg("")
	}))
	h = NewHandler(zerolog.New(out))(h)
	h.ServeHTTP(nil, r)
	if want, got := `{"request":"POST /path?foo=bar"}`+"\n", decodeIfBinary(out); want != got {
		t.Errorf("Invalid log output, got: %s, want: %s", got, want)
	}
}

func TestRemoteAddrHandler(t *testing.T) {
	out := &bytes.Buffer{}
	r := &http.Request{
		RemoteAddr: "1.2.3.4:1234",
	}
	h := RemoteAddrHandler("ip")(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		l := FromRequest(r)
		l.Log().Msg("")
	}))
	h = NewHandler(zerolog.New(out))(h)
	h.ServeHTTP(nil, r)
	if want, got := `{"ip":"1.2.3.4:1234"}`+"\n", decodeIfBinary(out); want != got {
		t.Errorf("Invalid log output, got: %s, want: %s", got, want)
	}
}

func TestRemoteAddrHandlerIPv6(t *testing.T) {
	out := &bytes.Buffer{}
	r := &http.Request{
		RemoteAddr: "[2001:db8:a0b:12f0::1]:1234",
	}
	h := RemoteAddrHandler("ip")(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		l := FromRequest(r)
		l.Log().Msg("")
	}))
	h = NewHandler(zerolog.New(out))(h)
	h.ServeHTTP(nil, r)
	if want, got := `{"ip":"[2001:db8:a0b:12f0::1]:1234"}`+"\n", decodeIfBinary(out); want != got {
		t.Errorf("Invalid log output, got: %s, want: %s", got, want)
	}
}

func TestRemoteIPHandler(t *testing.T) {
	out := &bytes.Buffer{}
	r := &http.Request{
		RemoteAddr: "1.2.3.4:1234",
	}
	h := RemoteIPHandler("ip")(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		l := FromRequest(r)
		l.Log().Msg("")
	}))
	h = NewHandler(zerolog.New(out))(h)
	h.ServeHTTP(nil, r)
	if want, got := `{"ip":"1.2.3.4"}`+"\n", decodeIfBinary(out); want != got {
		t.Errorf("Invalid log output, got: %s, want: %s", got, want)
	}
}

func TestRemoteIPHandlerIPv6(t *testing.T) {
	out := &bytes.Buffer{}
	r := &http.Request{
		RemoteAddr: "[2001:db8:a0b:12f0::1]:1234",
	}
	h := RemoteIPHandler("ip")(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		l := FromRequest(r)
		l.Log().Msg("")
	}))
	h = NewHandler(zerolog.New(out))(h)
	h.ServeHTTP(nil, r)
	if want, got := `{"ip":"2001:db8:a0b:12f0::1"}`+"\n", decodeIfBinary(out); want != got {
		t.Errorf("Invalid log output, got: %s, want: %s", got, want)
	}
}

func TestUserAgentHandler(t *testing.T) {
	out := &bytes.Buffer{}
	r := &http.Request{
		Header: http.Header{
			"User-Agent": []string{"some user agent string"},
		},
	}
	h := UserAgentHandler("ua")(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		l := FromRequest(r)
		l.Log().Msg("")
	}))
	h = NewHandler(zerolog.New(out))(h)
	h.ServeHTTP(nil, r)
	if want, got := `{"ua":"some user agent string"}`+"\n", decodeIfBinary(out); want != got {
		t.Errorf("Invalid log output, got: %s, want: %s", got, want)
	}
}

func TestRefererHandler(t *testing.T) {
	out := &bytes.Buffer{}
	r := &http.Request{
		Header: http.Header{
			"Referer": []string{"http://foo.com/bar"},
		},
	}
	h := RefererHandler("referer")(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		l := FromRequest(r)
		l.Log().Msg("")
	}))
	h = NewHandler(zerolog.New(out))(h)
	h.ServeHTTP(nil, r)
	if want, got := `{"referer":"http://foo.com/bar"}`+"\n", decodeIfBinary(out); want != got {
		t.Errorf("Invalid log output, got: %s, want: %s", got, want)
	}
}

func TestRequestIDHandler(t *testing.T) {
	out := &bytes.Buffer{}
	r := &http.Request{
		Header: http.Header{
			"Referer": []string{"http://foo.com/bar"},
		},
	}
	h := RequestIDHandler("id", "Request-Id")(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		id, ok := IDFromRequest(r)
		if !ok {
			t.Fatal("Missing id in request")
		}
		if want, got := id.String(), w.Header().Get("Request-Id"); got != want {
			t.Errorf("Invalid Request-Id header, got: %s, want: %s", got, want)
		}
		l := FromRequest(r)
		l.Log().Msg("")
		if want, got := fmt.Sprintf(`{"id":"%s"}`+"\n", id), decodeIfBinary(out); want != got {
			t.Errorf("Invalid log output, got: %s, want: %s", got, want)
		}
	}))
	h = NewHandler(zerolog.New(out))(h)
	h.ServeHTTP(httptest.NewRecorder(), r)
}

func TestCustomHeaderHandler(t *testing.T) {
	out := &bytes.Buffer{}
	r := &http.Request{
		Header: http.Header{
			"X-Request-Id": []string{"514bbe5bb5251c92bd07a9846f4a1ab6"},
		},
	}
	h := CustomHeaderHandler("reqID", "X-Request-Id")(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		l := FromRequest(r)
		l.Log().Msg("")
	}))
	h = NewHandler(zerolog.New(out))(h)
	h.ServeHTTP(nil, r)
	if want, got := `{"reqID":"514bbe5bb5251c92bd07a9846f4a1ab6"}`+"\n", decodeIfBinary(out); want != got {
		t.Errorf("Invalid log output, got: %s, want: %s", got, want)
	}
}

func TestEtagHandler(t *testing.T) {
	out := &bytes.Buffer{}
	w := httptest.NewRecorder()
	r := &http.Request{}
	h := EtagHandler("etag")(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Etag", `"abcdef"`)
		w.WriteHeader(http.StatusOK)
	}))
	h2 := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		h.ServeHTTP(w, r)
		l := FromRequest(r)
		l.Log().Msg("")
	})
	h3 := NewHandler(zerolog.New(out))(h2)
	h3.ServeHTTP(w, r)
	if want, got := `{"etag":"abcdef"}`+"\n", decodeIfBinary(out); want != got {
		t.Errorf("Invalid log output, got: %s, want: %s", got, want)
	}
}

func TestResponseHeaderHandler(t *testing.T) {
	out := &bytes.Buffer{}
	w := httptest.NewRecorder()
	r := &http.Request{}
	h := ResponseHeaderHandler("encoding", "Content-Encoding")(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Encoding", `gzip`)
		w.WriteHeader(http.StatusOK)
	}))
	h2 := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		h.ServeHTTP(w, r)
		l := FromRequest(r)
		l.Log().Msg("")
	})
	h3 := NewHandler(zerolog.New(out))(h2)
	h3.ServeHTTP(w, r)
	if want, got := `{"encoding":"gzip"}`+"\n", decodeIfBinary(out); want != got {
		t.Errorf("Invalid log output, got: %s, want: %s", got, want)
	}
}

func TestProtoHandler(t *testing.T) {
	out := &bytes.Buffer{}
	r := &http.Request{
		Proto: "test",
	}
	h := ProtoHandler("proto")(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		l := FromRequest(r)
		l.Log().Msg("")
	}))
	h = NewHandler(zerolog.New(out))(h)
	h.ServeHTTP(nil, r)
	if want, got := `{"proto":"test"}`+"\n", decodeIfBinary(out); want != got {
		t.Errorf("Invalid log output, got: %s, want: %s", got, want)
	}
}

func TestHTTPVersionHandler(t *testing.T) {
	out := &bytes.Buffer{}
	r := &http.Request{
		Proto: "HTTP/1.1",
	}
	h := HTTPVersionHandler("proto")(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		l := FromRequest(r)
		l.Log().Msg("")
	}))
	h = NewHandler(zerolog.New(out))(h)
	h.ServeHTTP(nil, r)
	if want, got := `{"proto":"1.1"}`+"\n", decodeIfBinary(out); want != got {
		t.Errorf("Invalid log output, got: %s, want: %s", got, want)
	}
}

func TestCombinedHandlers(t *testing.T) {
	out := &bytes.Buffer{}
	r := &http.Request{
		Method: "POST",
		URL:    &url.URL{Path: "/path", RawQuery: "foo=bar"},
	}
	h := MethodHandler("method")(RequestHandler("request")(URLHandler("url")(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		l := FromRequest(r)
		l.Log().Msg("")
	}))))
	h = NewHandler(zerolog.New(out))(h)
	h.ServeHTTP(nil, r)
	if want, got := `{"method":"POST","request":"POST /path?foo=bar","url":"/path?foo=bar"}`+"\n", decodeIfBinary(out); want != got {
		t.Errorf("Invalid log output, got: %s, want: %s", got, want)
	}
}

func BenchmarkHandlers(b *testing.B) {
	r := &http.Request{
		Method: "POST",
		URL:    &url.URL{Path: "/path", RawQuery: "foo=bar"},
	}
	h1 := URLHandler("url")(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		l := FromRequest(r)
		l.Log().Msg("")
	}))
	h2 := MethodHandler("method")(RequestHandler("request")(h1))
	handlers := map[string]http.Handler{
		"Single":           NewHandler(zerolog.New(io.Discard))(h1),
		"Combined":         NewHandler(zerolog.New(io.Discard))(h2),
		"SingleDisabled":   NewHandler(zerolog.New(io.Discard).Level(zerolog.Disabled))(h1),
		"CombinedDisabled": NewHandler(zerolog.New(io.Discard).Level(zerolog.Disabled))(h2),
	}
	for name := range handlers {
		h := handlers[name]
		b.Run(name, func(b *testing.B) {
			for i := 0; i < b.N; i++ {
				h.ServeHTTP(nil, r)
			}
		})
	}
}

func BenchmarkDataRace(b *testing.B) {
	log := zerolog.New(nil).With().
		Str("foo", "bar").
		Logger()
	lh := NewHandler(log)
	h := lh(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		l := FromRequest(r)
		l.UpdateContext(func(c zerolog.Context) zerolog.Context {
			return c.Str("bar", "baz")
		})
		l.Log().Msg("")
	}))

	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			h.ServeHTTP(nil, &http.Request{})
		}
	})
}

func TestCtxWithID(t *testing.T) {
	ctx := context.Background()

	id, _ := xid.FromString(`c0umremcie6smuu506pg`)

	want := context.Background()
	want = context.WithValue(want, idKey{}, id)

	if got := CtxWithID(ctx, id); !reflect.DeepEqual(got, want) {
		t.Errorf("CtxWithID() = %v, want %v", got, want)
	}
}

func TestHostHandler(t *testing.T) {
	out := &bytes.Buffer{}
	r := &http.Request{Host: "example.com:8080"}
	h := HostHandler("host")(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		l := FromRequest(r)
		l.Log().Msg("")
	}))
	h = NewHandler(zerolog.New(out))(h)
	h.ServeHTTP(nil, r)
	if want, got := `{"host":"example.com:8080"}`+"\n", decodeIfBinary(out); want != got {
		t.Errorf("Invalid log output, got: %s, want: %s", got, want)
	}
}

func TestHostHandlerWithoutPort(t *testing.T) {
	out := &bytes.Buffer{}
	r := &http.Request{Host: "example.com:8080"}
	h := HostHandler("host", true)(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		l := FromRequest(r)
		l.Log().Msg("")
	}))
	h = NewHandler(zerolog.New(out))(h)
	h.ServeHTTP(nil, r)
	if want, got := `{"host":"example.com"}`+"\n", decodeIfBinary(out); want != got {
		t.Errorf("Invalid log output, got: %s, want: %s", got, want)
	}
}

func TestGetHost(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"", ""},
		{"example.com:8080", "example.com"},
		{"example.com", "example.com"},
		{"invalid", "invalid"},
		{"192.168.0.1:8080", "192.168.0.1"},
		{"[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:8080", "2001:0db8:85a3:0000:0000:8a2e:0370:7334"},
		{"こんにちは.com:8080", "こんにちは.com"},
	}

	for _, tt := range tests {
		tt := tt
		t.Run(tt.input, func(t *testing.T) {
			result := getHost(tt.input)
			if tt.expected != result {
				t.Errorf("Invalid log output, got: %s, want: %s", result, tt.expected)
			}
		})
	}
}
