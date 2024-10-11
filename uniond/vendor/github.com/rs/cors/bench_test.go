package cors

import (
	"net/http"
	"strings"
	"testing"
)

type FakeResponse struct {
	header http.Header
}

func (r FakeResponse) Header() http.Header {
	return r.header
}

func (r FakeResponse) WriteHeader(n int) {
}

func (r FakeResponse) Write(b []byte) (n int, err error) {
	return len(b), nil
}

const (
	headerOrigin  = "Origin"
	headerACRM    = "Access-Control-Request-Method"
	headerACRH    = "Access-Control-Request-Headers"
	dummyEndpoint = "http://example.com/foo"
	dummyOrigin   = "https://somedomain.com"
)

func BenchmarkWithout(b *testing.B) {
	resps := makeFakeResponses(b.N)
	req, _ := http.NewRequest(http.MethodGet, dummyEndpoint, nil)

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		testHandler.ServeHTTP(resps[i], req)
	}
}

func BenchmarkDefault(b *testing.B) {
	resps := makeFakeResponses(b.N)
	req, _ := http.NewRequest(http.MethodGet, dummyEndpoint, nil)
	req.Header.Add(headerOrigin, dummyOrigin)
	handler := Default().Handler(testHandler)

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		handler.ServeHTTP(resps[i], req)
	}
}

func BenchmarkAllowedOrigin(b *testing.B) {
	resps := makeFakeResponses(b.N)
	req, _ := http.NewRequest(http.MethodGet, dummyEndpoint, nil)
	req.Header.Add(headerOrigin, dummyOrigin)
	c := New(Options{
		AllowedOrigins: []string{dummyOrigin},
	})
	handler := c.Handler(testHandler)

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		handler.ServeHTTP(resps[i], req)
	}
}

func BenchmarkPreflight(b *testing.B) {
	resps := makeFakeResponses(b.N)
	req, _ := http.NewRequest(http.MethodOptions, dummyEndpoint, nil)
	req.Header.Add(headerOrigin, dummyOrigin)
	req.Header.Add(headerACRM, http.MethodGet)
	handler := Default().Handler(testHandler)

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		handler.ServeHTTP(resps[i], req)
	}
}

func BenchmarkPreflightHeader(b *testing.B) {
	resps := makeFakeResponses(b.N)
	req, _ := http.NewRequest(http.MethodOptions, dummyEndpoint, nil)
	req.Header.Add(headerOrigin, dummyOrigin)
	req.Header.Add(headerACRM, http.MethodGet)
	req.Header.Add(headerACRH, "accept")
	handler := Default().Handler(testHandler)

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		handler.ServeHTTP(resps[i], req)
	}
}

func BenchmarkPreflightAdversarialACRH(b *testing.B) {
	resps := makeFakeResponses(b.N)
	req, _ := http.NewRequest(http.MethodOptions, dummyEndpoint, nil)
	req.Header.Add(headerOrigin, dummyOrigin)
	req.Header.Add(headerACRM, http.MethodGet)
	req.Header.Add(headerACRH, strings.Repeat(",", 1024))
	handler := Default().Handler(testHandler)

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		handler.ServeHTTP(resps[i], req)
	}
}

func makeFakeResponses(n int) []*FakeResponse {
	resps := make([]*FakeResponse, n)
	for i := 0; i < n; i++ {
		resps[i] = &FakeResponse{http.Header{
			"Content-Type": []string{"text/plain"},
		}}
	}
	return resps
}
