package beacon_api

import (
	"bytes"
	"context"
	"encoding/json"
	"io"
	"net/http"
	"strings"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/api"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
)

type JsonRestHandler interface {
	Get(ctx context.Context, endpoint string, resp interface{}) error
	Post(ctx context.Context, endpoint string, headers map[string]string, data *bytes.Buffer, resp interface{}) error
}

type BeaconApiJsonRestHandler struct {
	HttpClient http.Client
	Host       string
}

// Get sends a GET request and decodes the response body as a JSON object into the passed in object.
// If an HTTP error is returned, the body is decoded as a DefaultJsonError JSON object and returned as the first return value.
func (c BeaconApiJsonRestHandler) Get(ctx context.Context, endpoint string, resp interface{}) error {
	url := c.Host + endpoint
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		return errors.Wrapf(err, "failed to create request for endpoint %s", url)
	}

	httpResp, err := c.HttpClient.Do(req)
	if err != nil {
		return errors.Wrapf(err, "failed to perform request for endpoint %s", url)
	}
	defer func() {
		if err := httpResp.Body.Close(); err != nil {
			return
		}
	}()

	return decodeResp(httpResp, resp)
}

// Post sends a POST request and decodes the response body as a JSON object into the passed in object.
// If an HTTP error is returned, the body is decoded as a DefaultJsonError JSON object and returned as the first return value.
func (c BeaconApiJsonRestHandler) Post(
	ctx context.Context,
	apiEndpoint string,
	headers map[string]string,
	data *bytes.Buffer,
	resp interface{},
) error {
	if data == nil {
		return errors.New("data is nil")
	}

	url := c.Host + apiEndpoint
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, url, data)
	if err != nil {
		return errors.Wrapf(err, "failed to create request for endpoint %s", url)
	}

	for headerKey, headerValue := range headers {
		req.Header.Set(headerKey, headerValue)
	}
	req.Header.Set("Content-Type", api.JsonMediaType)

	httpResp, err := c.HttpClient.Do(req)
	if err != nil {
		return errors.Wrapf(err, "failed to perform request for endpoint %s", url)
	}
	defer func() {
		if err = httpResp.Body.Close(); err != nil {
			return
		}
	}()

	return decodeResp(httpResp, resp)
}

func decodeResp(httpResp *http.Response, resp interface{}) error {
	body, err := io.ReadAll(httpResp.Body)
	if err != nil {
		return errors.Wrapf(err, "failed to read response body for %s", httpResp.Request.URL)
	}

	if httpResp.Header.Get("Content-Type") != api.JsonMediaType {
		// 2XX codes are a success
		if strings.HasPrefix(httpResp.Status, "2") {
			return nil
		}
		return &httputil.DefaultJsonError{Code: httpResp.StatusCode, Message: string(body)}
	}

	decoder := json.NewDecoder(bytes.NewBuffer(body))
	// non-2XX codes are a failure
	if !strings.HasPrefix(httpResp.Status, "2") {
		errorJson := &httputil.DefaultJsonError{}
		if err = decoder.Decode(errorJson); err != nil {
			return errors.Wrapf(err, "failed to decode response body into error json for %s", httpResp.Request.URL)
		}
		return errorJson
	}
	// resp is nil for requests that do not return anything.
	if resp != nil {
		if err = decoder.Decode(resp); err != nil {
			return errors.Wrapf(err, "failed to decode response body into json for %s", httpResp.Request.URL)
		}
	}

	return nil
}
