package assert

import (
	"fmt"
	"io"
	"net/http"
	"net/url"
	"testing"
)

func httpOK(w http.ResponseWriter, r *http.Request) {
	w.WriteHeader(http.StatusOK)
}

func httpReadBody(w http.ResponseWriter, r *http.Request) {
	_, _ = io.Copy(io.Discard, r.Body)
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write([]byte("hello"))
}

func httpRedirect(w http.ResponseWriter, r *http.Request) {
	w.WriteHeader(http.StatusTemporaryRedirect)
}

func httpError(w http.ResponseWriter, r *http.Request) {
	w.WriteHeader(http.StatusInternalServerError)
}

func httpStatusCode(w http.ResponseWriter, r *http.Request) {
	w.WriteHeader(http.StatusSwitchingProtocols)
}

func TestHTTPSuccess(t *testing.T) {
	assert := New(t)

	mockT1 := new(testing.T)
	assert.Equal(HTTPSuccess(mockT1, httpOK, "GET", "/", nil), true)
	assert.False(mockT1.Failed())

	mockT2 := new(testing.T)
	assert.Equal(HTTPSuccess(mockT2, httpRedirect, "GET", "/", nil), false)
	assert.True(mockT2.Failed())

	mockT3 := new(mockTestingT)
	assert.Equal(HTTPSuccess(
		mockT3, httpError, "GET", "/", nil,
		"was not expecting a failure here",
	), false)
	assert.True(mockT3.Failed())
	assert.Contains(mockT3.errorString(), "was not expecting a failure here")

	mockT4 := new(testing.T)
	assert.Equal(HTTPSuccess(mockT4, httpStatusCode, "GET", "/", nil), false)
	assert.True(mockT4.Failed())

	mockT5 := new(testing.T)
	assert.Equal(HTTPSuccess(mockT5, httpReadBody, "POST", "/", nil), true)
	assert.False(mockT5.Failed())
}

func TestHTTPRedirect(t *testing.T) {
	assert := New(t)

	mockT1 := new(mockTestingT)
	assert.Equal(HTTPRedirect(
		mockT1, httpOK, "GET", "/", nil,
		"was expecting a 3xx status code. Got 200.",
	), false)
	assert.True(mockT1.Failed())
	assert.Contains(mockT1.errorString(), "was expecting a 3xx status code. Got 200.")

	mockT2 := new(testing.T)
	assert.Equal(HTTPRedirect(mockT2, httpRedirect, "GET", "/", nil), true)
	assert.False(mockT2.Failed())

	mockT3 := new(testing.T)
	assert.Equal(HTTPRedirect(mockT3, httpError, "GET", "/", nil), false)
	assert.True(mockT3.Failed())

	mockT4 := new(testing.T)
	assert.Equal(HTTPRedirect(mockT4, httpStatusCode, "GET", "/", nil), false)
	assert.True(mockT4.Failed())
}

func TestHTTPError(t *testing.T) {
	assert := New(t)

	mockT1 := new(testing.T)
	assert.Equal(HTTPError(mockT1, httpOK, "GET", "/", nil), false)
	assert.True(mockT1.Failed())

	mockT2 := new(mockTestingT)
	assert.Equal(HTTPError(
		mockT2, httpRedirect, "GET", "/", nil,
		"Expected this request to error out. But it didn't",
	), false)
	assert.True(mockT2.Failed())
	assert.Contains(mockT2.errorString(), "Expected this request to error out. But it didn't")

	mockT3 := new(testing.T)
	assert.Equal(HTTPError(mockT3, httpError, "GET", "/", nil), true)
	assert.False(mockT3.Failed())

	mockT4 := new(testing.T)
	assert.Equal(HTTPError(mockT4, httpStatusCode, "GET", "/", nil), false)
	assert.True(mockT4.Failed())
}

func TestHTTPStatusCode(t *testing.T) {
	assert := New(t)

	mockT1 := new(testing.T)
	assert.Equal(HTTPStatusCode(mockT1, httpOK, "GET", "/", nil, http.StatusSwitchingProtocols), false)
	assert.True(mockT1.Failed())

	mockT2 := new(testing.T)
	assert.Equal(HTTPStatusCode(mockT2, httpRedirect, "GET", "/", nil, http.StatusSwitchingProtocols), false)
	assert.True(mockT2.Failed())

	mockT3 := new(mockTestingT)
	assert.Equal(HTTPStatusCode(
		mockT3, httpError, "GET", "/", nil, http.StatusSwitchingProtocols,
		"Expected the status code to be %d", http.StatusSwitchingProtocols,
	), false)
	assert.True(mockT3.Failed())
	assert.Contains(mockT3.errorString(), "Expected the status code to be 101")

	mockT4 := new(testing.T)
	assert.Equal(HTTPStatusCode(mockT4, httpStatusCode, "GET", "/", nil, http.StatusSwitchingProtocols), true)
	assert.False(mockT4.Failed())
}

