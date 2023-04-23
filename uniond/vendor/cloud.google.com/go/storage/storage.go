// Copyright 2014 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package storage

import (
	"bytes"
	"context"
	"crypto"
	"crypto/rand"
	"crypto/rsa"
	"crypto/sha256"
	"crypto/x509"
	"encoding/base64"
	"encoding/hex"
	"encoding/pem"
	"errors"
	"fmt"
	"net/http"
	"net/url"
	"os"
	"reflect"
	"regexp"
	"sort"
	"strconv"
	"strings"
	"time"
	"unicode/utf8"

	"cloud.google.com/go/internal/optional"
	"cloud.google.com/go/internal/trace"
	"cloud.google.com/go/storage/internal"
	storagepb "cloud.google.com/go/storage/internal/apiv2/stubs"
	"github.com/googleapis/gax-go/v2"
	"golang.org/x/oauth2/google"
	"google.golang.org/api/googleapi"
	"google.golang.org/api/option"
	"google.golang.org/api/option/internaloption"
	raw "google.golang.org/api/storage/v1"
	"google.golang.org/api/transport"
	htransport "google.golang.org/api/transport/http"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/types/known/fieldmaskpb"
	"google.golang.org/protobuf/types/known/timestamppb"
)

// Methods which can be used in signed URLs.
var signedURLMethods = map[string]bool{"DELETE": true, "GET": true, "HEAD": true, "POST": true, "PUT": true}

var (
	// ErrBucketNotExist indicates that the bucket does not exist.
	ErrBucketNotExist = errors.New("storage: bucket doesn't exist")
	// ErrObjectNotExist indicates that the object does not exist.
	ErrObjectNotExist = errors.New("storage: object doesn't exist")
	// errMethodNotSupported indicates that the method called is not currently supported by the client.
	// TODO: Export this error when launching the transport-agnostic client.
	errMethodNotSupported = errors.New("storage: method is not currently supported")
	// errMethodNotValid indicates that given HTTP method is not valid.
	errMethodNotValid = fmt.Errorf("storage: HTTP method should be one of %v", reflect.ValueOf(signedURLMethods).MapKeys())
)

var userAgent = fmt.Sprintf("gcloud-golang-storage/%s", internal.Version)

const (
	// ScopeFullControl grants permissions to manage your
	// data and permissions in Google Cloud Storage.
	ScopeFullControl = raw.DevstorageFullControlScope

	// ScopeReadOnly grants permissions to
	// view your data in Google Cloud Storage.
	ScopeReadOnly = raw.DevstorageReadOnlyScope

	// ScopeReadWrite grants permissions to manage your
	// data in Google Cloud Storage.
	ScopeReadWrite = raw.DevstorageReadWriteScope

	// aes256Algorithm is the AES256 encryption algorithm used with the
	// Customer-Supplied Encryption Keys feature.
	aes256Algorithm = "AES256"

	// defaultGen indicates the latest object generation by default,
	// using a negative value.
	defaultGen = int64(-1)
)

// TODO: remove this once header with invocation ID is applied to all methods.
func setClientHeader(headers http.Header) {
	headers.Set("x-goog-api-client", xGoogDefaultHeader)
}

// Client is a client for interacting with Google Cloud Storage.
//
// Clients should be reused instead of created as needed.
// The methods of Client are safe for concurrent use by multiple goroutines.
type Client struct {
	hc  *http.Client
	raw *raw.Service
	// Scheme describes the scheme under the current host.
	scheme string
	// ReadHost is the default host used on the reader.
	readHost string
	// May be nil.
	creds *google.Credentials
	retry *retryConfig

	// tc is the transport-agnostic client implemented with either gRPC or HTTP.
	tc storageClient
	// useGRPC flags whether the client uses gRPC. This is needed while the
	// integration piece is only partially complete.
	// TODO: remove before merging to main.
	useGRPC bool
}

// NewClient creates a new Google Cloud Storage client.
// The default scope is ScopeFullControl. To use a different scope, like
// ScopeReadOnly, use option.WithScopes.
//
// Clients should be reused instead of created as needed. The methods of Client
// are safe for concurrent use by multiple goroutines.
func NewClient(ctx context.Context, opts ...option.ClientOption) (*Client, error) {

	// Use the experimental gRPC client if the env var is set.
	// This is an experimental API and not intended for public use.
	if withGRPC := os.Getenv("STORAGE_USE_GRPC"); withGRPC != "" {
		return newGRPCClient(ctx, opts...)
	}

	var creds *google.Credentials

	// In general, it is recommended to use raw.NewService instead of htransport.NewClient
	// since raw.NewService configures the correct default endpoints when initializing the
	// internal http client. However, in our case, "NewRangeReader" in reader.go needs to
	// access the http client directly to make requests, so we create the client manually
	// here so it can be re-used by both reader.go and raw.NewService. This means we need to
	// manually configure the default endpoint options on the http client. Furthermore, we
	// need to account for STORAGE_EMULATOR_HOST override when setting the default endpoints.
	if host := os.Getenv("STORAGE_EMULATOR_HOST"); host == "" {
		// Prepend default options to avoid overriding options passed by the user.
		opts = append([]option.ClientOption{option.WithScopes(ScopeFullControl, "https://www.googleapis.com/auth/cloud-platform"), option.WithUserAgent(userAgent)}, opts...)

		opts = append(opts, internaloption.WithDefaultEndpoint("https://storage.googleapis.com/storage/v1/"))
		opts = append(opts, internaloption.WithDefaultMTLSEndpoint("https://storage.mtls.googleapis.com/storage/v1/"))

		// Don't error out here. The user may have passed in their own HTTP
		// client which does not auth with ADC or other common conventions.
		c, err := transport.Creds(ctx, opts...)
		if err == nil {
			creds = c
			opts = append(opts, internaloption.WithCredentials(creds))
		}
	} else {
		var hostURL *url.URL

		if strings.Contains(host, "://") {
			h, err := url.Parse(host)
			if err != nil {
				return nil, err
			}
			hostURL = h
		} else {
			// Add scheme for user if not supplied in STORAGE_EMULATOR_HOST
			// URL is only parsed correctly if it has a scheme, so we build it ourselves
			hostURL = &url.URL{Scheme: "http", Host: host}
		}

		hostURL.Path = "storage/v1/"
		endpoint := hostURL.String()

		// Append the emulator host as default endpoint for the user
		opts = append([]option.ClientOption{option.WithoutAuthentication()}, opts...)

		opts = append(opts, internaloption.WithDefaultEndpoint(endpoint))
		opts = append(opts, internaloption.WithDefaultMTLSEndpoint(endpoint))
	}

	// htransport selects the correct endpoint among WithEndpoint (user override), WithDefaultEndpoint, and WithDefaultMTLSEndpoint.
	hc, ep, err := htransport.NewClient(ctx, opts...)
	if err != nil {
		return nil, fmt.Errorf("dialing: %w", err)
	}
	// RawService should be created with the chosen endpoint to take account of user override.
	rawService, err := raw.NewService(ctx, option.WithEndpoint(ep), option.WithHTTPClient(hc))
	if err != nil {
		return nil, fmt.Errorf("storage client: %w", err)
	}
	// Update readHost and scheme with the chosen endpoint.
	u, err := url.Parse(ep)
	if err != nil {
		return nil, fmt.Errorf("supplied endpoint %q is not valid: %w", ep, err)
	}

	tc, err := newHTTPStorageClient(ctx, withClientOptions(opts...))
	if err != nil {
		return nil, fmt.Errorf("storage: %w", err)
	}

	return &Client{
		hc:       hc,
		raw:      rawService,
		scheme:   u.Scheme,
		readHost: u.Host,
		creds:    creds,
		tc:       tc,
	}, nil
}

// newGRPCClient creates a new Storage client that initializes a gRPC-based
// client. Calls that have not been implemented in gRPC will panic.
//
// This is an experimental API and not intended for public use.
func newGRPCClient(ctx context.Context, opts ...option.ClientOption) (*Client, error) {
	opts = append(defaultGRPCOptions(), opts...)
	tc, err := newGRPCStorageClient(ctx, withClientOptions(opts...))
	if err != nil {
		return nil, err
	}

	return &Client{tc: tc, useGRPC: true}, nil
}

// Close closes the Client.
//
// Close need not be called at program exit.
func (c *Client) Close() error {
	// Set fields to nil so that subsequent uses will panic.
	c.hc = nil
	c.raw = nil
	c.creds = nil
	if c.tc != nil {
		return c.tc.Close()
	}
	return nil
}

// SigningScheme determines the API version to use when signing URLs.
type SigningScheme int

const (
	// SigningSchemeDefault is presently V2 and will change to V4 in the future.
	SigningSchemeDefault SigningScheme = iota

	// SigningSchemeV2 uses the V2 scheme to sign URLs.
	SigningSchemeV2

	// SigningSchemeV4 uses the V4 scheme to sign URLs.
	SigningSchemeV4
)

// URLStyle determines the style to use for the signed URL. pathStyle is the
// default. All non-default options work with V4 scheme only. See
// https://cloud.google.com/storage/docs/request-endpoints for details.
type URLStyle interface {
	// host should return the host portion of the signed URL, not including
	// the scheme (e.g. storage.googleapis.com).
	host(bucket string) string

	// path should return the path portion of the signed URL, which may include
	// both the bucket and object name or only the object name depending on the
	// style.
	path(bucket, object string) string
}

type pathStyle struct{}

type virtualHostedStyle struct{}

type bucketBoundHostname struct {
	hostname string
}

func (s pathStyle) host(bucket string) string {
	if host := os.Getenv("STORAGE_EMULATOR_HOST"); host != "" {
		return stripScheme(host)
	}

	return "storage.googleapis.com"
}

