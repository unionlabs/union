package network

import (
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/golang-jwt/jwt/v4"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestJWTAuthTransport(t *testing.T) {
	secret := bytesutil.PadTo([]byte("foo"), 32)
	authTransport := &jwtTransport{
		underlyingTransport: http.DefaultTransport,
		jwtSecret:           secret,
	}
	client := &http.Client{
		Timeout:   DefaultRPCHTTPTimeout,
		Transport: authTransport,
	}
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		reqToken := r.Header.Get("Authorization")
		splitToken := strings.Split(reqToken, "Bearer")
		// The format should be `Bearer ${token}`.
		require.Equal(t, 2, len(splitToken))
		reqToken = strings.TrimSpace(splitToken[1])
		token, err := jwt.Parse(reqToken, func(token *jwt.Token) (interface{}, error) {
			// We should be doing HMAC signing.
			_, ok := token.Method.(*jwt.SigningMethodHMAC)
			require.Equal(t, true, ok)
			return secret, nil
		})
		require.NoError(t, err)
		require.Equal(t, true, token.Valid)
		claims, ok := token.Claims.(jwt.MapClaims)
		require.Equal(t, true, ok)
		item, ok := claims["iat"]
		require.Equal(t, true, ok)
		iat, ok := item.(float64)
		require.Equal(t, true, ok)
		issuedAt := time.Unix(int64(iat), 0)
		// The claims should have an "iat" field (issued at) that is at most, 5 seconds ago.
		since := time.Since(issuedAt)
		require.Equal(t, true, since <= time.Second*5)
	}))
	defer srv.Close()
	_, err := client.Get(srv.URL)
	require.NoError(t, err)
}

func TestJWTWithId(t *testing.T) {
	secret := bytesutil.PadTo([]byte("foo"), 32)
	jwtId := "abc"
	authTransport := &jwtTransport{
		underlyingTransport: http.DefaultTransport,
		jwtSecret:           secret,
		jwtId:               jwtId,
	}
	client := &http.Client{
		Timeout:   DefaultRPCHTTPTimeout,
		Transport: authTransport,
	}
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		reqToken := r.Header.Get("Authorization")
		splitToken := strings.Split(reqToken, "Bearer")
		// The format should be `Bearer ${token}`.
		require.Equal(t, 2, len(splitToken))
		reqToken = strings.TrimSpace(splitToken[1])
		token, err := jwt.Parse(reqToken, func(token *jwt.Token) (interface{}, error) {
			// We should be doing HMAC signing.
			_, ok := token.Method.(*jwt.SigningMethodHMAC)
			require.Equal(t, true, ok)
			return secret, nil
		})
		require.NoError(t, err)
		require.Equal(t, true, token.Valid)
		claims, ok := token.Claims.(jwt.MapClaims)
		require.Equal(t, true, ok)
		item, ok := claims["iat"]
		require.Equal(t, true, ok)
		iat, ok := item.(float64)
		require.Equal(t, true, ok)
		issuedAt := time.Unix(int64(iat), 0)
		// The claims should have an "iat" field (issued at) that is at most, 5 seconds ago.
		since := time.Since(issuedAt)
		require.Equal(t, true, since <= time.Second*5)
		// check jwt claims id
		id, ok := claims["id"]
		require.Equal(t, true, ok)
		require.Equal(t, id, jwtId)
	}))
	defer srv.Close()
	_, err := client.Get(srv.URL)
	require.NoError(t, err)
}

func TestJWTWithoutId(t *testing.T) {
	secret := bytesutil.PadTo([]byte("foo"), 32)
	authTransport := &jwtTransport{
		underlyingTransport: http.DefaultTransport,
		jwtSecret:           secret,
	}
	client := &http.Client{
		Timeout:   DefaultRPCHTTPTimeout,
		Transport: authTransport,
	}
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		reqToken := r.Header.Get("Authorization")
		splitToken := strings.Split(reqToken, "Bearer")
		// The format should be `Bearer ${token}`.
		require.Equal(t, 2, len(splitToken))
		reqToken = strings.TrimSpace(splitToken[1])
		token, err := jwt.Parse(reqToken, func(token *jwt.Token) (interface{}, error) {
			// We should be doing HMAC signing.
			_, ok := token.Method.(*jwt.SigningMethodHMAC)
			require.Equal(t, true, ok)
			return secret, nil
		})
		require.NoError(t, err)
		require.Equal(t, true, token.Valid)
		claims, ok := token.Claims.(jwt.MapClaims)
		require.Equal(t, true, ok)
		item, ok := claims["iat"]
		require.Equal(t, true, ok)
		iat, ok := item.(float64)
		require.Equal(t, true, ok)
		issuedAt := time.Unix(int64(iat), 0)
		// The claims should have an "iat" field (issued at) that is at most, 5 seconds ago.
		since := time.Since(issuedAt)
		require.Equal(t, true, since <= time.Second*5)
		_, ok = claims["id"]
		require.Equal(t, false, ok)
	}))
	defer srv.Close()
	_, err := client.Get(srv.URL)
	require.NoError(t, err)
}
