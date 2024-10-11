package client

import (
	"fmt"
	"net/http"
	"time"
)

// ReqOption is a request functional option.
type ReqOption func(*http.Request)

// WithSSZEncoding is a request functional option that adds SSZ encoding header.
func WithSSZEncoding() ReqOption {
	return func(req *http.Request) {
		req.Header.Set("Accept", "application/octet-stream")
	}
}

// WithAuthorizationToken is a request functional option that adds header for authorization token.
func WithAuthorizationToken(token string) ReqOption {
	return func(req *http.Request) {
		req.Header.Set("Authorization", fmt.Sprintf("Bearer %s", token))
	}
}

// ClientOpt is a functional option for the Client type (http.Client wrapper)
type ClientOpt func(*Client)

// WithTimeout sets the .Timeout attribute of the wrapped http.Client.
func WithTimeout(timeout time.Duration) ClientOpt {
	return func(c *Client) {
		c.hc.Timeout = timeout
	}
}

// WithRoundTripper replaces the underlying HTTP's transport with a custom one.
func WithRoundTripper(t http.RoundTripper) ClientOpt {
	return func(c *Client) {
		c.hc.Transport = t
	}
}

// WithAuthenticationToken sets an oauth token to be used.
func WithAuthenticationToken(token string) ClientOpt {
	return func(c *Client) {
		c.token = token
	}
}