func (s virtualHostedStyle) host(bucket string) string {
	if host := os.Getenv("STORAGE_EMULATOR_HOST"); host != "" {
		return bucket + "." + stripScheme(host)
	}

	return bucket + ".storage.googleapis.com"
}

func (s bucketBoundHostname) host(bucket string) string {
	return s.hostname
}

func (s pathStyle) path(bucket, object string) string {
	p := bucket
	if object != "" {
		p += "/" + object
	}
	return p
}

func (s virtualHostedStyle) path(bucket, object string) string {
	return object
}

func (s bucketBoundHostname) path(bucket, object string) string {
	return object
}

// PathStyle is the default style, and will generate a URL of the form
// "storage.googleapis.com/<bucket-name>/<object-name>".
func PathStyle() URLStyle {
	return pathStyle{}
}

// VirtualHostedStyle generates a URL relative to the bucket's virtual
// hostname, e.g. "<bucket-name>.storage.googleapis.com/<object-name>".
func VirtualHostedStyle() URLStyle {
	return virtualHostedStyle{}
}

// BucketBoundHostname generates a URL with a custom hostname tied to a
// specific GCS bucket. The desired hostname should be passed in using the
// hostname argument. Generated urls will be of the form
// "<bucket-bound-hostname>/<object-name>". See
// https://cloud.google.com/storage/docs/request-endpoints#cname and
// https://cloud.google.com/load-balancing/docs/https/adding-backend-buckets-to-load-balancers
// for details. Note that for CNAMEs, only HTTP is supported, so Insecure must
// be set to true.
func BucketBoundHostname(hostname string) URLStyle {
	return bucketBoundHostname{hostname: hostname}
}

// Strips the scheme from a host if it contains it
func stripScheme(host string) string {
	if strings.Contains(host, "://") {
		host = strings.SplitN(host, "://", 2)[1]
	}
	return host
}

// SignedURLOptions allows you to restrict the access to the signed URL.
type SignedURLOptions struct {
	// GoogleAccessID represents the authorizer of the signed URL generation.
	// It is typically the Google service account client email address from
	// the Google Developers Console in the form of "xxx@developer.gserviceaccount.com".
	// Required.
	GoogleAccessID string

	// PrivateKey is the Google service account private key. It is obtainable
	// from the Google Developers Console.
	// At https://console.developers.google.com/project/<your-project-id>/apiui/credential,
	// create a service account client ID or reuse one of your existing service account
	// credentials. Click on the "Generate new P12 key" to generate and download
	// a new private key. Once you download the P12 file, use the following command
	// to convert it into a PEM file.
	//
	//    $ openssl pkcs12 -in key.p12 -passin pass:notasecret -out key.pem -nodes
	//
	// Provide the contents of the PEM file as a byte slice.
	// Exactly one of PrivateKey or SignBytes must be non-nil.
	PrivateKey []byte

	// SignBytes is a function for implementing custom signing. For example, if
	// your application is running on Google App Engine, you can use
	// appengine's internal signing function:
	//     ctx := appengine.NewContext(request)
	//     acc, _ := appengine.ServiceAccount(ctx)
	//     url, err := SignedURL("bucket", "object", &SignedURLOptions{
	//     	GoogleAccessID: acc,
	//     	SignBytes: func(b []byte) ([]byte, error) {
	//     		_, signedBytes, err := appengine.SignBytes(ctx, b)
	//     		return signedBytes, err
	//     	},
	//     	// etc.
	//     })
	//
	// Exactly one of PrivateKey or SignBytes must be non-nil.
	SignBytes func([]byte) ([]byte, error)

	// Method is the HTTP method to be used with the signed URL.
	// Signed URLs can be used with GET, HEAD, PUT, and DELETE requests.
	// Required.
	Method string

	// Expires is the expiration time on the signed URL. It must be
	// a datetime in the future. For SigningSchemeV4, the expiration may be no
	// more than seven days in the future.
	// Required.
	Expires time.Time

	// ContentType is the content type header the client must provide
	// to use the generated signed URL.
	// Optional.
	ContentType string

	// Headers is a list of extension headers the client must provide
	// in order to use the generated signed URL. Each must be a string of the
	// form "key:values", with multiple values separated by a semicolon.
	// Optional.
	Headers []string

	// QueryParameters is a map of additional query parameters. When
	// SigningScheme is V4, this is used in computing the signature, and the
	// client must use the same query parameters when using the generated signed
	// URL.
	// Optional.
	QueryParameters url.Values

	// MD5 is the base64 encoded MD5 checksum of the file.
	// If provided, the client should provide the exact value on the request
	// header in order to use the signed URL.
	// Optional.
	MD5 string

	// Style provides options for the type of URL to use. Options are
	// PathStyle (default), BucketBoundHostname, and VirtualHostedStyle. See
	// https://cloud.google.com/storage/docs/request-endpoints for details.
	// Only supported for V4 signing.
	// Optional.
	Style URLStyle

	// Insecure determines whether the signed URL should use HTTPS (default) or
	// HTTP.
	// Only supported for V4 signing.
	// Optional.
	Insecure bool

	// Scheme determines the version of URL signing to use. Default is
	// SigningSchemeV2.
	Scheme SigningScheme
}

func (opts *SignedURLOptions) clone() *SignedURLOptions {
	return &SignedURLOptions{
		GoogleAccessID:  opts.GoogleAccessID,
		SignBytes:       opts.SignBytes,
		PrivateKey:      opts.PrivateKey,
		Method:          opts.Method,
		Expires:         opts.Expires,
		ContentType:     opts.ContentType,
		Headers:         opts.Headers,
		QueryParameters: opts.QueryParameters,
		MD5:             opts.MD5,
		Style:           opts.Style,
		Insecure:        opts.Insecure,
		Scheme:          opts.Scheme,
	}
}

var (
	tabRegex = regexp.MustCompile(`[\t]+`)
	// I was tempted to call this spacex. :)
	spaceRegex = regexp.MustCompile(` +`)

	canonicalHeaderRegexp    = regexp.MustCompile(`(?i)^(x-goog-[^:]+):(.*)?$`)
	excludedCanonicalHeaders = map[string]bool{
		"x-goog-encryption-key":        true,
		"x-goog-encryption-key-sha256": true,
	}
)

// v2SanitizeHeaders applies the specifications for canonical extension headers at
// https://cloud.google.com/storage/docs/access-control/signed-urls-v2#about-canonical-extension-headers
func v2SanitizeHeaders(hdrs []string) []string {
	headerMap := map[string][]string{}
	for _, hdr := range hdrs {
		// No leading or trailing whitespaces.
		sanitizedHeader := strings.TrimSpace(hdr)

		var header, value string
		// Only keep canonical headers, discard any others.
		headerMatches := canonicalHeaderRegexp.FindStringSubmatch(sanitizedHeader)
		if len(headerMatches) == 0 {
			continue
		}
		header = headerMatches[1]
		value = headerMatches[2]

		header = strings.ToLower(strings.TrimSpace(header))
		value = strings.TrimSpace(value)

		if excludedCanonicalHeaders[header] {
			// Do not keep any deliberately excluded canonical headers when signing.
			continue
		}

		if len(value) > 0 {
			// Remove duplicate headers by appending the values of duplicates
			// in their order of appearance.
			headerMap[header] = append(headerMap[header], value)
		}
	}

	var sanitizedHeaders []string
	for header, values := range headerMap {
		// There should be no spaces around the colon separating the header name
		// from the header value or around the values themselves. The values
		// should be separated by commas.
		//
		// NOTE: The semantics for headers without a value are not clear.
		// However from specifications these should be edge-cases anyway and we
		// should assume that there will be no canonical headers using empty
		// values. Any such headers are discarded at the regexp stage above.
		sanitizedHeaders = append(sanitizedHeaders, fmt.Sprintf("%s:%s", header, strings.Join(values, ",")))
	}
	sort.Strings(sanitizedHeaders)
	return sanitizedHeaders
}

// v4SanitizeHeaders applies the specifications for canonical extension headers
// at https://cloud.google.com/storage/docs/authentication/canonical-requests#about-headers.
//
// V4 does a couple things differently from V2:
//   - Headers get sorted by key, instead of by key:value. We do this in
//     signedURLV4.
//   - There's no canonical regexp: we simply split headers on :.
//   - We don't exclude canonical headers.
//   - We replace leading and trailing spaces in header values, like v2, but also
//     all intermediate space duplicates get stripped. That is, there's only ever
//     a single consecutive space.
func v4SanitizeHeaders(hdrs []string) []string {
	headerMap := map[string][]string{}
	for _, hdr := range hdrs {
		// No leading or trailing whitespaces.
		sanitizedHeader := strings.TrimSpace(hdr)

		var key, value string
		headerMatches := strings.Split(sanitizedHeader, ":")
		if len(headerMatches) < 2 {
			continue
		}

		key = headerMatches[0]
		value = headerMatches[1]

		key = strings.ToLower(strings.TrimSpace(key))
		value = strings.TrimSpace(value)
		value = string(spaceRegex.ReplaceAll([]byte(value), []byte(" ")))
		value = string(tabRegex.ReplaceAll([]byte(value), []byte("\t")))

		if len(value) > 0 {
			// Remove duplicate headers by appending the values of duplicates
			// in their order of appearance.
			headerMap[key] = append(headerMap[key], value)
		}
	}

	var sanitizedHeaders []string
	for header, values := range headerMap {
		// There should be no spaces around the colon separating the header name
		// from the header value or around the values themselves. The values
		// should be separated by commas.
		//
		// NOTE: The semantics for headers without a value are not clear.
		// However from specifications these should be edge-cases anyway and we
		// should assume that there will be no canonical headers using empty
		// values. Any such headers are discarded at the regexp stage above.
		sanitizedHeaders = append(sanitizedHeaders, fmt.Sprintf("%s:%s", header, strings.Join(values, ",")))
	}
	return sanitizedHeaders
}

