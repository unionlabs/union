// Copyright (c) 2021 Klaus Post. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package gzhttp

import (
	"bytes"
	"io"
	"net/http"
	"net/http/httptest"
	"os"
	"runtime"
	"testing"

	"github.com/klauspost/compress/gzip"
	"github.com/klauspost/compress/zstd"
)

func TestTransport(t *testing.T) {
	bin, err := os.ReadFile("testdata/benchmark.json")
	if err != nil {
		t.Fatal(err)
	}

	server := httptest.NewServer(newTestHandler(bin))

	c := http.Client{Transport: Transport(http.DefaultTransport)}
	resp, err := c.Get(server.URL)
	if err != nil {
		t.Fatal(err)
	}
	got, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(got, bin) {
		t.Errorf("data mismatch")
	}
}

func TestTransportForced(t *testing.T) {
	raw, err := os.ReadFile("testdata/benchmark.json")
	if err != nil {
		t.Fatal(err)
	}

	var buf bytes.Buffer
	zw := gzip.NewWriter(&buf)
	zw.Write(raw)
	zw.Close()
	bin := buf.Bytes()

	server := httptest.NewServer(newTestHandler(bin))

	c := http.Client{Transport: Transport(http.DefaultTransport)}
	resp, err := c.Get(server.URL + "/gzipped")
	if err != nil {
		t.Fatal(err)
	}
	got, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(got, raw) {
		t.Errorf("data mismatch")
	}
}

func TestTransportForcedDisabled(t *testing.T) {
	raw, err := os.ReadFile("testdata/benchmark.json")
	if err != nil {
		t.Fatal(err)
	}

	var buf bytes.Buffer
	zw := gzip.NewWriter(&buf)
	zw.Write(raw)
	zw.Close()
	bin := buf.Bytes()

	server := httptest.NewServer(newTestHandler(bin))
	c := http.Client{Transport: Transport(http.DefaultTransport, TransportEnableGzip(false))}
	resp, err := c.Get(server.URL + "/gzipped")
	if err != nil {
		t.Fatal(err)
	}
	got, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(bin, got) {
		t.Errorf("data mismatch")
	}
}

func TestTransportZstd(t *testing.T) {
	bin, err := os.ReadFile("testdata/benchmark.json")
	if err != nil {
		t.Fatal(err)
	}
	enc, _ := zstd.NewWriter(nil)
	defer enc.Close()
	zsBin := enc.EncodeAll(bin, nil)
	server := httptest.NewServer(newTestHandler(zsBin))

	c := http.Client{Transport: Transport(http.DefaultTransport)}
	resp, err := c.Get(server.URL + "/zstd")
	if err != nil {
		t.Fatal(err)
	}
	got, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(got, bin) {
		t.Errorf("data mismatch")
	}
}

func TestTransportInvalid(t *testing.T) {
	bin, err := os.ReadFile("testdata/benchmark.json")
	if err != nil {
		t.Fatal(err)
	}

	server := httptest.NewServer(newTestHandler(bin))

	c := http.Client{Transport: Transport(http.DefaultTransport)}
	// Serves json as gzipped...
	resp, err := c.Get(server.URL + "/gzipped")
	if err != nil {
		t.Fatal(err)
	}
	_, err = io.ReadAll(resp.Body)
	if err == nil {
		t.Fatal("expected error, got nil")
	}
}

func TestTransportZstdDisabled(t *testing.T) {
	raw, err := os.ReadFile("testdata/benchmark.json")
	if err != nil {
		t.Fatal(err)
	}

	enc, _ := zstd.NewWriter(nil)
	defer enc.Close()
	zsBin := enc.EncodeAll(raw, nil)

	server := httptest.NewServer(newTestHandler(zsBin))
	c := http.Client{Transport: Transport(http.DefaultTransport, TransportEnableZstd(false))}
	resp, err := c.Get(server.URL + "/zstd")
	if err != nil {
		t.Fatal(err)
	}
	got, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(zsBin, got) {
		t.Errorf("data mismatch")
	}
}

func TestTransportZstdInvalid(t *testing.T) {
	bin, err := os.ReadFile("testdata/benchmark.json")
	if err != nil {
		t.Fatal(err)
	}
	// Do not encode...
	server := httptest.NewServer(newTestHandler(bin))

	c := http.Client{Transport: Transport(http.DefaultTransport)}
	resp, err := c.Get(server.URL + "/zstd")
	if err != nil {
		t.Fatal(err)
	}
	_, err = io.ReadAll(resp.Body)
	if err == nil {
		t.Fatal("expected error, got nil")
	}
	t.Log("expected error:", err)
}

