package clientstats

import (
	"bytes"
	"fmt"
	"io"
	"net/http"
)

type genericWriter struct {
	io.Writer
}

func (gw *genericWriter) Update(r io.Reader) error {
	_, err := io.Copy(gw, r)
	return err
}

// NewGenericClientStatsUpdater can Update any io.Writer.
// It is used by the cli to write to stdout when an http endpoint
// is not provided. The output could be piped into another program
// or used for debugging.
func NewGenericClientStatsUpdater(w io.Writer) Updater {
	return &genericWriter{w}
}

type httpPoster struct {
	url    string
	client *http.Client
}

func (gw *httpPoster) Update(r io.Reader) error {
	resp, err := gw.client.Post(gw.url, "application/json", r)
	if err != nil {
		return err
	}
	defer func() {
		if err := resp.Body.Close(); err != nil {
			return
		}
	}()
	if resp.StatusCode != http.StatusOK {
		buf := new(bytes.Buffer)
		_, err = io.Copy(buf, resp.Body)
		if err != nil {
			return fmt.Errorf("error reading response body for non-200 response status code (%d), err=%s", resp.StatusCode, err)
		}
		return fmt.Errorf("non-200 response status code (%d). response body=%s", resp.StatusCode, buf.String())
	}

	return nil
}

// NewClientStatsHTTPPostUpdater is used when the update endpoint
// is reachable via an HTTP POST request.
func NewClientStatsHTTPPostUpdater(u string) Updater {
	return &httpPoster{url: u, client: http.DefaultClient}
}