// SignedURL returns a URL for the specified object. Signed URLs allow anyone
// access to a restricted resource for a limited time without needing a
// Google account or signing in. For more information about signed URLs, see
// https://cloud.google.com/storage/docs/accesscontrol#signed_urls_query_string_authentication
// If initializing a Storage Client, instead use the Bucket.SignedURL method
// which uses the Client's credentials to handle authentication.
func SignedURL(bucket, object string, opts *SignedURLOptions) (string, error) {
	now := utcNow()
	if err := validateOptions(opts, now); err != nil {
		return "", err
	}

	switch opts.Scheme {
	case SigningSchemeV2:
		opts.Headers = v2SanitizeHeaders(opts.Headers)
		return signedURLV2(bucket, object, opts)
	case SigningSchemeV4:
		opts.Headers = v4SanitizeHeaders(opts.Headers)
		return signedURLV4(bucket, object, opts, now)
	default: // SigningSchemeDefault
		opts.Headers = v2SanitizeHeaders(opts.Headers)
		return signedURLV2(bucket, object, opts)
	}
}

func validateOptions(opts *SignedURLOptions, now time.Time) error {
	if opts == nil {
		return errors.New("storage: missing required SignedURLOptions")
	}
	if opts.GoogleAccessID == "" {
		return errors.New("storage: missing required GoogleAccessID")
	}
	if (opts.PrivateKey == nil) == (opts.SignBytes == nil) {
		return errors.New("storage: exactly one of PrivateKey or SignedBytes must be set")
	}
	opts.Method = strings.ToUpper(opts.Method)
	if _, ok := signedURLMethods[opts.Method]; !ok {
		return errMethodNotValid
	}
	if opts.Expires.IsZero() {
		return errors.New("storage: missing required expires option")
	}
	if opts.MD5 != "" {
		md5, err := base64.StdEncoding.DecodeString(opts.MD5)
		if err != nil || len(md5) != 16 {
			return errors.New("storage: invalid MD5 checksum")
		}
	}
	if opts.Style == nil {
		opts.Style = PathStyle()
	}
	if _, ok := opts.Style.(pathStyle); !ok && opts.Scheme == SigningSchemeV2 {
		return errors.New("storage: only path-style URLs are permitted with SigningSchemeV2")
	}
	if opts.Scheme == SigningSchemeV4 {
		cutoff := now.Add(604801 * time.Second) // 7 days + 1 second
		if !opts.Expires.Before(cutoff) {
			return errors.New("storage: expires must be within seven days from now")
		}
	}
	return nil
}

const (
	iso8601      = "20060102T150405Z"
	yearMonthDay = "20060102"
)

// utcNow returns the current time in UTC and is a variable to allow for
// reassignment in tests to provide deterministic signed URL values.
var utcNow = func() time.Time {
	return time.Now().UTC()
}

// extractHeaderNames takes in a series of key:value headers and returns the
// header names only.
func extractHeaderNames(kvs []string) []string {
	var res []string
	for _, header := range kvs {
		nameValue := strings.Split(header, ":")
		res = append(res, nameValue[0])
	}
	return res
}

// pathEncodeV4 creates an encoded string that matches the v4 signature spec.
// Following the spec precisely is necessary in order to ensure that the URL
// and signing string are correctly formed, and Go's url.PathEncode and
// url.QueryEncode don't generate an exact match without some additional logic.
func pathEncodeV4(path string) string {
	segments := strings.Split(path, "/")
	var encodedSegments []string
	for _, s := range segments {
		encodedSegments = append(encodedSegments, url.QueryEscape(s))
	}
	encodedStr := strings.Join(encodedSegments, "/")
	encodedStr = strings.Replace(encodedStr, "+", "%20", -1)
	return encodedStr
}

// signedURLV4 creates a signed URL using the sigV4 algorithm.
func signedURLV4(bucket, name string, opts *SignedURLOptions, now time.Time) (string, error) {
	buf := &bytes.Buffer{}
	fmt.Fprintf(buf, "%s\n", opts.Method)

	u := &url.URL{Path: opts.Style.path(bucket, name)}
	u.RawPath = pathEncodeV4(u.Path)

	// Note: we have to add a / here because GCS does so auto-magically, despite
	// our encoding not doing so (and we have to exactly match their
	// canonical query).
	fmt.Fprintf(buf, "/%s\n", u.RawPath)

	headerNames := append(extractHeaderNames(opts.Headers), "host")
	if opts.ContentType != "" {
		headerNames = append(headerNames, "content-type")
	}
	if opts.MD5 != "" {
		headerNames = append(headerNames, "content-md5")
	}
	sort.Strings(headerNames)
	signedHeaders := strings.Join(headerNames, ";")
	timestamp := now.Format(iso8601)
	credentialScope := fmt.Sprintf("%s/auto/storage/goog4_request", now.Format(yearMonthDay))
	canonicalQueryString := url.Values{
		"X-Goog-Algorithm":     {"GOOG4-RSA-SHA256"},
		"X-Goog-Credential":    {fmt.Sprintf("%s/%s", opts.GoogleAccessID, credentialScope)},
		"X-Goog-Date":          {timestamp},
		"X-Goog-Expires":       {fmt.Sprintf("%d", int(opts.Expires.Sub(now).Seconds()))},
		"X-Goog-SignedHeaders": {signedHeaders},
	}
	// Add user-supplied query parameters to the canonical query string. For V4,
	// it's necessary to include these.
	for k, v := range opts.QueryParameters {
		canonicalQueryString[k] = append(canonicalQueryString[k], v...)
	}
	// url.Values.Encode escaping is correct, except that a space must be replaced
	// by `%20` rather than `+`.
	escapedQuery := strings.Replace(canonicalQueryString.Encode(), "+", "%20", -1)
	fmt.Fprintf(buf, "%s\n", escapedQuery)

	// Fill in the hostname based on the desired URL style.
	u.Host = opts.Style.host(bucket)

	// Fill in the URL scheme.
	if opts.Insecure {
		u.Scheme = "http"
	} else {
		u.Scheme = "https"
	}

	var headersWithValue []string
	headersWithValue = append(headersWithValue, "host:"+u.Host)
	headersWithValue = append(headersWithValue, opts.Headers...)
	if opts.ContentType != "" {
		headersWithValue = append(headersWithValue, "content-type:"+opts.ContentType)
	}
	if opts.MD5 != "" {
		headersWithValue = append(headersWithValue, "content-md5:"+opts.MD5)
	}
	// Trim extra whitespace from headers and replace with a single space.
	var trimmedHeaders []string
	for _, h := range headersWithValue {
		trimmedHeaders = append(trimmedHeaders, strings.Join(strings.Fields(h), " "))
	}
	canonicalHeaders := strings.Join(sortHeadersByKey(trimmedHeaders), "\n")
	fmt.Fprintf(buf, "%s\n\n", canonicalHeaders)
	fmt.Fprintf(buf, "%s\n", signedHeaders)

	// If the user provides a value for X-Goog-Content-SHA256, we must use
	// that value in the request string. If not, we use UNSIGNED-PAYLOAD.
	sha256Header := false
	for _, h := range trimmedHeaders {
		if strings.HasPrefix(strings.ToLower(h), "x-goog-content-sha256") && strings.Contains(h, ":") {
			sha256Header = true
			fmt.Fprintf(buf, "%s", strings.SplitN(h, ":", 2)[1])
			break
		}
	}
	if !sha256Header {
		fmt.Fprint(buf, "UNSIGNED-PAYLOAD")
	}

	sum := sha256.Sum256(buf.Bytes())
	hexDigest := hex.EncodeToString(sum[:])
	signBuf := &bytes.Buffer{}
	fmt.Fprint(signBuf, "GOOG4-RSA-SHA256\n")
	fmt.Fprintf(signBuf, "%s\n", timestamp)
	fmt.Fprintf(signBuf, "%s\n", credentialScope)
	fmt.Fprintf(signBuf, "%s", hexDigest)

	signBytes := opts.SignBytes
	if opts.PrivateKey != nil {
		key, err := parseKey(opts.PrivateKey)
		if err != nil {
			return "", err
		}
		signBytes = func(b []byte) ([]byte, error) {
			sum := sha256.Sum256(b)
			return rsa.SignPKCS1v15(
				rand.Reader,
				key,
				crypto.SHA256,
				sum[:],
			)
		}
	}
	b, err := signBytes(signBuf.Bytes())
	if err != nil {
		return "", err
	}
	signature := hex.EncodeToString(b)
	canonicalQueryString.Set("X-Goog-Signature", string(signature))
	u.RawQuery = canonicalQueryString.Encode()
	return u.String(), nil
}

// takes a list of headerKey:headervalue1,headervalue2,etc and sorts by header
// key.
func sortHeadersByKey(hdrs []string) []string {
	headersMap := map[string]string{}
	var headersKeys []string
	for _, h := range hdrs {
		parts := strings.Split(h, ":")
		k := parts[0]
		v := parts[1]
		headersMap[k] = v
		headersKeys = append(headersKeys, k)
	}
	sort.Strings(headersKeys)
	var sorted []string
	for _, k := range headersKeys {
		v := headersMap[k]
		sorted = append(sorted, fmt.Sprintf("%s:%s", k, v))
	}
	return sorted
}

