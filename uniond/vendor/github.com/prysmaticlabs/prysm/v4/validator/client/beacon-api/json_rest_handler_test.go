package beacon_api

import (
	"bytes"
	"context"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/api"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/beacon"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestGet(t *testing.T) {
	ctx := context.Background()
	const endpoint = "/example/rest/api/endpoint"
	genesisJson := &beacon.GetGenesisResponse{
		Data: &beacon.Genesis{
			GenesisTime:           "123",
			GenesisValidatorsRoot: "0x456",
			GenesisForkVersion:    "0x789",
		},
	}
	mux := http.NewServeMux()
	mux.HandleFunc(endpoint, func(w http.ResponseWriter, r *http.Request) {
		marshalledJson, err := json.Marshal(genesisJson)
		require.NoError(t, err)

		w.Header().Set("Content-Type", api.JsonMediaType)
		_, err = w.Write(marshalledJson)
		require.NoError(t, err)
	})
	server := httptest.NewServer(mux)
	defer server.Close()

	jsonRestHandler := BeaconApiJsonRestHandler{
		HttpClient: http.Client{Timeout: time.Second * 5},
		Host:       server.URL,
	}
	resp := &beacon.GetGenesisResponse{}
	require.NoError(t, jsonRestHandler.Get(ctx, endpoint+"?arg1=abc&arg2=def", resp))
	assert.DeepEqual(t, genesisJson, resp)
}

func TestPost(t *testing.T) {
	ctx := context.Background()
	const endpoint = "/example/rest/api/endpoint"
	dataBytes := []byte{1, 2, 3, 4, 5}
	headers := map[string]string{"foo": "bar"}

	genesisJson := &beacon.GetGenesisResponse{
		Data: &beacon.Genesis{
			GenesisTime:           "123",
			GenesisValidatorsRoot: "0x456",
			GenesisForkVersion:    "0x789",
		},
	}

	mux := http.NewServeMux()
	mux.HandleFunc(endpoint, func(w http.ResponseWriter, r *http.Request) {
		// Make sure the request headers have been set
		assert.Equal(t, "bar", r.Header.Get("foo"))
		assert.Equal(t, api.JsonMediaType, r.Header.Get("Content-Type"))

		// Make sure the data matches
		receivedBytes := make([]byte, len(dataBytes))
		numBytes, err := r.Body.Read(receivedBytes)
		assert.Equal(t, io.EOF, err)
		assert.Equal(t, len(dataBytes), numBytes)
		assert.DeepEqual(t, dataBytes, receivedBytes)

		marshalledJson, err := json.Marshal(genesisJson)
		require.NoError(t, err)

		w.Header().Set("Content-Type", api.JsonMediaType)
		_, err = w.Write(marshalledJson)
		require.NoError(t, err)
	})
	server := httptest.NewServer(mux)
	defer server.Close()

	jsonRestHandler := BeaconApiJsonRestHandler{
		HttpClient: http.Client{Timeout: time.Second * 5},
		Host:       server.URL,
	}
	resp := &beacon.GetGenesisResponse{}
	require.NoError(t, jsonRestHandler.Post(ctx, endpoint, headers, bytes.NewBuffer(dataBytes), resp))
	assert.DeepEqual(t, genesisJson, resp)
}

func Test_decodeResp(t *testing.T) {
	type j struct {
		Foo string `json:"foo"`
	}

	t.Run("200 non-JSON", func(t *testing.T) {
		body := bytes.Buffer{}
		r := &http.Response{
			Status:     "200",
			StatusCode: http.StatusOK,
			Body:       io.NopCloser(&body),
			Header:     map[string][]string{"Content-Type": {api.OctetStreamMediaType}},
		}
		require.NoError(t, decodeResp(r, nil))
	})
	t.Run("204 non-JSON", func(t *testing.T) {
		body := bytes.Buffer{}
		r := &http.Response{
			Status:     "204",
			StatusCode: http.StatusNoContent,
			Body:       io.NopCloser(&body),
			Header:     map[string][]string{"Content-Type": {api.OctetStreamMediaType}},
		}
		require.NoError(t, decodeResp(r, nil))
	})
	t.Run("500 non-JSON", func(t *testing.T) {
		body := bytes.Buffer{}
		_, err := body.WriteString("foo")
		require.NoError(t, err)
		r := &http.Response{
			Status:     "500",
			StatusCode: http.StatusInternalServerError,
			Body:       io.NopCloser(&body),
			Header:     map[string][]string{"Content-Type": {api.OctetStreamMediaType}},
		}
		err = decodeResp(r, nil)
		errJson := &httputil.DefaultJsonError{}
		require.Equal(t, true, errors.As(err, &errJson))
		assert.Equal(t, http.StatusInternalServerError, errJson.Code)
		assert.Equal(t, "foo", errJson.Message)
	})
	t.Run("200 JSON with resp", func(t *testing.T) {
		body := bytes.Buffer{}
		b, err := json.Marshal(&j{Foo: "foo"})
		require.NoError(t, err)
		body.Write(b)
		r := &http.Response{
			Status:     "200",
			StatusCode: http.StatusOK,
			Body:       io.NopCloser(&body),
			Header:     map[string][]string{"Content-Type": {api.JsonMediaType}},
		}
		resp := &j{}
		require.NoError(t, decodeResp(r, resp))
		assert.Equal(t, "foo", resp.Foo)
	})
	t.Run("200 JSON without resp", func(t *testing.T) {
		body := bytes.Buffer{}
		r := &http.Response{
			Status:     "200",
			StatusCode: http.StatusOK,
			Body:       io.NopCloser(&body),
			Header:     map[string][]string{"Content-Type": {api.JsonMediaType}},
		}
		require.NoError(t, decodeResp(r, nil))
	})
	t.Run("204 JSON", func(t *testing.T) {
		body := bytes.Buffer{}
		r := &http.Response{
			Status:     "204",
			StatusCode: http.StatusNoContent,
			Body:       io.NopCloser(&body),
			Header:     map[string][]string{"Content-Type": {api.JsonMediaType}},
		}
		require.NoError(t, decodeResp(r, nil))
	})
	t.Run("500 JSON", func(t *testing.T) {
		body := bytes.Buffer{}
		b, err := json.Marshal(&httputil.DefaultJsonError{Code: http.StatusInternalServerError, Message: "error"})
		require.NoError(t, err)
		body.Write(b)
		r := &http.Response{
			Status:     "500",
			StatusCode: http.StatusInternalServerError,
			Body:       io.NopCloser(&body),
			Header:     map[string][]string{"Content-Type": {api.JsonMediaType}},
		}
		err = decodeResp(r, nil)
		errJson := &httputil.DefaultJsonError{}
		require.Equal(t, true, errors.As(err, &errJson))
		assert.Equal(t, http.StatusInternalServerError, errJson.Code)
		assert.Equal(t, "error", errJson.Message)
	})
	t.Run("200 JSON cannot decode", func(t *testing.T) {
		body := bytes.Buffer{}
		_, err := body.WriteString("foo")
		require.NoError(t, err)
		r := &http.Response{
			Status:     "200",
			StatusCode: http.StatusOK,
			Body:       io.NopCloser(&body),
			Header:     map[string][]string{"Content-Type": {api.JsonMediaType}},
			Request:    &http.Request{},
		}
		resp := &j{}
		err = decodeResp(r, resp)
		assert.ErrorContains(t, "failed to decode response body into json", err)
	})
	t.Run("500 JSON cannot decode", func(t *testing.T) {
		body := bytes.Buffer{}
		_, err := body.WriteString("foo")
		require.NoError(t, err)
		r := &http.Response{
			Status:     "500",
			StatusCode: http.StatusInternalServerError,
			Body:       io.NopCloser(&body),
			Header:     map[string][]string{"Content-Type": {api.JsonMediaType}},
			Request:    &http.Request{},
		}
		err = decodeResp(r, nil)
		assert.ErrorContains(t, "failed to decode response body into error json", err)
	})
}
