package httprp_test

import (
	"context"
	"io/ioutil"
	"net/http"
	"net/http/httptest"
	"net/url"
	"testing"

	httptransport "github.com/go-kit/kit/transport/httprp"
)

func TestServerHappyPathSingleServer(t *testing.T) {
	originServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("hey"))
	}))
	defer originServer.Close()
	originURL, _ := url.Parse(originServer.URL)

	handler := httptransport.NewServer(
		originURL,
	)
	proxyServer := httptest.NewServer(handler)
	defer proxyServer.Close()

	resp, _ := http.Get(proxyServer.URL)
	if want, have := http.StatusOK, resp.StatusCode; want != have {
		t.Errorf("want %d, have %d", want, have)
	}

	responseBody, _ := ioutil.ReadAll(resp.Body)
	if want, have := "hey", string(responseBody); want != have {
		t.Errorf("want %q, have %q", want, have)
	}
}

func TestServerHappyPathSingleServerWithServerOptions(t *testing.T) {
	const (
		headerKey = "X-TEST-HEADER"
		headerVal = "go-kit-proxy"
	)

	originServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if want, have := headerVal, r.Header.Get(headerKey); want != have {
			t.Errorf("want %q, have %q", want, have)
		}

		w.WriteHeader(http.StatusOK)
		w.Write([]byte("hey"))
	}))
	defer originServer.Close()
	originURL, _ := url.Parse(originServer.URL)

	handler := httptransport.NewServer(
		originURL,
		httptransport.ServerBefore(func(ctx context.Context, r *http.Request) context.Context {
			r.Header.Add(headerKey, headerVal)
			return ctx
		}),
	)
	proxyServer := httptest.NewServer(handler)
	defer proxyServer.Close()

	resp, _ := http.Get(proxyServer.URL)
	if want, have := http.StatusOK, resp.StatusCode; want != have {
		t.Errorf("want %d, have %d", want, have)
	}

	responseBody, _ := ioutil.ReadAll(resp.Body)
	if want, have := "hey", string(responseBody); want != have {
		t.Errorf("want %q, have %q", want, have)
	}
}

func TestServerOriginServerNotFoundResponse(t *testing.T) {
	originServer := httptest.NewServer(http.NotFoundHandler())
	defer originServer.Close()
	originURL, _ := url.Parse(originServer.URL)

	handler := httptransport.NewServer(
		originURL,
	)
	proxyServer := httptest.NewServer(handler)
	defer proxyServer.Close()

	resp, _ := http.Get(proxyServer.URL)
	if want, have := http.StatusNotFound, resp.StatusCode; want != have {
		t.Errorf("want %d, have %d", want, have)
	}
}

func TestServerOriginServerUnreachable(t *testing.T) {
	// create a server, then promptly shut it down
	originServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))
	originURL, _ := url.Parse(originServer.URL)
	originServer.Close()

	handler := httptransport.NewServer(
		originURL,
	)
	proxyServer := httptest.NewServer(handler)
	defer proxyServer.Close()

	resp, _ := http.Get(proxyServer.URL)
	switch resp.StatusCode {
	case http.StatusBadGateway: // go1.7 and beyond
		break
	case http.StatusInternalServerError: // to go1.7
		break
	default:
		t.Errorf("want %d or %d, have %d", http.StatusBadGateway, http.StatusInternalServerError, resp.StatusCode)
	}
}

func TestMultipleServerBefore(t *testing.T) {
	const (
		headerKey = "X-TEST-HEADER"
		headerVal = "go-kit-proxy"
	)

	originServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if want, have := headerVal, r.Header.Get(headerKey); want != have {
			t.Errorf("want %q, have %q", want, have)
		}

		w.WriteHeader(http.StatusOK)
		w.Write([]byte("hey"))
	}))
	defer originServer.Close()
	originURL, _ := url.Parse(originServer.URL)

	handler := httptransport.NewServer(
		originURL,
		httptransport.ServerBefore(func(ctx context.Context, r *http.Request) context.Context {
			r.Header.Add(headerKey, headerVal)
			return ctx
		}),
		httptransport.ServerBefore(func(ctx context.Context, r *http.Request) context.Context {
			return ctx
		}),
	)
	proxyServer := httptest.NewServer(handler)
	defer proxyServer.Close()

	resp, _ := http.Get(proxyServer.URL)
	if want, have := http.StatusOK, resp.StatusCode; want != have {
		t.Errorf("want %d, have %d", want, have)
	}

	responseBody, _ := ioutil.ReadAll(resp.Body)
	if want, have := "hey", string(responseBody); want != have {
		t.Errorf("want %q, have %q", want, have)
	}
}