func signedURLV2(bucket, name string, opts *SignedURLOptions) (string, error) {
	signBytes := opts.SignBytes
	if opts.PrivateKey != nil {
		key, err := parseKey(opts.PrivateKey)
		if err != nil {
			return "", err
		}
		signBytes = func(b []byte) ([]byte, error) {
			sum := sha256.Sum256(b)
			return rsa.SignPKCS1v15(
				rand.Reader,
				key,
				crypto.SHA256,
				sum[:],
			)
		}
	}

	u := &url.URL{
		Path: fmt.Sprintf("/%s/%s", bucket, name),
	}

	buf := &bytes.Buffer{}
	fmt.Fprintf(buf, "%s\n", opts.Method)
	fmt.Fprintf(buf, "%s\n", opts.MD5)
	fmt.Fprintf(buf, "%s\n", opts.ContentType)
	fmt.Fprintf(buf, "%d\n", opts.Expires.Unix())
	if len(opts.Headers) > 0 {
		fmt.Fprintf(buf, "%s\n", strings.Join(opts.Headers, "\n"))
	}
	fmt.Fprintf(buf, "%s", u.String())

	b, err := signBytes(buf.Bytes())
	if err != nil {
		return "", err
	}
	encoded := base64.StdEncoding.EncodeToString(b)
	u.Scheme = "https"
	u.Host = PathStyle().host(bucket)
	q := u.Query()
	q.Set("GoogleAccessId", opts.GoogleAccessID)
	q.Set("Expires", fmt.Sprintf("%d", opts.Expires.Unix()))
	q.Set("Signature", string(encoded))
	u.RawQuery = q.Encode()
	return u.String(), nil
}

// ObjectHandle provides operations on an object in a Google Cloud Storage bucket.
// Use BucketHandle.Object to get a handle.
type ObjectHandle struct {
	c              *Client
	bucket         string
	object         string
	acl            ACLHandle
	gen            int64 // a negative value indicates latest
	conds          *Conditions
	encryptionKey  []byte // AES-256 key
	userProject    string // for requester-pays buckets
	readCompressed bool   // Accept-Encoding: gzip
	retry          *retryConfig
}

// ACL provides access to the object's access control list.
// This controls who can read and write this object.
// This call does not perform any network operations.
func (o *ObjectHandle) ACL() *ACLHandle {
	return &o.acl
}

// Generation returns a new ObjectHandle that operates on a specific generation
// of the object.
// By default, the handle operates on the latest generation. Not
// all operations work when given a specific generation; check the API
// endpoints at https://cloud.google.com/storage/docs/json_api/ for details.
func (o *ObjectHandle) Generation(gen int64) *ObjectHandle {
	o2 := *o
	o2.gen = gen
	return &o2
}

// If returns a new ObjectHandle that applies a set of preconditions.
// Preconditions already set on the ObjectHandle are ignored.
// Operations on the new handle will return an error if the preconditions are not
// satisfied. See https://cloud.google.com/storage/docs/generations-preconditions
// for more details.
func (o *ObjectHandle) If(conds Conditions) *ObjectHandle {
	o2 := *o
	o2.conds = &conds
	return &o2
}

// Key returns a new ObjectHandle that uses the supplied encryption
// key to encrypt and decrypt the object's contents.
//
// Encryption key must be a 32-byte AES-256 key.
// See https://cloud.google.com/storage/docs/encryption for details.
func (o *ObjectHandle) Key(encryptionKey []byte) *ObjectHandle {
	o2 := *o
	o2.encryptionKey = encryptionKey
	return &o2
}

// Attrs returns meta information about the object.
// ErrObjectNotExist will be returned if the object is not found.
func (o *ObjectHandle) Attrs(ctx context.Context) (attrs *ObjectAttrs, err error) {
	ctx = trace.StartSpan(ctx, "cloud.google.com/go/storage.Object.Attrs")
	defer func() { trace.EndSpan(ctx, err) }()

	if err := o.validate(); err != nil {
		return nil, err
	}
	opts := makeStorageOpts(true, o.retry, o.userProject)
	return o.c.tc.GetObject(ctx, o.bucket, o.object, o.gen, o.encryptionKey, o.conds, opts...)
}

// Update updates an object with the provided attributes. See
// ObjectAttrsToUpdate docs for details on treatment of zero values.
// ErrObjectNotExist will be returned if the object is not found.
func (o *ObjectHandle) Update(ctx context.Context, uattrs ObjectAttrsToUpdate) (oa *ObjectAttrs, err error) {
	ctx = trace.StartSpan(ctx, "cloud.google.com/go/storage.Object.Update")
	defer func() { trace.EndSpan(ctx, err) }()

	if err := o.validate(); err != nil {
		return nil, err
	}
	isIdempotent := o.conds != nil && o.conds.MetagenerationMatch != 0
	opts := makeStorageOpts(isIdempotent, o.retry, o.userProject)
	return o.c.tc.UpdateObject(ctx, o.bucket, o.object, &uattrs, o.gen, o.encryptionKey, o.conds, opts...)
}

// BucketName returns the name of the bucket.
func (o *ObjectHandle) BucketName() string {
	return o.bucket
}

// ObjectName returns the name of the object.
func (o *ObjectHandle) ObjectName() string {
	return o.object
}

// ObjectAttrsToUpdate is used to update the attributes of an object.
// Only fields set to non-nil values will be updated.
// For all fields except CustomTime, set the field to its zero value to delete
// it. CustomTime cannot be deleted or changed to an earlier time once set.
//
// For example, to change ContentType and delete ContentEncoding and
// Metadata, use
//
//	ObjectAttrsToUpdate{
//	    ContentType: "text/html",
//	    ContentEncoding: "",
//	    Metadata: map[string]string{},
//	}
type ObjectAttrsToUpdate struct {
	EventBasedHold     optional.Bool
	TemporaryHold      optional.Bool
	ContentType        optional.String
	ContentLanguage    optional.String
	ContentEncoding    optional.String
	ContentDisposition optional.String
	CacheControl       optional.String
	CustomTime         time.Time         // Cannot be deleted or backdated from its current value.
	Metadata           map[string]string // Set to map[string]string{} to delete.
	ACL                []ACLRule

	// If not empty, applies a predefined set of access controls. ACL must be nil.
	// See https://cloud.google.com/storage/docs/json_api/v1/objects/patch.
	PredefinedACL string
}

// Delete deletes the single specified object.
func (o *ObjectHandle) Delete(ctx context.Context) error {
	if err := o.validate(); err != nil {
		return err
	}
	// Delete is idempotent if GenerationMatch or Generation have been passed in.
	// The default generation is negative to get the latest version of the object.
	isIdempotent := (o.conds != nil && o.conds.GenerationMatch != 0) || o.gen >= 0
	opts := makeStorageOpts(isIdempotent, o.retry, o.userProject)
	return o.c.tc.DeleteObject(ctx, o.bucket, o.object, o.gen, o.conds, opts...)
}

// ReadCompressed when true causes the read to happen without decompressing.
func (o *ObjectHandle) ReadCompressed(compressed bool) *ObjectHandle {
	o2 := *o
	o2.readCompressed = compressed
	return &o2
}

// NewWriter returns a storage Writer that writes to the GCS object
// associated with this ObjectHandle.
//
// A new object will be created unless an object with this name already exists.
// Otherwise any previous object with the same name will be replaced.
// The object will not be available (and any previous object will remain)
// until Close has been called.
//
// Attributes can be set on the object by modifying the returned Writer's
// ObjectAttrs field before the first call to Write. If no ContentType
// attribute is specified, the content type will be automatically sniffed
// using net/http.DetectContentType.
//
// Note that each Writer allocates an internal buffer of size Writer.ChunkSize.
// See the ChunkSize docs for more information.
//
// It is the caller's responsibility to call Close when writing is done. To
// stop writing without saving the data, cancel the context.
func (o *ObjectHandle) NewWriter(ctx context.Context) *Writer {
	return &Writer{
		ctx:         ctx,
		o:           o,
		donec:       make(chan struct{}),
		ObjectAttrs: ObjectAttrs{Name: o.object},
		ChunkSize:   googleapi.DefaultUploadChunkSize,
	}
}

func (o *ObjectHandle) validate() error {
	if o.bucket == "" {
		return errors.New("storage: bucket name is empty")
	}
	if o.object == "" {
		return errors.New("storage: object name is empty")
	}
	if !utf8.ValidString(o.object) {
		return fmt.Errorf("storage: object name %q is not valid UTF-8", o.object)
	}
	return nil
}

// parseKey converts the binary contents of a private key file to an
// *rsa.PrivateKey. It detects whether the private key is in a PEM container or
// not. If so, it extracts the private key from PEM container before
// conversion. It only supports PEM containers with no passphrase.
func parseKey(key []byte) (*rsa.PrivateKey, error) {
	if block, _ := pem.Decode(key); block != nil {
		key = block.Bytes
	}
	parsedKey, err := x509.ParsePKCS8PrivateKey(key)
	if err != nil {
		parsedKey, err = x509.ParsePKCS1PrivateKey(key)
		if err != nil {
			return nil, err
		}
	}
	parsed, ok := parsedKey.(*rsa.PrivateKey)
	if !ok {
		return nil, errors.New("oauth2: private key is invalid")
	}
	return parsed, nil
}

// toRawObject copies the editable attributes from o to the raw library's Object type.
func (o *ObjectAttrs) toRawObject(bucket string) *raw.Object {
	var ret string
	if !o.RetentionExpirationTime.IsZero() {
		ret = o.RetentionExpirationTime.Format(time.RFC3339)
	}
	var ct string
	if !o.CustomTime.IsZero() {
		ct = o.CustomTime.Format(time.RFC3339)
	}
	return &raw.Object{
		Bucket:                  bucket,
		Name:                    o.Name,
		EventBasedHold:          o.EventBasedHold,
		TemporaryHold:           o.TemporaryHold,
		RetentionExpirationTime: ret,
		ContentType:             o.ContentType,
		ContentEncoding:         o.ContentEncoding,
		ContentLanguage:         o.ContentLanguage,
		CacheControl:            o.CacheControl,
		ContentDisposition:      o.ContentDisposition,
		StorageClass:            o.StorageClass,
		Acl:                     toRawObjectACL(o.ACL),
		Metadata:                o.Metadata,
		CustomTime:              ct,
	}
}

