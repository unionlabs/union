package client

import (
	"fmt"
	"io"
	"net/http"

	"github.com/pkg/errors"
)

// ErrMalformedHostname is used to indicate if a host name's format is incorrect.
var ErrMalformedHostname = errors.New("hostname must include port, separated by one colon, like example.com:3500")

// ErrNotOK is used to indicate when an HTTP request to the API failed with any non-2xx response code.
// More specific errors may be returned, but an error in reaction to a non-2xx response will always wrap ErrNotOK.
var ErrNotOK = errors.New("did not receive 2xx response from API")

// ErrNotFound specifically means that a '404 - NOT FOUND' response was received from the API.
var ErrNotFound = errors.Wrap(ErrNotOK, "recv 404 NotFound response from API")

// ErrInvalidNodeVersion indicates that the /eth/v1/node/version API response format was not recognized.
var ErrInvalidNodeVersion = errors.New("invalid node version response")

// Non200Err is a function that parses an HTTP response to handle responses that are not 200 with a formatted error.
func Non200Err(response *http.Response) error {
	bodyBytes, err := io.ReadAll(response.Body)
	var body string
	if err != nil {
		body = "(Unable to read response body.)"
	} else {
		body = "response body:\n" + string(bodyBytes)
	}
	msg := fmt.Sprintf("code=%d, url=%s, body=%s", response.StatusCode, response.Request.URL, body)
	switch response.StatusCode {
	case http.StatusNotFound:
		return errors.Wrap(ErrNotFound, msg)
	default:
		return errors.Wrap(ErrNotOK, msg)
	}
}
