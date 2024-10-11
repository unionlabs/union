package internal

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/http/httputil"
	"net/url"
	"strconv"
	"strings"
	"time"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/pkg/errors"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	"github.com/sirupsen/logrus"
	"go.opencensus.io/trace"
)

const (
	ethApiNamespace = "/api/v1/eth2/sign/"
)

type SignRequestJson []byte

// SignatureResponse is the struct representing the signing request response in json format
type SignatureResponse struct {
	Signature hexutil.Bytes `json:"signature"`
}

// HttpSignerClient defines the interface for interacting with a remote web3signer.
type HttpSignerClient interface {
	Sign(ctx context.Context, pubKey string, request SignRequestJson) (bls.Signature, error)
	GetPublicKeys(ctx context.Context, url string) ([][48]byte, error)
}

// ApiClient a wrapper object around web3signer APIs. Please refer to the docs from Consensys' web3signer project.
type ApiClient struct {
	BaseURL    *url.URL
	RestClient *http.Client
}

// NewApiClient method instantiates a new ApiClient object.
func NewApiClient(baseEndpoint string) (*ApiClient, error) {
	u, err := url.ParseRequestURI(baseEndpoint)
	if err != nil {
		return nil, errors.Wrap(err, "invalid format, unable to parse url")
	}
	if u.Scheme == "" || u.Host == "" {
		return nil, fmt.Errorf("web3signer url must be in the format of http(s)://host:port url used: %v", baseEndpoint)
	}
	return &ApiClient{
		BaseURL:    u,
		RestClient: &http.Client{},
	}, nil
}

// Sign is a wrapper method around the web3signer sign api.
func (client *ApiClient) Sign(ctx context.Context, pubKey string, request SignRequestJson) (bls.Signature, error) {
	requestPath := ethApiNamespace + pubKey
	resp, err := client.doRequest(ctx, http.MethodPost, client.BaseURL.String()+requestPath, bytes.NewBuffer(request))
	if err != nil {
		return nil, err
	}
	if resp.StatusCode == http.StatusNotFound {
		return nil, fmt.Errorf("public key not found")
	}
	if resp.StatusCode == http.StatusPreconditionFailed {
		return nil, fmt.Errorf("signing operation failed due to slashing protection rules,  Signing Request URL: %v, Status: %v", client.BaseURL.String()+requestPath, resp.StatusCode)
	}
	contentType := resp.Header.Get("Content-Type")
	if strings.HasPrefix(contentType, "application/json") {
		var sigResp SignatureResponse
		if err := unmarshalResponse(resp.Body, &sigResp); err != nil {
			return nil, err
		}
		return bls.SignatureFromBytes(sigResp.Signature)
	} else {
		return unmarshalSignatureResponse(resp.Body)
	}
}

// GetPublicKeys is a wrapper method around the web3signer publickeys api (this may be removed in the future or moved to another location due to its usage).
func (client *ApiClient) GetPublicKeys(ctx context.Context, url string) ([][fieldparams.BLSPubkeyLength]byte, error) {
	resp, err := client.doRequest(ctx, http.MethodGet, url, nil /* no body needed on get request */)
	if err != nil {
		return nil, err
	}
	var publicKeys []string
	if err := unmarshalResponse(resp.Body, &publicKeys); err != nil {
		return nil, err
	}
	decodedKeys := make([][fieldparams.BLSPubkeyLength]byte, len(publicKeys))
	var errorKeyPositions string
	for i, value := range publicKeys {
		decodedKey, err := hexutil.Decode(value)
		if err != nil {
			errorKeyPositions += fmt.Sprintf("%v, ", i)
			continue
		}
		decodedKeys[i] = bytesutil.ToBytes48(decodedKey)
	}
	if errorKeyPositions != "" {
		return nil, errors.New("failed to decode from Hex from the following public key index locations: " + errorKeyPositions)
	}
	return decodedKeys, nil
}

// ReloadSignerKeys is a wrapper method around the web3signer reload api.
func (client *ApiClient) ReloadSignerKeys(ctx context.Context) error {
	const requestPath = "/reload"
	if _, err := client.doRequest(ctx, http.MethodPost, client.BaseURL.String()+requestPath, nil); err != nil {
		return err
	}
	return nil
}