// toProtoObject copies the editable attributes from o to the proto library's Object type.
func (o *ObjectAttrs) toProtoObject(b string) *storagepb.Object {
	// For now, there are only globally unique buckets, and "_" is the alias
	// project ID for such buckets. If the bucket is not provided, like in the
	// destination ObjectAttrs of a Copy, do not attempt to format it.
	if b != "" {
		b = bucketResourceName(globalProjectAlias, b)
	}

	return &storagepb.Object{
		Bucket:              b,
		Name:                o.Name,
		EventBasedHold:      proto.Bool(o.EventBasedHold),
		TemporaryHold:       o.TemporaryHold,
		ContentType:         o.ContentType,
		ContentEncoding:     o.ContentEncoding,
		ContentLanguage:     o.ContentLanguage,
		CacheControl:        o.CacheControl,
		ContentDisposition:  o.ContentDisposition,
		StorageClass:        o.StorageClass,
		Acl:                 toProtoObjectACL(o.ACL),
		Metadata:            o.Metadata,
		CreateTime:          toProtoTimestamp(o.Created),
		CustomTime:          toProtoTimestamp(o.CustomTime),
		DeleteTime:          toProtoTimestamp(o.Deleted),
		RetentionExpireTime: toProtoTimestamp(o.RetentionExpirationTime),
		UpdateTime:          toProtoTimestamp(o.Updated),
		KmsKey:              o.KMSKeyName,
		Generation:          o.Generation,
		Size:                o.Size,
	}
}

// toProtoObject copies the attributes to update from uattrs to the proto library's Object type.
func (uattrs *ObjectAttrsToUpdate) toProtoObject(bucket, object string) *storagepb.Object {
	o := &storagepb.Object{
		Name:   object,
		Bucket: bucket,
	}
	if uattrs == nil {
		return o
	}

	if uattrs.EventBasedHold != nil {
		o.EventBasedHold = proto.Bool(optional.ToBool(uattrs.EventBasedHold))
	}
	if uattrs.TemporaryHold != nil {
		o.TemporaryHold = optional.ToBool(uattrs.TemporaryHold)
	}
	if uattrs.ContentType != nil {
		o.ContentType = optional.ToString(uattrs.ContentType)
	}
	if uattrs.ContentLanguage != nil {
		o.ContentLanguage = optional.ToString(uattrs.ContentLanguage)
	}
	if uattrs.ContentEncoding != nil {
		o.ContentEncoding = optional.ToString(uattrs.ContentEncoding)
	}
	if uattrs.ContentDisposition != nil {
		o.ContentDisposition = optional.ToString(uattrs.ContentDisposition)
	}
	if uattrs.CacheControl != nil {
		o.CacheControl = optional.ToString(uattrs.CacheControl)
	}
	if !uattrs.CustomTime.IsZero() {
		o.CustomTime = toProtoTimestamp(uattrs.CustomTime)
	}
	if uattrs.ACL != nil {
		o.Acl = toProtoObjectACL(uattrs.ACL)
	}

	// TODO(cathyo): Handle metadata. Pending b/230510191.

	return o
}

// ObjectAttrs represents the metadata for a Google Cloud Storage (GCS) object.
type ObjectAttrs struct {
	// Bucket is the name of the bucket containing this GCS object.
	// This field is read-only.
	Bucket string

	// Name is the name of the object within the bucket.
	// This field is read-only.
	Name string

	// ContentType is the MIME type of the object's content.
	ContentType string

	// ContentLanguage is the content language of the object's content.
	ContentLanguage string

	// CacheControl is the Cache-Control header to be sent in the response
	// headers when serving the object data.
	CacheControl string

	// EventBasedHold specifies whether an object is under event-based hold. New
	// objects created in a bucket whose DefaultEventBasedHold is set will
	// default to that value.
	EventBasedHold bool

	// TemporaryHold specifies whether an object is under temporary hold. While
	// this flag is set to true, the object is protected against deletion and
	// overwrites.
	TemporaryHold bool

	// RetentionExpirationTime is a server-determined value that specifies the
	// earliest time that the object's retention period expires.
	// This is a read-only field.
	RetentionExpirationTime time.Time

	// ACL is the list of access control rules for the object.
	ACL []ACLRule

	// If not empty, applies a predefined set of access controls. It should be set
	// only when writing, copying or composing an object. When copying or composing,
	// it acts as the destinationPredefinedAcl parameter.
	// PredefinedACL is always empty for ObjectAttrs returned from the service.
	// See https://cloud.google.com/storage/docs/json_api/v1/objects/insert
	// for valid values.
	PredefinedACL string

	// Owner is the owner of the object. This field is read-only.
	//
	// If non-zero, it is in the form of "user-<userId>".
	Owner string

	// Size is the length of the object's content. This field is read-only.
	Size int64

	// ContentEncoding is the encoding of the object's content.
	ContentEncoding string

	// ContentDisposition is the optional Content-Disposition header of the object
	// sent in the response headers.
	ContentDisposition string

	// MD5 is the MD5 hash of the object's content. This field is read-only,
	// except when used from a Writer. If set on a Writer, the uploaded
	// data is rejected if its MD5 hash does not match this field.
	MD5 []byte

	// CRC32C is the CRC32 checksum of the object's content using the Castagnoli93
	// polynomial. This field is read-only, except when used from a Writer or
	// Composer. In those cases, if the SendCRC32C field in the Writer or Composer
	// is set to is true, the uploaded data is rejected if its CRC32C hash does
	// not match this field.
	//
	// Note: For a Writer, SendCRC32C must be set to true BEFORE the first call to
	// Writer.Write() in order to send the checksum.
	CRC32C uint32

	// MediaLink is an URL to the object's content. This field is read-only.
	MediaLink string

	// Metadata represents user-provided metadata, in key/value pairs.
	// It can be nil if no metadata is provided.
	//
	// For object downloads using Reader, metadata keys are sent as headers.
	// Therefore, avoid setting metadata keys using characters that are not valid
	// for headers. See https://www.rfc-editor.org/rfc/rfc7230#section-3.2.6.
	Metadata map[string]string

	// Generation is the generation number of the object's content.
	// This field is read-only.
	Generation int64

	// Metageneration is the version of the metadata for this
	// object at this generation. This field is used for preconditions
	// and for detecting changes in metadata. A metageneration number
	// is only meaningful in the context of a particular generation
	// of a particular object. This field is read-only.
	Metageneration int64

	// StorageClass is the storage class of the object. This defines
	// how objects are stored and determines the SLA and the cost of storage.
	// Typical values are "STANDARD", "NEARLINE", "COLDLINE" and "ARCHIVE".
	// Defaults to "STANDARD".
	// See https://cloud.google.com/storage/docs/storage-classes for all
	// valid values.
	StorageClass string

	// Created is the time the object was created. This field is read-only.
	Created time.Time

	// Deleted is the time the object was deleted.
	// If not deleted, it is the zero value. This field is read-only.
	Deleted time.Time

	// Updated is the creation or modification time of the object.
	// For buckets with versioning enabled, changing an object's
	// metadata does not change this property. This field is read-only.
	Updated time.Time

	// CustomerKeySHA256 is the base64-encoded SHA-256 hash of the
	// customer-supplied encryption key for the object. It is empty if there is
	// no customer-supplied encryption key.
	// See // https://cloud.google.com/storage/docs/encryption for more about
	// encryption in Google Cloud Storage.
	CustomerKeySHA256 string

	// Cloud KMS key name, in the form
	// projects/P/locations/L/keyRings/R/cryptoKeys/K, used to encrypt this object,
	// if the object is encrypted by such a key.
	//
	// Providing both a KMSKeyName and a customer-supplied encryption key (via
	// ObjectHandle.Key) will result in an error when writing an object.
	KMSKeyName string

	// Prefix is set only for ObjectAttrs which represent synthetic "directory
	// entries" when iterating over buckets using Query.Delimiter. See
	// ObjectIterator.Next. When set, no other fields in ObjectAttrs will be
	// populated.
	Prefix string

	// Etag is the HTTP/1.1 Entity tag for the object.
	// This field is read-only.
	Etag string

	// A user-specified timestamp which can be applied to an object. This is
	// typically set in order to use the CustomTimeBefore and DaysSinceCustomTime
	// LifecycleConditions to manage object lifecycles.
	//
	// CustomTime cannot be removed once set on an object. It can be updated to a
	// later value but not to an earlier one. For more information see
	// https://cloud.google.com/storage/docs/metadata#custom-time .
	CustomTime time.Time

	// ComponentCount is the number of objects contained within a composite object.
	// For non-composite objects, the value will be zero.
	// This field is read-only.
	ComponentCount int64
}

// convertTime converts a time in RFC3339 format to time.Time.
// If any error occurs in parsing, the zero-value time.Time is silently returned.
func convertTime(t string) time.Time {
	var r time.Time
	if t != "" {
		r, _ = time.Parse(time.RFC3339, t)
	}
	return r
}

func convertProtoTime(t *timestamppb.Timestamp) time.Time {
	var r time.Time
	if t != nil {
		r = t.AsTime()
	}
	return r
}

func toProtoTimestamp(t time.Time) *timestamppb.Timestamp {
	if t.IsZero() {
		return nil
	}

	return timestamppb.New(t)
}