func TestHTTPStatusesWrapper(t *testing.T) {
	assert := New(t)
	mockAssert := New(new(testing.T))

	assert.Equal(mockAssert.HTTPSuccess(httpOK, "GET", "/", nil), true)
	assert.Equal(mockAssert.HTTPSuccess(httpRedirect, "GET", "/", nil), false)
	assert.Equal(mockAssert.HTTPSuccess(httpError, "GET", "/", nil), false)

	assert.Equal(mockAssert.HTTPRedirect(httpOK, "GET", "/", nil), false)
	assert.Equal(mockAssert.HTTPRedirect(httpRedirect, "GET", "/", nil), true)
	assert.Equal(mockAssert.HTTPRedirect(httpError, "GET", "/", nil), false)

	assert.Equal(mockAssert.HTTPError(httpOK, "GET", "/", nil), false)
	assert.Equal(mockAssert.HTTPError(httpRedirect, "GET", "/", nil), false)
	assert.Equal(mockAssert.HTTPError(httpError, "GET", "/", nil), true)
}

func httpHelloName(w http.ResponseWriter, r *http.Request) {
	name := r.FormValue("name")
	_, _ = fmt.Fprintf(w, "Hello, %s!", name)
}

func TestHTTPRequestWithNoParams(t *testing.T) {
	var got *http.Request
	handler := func(w http.ResponseWriter, r *http.Request) {
		got = r
		w.WriteHeader(http.StatusOK)
	}

	True(t, HTTPSuccess(t, handler, "GET", "/url", nil))

	Empty(t, got.URL.Query())
	Equal(t, "/url", got.URL.RequestURI())
}

func TestHTTPRequestWithParams(t *testing.T) {
	var got *http.Request
	handler := func(w http.ResponseWriter, r *http.Request) {
		got = r
		w.WriteHeader(http.StatusOK)
	}
	params := url.Values{}
	params.Add("id", "12345")

	True(t, HTTPSuccess(t, handler, "GET", "/url", params))

	Equal(t, url.Values{"id": []string{"12345"}}, got.URL.Query())
	Equal(t, "/url?id=12345", got.URL.String())
	Equal(t, "/url?id=12345", got.URL.RequestURI())
}

func TestHttpBody(t *testing.T) {
	assert := New(t)
	mockT := new(mockTestingT)

	assert.True(HTTPBodyContains(mockT, httpHelloName, "GET", "/", url.Values{"name": []string{"World"}}, "Hello, World!"))
	assert.True(HTTPBodyContains(mockT, httpHelloName, "GET", "/", url.Values{"name": []string{"World"}}, "World"))
	assert.False(HTTPBodyContains(mockT, httpHelloName, "GET", "/", url.Values{"name": []string{"World"}}, "world"))

	assert.False(HTTPBodyNotContains(mockT, httpHelloName, "GET", "/", url.Values{"name": []string{"World"}}, "Hello, World!"))
	assert.False(HTTPBodyNotContains(
		mockT, httpHelloName, "GET", "/", url.Values{"name": []string{"World"}}, "World",
		"Expected the request body to not contain 'World'. But it did.",
	))
	assert.True(HTTPBodyNotContains(mockT, httpHelloName, "GET", "/", url.Values{"name": []string{"World"}}, "world"))
	assert.Contains(mockT.errorString(), "Expected the request body to not contain 'World'. But it did.")

	assert.True(HTTPBodyContains(mockT, httpReadBody, "GET", "/", nil, "hello"))
}

func TestHttpBodyWrappers(t *testing.T) {
	assert := New(t)
	mockAssert := New(new(testing.T))

	assert.True(mockAssert.HTTPBodyContains(httpHelloName, "GET", "/", url.Values{"name": []string{"World"}}, "Hello, World!"))
	assert.True(mockAssert.HTTPBodyContains(httpHelloName, "GET", "/", url.Values{"name": []string{"World"}}, "World"))
	assert.False(mockAssert.HTTPBodyContains(httpHelloName, "GET", "/", url.Values{"name": []string{"World"}}, "world"))

	assert.False(mockAssert.HTTPBodyNotContains(httpHelloName, "GET", "/", url.Values{"name": []string{"World"}}, "Hello, World!"))
	assert.False(mockAssert.HTTPBodyNotContains(httpHelloName, "GET", "/", url.Values{"name": []string{"World"}}, "World"))
	assert.True(mockAssert.HTTPBodyNotContains(httpHelloName, "GET", "/", url.Values{"name": []string{"World"}}, "world"))
}