// GetServerStatus is a wrapper method around the web3signer upcheck api
func (client *ApiClient) GetServerStatus(ctx context.Context) (string, error) {
	const requestPath = "/upcheck"
	resp, err := client.doRequest(ctx, http.MethodGet, client.BaseURL.String()+requestPath, nil /* no body needed on get request */)
	if err != nil {
		return "", err
	}
	var status string
	if err := unmarshalResponse(resp.Body, &status); err != nil {
		return "", err
	}
	return status, nil
}

// doRequest is a utility method for requests.
func (client *ApiClient) doRequest(ctx context.Context, httpMethod, fullPath string, body io.Reader) (*http.Response, error) {
	var requestDump []byte
	ctx, span := trace.StartSpan(ctx, "remote_web3signer.Client.doRequest")
	defer span.End()
	span.AddAttributes(
		trace.StringAttribute("httpMethod", httpMethod),
		trace.StringAttribute("fullPath", fullPath),
		trace.BoolAttribute("hasBody", body != nil),
	)
	req, err := http.NewRequestWithContext(ctx, httpMethod, fullPath, body)
	if err != nil {
		return nil, errors.Wrap(err, "invalid format, failed to create new Post Request Object")
	}
	req.Header.Set("Content-Type", "application/json")

	start := time.Now()
	resp, err := client.RestClient.Do(req)
	duration := time.Since(start)
	if err != nil {
		signRequestDurationSeconds.WithLabelValues(req.Method, "error").Observe(duration.Seconds())
		err = errors.Wrap(err, "failed to execute json request")
		tracing.AnnotateError(span, err)
		return resp, err
	} else {
		signRequestDurationSeconds.WithLabelValues(req.Method, strconv.Itoa(resp.StatusCode)).Observe(duration.Seconds())
	}
	if resp.StatusCode != http.StatusOK {
		requestDump, err = httputil.DumpRequestOut(req, true)
		if err != nil {
			return nil, err
		}
		responseDump, err := httputil.DumpResponse(resp, true)
		if err != nil {
			return nil, err
		}
		log.WithFields(logrus.Fields{
			"status":   resp.StatusCode,
			"request":  string(requestDump),
			"response": string(responseDump),
		}).Error("web3signer request failed")
	}
	if resp.StatusCode == http.StatusInternalServerError {
		err = fmt.Errorf("internal Web3Signer server error, Signing Request URL: %v Status: %v", fullPath, resp.StatusCode)
		tracing.AnnotateError(span, err)
		return nil, err
	} else if resp.StatusCode == http.StatusBadRequest {
		err = fmt.Errorf("bad request format, Signing Request URL: %v Status: %v", fullPath, resp.StatusCode)
		tracing.AnnotateError(span, err)
		return nil, err
	}
	return resp, nil
}

// unmarshalResponse is a utility method for unmarshalling responses.
func unmarshalResponse(responseBody io.ReadCloser, unmarshalledResponseObject interface{}) error {
	defer closeBody(responseBody)
	if err := json.NewDecoder(responseBody).Decode(&unmarshalledResponseObject); err != nil {
		body, err := io.ReadAll(responseBody)
		if err != nil {
			return errors.Wrap(err, "failed to read response body")
		}
		return errors.Wrap(err, fmt.Sprintf("invalid format, unable to read response body: %v", string(body)))
	}
	return nil
}

func unmarshalSignatureResponse(responseBody io.ReadCloser) (bls.Signature, error) {
	defer closeBody(responseBody)
	body, err := io.ReadAll(responseBody)
	if err != nil {
		return nil, err
	}
	sigBytes, err := hexutil.Decode(string(body))
	if err != nil {
		return nil, err
	}
	return bls.SignatureFromBytes(sigBytes)
}

// closeBody a utility method to wrap an error for closing
func closeBody(body io.Closer) {
	if err := body.Close(); err != nil {
		log.WithError(err).Error("could not close response body")
	}
}