func newObject(o *raw.Object) *ObjectAttrs {
	if o == nil {
		return nil
	}
	owner := ""
	if o.Owner != nil {
		owner = o.Owner.Entity
	}
	md5, _ := base64.StdEncoding.DecodeString(o.Md5Hash)
	crc32c, _ := decodeUint32(o.Crc32c)
	var sha256 string
	if o.CustomerEncryption != nil {
		sha256 = o.CustomerEncryption.KeySha256
	}
	return &ObjectAttrs{
		Bucket:                  o.Bucket,
		Name:                    o.Name,
		ContentType:             o.ContentType,
		ContentLanguage:         o.ContentLanguage,
		CacheControl:            o.CacheControl,
		EventBasedHold:          o.EventBasedHold,
		TemporaryHold:           o.TemporaryHold,
		RetentionExpirationTime: convertTime(o.RetentionExpirationTime),
		ACL:                     toObjectACLRules(o.Acl),
		Owner:                   owner,
		ContentEncoding:         o.ContentEncoding,
		ContentDisposition:      o.ContentDisposition,
		Size:                    int64(o.Size),
		MD5:                     md5,
		CRC32C:                  crc32c,
		MediaLink:               o.MediaLink,
		Metadata:                o.Metadata,
		Generation:              o.Generation,
		Metageneration:          o.Metageneration,
		StorageClass:            o.StorageClass,
		CustomerKeySHA256:       sha256,
		KMSKeyName:              o.KmsKeyName,
		Created:                 convertTime(o.TimeCreated),
		Deleted:                 convertTime(o.TimeDeleted),
		Updated:                 convertTime(o.Updated),
		Etag:                    o.Etag,
		CustomTime:              convertTime(o.CustomTime),
		ComponentCount:          o.ComponentCount,
	}
}

func newObjectFromProto(o *storagepb.Object) *ObjectAttrs {
	if o == nil {
		return nil
	}
	return &ObjectAttrs{
		Bucket:                  parseBucketName(o.Bucket),
		Name:                    o.Name,
		ContentType:             o.ContentType,
		ContentLanguage:         o.ContentLanguage,
		CacheControl:            o.CacheControl,
		EventBasedHold:          o.GetEventBasedHold(),
		TemporaryHold:           o.TemporaryHold,
		RetentionExpirationTime: convertProtoTime(o.GetRetentionExpireTime()),
		ACL:                     toObjectACLRulesFromProto(o.GetAcl()),
		Owner:                   o.GetOwner().GetEntity(),
		ContentEncoding:         o.ContentEncoding,
		ContentDisposition:      o.ContentDisposition,
		Size:                    int64(o.Size),
		MD5:                     o.GetChecksums().GetMd5Hash(),
		CRC32C:                  o.GetChecksums().GetCrc32C(),
		Metadata:                o.Metadata,
		Generation:              o.Generation,
		Metageneration:          o.Metageneration,
		StorageClass:            o.StorageClass,
		// CustomerKeySHA256 needs to be presented as base64 encoded, but the response from gRPC is not.
		CustomerKeySHA256: base64.StdEncoding.EncodeToString(o.GetCustomerEncryption().GetKeySha256Bytes()),
		KMSKeyName:        o.GetKmsKey(),
		Created:           convertProtoTime(o.GetCreateTime()),
		Deleted:           convertProtoTime(o.GetDeleteTime()),
		Updated:           convertProtoTime(o.GetUpdateTime()),
		CustomTime:        convertProtoTime(o.GetCustomTime()),
		ComponentCount:    int64(o.ComponentCount),
	}
}

// Decode a uint32 encoded in Base64 in big-endian byte order.
func decodeUint32(b64 string) (uint32, error) {
	d, err := base64.StdEncoding.DecodeString(b64)
	if err != nil {
		return 0, err
	}
	if len(d) != 4 {
		return 0, fmt.Errorf("storage: %q does not encode a 32-bit value", d)
	}
	return uint32(d[0])<<24 + uint32(d[1])<<16 + uint32(d[2])<<8 + uint32(d[3]), nil
}

// Encode a uint32 as Base64 in big-endian byte order.
func encodeUint32(u uint32) string {
	b := []byte{byte(u >> 24), byte(u >> 16), byte(u >> 8), byte(u)}
	return base64.StdEncoding.EncodeToString(b)
}

// Projection is enumerated type for Query.Projection.
type Projection int

const (
	// ProjectionDefault returns all fields of objects.
	ProjectionDefault Projection = iota

	// ProjectionFull returns all fields of objects.
	ProjectionFull

	// ProjectionNoACL returns all fields of objects except for Owner and ACL.
	ProjectionNoACL
)

func (p Projection) String() string {
	switch p {
	case ProjectionFull:
		return "full"
	case ProjectionNoACL:
		return "noAcl"
	default:
		return ""
	}
}

// Query represents a query to filter objects from a bucket.
type Query struct {
	// Delimiter returns results in a directory-like fashion.
	// Results will contain only objects whose names, aside from the
	// prefix, do not contain delimiter. Objects whose names,
	// aside from the prefix, contain delimiter will have their name,
	// truncated after the delimiter, returned in prefixes.
	// Duplicate prefixes are omitted.
	// Optional.
	Delimiter string

	// Prefix is the prefix filter to query objects
	// whose names begin with this prefix.
	// Optional.
	Prefix string

	// Versions indicates whether multiple versions of the same
	// object will be included in the results.
	Versions bool

	// attrSelection is used to select only specific fields to be returned by
	// the query. It is set by the user calling calling SetAttrSelection. These
	// are used by toFieldMask and toFieldSelection for gRPC and HTTP/JSON
	// clients repsectively.
	attrSelection []string

	// StartOffset is used to filter results to objects whose names are
	// lexicographically equal to or after startOffset. If endOffset is also set,
	// the objects listed will have names between startOffset (inclusive) and
	// endOffset (exclusive).
	StartOffset string

	// EndOffset is used to filter results to objects whose names are
	// lexicographically before endOffset. If startOffset is also set, the objects
	// listed will have names between startOffset (inclusive) and endOffset (exclusive).
	EndOffset string

	// Projection defines the set of properties to return. It will default to ProjectionFull,
	// which returns all properties. Passing ProjectionNoACL will omit Owner and ACL,
	// which may improve performance when listing many objects.
	Projection Projection

	// IncludeTrailingDelimiter controls how objects which end in a single
	// instance of Delimiter (for example, if Query.Delimiter = "/" and the
	// object name is "foo/bar/") are included in the results. By default, these
	// objects only show up as prefixes. If IncludeTrailingDelimiter is set to
	// true, they will also be included as objects and their metadata will be
	// populated in the returned ObjectAttrs.
	IncludeTrailingDelimiter bool
}

// attrToFieldMap maps the field names of ObjectAttrs to the underlying field
// names in the API call. Only the ObjectAttrs field names are visible to users
// because they are already part of the public API of the package.
var attrToFieldMap = map[string]string{
	"Bucket":                  "bucket",
	"Name":                    "name",
	"ContentType":             "contentType",
	"ContentLanguage":         "contentLanguage",
	"CacheControl":            "cacheControl",
	"EventBasedHold":          "eventBasedHold",
	"TemporaryHold":           "temporaryHold",
	"RetentionExpirationTime": "retentionExpirationTime",
	"ACL":                     "acl",
	"Owner":                   "owner",
	"ContentEncoding":         "contentEncoding",
	"ContentDisposition":      "contentDisposition",
	"Size":                    "size",
	"MD5":                     "md5Hash",
	"CRC32C":                  "crc32c",
	"MediaLink":               "mediaLink",
	"Metadata":                "metadata",
	"Generation":              "generation",
	"Metageneration":          "metageneration",
	"StorageClass":            "storageClass",
	"CustomerKeySHA256":       "customerEncryption",
	"KMSKeyName":              "kmsKeyName",
	"Created":                 "timeCreated",
	"Deleted":                 "timeDeleted",
	"Updated":                 "updated",
	"Etag":                    "etag",
	"CustomTime":              "customTime",
	"ComponentCount":          "componentCount",
}

// attrToProtoFieldMap maps the field names of ObjectAttrs to the underlying field
// names in the protobuf Object message.
var attrToProtoFieldMap = map[string]string{
	"Name":                    "name",
	"Bucket":                  "bucket",
	"Etag":                    "etag",
	"Generation":              "generation",
	"Metageneration":          "metageneration",
	"StorageClass":            "storage_class",
	"Size":                    "size",
	"ContentEncoding":         "content_encoding",
	"ContentDisposition":      "content_disposition",
	"CacheControl":            "cache_control",
	"ACL":                     "acl",
	"ContentLanguage":         "content_language",
	"Deleted":                 "delete_time",
	"ContentType":             "content_type",
	"Created":                 "create_time",
	"CRC32C":                  "checksums.crc32c",
	"MD5":                     "checksums.md5_hash",
	"Updated":                 "update_time",
	"KMSKeyName":              "kms_key",
	"TemporaryHold":           "temporary_hold",
	"RetentionExpirationTime": "retention_expire_time",
	"Metadata":                "metadata",
	"EventBasedHold":          "event_based_hold",
	"Owner":                   "owner",
	"CustomerKeySHA256":       "customer_encryption",
	"CustomTime":              "custom_time",
	"ComponentCount":          "component_count",
	// MediaLink was explicitly excluded from the proto as it is an HTTP-ism.
	// "MediaLink":               "mediaLink",
}

// SetAttrSelection makes the query populate only specific attributes of
// objects. When iterating over objects, if you only need each object's name
// and size, pass []string{"Name", "Size"} to this method. Only these fields
// will be fetched for each object across the network; the other fields of
// ObjectAttr will remain at their default values. This is a performance
// optimization; for more information, see
// https://cloud.google.com/storage/docs/json_api/v1/how-tos/performance
func (q *Query) SetAttrSelection(attrs []string) error {
	// Validate selections.
	for _, attr := range attrs {
		// If the attr is acceptable for one of the two sets, then it is OK.
		// If it is not acceptable for either, then return an error.
		// The respective masking implementations ignore unknown attrs which
		// makes switching between transports a little easier.
		_, okJSON := attrToFieldMap[attr]
		_, okGRPC := attrToProtoFieldMap[attr]

		if !okJSON && !okGRPC {
			return fmt.Errorf("storage: attr %v is not valid", attr)
		}
	}

	q.attrSelection = attrs

	return nil
}

