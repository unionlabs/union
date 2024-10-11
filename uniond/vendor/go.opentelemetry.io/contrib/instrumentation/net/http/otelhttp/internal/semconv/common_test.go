// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package semconv

import (
	"net/http"
	"net/http/httptest"
	"net/url"
	"strconv"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"go.opentelemetry.io/otel/attribute"
)

type testServerReq struct {
	hostname   string
	serverPort int
	peerAddr   string
	peerPort   int
	clientIP   string
}

func testTraceRequest(t *testing.T, serv HTTPServer, want func(testServerReq) []attribute.KeyValue) {
	t.Helper()

	got := make(chan *http.Request, 1)
	handler := func(w http.ResponseWriter, r *http.Request) {
		got <- r
		close(got)
		w.WriteHeader(http.StatusOK)
	}

	srv := httptest.NewServer(http.HandlerFunc(handler))
	defer srv.Close()

	srvURL, err := url.Parse(srv.URL)
	require.NoError(t, err)
	srvPort, err := strconv.ParseInt(srvURL.Port(), 10, 32)
	require.NoError(t, err)

	resp, err := srv.Client().Get(srv.URL)
	require.NoError(t, err)
	require.NoError(t, resp.Body.Close())

	req := <-got
	peer, peerPort := splitHostPort(req.RemoteAddr)

	const user = "alice"
	req.SetBasicAuth(user, "pswrd")

	const clientIP = "127.0.0.5"
	req.Header.Add("X-Forwarded-For", clientIP)

	srvReq := testServerReq{
		hostname:   srvURL.Hostname(),
		serverPort: int(srvPort),
		peerAddr:   peer,
		peerPort:   peerPort,
		clientIP:   clientIP,
	}

	assert.ElementsMatch(t, want(srvReq), serv.RequestTraceAttrs("", req))
}
