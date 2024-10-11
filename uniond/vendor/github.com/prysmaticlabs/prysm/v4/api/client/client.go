package client

import (
	"context"
	"io"
	"net"
	"net/http"
	"net/url"

	"github.com/pkg/errors"
)

// Client is a wrapper object around the HTTP client.
type Client struct {
	hc      *http.Client
	baseURL *url.URL
	token   string
}

// NewClient constructs a new client with the provided options (ex WithTimeout).
// `host` is the base host + port used to construct request urls. This value can be
// a URL string, or NewClient will assume an http endpoint if just `host:port` is used.
func NewClient(host string, opts ...ClientOpt) (*Client, error) {
	u, err := urlForHost(host)
	if err != nil {
		return nil, err
	}
	c := &Client{
		hc:      &http.Client{},
		baseURL: u,
	}
	for _, o := range opts {
		o(c)
	}
	return c, nil
}

// Token returns the bearer token used for jwt authentication
func (c *Client) Token() string {
	return c.token
}

// BaseURL returns the base url of the client
func (c *Client) BaseURL() *url.URL {
	return c.baseURL
}

// Do execute the request against the http client
func (c *Client) Do(req *http.Request) (*http.Response, error) {
	return c.hc.Do(req)
}

func urlForHost(h string) (*url.URL, error) {
	// try to parse as url (being permissive)
	u, err := url.Parse(h)
	if err == nil && u.Host != "" {
		return u, nil
	}
	// try to parse as host:port
	host, port, err := net.SplitHostPort(h)
	if err != nil {
		return nil, ErrMalformedHostname
	}
	return &url.URL{Host: net.JoinHostPort(host, port), Scheme: "http"}, nil
}

// NodeURL returns a human-readable string representation of the beacon node base url.
func (c *Client) NodeURL() string {
	return c.baseURL.String()
}

// Get is a generic, opinionated GET function to reduce boilerplate amongst the getters in this package.
func (c *Client) Get(ctx context.Context, path string, opts ...ReqOption) ([]byte, error) {
	u := c.baseURL.ResolveReference(&url.URL{Path: path})
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, u.String(), nil)
	if err != nil {
		return nil, err
	}
	for _, o := range opts {
		o(req)
	}
	r, err := c.hc.Do(req)
	if err != nil {
		return nil, err
	}
	defer func() {
		err = r.Body.Close()
	}()
	if r.StatusCode != http.StatusOK {
		return nil, Non200Err(r)
	}
	b, err := io.ReadAll(r.Body)
	if err != nil {
		return nil, errors.Wrap(err, "error reading http response body")
	}
	return b, nil
}
