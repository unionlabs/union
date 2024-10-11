package network

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/network/authorization"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func TestToHeaderValue(t *testing.T) {
	t.Run("None", func(t *testing.T) {
		data := &AuthorizationData{
			Method: authorization.None,
			Value:  "foo",
		}
		header, err := data.ToHeaderValue()
		require.NoError(t, err)
		assert.Equal(t, "", header)
	})
	t.Run("Basic", func(t *testing.T) {
		data := &AuthorizationData{
			Method: authorization.Basic,
			Value:  "foo",
		}
		header, err := data.ToHeaderValue()
		require.NoError(t, err)
		assert.Equal(t, "Basic foo", header)
	})
	t.Run("Bearer", func(t *testing.T) {
		data := &AuthorizationData{
			Method: authorization.Bearer,
			Value:  "foo",
		}
		header, err := data.ToHeaderValue()
		require.NoError(t, err)
		assert.Equal(t, "Bearer foo", header)
	})
	t.Run("Unknown", func(t *testing.T) {
		data := &AuthorizationData{
			Method: 99,
			Value:  "foo",
		}
		_, err := data.ToHeaderValue()
		require.NotNil(t, err)
	})
}

func TestMethod(t *testing.T) {
	t.Run("None", func(t *testing.T) {
		method := Method("")
		assert.Equal(t, authorization.None, method)
		method = Method("foo")
		assert.Equal(t, authorization.None, method)
	})
	t.Run("Basic", func(t *testing.T) {
		method := Method("Basic")
		assert.Equal(t, authorization.Basic, method)
	})
	t.Run("Basic different text case", func(t *testing.T) {
		method := Method("bAsIc")
		assert.Equal(t, authorization.Basic, method)
	})
	t.Run("Bearer", func(t *testing.T) {
		method := Method("Bearer")
		assert.Equal(t, authorization.Bearer, method)
	})
	t.Run("Bearer different text case", func(t *testing.T) {
		method := Method("bEaReR")
		assert.Equal(t, authorization.Bearer, method)
	})
}

func TestEndpointEquals(t *testing.T) {
	e := Endpoint{
		Url: "Url",
		Auth: AuthorizationData{
			Method: authorization.Basic,
			Value:  "Basic username:password",
		},
	}

	t.Run("equal", func(t *testing.T) {
		other := Endpoint{
			Url: "Url",
			Auth: AuthorizationData{
				Method: authorization.Basic,
				Value:  "Basic username:password",
			},
		}
		assert.Equal(t, true, e.Equals(other))
	})
	t.Run("different URL", func(t *testing.T) {
		other := Endpoint{
			Url: "Different",
			Auth: AuthorizationData{
				Method: authorization.Basic,
				Value:  "Basic username:password",
			},
		}
		assert.Equal(t, false, e.Equals(other))
	})
	t.Run("different auth data", func(t *testing.T) {
		other := Endpoint{
			Url: "Url",
			Auth: AuthorizationData{
				Method: authorization.Bearer,
				Value:  "Bearer token",
			},
		}
		assert.Equal(t, false, e.Equals(other))
	})
}

func TestAuthorizationDataEquals(t *testing.T) {
	d := AuthorizationData{
		Method: authorization.Basic,
		Value:  "username:password",
	}

	t.Run("equal", func(t *testing.T) {
		other := AuthorizationData{
			Method: authorization.Basic,
			Value:  "username:password",
		}
		assert.Equal(t, true, d.Equals(other))
	})
	t.Run("different method", func(t *testing.T) {
		other := AuthorizationData{
			Method: authorization.None,
			Value:  "username:password",
		}
		assert.Equal(t, false, d.Equals(other))
	})
	t.Run("different value", func(t *testing.T) {
		other := AuthorizationData{
			Method: authorization.Basic,
			Value:  "different:different",
		}
		assert.Equal(t, false, d.Equals(other))
	})
}

func TestHttpEndpoint(t *testing.T) {
	hook := logTest.NewGlobal()
	url := "http://test"

	t.Run("URL", func(t *testing.T) {
		endpoint := HttpEndpoint(url)
		assert.Equal(t, url, endpoint.Url)
		assert.Equal(t, authorization.None, endpoint.Auth.Method)
	})
	t.Run("URL with separator", func(t *testing.T) {
		endpoint := HttpEndpoint(url + ",")
		assert.Equal(t, url, endpoint.Url)
		assert.Equal(t, authorization.None, endpoint.Auth.Method)
	})
	t.Run("URL with whitespace", func(t *testing.T) {
		endpoint := HttpEndpoint("   " + url + "   ,")
		assert.Equal(t, url, endpoint.Url)
		assert.Equal(t, authorization.None, endpoint.Auth.Method)
	})
	t.Run("Basic auth", func(t *testing.T) {
		endpoint := HttpEndpoint(url + ",Basic username:password")
		assert.Equal(t, url, endpoint.Url)
		assert.Equal(t, authorization.Basic, endpoint.Auth.Method)
		assert.Equal(t, "dXNlcm5hbWU6cGFzc3dvcmQ=", endpoint.Auth.Value)
	})
	t.Run("Basic auth with whitespace", func(t *testing.T) {
		endpoint := HttpEndpoint(url + ",   Basic username:password   ")
		assert.Equal(t, url, endpoint.Url)
		assert.Equal(t, authorization.Basic, endpoint.Auth.Method)
		assert.Equal(t, "dXNlcm5hbWU6cGFzc3dvcmQ=", endpoint.Auth.Value)
	})
	t.Run("Basic auth with incorrect format", func(t *testing.T) {
		hook.Reset()
		endpoint := HttpEndpoint(url + ",Basic username:password foo")
		assert.Equal(t, url, endpoint.Url)
		assert.Equal(t, authorization.None, endpoint.Auth.Method)
		assert.LogsContain(t, hook, "Skipping authorization")
	})
	t.Run("Bearer auth", func(t *testing.T) {
		endpoint := HttpEndpoint(url + ",Bearer token")
		assert.Equal(t, url, endpoint.Url)
		assert.Equal(t, authorization.Bearer, endpoint.Auth.Method)
		assert.Equal(t, "token", endpoint.Auth.Value)
	})
	t.Run("Bearer auth with whitespace", func(t *testing.T) {
		endpoint := HttpEndpoint(url + ",   Bearer token   ")
		assert.Equal(t, url, endpoint.Url)
		assert.Equal(t, authorization.Bearer, endpoint.Auth.Method)
		assert.Equal(t, "token", endpoint.Auth.Value)
	})
	t.Run("Bearer auth with incorrect format", func(t *testing.T) {
		hook.Reset()
		endpoint := HttpEndpoint(url + ",Bearer token foo")
		assert.Equal(t, url, endpoint.Url)
		assert.Equal(t, authorization.None, endpoint.Auth.Method)
		assert.LogsContain(t, hook, "Skipping authorization")
	})
	t.Run("Too many separators", func(t *testing.T) {
		endpoint := HttpEndpoint(url + ",Bearer token,foo")
		assert.Equal(t, url, endpoint.Url)
		assert.Equal(t, authorization.None, endpoint.Auth.Method)
		assert.LogsContain(t, hook, "Skipping authorization")
	})
}
