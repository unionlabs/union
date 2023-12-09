package testutil

import (
	"bytes"
	"fmt"
	"io"
	"net/http"
)

// GetRequestWithHeaders defines a wrapper around an HTTP GET request with a provided URL
// and custom headers
// An error is returned if the request or reading the body fails.
func GetRequestWithHeaders(url string, headers map[string]string) ([]byte, error) {
	req, err := http.NewRequest("GET", url, nil)
	if err != nil {
		return nil, err
	}

	client := &http.Client{}

	for key, value := range headers {
		req.Header.Set(key, value)
	}

	res, err := client.Do(req)
	if err != nil {
		return nil, err
	}

	body, err := io.ReadAll(res.Body)
	if err != nil {
		return nil, err
	}

	if err = res.Body.Close(); err != nil {
		return nil, err
	}

	return body, nil
}

// GetRequest defines a wrapper around an HTTP GET request with a provided URL.
// An error is returned if the request or reading the body fails.
func GetRequest(url string) ([]byte, error) {
	res, err := http.Get(url) //nolint:gosec // only used for testing
	if err != nil {
		return nil, err
	}
	defer func() {
		_ = res.Body.Close()
	}()

	body, err := io.ReadAll(res.Body)
	if err != nil {
		return nil, err
	}

	return body, nil
}

// PostRequest defines a wrapper around an HTTP POST request with a provided URL and data.
// An error is returned if the request or reading the body fails.
func PostRequest(url, contentType string, data []byte) ([]byte, error) {
	res, err := http.Post(url, contentType, bytes.NewBuffer(data)) //nolint:gosec // only used	for testing
	if err != nil {
		return nil, fmt.Errorf("error while sending post request: %w", err)
	}
	defer func() {
		_ = res.Body.Close()
	}()

	bz, err := io.ReadAll(res.Body)
	if err != nil {
		return nil, fmt.Errorf("error reading response body: %w", err)
	}

	return bz, nil
}
