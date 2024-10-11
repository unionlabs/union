package prometheus

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"

	"github.com/golang/gddo/httputil"
)

const (
	contentTypePlainText = "text/plain"
	contentTypeJSON      = "application/json"
)

// generatedResponse is a container for response output.
type generatedResponse struct {
	// Err is protocol error, if any.
	Err string `json:"error"`

	// Data is response output, if any.
	Data interface{} `json:"data"`
}

// negotiateContentType parses "Accept:" header and returns preferred content type string.
func negotiateContentType(r *http.Request) string {
	contentTypes := []string{
		contentTypePlainText,
		contentTypeJSON,
	}
	return httputil.NegotiateContentType(r, contentTypes, contentTypePlainText)
}

// writeResponse is content-type aware response writer.
func writeResponse(w http.ResponseWriter, r *http.Request, response generatedResponse) error {
	switch negotiateContentType(r) {
	case contentTypePlainText:
		buf, ok := response.Data.(bytes.Buffer)
		if !ok {
			return fmt.Errorf("unexpected data: %v", response.Data)
		}
		if _, err := w.Write(buf.Bytes()); err != nil {
			return fmt.Errorf("could not write response body: %w", err)
		}
	case contentTypeJSON:
		w.Header().Set("Content-Type", contentTypeJSON)
		if err := json.NewEncoder(w).Encode(response); err != nil {
			return err
		}
	}
	return nil
}
