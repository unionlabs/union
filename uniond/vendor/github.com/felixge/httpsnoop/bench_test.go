package httpsnoop

import (
	"net/http"
	"net/http/httptest"
	"testing"
)

func BenchmarkBaseline(b *testing.B) {
	benchmark(b, 0)
}

func BenchmarkCaptureMetrics(b *testing.B) {
	benchmark(b, 1)
}

func BenchmarkCaptureMetricsTwice(b *testing.B) {
	benchmark(b, 2)
}

func BenchmarkWrap(b *testing.B) {
	b.StopTimer()
	doneCh := make(chan struct{}, 1)
	h := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		b.StartTimer()
		for i := 0; i < b.N; i++ {
			Wrap(w, Hooks{})
		}
		doneCh <- struct{}{}
	})
	s := httptest.NewServer(h)
	defer s.Close()
	if _, err := http.Get(s.URL); err != nil {
		b.Fatal(err)
	}
	<-doneCh
}

func benchmark(b *testing.B, wrappings int) {
	dummyH := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {})
	h := dummyH
	for x := 0; x < wrappings; x++ {
		hCopy := h
		h = http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			CaptureMetrics(hCopy, w, r)
		})
	}

	req := httptest.NewRequest(http.MethodGet, "/", http.NoBody)
	resp := httptest.NewRecorder() // ok to reuse; we're not writing anything to it

	b.ReportAllocs()
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		h.ServeHTTP(resp, req)
	}
}
