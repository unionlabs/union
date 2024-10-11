package httputil

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strconv"

	"github.com/prysmaticlabs/prysm/v4/api"
	log "github.com/sirupsen/logrus"
)

type HasStatusCode interface {
	StatusCode() int
}

// DefaultJsonError is a JSON representation of a simple error value, containing only a message and an error code.
type DefaultJsonError struct {
	Message string `json:"message"`
	Code    int    `json:"code"`
}

func (e *DefaultJsonError) StatusCode() int {
	return e.Code
}

func (e *DefaultJsonError) Error() string {
	return fmt.Sprintf("HTTP request unsuccessful (%d: %s)", e.Code, e.Message)
}

// WriteJson writes the response message in JSON format.
func WriteJson(w http.ResponseWriter, v any) {
	w.Header().Set("Content-Type", api.JsonMediaType)
	w.WriteHeader(http.StatusOK)
	if err := json.NewEncoder(w).Encode(v); err != nil {
		log.WithError(err).Error("Could not write response message")
	}
}

// WriteSsz writes the response message in ssz format
func WriteSsz(w http.ResponseWriter, respSsz []byte, fileName string) {
	w.Header().Set("Content-Length", strconv.Itoa(len(respSsz)))
	w.Header().Set("Content-Type", api.OctetStreamMediaType)
	w.Header().Set("Content-Disposition", "attachment; filename="+fileName)
	if _, err := io.Copy(w, io.NopCloser(bytes.NewReader(respSsz))); err != nil {
		log.WithError(err).Error("could not write response message")
	}
}

// WriteError writes the error by manipulating headers and the body of the final response.
func WriteError(w http.ResponseWriter, errJson HasStatusCode) {
	j, err := json.Marshal(errJson)
	if err != nil {
		log.WithError(err).Error("Could not marshal error message")
		return
	}
	w.Header().Set("Content-Length", strconv.Itoa(len(j)))
	w.Header().Set("Content-Type", api.JsonMediaType)
	w.WriteHeader(errJson.StatusCode())
	if _, err := io.Copy(w, io.NopCloser(bytes.NewReader(j))); err != nil {
		log.WithError(err).Error("Could not write error message")
	}
}
