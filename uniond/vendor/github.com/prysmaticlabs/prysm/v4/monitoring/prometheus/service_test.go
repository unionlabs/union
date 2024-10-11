package prometheus

import (
	"errors"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/prysmaticlabs/prysm/v4/runtime"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/sirupsen/logrus"
)

func init() {
	logrus.SetLevel(logrus.DebugLevel)
	logrus.SetOutput(io.Discard)
}

func TestLifecycle(t *testing.T) {
	prometheusService := NewService(":2112", nil)
	prometheusService.Start()
	// Give service time to start.
	time.Sleep(time.Second)

	// Query the service to ensure it really started.
	resp, err := http.Get("http://localhost:2112/metrics")
	require.NoError(t, err)
	assert.NotEqual(t, uint64(0), resp.ContentLength, "Unexpected content length 0")

	err = prometheusService.Stop()
	require.NoError(t, err)
	// Give service time to stop.
	time.Sleep(time.Second)

	// Query the service to ensure it really stopped.
	_, err = http.Get("http://localhost:2112/metrics")
	assert.NotNil(t, err, "Service still running after Stop()")
}

type mockService struct {
	status error
}

func (_ *mockService) Start() {
}

func (_ *mockService) Stop() error {
	return nil
}

func (m *mockService) Status() error {
	return m.status
}

func TestHealthz(t *testing.T) {
	registry := runtime.NewServiceRegistry()
	m := &mockService{}
	require.NoError(t, registry.RegisterService(m), "Failed to register service")
	s := NewService("" /*addr*/, registry)

	req, err := http.NewRequest("GET", "/healthz", nil /*reader*/)
	require.NoError(t, err)

	handler := http.HandlerFunc(s.healthzHandler)

	rr := httptest.NewRecorder()
	handler.ServeHTTP(rr, req)

	if status := rr.Code; status != http.StatusOK {
		t.Errorf("expected OK status but got %v", rr.Code)
	}

	body := rr.Body.String()
	if !strings.Contains(body, "*prometheus.mockService: OK") {
		t.Errorf("Expected body to contain mockService status, but got %v", body)
	}

	m.status = errors.New("something really bad has happened")

	rr = httptest.NewRecorder()
	handler.ServeHTTP(rr, req)

	if status := rr.Code; status != http.StatusServiceUnavailable {
		t.Errorf("expected StatusServiceUnavailable status but got %v", rr.Code)
	}

	body = rr.Body.String()
	if !strings.Contains(
		body,
		"*prometheus.mockService: ERROR, something really bad has happened",
	) {
		t.Errorf("Expected body to contain mockService status, but got %v", body)
	}

}

func TestStatus(t *testing.T) {
	failError := errors.New("failure")
	s := &Service{failStatus: failError}

	if err := s.Status(); err != s.failStatus {
		t.Errorf("Wanted: %v, got: %v", s.failStatus, s.Status())
	}
}

func TestContentNegotiation(t *testing.T) {
	t.Run("/healthz all services are ok", func(t *testing.T) {
		registry := runtime.NewServiceRegistry()
		m := &mockService{}
		require.NoError(t, registry.RegisterService(m), "Failed to register service")
		s := NewService("", registry)

		req, err := http.NewRequest("GET", "/healthz", nil /* body */)
		require.NoError(t, err)

		handler := http.HandlerFunc(s.healthzHandler)
		rr := httptest.NewRecorder()
		handler.ServeHTTP(rr, req)

		body := rr.Body.String()
		if !strings.Contains(body, "*prometheus.mockService: OK") {
			t.Errorf("Expected body to contain mockService status, but got %q", body)
		}

		// Request response as JSON.
		req.Header.Add("Accept", "application/json, */*;q=0.5")
		rr = httptest.NewRecorder()
		handler.ServeHTTP(rr, req)

		body = rr.Body.String()
		expectedJSON := "{\"error\":\"\",\"data\":[{\"service\":\"*prometheus.mockService\",\"status\":true,\"error\":\"\"}]}"
		if !strings.Contains(body, expectedJSON) {
			t.Errorf("Unexpected data, want: %q got %q", expectedJSON, body)
		}
	})

	t.Run("/healthz failed service", func(t *testing.T) {
		registry := runtime.NewServiceRegistry()
		m := &mockService{}
		m.status = errors.New("something is wrong")
		require.NoError(t, registry.RegisterService(m), "Failed to register service")
		s := NewService("", registry)

		req, err := http.NewRequest("GET", "/healthz", nil /* body */)
		require.NoError(t, err)

		handler := http.HandlerFunc(s.healthzHandler)
		rr := httptest.NewRecorder()
		handler.ServeHTTP(rr, req)

		body := rr.Body.String()
		if !strings.Contains(body, "*prometheus.mockService: ERROR, something is wrong") {
			t.Errorf("Expected body to contain mockService status, but got %q", body)
		}

		// Request response as JSON.
		req.Header.Add("Accept", "application/json, */*;q=0.5")
		rr = httptest.NewRecorder()
		handler.ServeHTTP(rr, req)

		body = rr.Body.String()
		expectedJSON := "{\"error\":\"\",\"data\":[{\"service\":\"*prometheus.mockService\",\"status\":false,\"error\":\"something is wrong\"}]}"
		if !strings.Contains(body, expectedJSON) {
			t.Errorf("Unexpected data, want: %q got %q", expectedJSON, body)
		}
		if rr.Code < 500 {
			t.Errorf("Expected a server error response code, but got %d", rr.Code)
		}
	})
}
