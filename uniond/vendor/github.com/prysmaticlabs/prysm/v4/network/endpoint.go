package network

import (
	"context"
	"encoding/base64"
	"errors"
	"fmt"
	"net/http"
	"net/url"
	"strings"

	gethRPC "github.com/ethereum/go-ethereum/rpc"
	"github.com/prysmaticlabs/prysm/v4/network/authorization"
	log "github.com/sirupsen/logrus"
)

// Endpoint is an endpoint with authorization data.
type Endpoint struct {
	Url  string
	Auth AuthorizationData
}

// AuthorizationData holds all information necessary to authorize with HTTP.
type AuthorizationData struct {
	Method authorization.Method
	Value  string
	JwtId  string
}

// Equals compares two endpoints for equality.
func (e Endpoint) Equals(other Endpoint) bool {
	return e.Url == other.Url && e.Auth.Equals(other.Auth)
}

// HttpClient creates a http client object dependant
// on the properties of the network endpoint.
func (e Endpoint) HttpClient() *http.Client {
	if e.Auth.Method != authorization.Bearer {
		return http.DefaultClient
	}
	return NewHttpClientWithSecret(e.Auth.Value, e.Auth.JwtId)
}

// Equals compares two authorization data objects for equality.
func (d *AuthorizationData) Equals(other AuthorizationData) bool {
	return d.Method == other.Method && d.Value == other.Value
}

// ToHeaderValue retrieves the value of the authorization header from AuthorizationData.
func (d *AuthorizationData) ToHeaderValue() (string, error) {
	switch d.Method {
	case authorization.Basic:
		return "Basic " + d.Value, nil
	case authorization.Bearer:
		return "Bearer " + d.Value, nil
	case authorization.None:
		return "", nil
	}

	return "", errors.New("could not create HTTP header for unknown authorization method")
}

// HttpEndpoint extracts an httputils.Endpoint from the provider parameter.
func HttpEndpoint(eth1Provider string) Endpoint {
	endpoint := Endpoint{
		Url: "",
		Auth: AuthorizationData{
			Method: authorization.None,
			Value:  "",
		}}

	authValues := strings.Split(eth1Provider, ",")
	endpoint.Url = strings.TrimSpace(authValues[0])
	if len(authValues) > 2 {
		log.Errorf(
			"ETH1 endpoint string can contain one comma for specifying the authorization header to access the provider."+
				" String contains too many commas: %d. Skipping authorization.", len(authValues)-1)
	} else if len(authValues) == 2 {
		switch Method(strings.TrimSpace(authValues[1])) {
		case authorization.Basic:
			basicAuthValues := strings.Split(strings.TrimSpace(authValues[1]), " ")
			if len(basicAuthValues) != 2 {
				log.Errorf("Basic Authentication has incorrect format. Skipping authorization.")
			} else {
				endpoint.Auth.Method = authorization.Basic
				endpoint.Auth.Value = base64.StdEncoding.EncodeToString([]byte(basicAuthValues[1]))
			}
		case authorization.Bearer:
			bearerAuthValues := strings.Split(strings.TrimSpace(authValues[1]), " ")
			if len(bearerAuthValues) != 2 {
				log.Errorf("Bearer Authentication has incorrect format. Skipping authorization.")
			} else {
				endpoint.Auth.Method = authorization.Bearer
				endpoint.Auth.Value = bearerAuthValues[1]
			}
		case authorization.None:
			log.Errorf("Authorization has incorrect format or authorization type is not supported.")
		}
	}
	return endpoint
}

// Method returns the authorizationmethod.AuthorizationMethod corresponding with the parameter value.
func Method(auth string) authorization.Method {
	if strings.HasPrefix(strings.ToLower(auth), "basic") {
		return authorization.Basic
	}
	if strings.HasPrefix(strings.ToLower(auth), "bearer") {
		return authorization.Bearer
	}
	return authorization.None
}

// NewHttpClientWithSecret returns a http client that utilizes
// jwt authentication.
func NewHttpClientWithSecret(secret, id string) *http.Client {
	authTransport := &jwtTransport{
		underlyingTransport: http.DefaultTransport,
		jwtSecret:           []byte(secret),
		jwtId:               id,
	}
	return &http.Client{
		Timeout:   DefaultRPCHTTPTimeout,
		Transport: authTransport,
	}
}

func NewExecutionRPCClient(ctx context.Context, endpoint Endpoint, headers http.Header) (*gethRPC.Client, error) {
	// Need to handle ipc and http
	var client *gethRPC.Client
	u, err := url.Parse(endpoint.Url)
	if err != nil {
		return nil, err
	}
	switch u.Scheme {
	case "http", "https":
		client, err = gethRPC.DialOptions(ctx, endpoint.Url, gethRPC.WithHTTPClient(endpoint.HttpClient()), gethRPC.WithHeaders(headers))
		if err != nil {
			return nil, err
		}
	case "", "ipc":
		client, err = gethRPC.DialIPC(ctx, endpoint.Url)
		if err != nil {
			return nil, err
		}
	default:
		return nil, fmt.Errorf("no known transport for URL scheme %q", u.Scheme)
	}
	return client, nil
}
