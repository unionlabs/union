package rpc

import (
	"encoding/json"
	"fmt"
	"net/http"

	"github.com/prysmaticlabs/prysm/v4/api"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	pb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"go.opencensus.io/trace"
	"google.golang.org/protobuf/types/known/emptypb"
)

// GetVersion returns the beacon node and validator client versions
func (s *Server) GetVersion(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.web.health.GetVersion")
	defer span.End()

	beacon, err := s.beaconNodeClient.GetVersion(ctx, &emptypb.Empty{})
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}

	httputil.WriteJson(w, struct {
		Beacon    string `json:"beacon"`
		Validator string `json:"validator"`
	}{
		Beacon:    beacon.Version,
		Validator: version.Version(),
	})
}

// StreamBeaconLogs from the beacon node via server-side events.
func (s *Server) StreamBeaconLogs(w http.ResponseWriter, r *http.Request) {
	// Wrap service context with a cancel in order to propagate the exiting of
	// this method properly to the beacon node server.
	ctx, span := trace.StartSpan(r.Context(), "validator.web.health.StreamBeaconLogs")
	defer span.End()
	// Set up SSE response headers
	w.Header().Set("Content-Type", api.EventStreamMediaType)
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("Connection", api.KeepAlive)

	// Flush helper function to ensure data is sent to client
	flusher, ok := w.(http.Flusher)
	if !ok {
		httputil.HandleError(w, "Streaming unsupported!", http.StatusInternalServerError)
		return
	}
	// TODO: StreamBeaconLogs grpc will need to be replaced in the future
	client, err := s.beaconNodeHealthClient.StreamBeaconLogs(ctx, &emptypb.Empty{})
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}

	for {
		select {
		case <-s.ctx.Done():
			return
		case <-ctx.Done():
			return
		case <-client.Context().Done():
			return
		default:
			logResp, err := client.Recv()
			if err != nil {
				httputil.HandleError(w, "could not receive beacon logs from stream: "+err.Error(), http.StatusInternalServerError)
				return
			}
			jsonResp, err := json.Marshal(logResp)
			if err != nil {
				httputil.HandleError(w, "could not encode log response into JSON: "+err.Error(), http.StatusInternalServerError)
				return
			}

			// Send the response as an SSE event
			// Assuming resp has a String() method for simplicity
			_, err = fmt.Fprintf(w, "%s\n", jsonResp)
			if err != nil {
				httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
				return
			}
			// Flush the data to the client immediately
			flusher.Flush()
		}
	}
}

// StreamValidatorLogs from the validator client via server-side events.
func (s *Server) StreamValidatorLogs(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.web.health.StreamValidatorLogs")
	defer span.End()

	// Ensure that the writer supports flushing.
	flusher, ok := w.(http.Flusher)
	if !ok {
		httputil.HandleError(w, "Streaming unsupported!", http.StatusInternalServerError)
		return
	}

	ch := make(chan []byte, s.streamLogsBufferSize)
	sub := s.logsStreamer.LogsFeed().Subscribe(ch)
	defer func() {
		sub.Unsubscribe()
		close(ch)
	}()
	// Set up SSE response headers
	w.Header().Set("Content-Type", api.EventStreamMediaType)
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("Connection", api.KeepAlive)

	recentLogs := s.logsStreamer.GetLastFewLogs()
	logStrings := make([]string, len(recentLogs))
	for i, l := range recentLogs {
		logStrings[i] = string(l)
	}
	ls := &pb.LogsResponse{
		Logs: logStrings,
	}
	jsonLogs, err := json.Marshal(ls)
	if err != nil {
		httputil.HandleError(w, "Failed to marshal logs: "+err.Error(), http.StatusInternalServerError)
		return
	}
	_, err = fmt.Fprintf(w, "%s\n", jsonLogs)
	if err != nil {
		httputil.HandleError(w, "Error sending data: "+err.Error(), http.StatusInternalServerError)
		return
	}
	flusher.Flush()

	for {
		select {
		case log := <-ch:
			// Set up SSE response headers
			ls = &pb.LogsResponse{
				Logs: []string{string(log)},
			}
			jsonLogs, err = json.Marshal(ls)
			if err != nil {
				httputil.HandleError(w, "Failed to marshal logs: "+err.Error(), http.StatusInternalServerError)
				return
			}
			_, err = fmt.Fprintf(w, "%s\n", jsonLogs)
			if err != nil {
				httputil.HandleError(w, "Error sending data: "+err.Error(), http.StatusInternalServerError)
				return
			}

			flusher.Flush()
		case <-s.ctx.Done():
			return
		case err := <-sub.Err():
			httputil.HandleError(w, "Subscriber error: "+err.Error(), http.StatusInternalServerError)
			return
		case <-ctx.Done():
			return
		}
	}
}
