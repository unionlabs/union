// Package prometheus defines a service which is used for metrics collection
// and health of a node in Prysm.
package prometheus

import (
	"bytes"
	"context"
	"fmt"
	"net"
	"net/http"
	"runtime/debug"
	"runtime/pprof"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"github.com/prysmaticlabs/prysm/v4/runtime"
	"github.com/sirupsen/logrus"
)

var log = logrus.WithField("prefix", "prometheus")

// Service provides Prometheus metrics via the /metrics route. This route will
// show all the metrics registered with the Prometheus DefaultRegisterer.
type Service struct {
	server      *http.Server
	svcRegistry *runtime.ServiceRegistry
	failStatus  error
}

// Handler represents a path and handler func to serve on the same port as /metrics, /healthz, /goroutinez, etc.
type Handler struct {
	Path    string
	Handler func(http.ResponseWriter, *http.Request)
}

// NewService sets up a new instance for a given address host:port.
// An empty host will match with any IP so an address like ":2121" is perfectly acceptable.
func NewService(addr string, svcRegistry *runtime.ServiceRegistry, additionalHandlers ...Handler) *Service {
	s := &Service{svcRegistry: svcRegistry}

	mux := http.NewServeMux()
	mux.Handle("/metrics", promhttp.HandlerFor(prometheus.DefaultGatherer, promhttp.HandlerOpts{
		MaxRequestsInFlight: 5,
		Timeout:             30 * time.Second,
	}))
	mux.HandleFunc("/healthz", s.healthzHandler)
	mux.HandleFunc("/goroutinez", s.goroutinezHandler)

	// Register additional handlers.
	for _, h := range additionalHandlers {
		mux.HandleFunc(h.Path, h.Handler)
	}

	s.server = &http.Server{Addr: addr, Handler: mux, ReadHeaderTimeout: time.Second}

	return s
}

func (s *Service) healthzHandler(w http.ResponseWriter, r *http.Request) {
	response := generatedResponse{}

	type serviceStatus struct {
		Name   string `json:"service"`
		Status bool   `json:"status"`
		Err    string `json:"error"`
	}
	var hasError bool
	statuses := make([]serviceStatus, 0, len(s.svcRegistry.Statuses()))
	for k, v := range s.svcRegistry.Statuses() {
		s := serviceStatus{
			Name:   k.String(),
			Status: true,
		}
		if v != nil {
			s.Status = false
			s.Err = v.Error()
			if s.Err != "" {
				hasError = true
			}
		}
		statuses = append(statuses, s)
	}
	response.Data = statuses

	if hasError {
		w.WriteHeader(http.StatusServiceUnavailable)
	} else {
		w.WriteHeader(http.StatusOK)
	}

	// Handle plain text content.
	if contentType := negotiateContentType(r); contentType == contentTypePlainText {
		var buf bytes.Buffer
		for _, s := range statuses {
			var status string
			if s.Status {
				status = "OK"
			} else {
				status = "ERROR, " + s.Err
			}

			if _, err := buf.WriteString(fmt.Sprintf("%s: %s\n", s.Name, status)); err != nil {
				response.Err = err.Error()
				break
			}
		}
		response.Data = buf
	}

	if err := writeResponse(w, r, response); err != nil {
		log.WithError(err).Error("Error writing response")
	}
}

func (_ *Service) goroutinezHandler(w http.ResponseWriter, _ *http.Request) {
	stack := debug.Stack()
	if _, err := w.Write(stack); err != nil {
		log.WithError(err).Error("Failed to write goroutines stack")
	}
	if err := pprof.Lookup("goroutine").WriteTo(w, 2); err != nil {
		log.WithError(err).Error("Failed to write pprof goroutines")
	}
}

// Start the prometheus service.
func (s *Service) Start() {
	go func() {
		// See if the port is already used.
		conn, err := net.DialTimeout("tcp", s.server.Addr, time.Second)
		if err == nil {
			if err := conn.Close(); err != nil {
				log.WithError(err).Error("Failed to close connection")
			}
			// Something on the port; we cannot use it.
			log.WithField("address", s.server.Addr).Warn("Port already in use; cannot start prometheus service")
		} else {
			// Nothing on that port; we can use it.
			log.WithField("address", s.server.Addr).Debug("Starting prometheus service")
			err := s.server.ListenAndServe()
			if err != nil && err != http.ErrServerClosed {
				log.WithError(err).Errorf("Could not listen to host:port :%s", s.server.Addr)
				s.failStatus = err
			}
		}
	}()
}

// Stop the service gracefully.
func (s *Service) Stop() error {
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()
	return s.server.Shutdown(ctx)
}

// Status checks for any service failure conditions.
func (s *Service) Status() error {
	if s.failStatus != nil {
		return s.failStatus
	}
	return nil
}
