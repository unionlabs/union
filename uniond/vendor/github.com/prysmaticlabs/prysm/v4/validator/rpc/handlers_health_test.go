package rpc

import (
	"bytes"
	"context"
	"io"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/golang/mock/gomock"
	"github.com/golang/protobuf/ptypes/empty"
	"github.com/prysmaticlabs/prysm/v4/api"
	"github.com/prysmaticlabs/prysm/v4/io/logs/mock"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	pb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	validatormock "github.com/prysmaticlabs/prysm/v4/testing/validator-mock"
	"google.golang.org/grpc"
)

type MockBeaconNodeHealthClient struct {
	grpc.ClientStream
	logs []*pb.LogsResponse
	err  error
}

func (m *MockBeaconNodeHealthClient) StreamBeaconLogs(ctx context.Context, in *empty.Empty, opts ...grpc.CallOption) (eth.Health_StreamBeaconLogsClient, error) {
	return m, m.err
}

func (m *MockBeaconNodeHealthClient) Recv() (*eth.LogsResponse, error) {
	if len(m.logs) == 0 {
		return nil, io.EOF
	}
	log := m.logs[0]
	m.logs = m.logs[1:]
	return log, nil
}

func (m *MockBeaconNodeHealthClient) SendMsg(_ interface{}) error {
	return m.err
}

func (m *MockBeaconNodeHealthClient) Context() context.Context {
	return context.Background()
}

type flushableResponseRecorder struct {
	*httptest.ResponseRecorder
	flushed bool
}

func (f *flushableResponseRecorder) Flush() {
	f.flushed = true
}

func TestStreamBeaconLogs(t *testing.T) {
	logs := []*pb.LogsResponse{
		{
			Logs: []string{"log1", "log2"},
		},
		{
			Logs: []string{"log3", "log4"},
		},
	}

	mockClient := &MockBeaconNodeHealthClient{
		logs: logs,
		err:  nil,
	}

	// Setting up the mock in the server struct
	s := Server{
		ctx:                    context.Background(),
		beaconNodeHealthClient: mockClient,
	}

	// Create a mock ResponseWriter and Request
	w := &flushableResponseRecorder{
		ResponseRecorder: httptest.NewRecorder(),
	}
	r := httptest.NewRequest("GET", "/v2/validator/health/logs/beacon/stream", nil)

	// Call the function
	s.StreamBeaconLogs(w, r)

	// Assert the results
	resp := w.Result()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("Expected status OK but got %v", resp.StatusCode)
	}
	ct, ok := resp.Header["Content-Type"]
	require.Equal(t, ok, true)
	require.Equal(t, ct[0], api.EventStreamMediaType)
	cn, ok := resp.Header["Connection"]
	require.Equal(t, ok, true)
	require.Equal(t, cn[0], api.KeepAlive)
	body, err := io.ReadAll(resp.Body)
	require.NoError(t, err)
	require.NotNil(t, body)
	require.StringContains(t, `{"logs":["log1","log2"]}`, string(body))
	require.StringContains(t, `{"logs":["log3","log4"]}`, string(body))
	if !w.flushed {
		t.Fatal("Flush was not called")
	}
}

func TestStreamValidatorLogs(t *testing.T) {
	ctx := context.Background()
	mockLogs := [][]byte{
		[]byte("[2023-10-31 10:00:00] INFO: Starting server..."),
		[]byte("[2023-10-31 10:01:23] DEBUG: Database connection established."),
		[]byte("[2023-10-31 10:05:45] WARN: High memory usage detected."),
		[]byte("[2023-10-31 10:10:12] INFO: New user registered: user123."),
		[]byte("[2023-10-31 10:15:30] ERROR: Failed to send email."),
	}
	logStreamer := mock.NewMockStreamer(mockLogs)
	// Setting up the mock in the server struct
	s := Server{
		ctx:                  ctx,
		logsStreamer:         logStreamer,
		streamLogsBufferSize: 100,
	}

	w := &flushableResponseRecorder{
		ResponseRecorder: httptest.NewRecorder(),
	}
	r := httptest.NewRequest("GET", "/v2/validator/health/logs/validator/stream", nil)
	go func() {
		s.StreamValidatorLogs(w, r)
	}()
	// wait for initiation of StreamValidatorLogs
	time.Sleep(100 * time.Millisecond)
	logStreamer.LogsFeed().Send([]byte("Some mock event data"))
	// wait for feed
	time.Sleep(100 * time.Millisecond)
	s.ctx.Done()
	// Assert the results
	resp := w.Result()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("Expected status OK but got %v", resp.StatusCode)
	}
	ct, ok := resp.Header["Content-Type"]
	require.Equal(t, ok, true)
	require.Equal(t, ct[0], api.EventStreamMediaType)
	cn, ok := resp.Header["Connection"]
	require.Equal(t, ok, true)
	require.Equal(t, cn[0], api.KeepAlive)
	// Check if data was written
	body, err := io.ReadAll(resp.Body)
	require.NoError(t, err)
	require.NotNil(t, body)

	require.StringContains(t, `{"logs":["[2023-10-31 10:00:00] INFO: Starting server...","[2023-10-31 10:01:23] DEBUG: Database connection established.",`+
		`"[2023-10-31 10:05:45] WARN: High memory usage detected.","[2023-10-31 10:10:12] INFO: New user registered: user123.","[2023-10-31 10:15:30] ERROR: Failed to send email."]}`, string(body))
	require.StringContains(t, `{"logs":["Some mock event data"]}`, string(body))

	// Check if Flush was called
	if !w.flushed {
		t.Fatal("Flush was not called")
	}

}

func TestServer_GetVersion(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	ctx := context.Background()
	mockNodeClient := validatormock.NewMockNodeClient(ctrl)
	s := Server{
		ctx:              ctx,
		beaconNodeClient: mockNodeClient,
	}
	mockNodeClient.EXPECT().GetVersion(gomock.Any(), gomock.Any()).Return(&eth.Version{
		Version:  "4.10.1",
		Metadata: "beacon node",
	}, nil)
	r := httptest.NewRequest("GET", "/v2/validator/health/version", nil)
	w := httptest.NewRecorder()
	w.Body = &bytes.Buffer{}
	s.GetVersion(w, r)
	resp := w.Result()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("Expected status OK but got %v", resp.StatusCode)
	}
	body, err := io.ReadAll(resp.Body)
	require.NoError(t, err)
	require.NotNil(t, body)
	require.StringContains(t, `{"beacon":"4.10.1","validator":"Prysm/Unknown/Local build. Built at: Moments ago"}`, string(body))
}
