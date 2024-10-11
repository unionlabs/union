package httputil

import (
	"bytes"
	"fmt"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/api"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestRespondWithSsz(t *testing.T) {
	t.Run("ssz_requested", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodGet, "http://foo.example", nil)
		request.Header["Accept"] = []string{api.OctetStreamMediaType}
		result := RespondWithSsz(request)
		assert.Equal(t, true, result)
	})

	t.Run("ssz_content_type_first", func(t *testing.T) {
		request := httptest.NewRequest("GET", "http://foo.example", nil)
		request.Header["Accept"] = []string{fmt.Sprintf("%s,%s", api.OctetStreamMediaType, api.JsonMediaType)}
		result := RespondWithSsz(request)
		assert.Equal(t, true, result)
	})

	t.Run("ssz_content_type_preferred_1", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodGet, "http://foo.example", nil)
		request.Header["Accept"] = []string{fmt.Sprintf("%s;q=0.9,%s", api.JsonMediaType, api.OctetStreamMediaType)}
		result := RespondWithSsz(request)
		assert.Equal(t, true, result)
	})

	t.Run("ssz_content_type_preferred_2", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodGet, "http://foo.example", nil)
		request.Header["Accept"] = []string{fmt.Sprintf("%s;q=0.95,%s;q=0.9", api.OctetStreamMediaType, api.JsonMediaType)}
		result := RespondWithSsz(request)
		assert.Equal(t, true, result)
	})

	t.Run("other_content_type_preferred", func(t *testing.T) {
		request := httptest.NewRequest("GET", "http://foo.example", nil)
		request.Header["Accept"] = []string{fmt.Sprintf("%s,%s;q=0.9", api.JsonMediaType, api.OctetStreamMediaType)}
		result := RespondWithSsz(request)
		assert.Equal(t, false, result)
	})

	t.Run("other_params", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodGet, "http://foo.example", nil)
		request.Header["Accept"] = []string{fmt.Sprintf("%s,%s;q=0.9,otherparam=xyz", api.JsonMediaType, api.OctetStreamMediaType)}
		result := RespondWithSsz(request)
		assert.Equal(t, false, result)
	})

	t.Run("no_header", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodGet, "http://foo.example", nil)
		result := RespondWithSsz(request)
		assert.Equal(t, false, result)
	})

	t.Run("empty_header", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodGet, "http://foo.example", nil)
		request.Header["Accept"] = []string{}
		result := RespondWithSsz(request)
		assert.Equal(t, false, result)
	})

	t.Run("empty_header_value", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodGet, "http://foo.example", nil)
		request.Header["Accept"] = []string{""}
		result := RespondWithSsz(request)
		assert.Equal(t, false, result)
	})

	t.Run("other_content_type", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodGet, "http://foo.example", nil)
		request.Header["Accept"] = []string{"application/other"}
		result := RespondWithSsz(request)
		assert.Equal(t, false, result)
	})

	t.Run("garbage", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodGet, "http://foo.example", nil)
		request.Header["Accept"] = []string{"This is Sparta!!!"}
		result := RespondWithSsz(request)
		assert.Equal(t, false, result)
	})
}

func TestIsRequestSsz(t *testing.T) {
	t.Run("ssz Post happy path", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString("something")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://foo.example", &body)
		request.Header["Content-Type"] = []string{api.OctetStreamMediaType}
		result := IsRequestSsz(request)
		assert.Equal(t, true, result)
	})

	t.Run("ssz Post missing header", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodPost, "http://foo.example", nil)
		result := IsRequestSsz(request)
		assert.Equal(t, false, result)
	})

	t.Run("ssz Post wrong content type", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodPost, "http://foo.example", nil)
		request.Header["Content-Type"] = []string{"application/other"}
		result := IsRequestSsz(request)
		assert.Equal(t, false, result)
	})

	t.Run("ssz Post json content type", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodPost, "http://foo.example", nil)
		request.Header["Content-Type"] = []string{api.JsonMediaType}
		result := IsRequestSsz(request)
		assert.Equal(t, false, result)
	})
}