func TestDefaultTransport(t *testing.T) {
	bin, err := os.ReadFile("testdata/benchmark.json")
	if err != nil {
		t.Fatal(err)
	}

	server := httptest.NewServer(newTestHandler(bin))

	// Not wrapped...
	c := http.Client{Transport: http.DefaultTransport}
	resp, err := c.Get(server.URL)
	if err != nil {
		t.Fatal(err)
	}
	got, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(got, bin) {
		t.Errorf("data mismatch")
	}
}

func TestTransportCustomEval(t *testing.T) {
	bin, err := os.ReadFile("testdata/benchmark.json")
	if err != nil {
		t.Fatal(err)
	}

	server := httptest.NewServer(newTestHandler(bin))
	calledWith := ""
	c := http.Client{Transport: Transport(http.DefaultTransport, TransportEnableZstd(false), TransportCustomEval(func(h http.Header) bool {
		calledWith = h.Get("Content-Encoding")
		return true
	}))}
	resp, err := c.Get(server.URL)
	if err != nil {
		t.Fatal(err)
	}
	got, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(got, bin) {
		t.Errorf("data mismatch")
	}
	if calledWith != "gzip" {
		t.Fatalf("Expected encoding %q, got %q", "gzip", calledWith)
	}
	// Test returning false
	c = http.Client{Transport: Transport(http.DefaultTransport, TransportCustomEval(func(h http.Header) bool {
		calledWith = h.Get("Content-Encoding")
		return false
	}))}
	resp, err = c.Get(server.URL)
	if err != nil {
		t.Fatal(err)
	}
	// Check we got the compressed data
	gotCE := resp.Header.Get("Content-Encoding")
	if gotCE != "gzip" {
		t.Fatalf("Expected encoding %q, got %q", "gzip", gotCE)
	}
	if calledWith != "gzip" {
		t.Fatalf("Expected encoding %q, got %q", "gzip", calledWith)
	}
}

func TestTransportTransportAlwaysDecompress(t *testing.T) {
	bin, err := os.ReadFile("testdata/benchmark.json")
	if err != nil {
		t.Fatal(err)
	}

	// We will serve the data as zstd+gzip, but the client will not request it.
	server := httptest.NewServer(newTestHandler(bin))
	c := http.Client{Transport: Transport(http.DefaultTransport, TransportEnableZstd(false), TransportEnableGzip(false), TransportAlwaysDecompress(true))}
	resp, err := c.Get(server.URL + "/zstd/do")
	if err != nil {
		t.Fatal(err)
	}
	got, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(got, bin) {
		t.Errorf("data mismatch")
	}
	resp.Body.Close()

	resp, err = c.Get(server.URL + "/gzip/do")
	if err != nil {
		t.Fatal(err)
	}
	got, err = io.ReadAll(resp.Body)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(got, bin) {
		t.Errorf("data mismatch")
	}
	resp.Body.Close()

	// We will serve the data as zstd+gzip, but the client will not request it.
	// With TransportAlwaysDecompress(false) it should not be decompressed.
	c = http.Client{Transport: Transport(http.DefaultTransport, TransportEnableZstd(false), TransportEnableGzip(false), TransportAlwaysDecompress(false))}
	resp, err = c.Get(server.URL + "/zstd/do")
	if err != nil {
		t.Fatal(err)
	}
	got, err = io.ReadAll(resp.Body)
	if err != nil {
		t.Fatal(err)
	}
	if bytes.Equal(got, bin) {
		t.Errorf("data matches")
	}
	resp.Body.Close()

	resp, err = c.Get(server.URL + "/gzip/do")
	if err != nil {
		t.Fatal(err)
	}
	got, err = io.ReadAll(resp.Body)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(got, bin) {
		t.Errorf("data matches")
	}
	resp.Body.Close()
}

