package beacon

import (
	"net/url"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/api/client"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestParseNodeVersion(t *testing.T) {
	cases := []struct {
		name string
		v    string
		err  error
		nv   *NodeVersion
	}{
		{
			name: "empty string",
			v:    "",
			err:  client.ErrInvalidNodeVersion,
		},
		{
			name: "Prysm as the version string",
			v:    "Prysm",
			err:  client.ErrInvalidNodeVersion,
		},
		{
			name: "semver only",
			v:    "v2.0.6",
			err:  client.ErrInvalidNodeVersion,
		},
		{
			name: "complete version",
			v:    "Prysm/v2.0.6 (linux amd64)",
			nv: &NodeVersion{
				implementation: "Prysm",
				semver:         "v2.0.6",
				systemInfo:     "(linux amd64)",
			},
		},
		{
			name: "nimbus version",
			v:    "Nimbus/v22.4.0-039bec-stateofus",
			nv: &NodeVersion{
				implementation: "Nimbus",
				semver:         "v22.4.0-039bec-stateofus",
				systemInfo:     "",
			},
		},
		{
			name: "teku version",
			v:    "teku/v22.3.2/linux-x86_64/oracle-java-11",
			nv: &NodeVersion{
				implementation: "teku",
				semver:         "v22.3.2",
				systemInfo:     "linux-x86_64/oracle-java-11",
			},
		},
		{
			name: "lighthouse version",
			v:    "Lighthouse/v2.1.1-5f628a7/x86_64-linux",
			nv: &NodeVersion{
				implementation: "Lighthouse",
				semver:         "v2.1.1-5f628a7",
				systemInfo:     "x86_64-linux",
			},
		},
	}

	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			nv, err := parseNodeVersion(c.v)
			if c.err != nil {
				require.ErrorIs(t, err, c.err)
			} else {
				require.NoError(t, err)
				require.DeepEqual(t, c.nv, nv)
			}
		})
	}
}

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
			err:     client.ErrMalformedHostname,
		},
		{
			name:    "hostname with port",
			hostArg: "mydomain.org:3500",
			path:    getNodeVersionPath,
			joined:  "http://mydomain.org:3500/eth/v1/node/version",
		},
		{
			name:    "https scheme, hostname with port",
			hostArg: "https://mydomain.org:3500",
			path:    getNodeVersionPath,
			joined:  "https://mydomain.org:3500/eth/v1/node/version",
		},
		{
			name:    "http scheme, hostname without port",
			hostArg: "http://mydomain.org",
			path:    getNodeVersionPath,
			joined:  "http://mydomain.org/eth/v1/node/version",
		},
		{
			name:    "http scheme, trailing slash, hostname without port",
			hostArg: "http://mydomain.org/",
			path:    getNodeVersionPath,
			joined:  "http://mydomain.org/eth/v1/node/version",
		},
		{
			name:    "http scheme, hostname with basic auth creds and no port",
			hostArg: "http://username:pass@mydomain.org/",
			path:    getNodeVersionPath,
			joined:  "http://username:pass@mydomain.org/eth/v1/node/version",
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