func (q *Query) toFieldSelection() string {
	if q == nil || len(q.attrSelection) == 0 {
		return ""
	}
	fieldSet := make(map[string]bool)

	for _, attr := range q.attrSelection {
		field, ok := attrToFieldMap[attr]
		if !ok {
			// Future proofing, skip unknown fields, let SetAttrSelection handle
			// error modes.
			continue
		}
		fieldSet[field] = true
	}

	var s string
	if len(fieldSet) > 0 {
		var b bytes.Buffer
		b.WriteString("prefixes,items(")
		first := true
		for field := range fieldSet {
			if !first {
				b.WriteString(",")
			}
			first = false
			b.WriteString(field)
		}
		b.WriteString(")")
		s = b.String()
	}
	return s
}

func (q *Query) toFieldMask() *fieldmaskpb.FieldMask {
	// The default behavior with no Query is ProjectionDefault (i.e. ProjectionFull).
	if q == nil {
		return &fieldmaskpb.FieldMask{Paths: []string{"*"}}
	}

	// User selected attributes via q.SetAttrSeleciton. This takes precedence
	// over the Projection.
	if numSelected := len(q.attrSelection); numSelected > 0 {
		protoFieldPaths := make([]string, 0, numSelected)

		for _, attr := range q.attrSelection {
			pf, ok := attrToProtoFieldMap[attr]
			if !ok {
				// Future proofing, skip unknown fields, let SetAttrSelection
				// handle error modes.
				continue
			}
			protoFieldPaths = append(protoFieldPaths, pf)
		}

		return &fieldmaskpb.FieldMask{Paths: protoFieldPaths}
	}

	// ProjectDefault == ProjectionFull which means all fields.
	fm := &fieldmaskpb.FieldMask{Paths: []string{"*"}}
	if q.Projection == ProjectionNoACL {
		paths := make([]string, 0, len(attrToProtoFieldMap)-2) // omitting two fields
		for _, f := range attrToProtoFieldMap {
			// Skip the acl and owner fields for "NoACL".
			if f == "acl" || f == "owner" {
				continue
			}
			paths = append(paths, f)
		}
		fm.Paths = paths
	}

	return fm
}

// Conditions constrain methods to act on specific generations of
// objects.
//
// The zero value is an empty set of constraints. Not all conditions or
// combinations of conditions are applicable to all methods.
// See https://cloud.google.com/storage/docs/generations-preconditions
// for details on how these operate.
type Conditions struct {
	// Generation constraints.
	// At most one of the following can be set to a non-zero value.

	// GenerationMatch specifies that the object must have the given generation
	// for the operation to occur.
	// If GenerationMatch is zero, it has no effect.
	// Use DoesNotExist to specify that the object does not exist in the bucket.
	GenerationMatch int64

	// GenerationNotMatch specifies that the object must not have the given
	// generation for the operation to occur.
	// If GenerationNotMatch is zero, it has no effect.
	GenerationNotMatch int64

	// DoesNotExist specifies that the object must not exist in the bucket for
	// the operation to occur.
	// If DoesNotExist is false, it has no effect.
	DoesNotExist bool

	// Metadata generation constraints.
	// At most one of the following can be set to a non-zero value.

	// MetagenerationMatch specifies that the object must have the given
	// metageneration for the operation to occur.
	// If MetagenerationMatch is zero, it has no effect.
	MetagenerationMatch int64

	// MetagenerationNotMatch specifies that the object must not have the given
	// metageneration for the operation to occur.
	// If MetagenerationNotMatch is zero, it has no effect.
	MetagenerationNotMatch int64
}

func (c *Conditions) validate(method string) error {
	if *c == (Conditions{}) {
		return fmt.Errorf("storage: %s: empty conditions", method)
	}
	if !c.isGenerationValid() {
		return fmt.Errorf("storage: %s: multiple conditions specified for generation", method)
	}
	if !c.isMetagenerationValid() {
		return fmt.Errorf("storage: %s: multiple conditions specified for metageneration", method)
	}
	return nil
}

func (c *Conditions) isGenerationValid() bool {
	n := 0
	if c.GenerationMatch != 0 {
		n++
	}
	if c.GenerationNotMatch != 0 {
		n++
	}
	if c.DoesNotExist {
		n++
	}
	return n <= 1
}

func (c *Conditions) isMetagenerationValid() bool {
	return c.MetagenerationMatch == 0 || c.MetagenerationNotMatch == 0
}

// applyConds modifies the provided call using the conditions in conds.
// call is something that quacks like a *raw.WhateverCall.
func applyConds(method string, gen int64, conds *Conditions, call interface{}) error {
	cval := reflect.ValueOf(call)
	if gen >= 0 {
		if !setConditionField(cval, "Generation", gen) {
			return fmt.Errorf("storage: %s: generation not supported", method)
		}
	}
	if conds == nil {
		return nil
	}
	if err := conds.validate(method); err != nil {
		return err
	}
	switch {
	case conds.GenerationMatch != 0:
		if !setConditionField(cval, "IfGenerationMatch", conds.GenerationMatch) {
			return fmt.Errorf("storage: %s: ifGenerationMatch not supported", method)
		}
	case conds.GenerationNotMatch != 0:
		if !setConditionField(cval, "IfGenerationNotMatch", conds.GenerationNotMatch) {
			return fmt.Errorf("storage: %s: ifGenerationNotMatch not supported", method)
		}
	case conds.DoesNotExist:
		if !setConditionField(cval, "IfGenerationMatch", int64(0)) {
			return fmt.Errorf("storage: %s: DoesNotExist not supported", method)
		}
	}
	switch {
	case conds.MetagenerationMatch != 0:
		if !setConditionField(cval, "IfMetagenerationMatch", conds.MetagenerationMatch) {
			return fmt.Errorf("storage: %s: ifMetagenerationMatch not supported", method)
		}
	case conds.MetagenerationNotMatch != 0:
		if !setConditionField(cval, "IfMetagenerationNotMatch", conds.MetagenerationNotMatch) {
			return fmt.Errorf("storage: %s: ifMetagenerationNotMatch not supported", method)
		}
	}
	return nil
}

func applySourceConds(gen int64, conds *Conditions, call *raw.ObjectsRewriteCall) error {
	if gen >= 0 {
		call.SourceGeneration(gen)
	}
	if conds == nil {
		return nil
	}
	if err := conds.validate("CopyTo source"); err != nil {
		return err
	}
	switch {
	case conds.GenerationMatch != 0:
		call.IfSourceGenerationMatch(conds.GenerationMatch)
	case conds.GenerationNotMatch != 0:
		call.IfSourceGenerationNotMatch(conds.GenerationNotMatch)
	case conds.DoesNotExist:
		call.IfSourceGenerationMatch(0)
	}
	switch {
	case conds.MetagenerationMatch != 0:
		call.IfSourceMetagenerationMatch(conds.MetagenerationMatch)
	case conds.MetagenerationNotMatch != 0:
		call.IfSourceMetagenerationNotMatch(conds.MetagenerationNotMatch)
	}
	return nil
}

func applySourceCondsProto(gen int64, conds *Conditions, call *storagepb.RewriteObjectRequest) error {
	if gen >= 0 {
		call.SourceGeneration = gen
	}
	if conds == nil {
		return nil
	}
	if err := conds.validate("CopyTo source"); err != nil {
		return err
	}
	switch {
	case conds.GenerationMatch != 0:
		call.IfSourceGenerationMatch = proto.Int64(conds.GenerationMatch)
	case conds.GenerationNotMatch != 0:
		call.IfSourceGenerationNotMatch = proto.Int64(conds.GenerationNotMatch)
	case conds.DoesNotExist:
		call.IfSourceGenerationMatch = proto.Int64(0)
	}
	switch {
	case conds.MetagenerationMatch != 0:
		call.IfSourceMetagenerationMatch = proto.Int64(conds.MetagenerationMatch)
	case conds.MetagenerationNotMatch != 0:
		call.IfSourceMetagenerationNotMatch = proto.Int64(conds.MetagenerationNotMatch)
	}
	return nil
}

// setConditionField sets a field on a *raw.WhateverCall.
// We can't use anonymous interfaces because the return type is
// different, since the field setters are builders.
func setConditionField(call reflect.Value, name string, value interface{}) bool {
	m := call.MethodByName(name)
	if !m.IsValid() {
		return false
	}
	m.Call([]reflect.Value{reflect.ValueOf(value)})
	return true
}

// Retryer returns an object handle that is configured with custom retry
// behavior as specified by the options that are passed to it. All operations
// on the new handle will use the customized retry configuration.
// These retry options will merge with the bucket's retryer (if set) for the
// returned handle. Options passed into this method will take precedence over
// retry options on the bucket and client. Note that you must explicitly pass in
// each option you want to override.
func (o *ObjectHandle) Retryer(opts ...RetryOption) *ObjectHandle {
	o2 := *o
	var retry *retryConfig
	if o.retry != nil {
		// merge the options with the existing retry
		retry = o.retry
	} else {
		retry = &retryConfig{}
	}
	for _, opt := range opts {
		opt.apply(retry)
	}
	o2.retry = retry
	o2.acl.retry = retry
	return &o2
}

// SetRetry configures the client with custom retry behavior as specified by the
// options that are passed to it. All operations using this client will use the
// customized retry configuration.
// This should be called once before using the client for network operations, as
// there could be indeterminate behaviour with operations in progress.
// Retry options set on a bucket or object handle will take precedence over
// these options.
func (c *Client) SetRetry(opts ...RetryOption) {
	var retry *retryConfig
	if c.retry != nil {
		// merge the options with the existing retry
		retry = c.retry
	} else {
		retry = &retryConfig{}
	}
	for _, opt := range opts {
		opt.apply(retry)
	}
	c.retry = retry
}

