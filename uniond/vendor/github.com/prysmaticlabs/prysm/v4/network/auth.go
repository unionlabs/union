package network

import (
	"net/http"
	"time"

	"github.com/golang-jwt/jwt/v4"
	"github.com/pkg/errors"
)

// DefaultRPCHTTPTimeout for HTTP requests via an RPC connection to an execution node.
const DefaultRPCHTTPTimeout = time.Second * 30

// This creates a custom HTTP transport which we can attach to our HTTP client
// in order to inject JWT auth strings into our HTTP request headers. Authentication
// is required when interacting with an Ethereum engine API server via HTTP, and JWT
// is chosen as the scheme of choice.
// For more details on the requirements of authentication when using the engine API, see
// the specification here: https://github.com/ethereum/execution-apis/blob/main/src/engine/authentication.md
//
// To use this transport, initialize a new &http.Client{} from the standard library
// and set the Transport field to &jwtTransport{} with values
// http.DefaultTransport and a JWT secret.
type jwtTransport struct {
	underlyingTransport http.RoundTripper
	jwtSecret           []byte
	jwtId               string
}

// RoundTrip ensures our transport implements http.RoundTripper interface from the
// standard library. When used as the transport for an HTTP client, the code below
// will run every time our client makes an HTTP request. This is used to inject
// an JWT bearer token in the Authorization request header of every outgoing request
// our HTTP client makes.
func (t *jwtTransport) RoundTrip(req *http.Request) (*http.Response, error) {
	claims := jwt.MapClaims{
		// Required claim for engine API auth. "iat" stands for issued at
		// and it must be a unix timestamp that is +/- 5 seconds from the current
		// timestamp at the moment the server verifies this value.
		"iat": time.Now().Unix(),
	}
	if len(t.jwtId) > 0 {
		claims["id"] = t.jwtId
	}
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	tokenString, err := token.SignedString(t.jwtSecret)
	if err != nil {
		return nil, errors.Wrap(err, "could not produce signed JWT token")
	}
	req.Header.Set("Authorization", "Bearer "+tokenString)
	return t.underlyingTransport.RoundTrip(req)
}
