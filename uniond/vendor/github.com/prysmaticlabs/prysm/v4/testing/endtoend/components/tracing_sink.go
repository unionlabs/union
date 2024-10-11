package components

import (
	"bytes"
	"context"
	"encoding/base64"
	"io"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/helpers"
	e2e "github.com/prysmaticlabs/prysm/v4/testing/endtoend/params"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/types"
)

var _ types.ComponentRunner = &TracingSink{}

// TracingSink to capture HTTP requests from opentracing pushes. This is meant
// to capture all opentracing spans from Prysm during an end-to-end test. Spans
// are normally sent to a jaeger (https://www.jaegertracing.io/docs/1.25/getting-started/)
// endpoint, but here we instead replace that with our own http request sink.
// The request sink receives any requests, raw marshals them and base64-encodes them,
// then writes them newline-delimited into a file.
//
// The output file from this component can then be used by tools/replay-http in
// the Prysm repository to replay requests to a jaeger collector endpoint. This
// can then be used to visualize the spans themselves in the jaeger UI.
type TracingSink struct {
	cancel   context.CancelFunc
	started  chan struct{}
	endpoint string
	server   *http.Server
}

// NewTracingSink initializes the tracing sink component.
func NewTracingSink(endpoint string) *TracingSink {
	return &TracingSink{
		started:  make(chan struct{}, 1),
		endpoint: endpoint,
	}
}

// Start the tracing sink.
func (ts *TracingSink) Start(ctx context.Context) error {
	if ts.endpoint == "" {
		return errors.New("empty endpoint provided")
	}
	ctx, cancelF := context.WithCancel(ctx)
	ts.cancel = cancelF
	go ts.initializeSink(ctx)
	close(ts.started)
	return nil
}

// Started checks whether a tracing sink is started and ready to be queried.
func (ts *TracingSink) Started() <-chan struct{} {
	return ts.started
}

// Pause pauses the component and its underlying process.
func (ts *TracingSink) Pause() error {
	return nil
}

// Resume resumes the component and its underlying process.
func (ts *TracingSink) Resume() error {
	return nil
}

// Stop stops the component and its underlying process.
func (ts *TracingSink) Stop() error {
	ts.cancel()
	return nil
}

// Initialize an http handler that writes all requests to a file.
func (ts *TracingSink) initializeSink(ctx context.Context) {
	mux := &http.ServeMux{}
	ts.server = &http.Server{
		Addr:              ts.endpoint,
		Handler:           mux,
		ReadHeaderTimeout: time.Second,
	}
	defer func() {
		if err := ts.server.Close(); err != nil {
			log.WithError(err).Error("Failed to close http server")
			return
		}
	}()
	stdOutFile, err := helpers.DeleteAndCreateFile(e2e.TestParams.LogPath, e2e.TracingRequestSinkFileName)
	if err != nil {
		log.WithError(err).Error("Failed to create stdout file")
		return
	}
	cleanup := func() {
		if err := stdOutFile.Close(); err != nil {
			log.WithError(err).Error("Could not close stdout file")
		}
		if err := ts.server.Close(); err != nil {
			log.WithError(err).Error("Could not close http server")
		}
	}
	mux.HandleFunc("/", func(_ http.ResponseWriter, r *http.Request) {
		if err := captureRequest(stdOutFile, r); err != nil {
			log.WithError(err).Error("Failed to capture http request")
			return
		}
	})
	sigs := make(chan os.Signal, 1)
	signal.Notify(sigs, syscall.SIGINT, syscall.SIGTERM)
	go func() {
		for {
			select {
			case <-ctx.Done():
				cleanup()
				return
			case <-sigs:
				cleanup()
				return
			default:
				// Sleep for 100ms and do nothing while waiting for
				// cancellation.
				time.Sleep(100 * time.Millisecond)
			}
		}
	}()
	if err := ts.server.ListenAndServe(); err != http.ErrServerClosed {
		log.WithError(err).Error("Failed to serve http")
	}
}

// Captures raw requests in base64 encoded form in a line-delimited file.
func captureRequest(f io.Writer, r *http.Request) error {
	buf := bytes.NewBuffer(nil)
	err := r.Write(buf)
	if err != nil {
		return err
	}
	encoded := make([]byte, base64.StdEncoding.EncodedLen(len(buf.Bytes())))
	base64.StdEncoding.Encode(encoded, buf.Bytes())
	encoded = append(encoded, []byte("\n")...)
	_, err = f.Write(encoded)
	if err != nil {
		return err
	}
	return nil
}

func (ts *TracingSink) UnderlyingProcess() *os.Process {
	return nil // No subprocess for this component.
}
