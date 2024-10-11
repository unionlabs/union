package client

import (
	"net/url"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestValidHostname(t *testing.T) {
	cases := []struct {
		name    string
		hostArg string
		path    string
		joined  string
		err     error
	}{
		{
			name:    "hostname without port",
			hostArg: "mydomain.org",
			err:     ErrMalformedHostname,
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			cl, err := NewClient(c.hostArg)
			if c.err != nil {
				require.ErrorIs(t, err, c.err)
				return
			}
			require.NoError(t, err)
			require.Equal(t, c.joined, cl.BaseURL().ResolveReference(&url.URL{Path: c.path}).String())
		})
	}
}

func TestWithAuthenticationToken(t *testing.T) {
	cl, err := NewClient("https://www.offchainlabs.com:3500", WithAuthenticationToken("my token"))
	require.NoError(t, err)
	require.Equal(t, cl.Token(), "my token")
}

func TestBaseURL(t *testing.T) {
	cl, err := NewClient("https://www.offchainlabs.com:3500")
	require.NoError(t, err)
	require.Equal(t, "www.offchainlabs.com", cl.BaseURL().Hostname())
	require.Equal(t, "3500", cl.BaseURL().Port())
}
