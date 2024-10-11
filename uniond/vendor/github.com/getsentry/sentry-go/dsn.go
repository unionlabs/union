package sentry

import (
	"encoding/json"
	"fmt"
	"net/url"
	"strconv"
	"strings"
	"time"
)

type scheme string

const (
	schemeHTTP  scheme = "http"
	schemeHTTPS scheme = "https"
)

func (scheme scheme) defaultPort() int {
	switch scheme {
	case schemeHTTPS:
		return 443
	case schemeHTTP:
		return 80
	default:
		return 80
	}
}

// DsnParseError represents an error that occurs if a Sentry
// DSN cannot be parsed.
type DsnParseError struct {
	Message string
}

func (e DsnParseError) Error() string {
	return "[Sentry] DsnParseError: " + e.Message
}

// Dsn is used as the remote address source to client transport.
type Dsn struct {
	scheme    scheme
	publicKey string
	secretKey string
	host      string
	port      int
	path      string
	projectID string
}

// NewDsn creates a Dsn by parsing rawURL. Most users will never call this
// function directly. It is provided for use in custom Transport
// implementations.
func NewDsn(rawURL string) (*Dsn, error) {
	// Parse
	parsedURL, err := url.Parse(rawURL)
	if err != nil {
		return nil, &DsnParseError{fmt.Sprintf("invalid url: %v", err)}
	}

	// Scheme
	var scheme scheme
	switch parsedURL.Scheme {
	case "http":
		scheme = schemeHTTP
	case "https":
		scheme = schemeHTTPS
	default:
		return nil, &DsnParseError{"invalid scheme"}
	}

	// PublicKey
	publicKey := parsedURL.User.Username()
	if publicKey == "" {
		return nil, &DsnParseError{"empty username"}
	}

	// SecretKey
	var secretKey string
	if parsedSecretKey, ok := parsedURL.User.Password(); ok {
		secretKey = parsedSecretKey
	}

	// Host
	host := parsedURL.Hostname()
	if host == "" {
		return nil, &DsnParseError{"empty host"}
	}

	// Port
	var port int
	if parsedURL.Port() != "" {
		port, err = strconv.Atoi(parsedURL.Port())
		if err != nil {
			return nil, &DsnParseError{"invalid port"}
		}
	} else {
		port = scheme.defaultPort()
	}

	// ProjectID
	if parsedURL.Path == "" || parsedURL.Path == "/" {
		return nil, &DsnParseError{"empty project id"}
	}
	pathSegments := strings.Split(parsedURL.Path[1:], "/")
	projectID := pathSegments[len(pathSegments)-1]

	if projectID == "" {
		return nil, &DsnParseError{"empty project id"}
	}

	// Path
	var path string
	if len(pathSegments) > 1 {
		path = "/" + strings.Join(pathSegments[0:len(pathSegments)-1], "/")
	}

	return &Dsn{
		scheme:    scheme,
		publicKey: publicKey,
		secretKey: secretKey,
		host:      host,
		port:      port,
		path:      path,
		projectID: projectID,
	}, nil
}

// String formats Dsn struct into a valid string url.
func (dsn Dsn) String() string {
	var url string
	url += fmt.Sprintf("%s://%s", dsn.scheme, dsn.publicKey)
	if dsn.secretKey != "" {
		url += fmt.Sprintf(":%s", dsn.secretKey)
	}
	url += fmt.Sprintf("@%s", dsn.host)
	if dsn.port != dsn.scheme.defaultPort() {
		url += fmt.Sprintf(":%d", dsn.port)
	}
	if dsn.path != "" {
		url += dsn.path
	}
	url += fmt.Sprintf("/%s", dsn.projectID)
	return url
}

// Get the scheme of the DSN.
func (dsn Dsn) GetScheme() string {
	return string(dsn.scheme)
}

// Get the public key of the DSN.
func (dsn Dsn) GetPublicKey() string {
	return dsn.publicKey
}

// Get the secret key of the DSN.
func (dsn Dsn) GetSecretKey() string {
	return dsn.secretKey
}

// Get the host of the DSN.
func (dsn Dsn) GetHost() string {
	return dsn.host
}

// Get the port of the DSN.
func (dsn Dsn) GetPort() int {
	return dsn.port
}

// Get the path of the DSN.
func (dsn Dsn) GetPath() string {
	return dsn.path
}

// Get the project ID of the DSN.
func (dsn Dsn) GetProjectID() string {
	return dsn.projectID
}

// GetAPIURL returns the URL of the envelope endpoint of the project
// associated with the DSN.
func (dsn Dsn) GetAPIURL() *url.URL {
	var rawURL string
	rawURL += fmt.Sprintf("%s://%s", dsn.scheme, dsn.host)
	if dsn.port != dsn.scheme.defaultPort() {
		rawURL += fmt.Sprintf(":%d", dsn.port)
	}
	if dsn.path != "" {
		rawURL += dsn.path
	}
	rawURL += fmt.Sprintf("/api/%s/%s/", dsn.projectID, "envelope")
	parsedURL, _ := url.Parse(rawURL)
	return parsedURL
}

// RequestHeaders returns all the necessary headers that have to be used in the transport when seinding events
// to the /store endpoint.
//
// Deprecated: This method shall only be used if you want to implement your own transport that sends events to
// the /store endpoint. If you're using the transport provided by the SDK, all necessary headers to authenticate
// against the /envelope endpoint are added automatically.
func (dsn Dsn) RequestHeaders() map[string]string {
	auth := fmt.Sprintf("Sentry sentry_version=%s, sentry_timestamp=%d, "+
		"sentry_client=sentry.go/%s, sentry_key=%s", apiVersion, time.Now().Unix(), SDKVersion, dsn.publicKey)

	if dsn.secretKey != "" {
		auth = fmt.Sprintf("%s, sentry_secret=%s", auth, dsn.secretKey)
	}

	return map[string]string{
		"Content-Type":  "application/json",
		"X-Sentry-Auth": auth,
	}
}

// MarshalJSON converts the Dsn struct to JSON.
func (dsn Dsn) MarshalJSON() ([]byte, error) {
	return json.Marshal(dsn.String())
}

// UnmarshalJSON converts JSON data to the Dsn struct.
func (dsn *Dsn) UnmarshalJSON(data []byte) error {
	var str string
	_ = json.Unmarshal(data, &str)
	newDsn, err := NewDsn(str)
	if err != nil {
		return err
	}
	*dsn = *newDsn
	return nil
}