// RetryOption allows users to configure non-default retry behavior for API
// calls made to GCS.
type RetryOption interface {
	apply(config *retryConfig)
}

// WithBackoff allows configuration of the backoff timing used for retries.
// Available configuration options (Initial, Max and Multiplier) are described
// at https://pkg.go.dev/github.com/googleapis/gax-go/v2#Backoff. If any fields
// are not supplied by the user, gax default values will be used.
func WithBackoff(backoff gax.Backoff) RetryOption {
	return &withBackoff{
		backoff: backoff,
	}
}

type withBackoff struct {
	backoff gax.Backoff
}

func (wb *withBackoff) apply(config *retryConfig) {
	config.backoff = &wb.backoff
}

// RetryPolicy describes the available policies for which operations should be
// retried. The default is `RetryIdempotent`.
type RetryPolicy int

const (
	// RetryIdempotent causes only idempotent operations to be retried when the
	// service returns a transient error. Using this policy, fully idempotent
	// operations (such as `ObjectHandle.Attrs()`) will always be retried.
	// Conditionally idempotent operations (for example `ObjectHandle.Update()`)
	// will be retried only if the necessary conditions have been supplied (in
	// the case of `ObjectHandle.Update()` this would mean supplying a
	// `Conditions.MetagenerationMatch` condition is required).
	RetryIdempotent RetryPolicy = iota

	// RetryAlways causes all operations to be retried when the service returns a
	// transient error, regardless of idempotency considerations.
	RetryAlways

	// RetryNever causes the client to not perform retries on failed operations.
	RetryNever
)

// WithPolicy allows the configuration of which operations should be performed
// with retries for transient errors.
func WithPolicy(policy RetryPolicy) RetryOption {
	return &withPolicy{
		policy: policy,
	}
}

type withPolicy struct {
	policy RetryPolicy
}

func (ws *withPolicy) apply(config *retryConfig) {
	config.policy = ws.policy
}

// WithErrorFunc allows users to pass a custom function to the retryer. Errors
// will be retried if and only if `shouldRetry(err)` returns true.
// By default, the following errors are retried (see ShouldRetry for the default
// function):
//
// - HTTP responses with codes 408, 429, 502, 503, and 504.
//
// - Transient network errors such as connection reset and io.ErrUnexpectedEOF.
//
// - Errors which are considered transient using the Temporary() interface.
//
// - Wrapped versions of these errors.
//
// This option can be used to retry on a different set of errors than the
// default. Users can use the default ShouldRetry function inside their custom
// function if they only want to make minor modifications to default behavior.
func WithErrorFunc(shouldRetry func(err error) bool) RetryOption {
	return &withErrorFunc{
		shouldRetry: shouldRetry,
	}
}

type withErrorFunc struct {
	shouldRetry func(err error) bool
}

func (wef *withErrorFunc) apply(config *retryConfig) {
	config.shouldRetry = wef.shouldRetry
}

type retryConfig struct {
	backoff     *gax.Backoff
	policy      RetryPolicy
	shouldRetry func(err error) bool
}

func (r *retryConfig) clone() *retryConfig {
	if r == nil {
		return nil
	}

	var bo *gax.Backoff
	if r.backoff != nil {
		bo = &gax.Backoff{
			Initial:    r.backoff.Initial,
			Max:        r.backoff.Max,
			Multiplier: r.backoff.Multiplier,
		}
	}

	return &retryConfig{
		backoff:     bo,
		policy:      r.policy,
		shouldRetry: r.shouldRetry,
	}
}

// composeSourceObj wraps a *raw.ComposeRequestSourceObjects, but adds the methods
// that modifyCall searches for by name.
type composeSourceObj struct {
	src *raw.ComposeRequestSourceObjects
}

func (c composeSourceObj) Generation(gen int64) {
	c.src.Generation = gen
}

func (c composeSourceObj) IfGenerationMatch(gen int64) {
	// It's safe to overwrite ObjectPreconditions, since its only field is
	// IfGenerationMatch.
	c.src.ObjectPreconditions = &raw.ComposeRequestSourceObjectsObjectPreconditions{
		IfGenerationMatch: gen,
	}
}

func setEncryptionHeaders(headers http.Header, key []byte, copySource bool) error {
	if key == nil {
		return nil
	}
	// TODO(jbd): Ask the API team to return a more user-friendly error
	// and avoid doing this check at the client level.
	if len(key) != 32 {
		return errors.New("storage: not a 32-byte AES-256 key")
	}
	var cs string
	if copySource {
		cs = "copy-source-"
	}
	headers.Set("x-goog-"+cs+"encryption-algorithm", aes256Algorithm)
	headers.Set("x-goog-"+cs+"encryption-key", base64.StdEncoding.EncodeToString(key))
	keyHash := sha256.Sum256(key)
	headers.Set("x-goog-"+cs+"encryption-key-sha256", base64.StdEncoding.EncodeToString(keyHash[:]))
	return nil
}

// toProtoCommonObjectRequestParams sets customer-supplied encryption to the proto library's CommonObjectRequestParams.
func toProtoCommonObjectRequestParams(key []byte) *storagepb.CommonObjectRequestParams {
	if key == nil {
		return nil
	}
	keyHash := sha256.Sum256(key)
	return &storagepb.CommonObjectRequestParams{
		EncryptionAlgorithm:      aes256Algorithm,
		EncryptionKeyBytes:       key,
		EncryptionKeySha256Bytes: keyHash[:],
	}
}

func toProtoChecksums(sendCRC32C bool, attrs *ObjectAttrs) *storagepb.ObjectChecksums {
	var checksums *storagepb.ObjectChecksums
	if sendCRC32C {
		checksums = &storagepb.ObjectChecksums{
			Crc32C: proto.Uint32(attrs.CRC32C),
		}
	}
	if len(attrs.MD5) != 0 {
		if checksums == nil {
			checksums = &storagepb.ObjectChecksums{
				Md5Hash: attrs.MD5,
			}
		} else {
			checksums.Md5Hash = attrs.MD5
		}
	}
	return checksums
}

// ServiceAccount fetches the email address of the given project's Google Cloud Storage service account.
func (c *Client) ServiceAccount(ctx context.Context, projectID string) (string, error) {
	o := makeStorageOpts(true, c.retry, "")
	return c.tc.GetServiceAccount(ctx, projectID, o...)

}

// bucketResourceName formats the given project ID and bucketResourceName ID
// into a Bucket resource name. This is the format necessary for the gRPC API as
// it conforms to the Resource-oriented design practices in https://google.aip.dev/121.
func bucketResourceName(p, b string) string {
	return fmt.Sprintf("projects/%s/buckets/%s", p, b)
}

// parseBucketName strips the leading resource path segment and returns the
// bucket ID, which is the simple Bucket name typical of the v1 API.
func parseBucketName(b string) string {
	sep := strings.LastIndex(b, "/")
	return b[sep+1:]
}

// parseProjectNumber consume the given resource name and parses out the project
// number if one is present i.e. it is not a project ID.
func parseProjectNumber(r string) uint64 {
	projectID := regexp.MustCompile(`projects\/([0-9]+)\/?`)
	if matches := projectID.FindStringSubmatch(r); len(matches) > 0 {
		// Capture group follows the matched segment. For example:
		// input: projects/123/bars/456
		// output: [projects/123/, 123]
		number, err := strconv.ParseUint(matches[1], 10, 64)
		if err != nil {
			return 0
		}
		return number
	}

	return 0
}

// toProjectResource accepts a project ID and formats it as a Project resource
// name.
func toProjectResource(project string) string {
	return fmt.Sprintf("projects/%s", project)
}

// setConditionProtoField uses protobuf reflection to set named condition field
// to the given condition value if supported on the protobuf message.
//
// This is an experimental API and not intended for public use.
func setConditionProtoField(m protoreflect.Message, f string, v int64) bool {
	fields := m.Descriptor().Fields()
	if rf := fields.ByName(protoreflect.Name(f)); rf != nil {
		m.Set(rf, protoreflect.ValueOfInt64(v))
		return true
	}

	return false
}

// applyCondsProto validates and attempts to set the conditions on a protobuf
// message using protobuf reflection.
//
// This is an experimental API and not intended for public use.
func applyCondsProto(method string, gen int64, conds *Conditions, msg proto.Message) error {
	rmsg := msg.ProtoReflect()

	if gen >= 0 {
		if !setConditionProtoField(rmsg, "generation", gen) {
			return fmt.Errorf("storage: %s: generation not supported", method)
		}
	}
	if conds == nil {
		return nil
	}
	if err := conds.validate(method); err != nil {
		return err
	}

	switch {
	case conds.GenerationMatch != 0:
		if !setConditionProtoField(rmsg, "if_generation_match", conds.GenerationMatch) {
			return fmt.Errorf("storage: %s: ifGenerationMatch not supported", method)
		}
	case conds.GenerationNotMatch != 0:
		if !setConditionProtoField(rmsg, "if_generation_not_match", conds.GenerationNotMatch) {
			return fmt.Errorf("storage: %s: ifGenerationNotMatch not supported", method)
		}
	case conds.DoesNotExist:
		if !setConditionProtoField(rmsg, "if_generation_match", int64(0)) {
			return fmt.Errorf("storage: %s: DoesNotExist not supported", method)
		}
	}
	switch {
	case conds.MetagenerationMatch != 0:
		if !setConditionProtoField(rmsg, "if_metageneration_match", conds.MetagenerationMatch) {
			return fmt.Errorf("storage: %s: ifMetagenerationMatch not supported", method)
		}
	case conds.MetagenerationNotMatch != 0:
		if !setConditionProtoField(rmsg, "if_metageneration_not_match", conds.MetagenerationNotMatch) {
			return fmt.Errorf("storage: %s: ifMetagenerationNotMatch not supported", method)
		}
	}
	return nil
}