func BenchmarkTransport(b *testing.B) {
	raw, err := os.ReadFile("testdata/benchmark.json")
	if err != nil {
		b.Fatal(err)
	}
	sz := int64(len(raw))
	var buf bytes.Buffer
	zw := gzip.NewWriter(&buf)
	zw.Write(raw)
	zw.Close()
	bin := buf.Bytes()
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		r.Body.Close()
		w.Header().Set("Content-Encoding", "gzip")
		w.WriteHeader(http.StatusOK)
		w.Write(bin)
	}))
	enc, _ := zstd.NewWriter(nil, zstd.WithWindowSize(128<<10), zstd.WithEncoderLevel(zstd.SpeedBestCompression))
	defer enc.Close()
	zsBin := enc.EncodeAll(raw, nil)
	serverZstd := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		r.Body.Close()
		w.Header().Set("Content-Encoding", "zstd")
		w.WriteHeader(http.StatusOK)
		w.Write(zsBin)
	}))
	b.Run("gzhttp", func(b *testing.B) {
		c := http.Client{Transport: Transport(http.DefaultTransport)}

		b.SetBytes(int64(sz))
		b.ReportAllocs()
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			resp, err := c.Get(server.URL + "/gzipped")
			if err != nil {
				b.Fatal(err)
			}
			n, err := io.Copy(io.Discard, resp.Body)
			if err != nil {
				b.Fatal(err)
			}
			if n != sz {
				b.Fatalf("size mismatch: want %d, got %d", sz, n)
			}
			resp.Body.Close()
		}
		b.ReportMetric(100*float64(len(bin))/float64(len(raw)), "pct")
	})
	b.Run("stdlib", func(b *testing.B) {
		c := http.Client{Transport: http.DefaultTransport}
		b.SetBytes(int64(sz))
		b.ReportAllocs()
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			resp, err := c.Get(server.URL + "/gzipped")
			if err != nil {
				b.Fatal(err)
			}
			n, err := io.Copy(io.Discard, resp.Body)
			if err != nil {
				b.Fatal(err)
			}
			if n != sz {
				b.Fatalf("size mismatch: want %d, got %d", sz, n)
			}
			resp.Body.Close()
		}
		b.ReportMetric(100*float64(len(bin))/float64(len(raw)), "pct")
	})
	b.Run("zstd", func(b *testing.B) {
		c := http.Client{Transport: Transport(http.DefaultTransport)}

		b.SetBytes(int64(sz))
		b.ReportAllocs()
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			resp, err := c.Get(serverZstd.URL + "/zstd")
			if err != nil {
				b.Fatal(err)
			}
			n, err := io.Copy(io.Discard, resp.Body)
			if err != nil {
				b.Fatal(err)
			}
			if n != sz {
				b.Fatalf("size mismatch: want %d, got %d", sz, n)
			}
			resp.Body.Close()
		}
		b.ReportMetric(100*float64(len(zsBin))/float64(len(raw)), "pct")
	})
	b.Run("gzhttp-par", func(b *testing.B) {
		c := http.Client{
			Transport: Transport(&http.Transport{
				MaxConnsPerHost:     runtime.GOMAXPROCS(0),
				MaxIdleConnsPerHost: runtime.GOMAXPROCS(0),
			}),
		}

		b.SetBytes(int64(sz))
		b.ReportAllocs()
		b.ResetTimer()
		b.RunParallel(func(pb *testing.PB) {
			for pb.Next() {
				resp, err := c.Get(server.URL + "/gzipped")
				if err != nil {
					b.Fatal(err)
				}
				n, err := io.Copy(io.Discard, resp.Body)
				if err != nil {
					b.Fatal(err)
				}
				if n != sz {
					b.Fatalf("size mismatch: want %d, got %d", sz, n)
				}
				resp.Body.Close()
			}
		})
		b.ReportMetric(100*float64(len(bin))/float64(len(raw)), "pct")
	})
	b.Run("stdlib-par", func(b *testing.B) {
		c := http.Client{Transport: &http.Transport{
			MaxConnsPerHost:     runtime.GOMAXPROCS(0),
			MaxIdleConnsPerHost: runtime.GOMAXPROCS(0),
		}}
		b.SetBytes(int64(sz))
		b.ReportAllocs()
		b.ResetTimer()
		b.RunParallel(func(pb *testing.PB) {
			for pb.Next() {
				resp, err := c.Get(server.URL + "/gzipped")
				if err != nil {
					b.Fatal(err)
				}
				n, err := io.Copy(io.Discard, resp.Body)
				if err != nil {
					b.Fatal(err)
				}
				if n != sz {
					b.Fatalf("size mismatch: want %d, got %d", sz, n)
				}
				resp.Body.Close()
			}
		})
		b.ReportMetric(100*float64(len(bin))/float64(len(raw)), "pct")
	})
	b.Run("zstd-par", func(b *testing.B) {
		c := http.Client{
			Transport: Transport(&http.Transport{
				MaxConnsPerHost:     runtime.GOMAXPROCS(0),
				MaxIdleConnsPerHost: runtime.GOMAXPROCS(0),
			}),
		}

		b.SetBytes(int64(sz))
		b.ReportAllocs()
		b.ResetTimer()
		b.RunParallel(func(pb *testing.PB) {
			for pb.Next() {
				resp, err := c.Get(serverZstd.URL + "/zstd")
				if err != nil {
					b.Fatal(err)
				}
				n, err := io.Copy(io.Discard, resp.Body)
				if err != nil {
					b.Fatal(err)
				}
				if n != sz {
					b.Fatalf("size mismatch: want %d, got %d", sz, n)
				}
				resp.Body.Close()
			}
		})
		b.ReportMetric(100*float64(len(zsBin))/float64(len(raw)), "pct")
	})
}
