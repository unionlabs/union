package rpc

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/golang-jwt/jwt/v4"
	"github.com/prysmaticlabs/prysm/v4/api"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"
)

func TestServer_JWTInterceptor_Verify(t *testing.T) {
	s := Server{
		jwtSecret: []byte("testKey"),
	}
	interceptor := s.JWTInterceptor()

	unaryInfo := &grpc.UnaryServerInfo{
		FullMethod: "Proto.CreateWallet",
	}
	unaryHandler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return nil, nil
	}
	token, err := createTokenString(s.jwtSecret)
	require.NoError(t, err)
	ctxMD := map[string][]string{
		"authorization": {"Bearer " + token},
	}
	ctx := context.Background()
	ctx = metadata.NewIncomingContext(ctx, ctxMD)
	_, err = interceptor(ctx, "xyz", unaryInfo, unaryHandler)
	require.NoError(t, err)
}

func TestServer_JWTInterceptor_BadToken(t *testing.T) {
	s := Server{
		jwtSecret: []byte("testKey"),
	}
	interceptor := s.JWTInterceptor()

	unaryInfo := &grpc.UnaryServerInfo{
		FullMethod: "Proto.CreateWallet",
	}
	unaryHandler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return nil, nil
	}

	badServer := Server{
		jwtSecret: []byte("badTestKey"),
	}
	token, err := createTokenString(badServer.jwtSecret)
	require.NoError(t, err)
	ctxMD := map[string][]string{
		"authorization": {"Bearer " + token},
	}
	ctx := context.Background()
	ctx = metadata.NewIncomingContext(ctx, ctxMD)
	_, err = interceptor(ctx, "xyz", unaryInfo, unaryHandler)
	require.ErrorContains(t, "signature is invalid", err)
}

func TestServer_JWTInterceptor_InvalidSigningType(t *testing.T) {
	ss := &Server{jwtSecret: make([]byte, 32)}
	// Use a different signing type than the expected, HMAC.
	token := jwt.NewWithClaims(jwt.SigningMethodRS256, jwt.RegisteredClaims{})
	_, err := ss.validateJWT(token)
	require.ErrorContains(t, "unexpected JWT signing method", err)
}

func TestServer_JwtHttpInterceptor(t *testing.T) {
	jwtKey, err := createRandomJWTSecret()
	require.NoError(t, err)

	s := &Server{jwtSecret: jwtKey}
	testHandler := s.JwtHttpInterceptor(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Your test handler logic here
		w.WriteHeader(http.StatusOK)
		_, err := w.Write([]byte("Test Response"))
		require.NoError(t, err)
	}))
	t.Run("no jwt was sent", func(t *testing.T) {
		rr := httptest.NewRecorder()
		req, err := http.NewRequest(http.MethodGet, "/eth/v1/keystores", nil)
		require.NoError(t, err)
		testHandler.ServeHTTP(rr, req)
		require.Equal(t, http.StatusUnauthorized, rr.Code)
	})
	t.Run("wrong jwt was sent", func(t *testing.T) {
		rr := httptest.NewRecorder()
		req, err := http.NewRequest(http.MethodGet, "/eth/v1/keystores", nil)
		require.NoError(t, err)
		req.Header.Set("Authorization", "Bearer YOUR_JWT_TOKEN") // Replace with a valid JWT token
		testHandler.ServeHTTP(rr, req)
		require.Equal(t, http.StatusForbidden, rr.Code)
	})
	t.Run("jwt was sent", func(t *testing.T) {
		rr := httptest.NewRecorder()
		req, err := http.NewRequest(http.MethodGet, "/eth/v1/keystores", nil)
		require.NoError(t, err)
		token, err := createTokenString(jwtKey)
		require.NoError(t, err)
		req.Header.Set("Authorization", "Bearer "+token) // Replace with a valid JWT token
		testHandler.ServeHTTP(rr, req)
		require.Equal(t, http.StatusOK, rr.Code)
	})
	t.Run("initialize does not need jwt", func(t *testing.T) {
		rr := httptest.NewRecorder()
		req, err := http.NewRequest(http.MethodGet, api.WebUrlPrefix+"initialize", nil)
		require.NoError(t, err)
		testHandler.ServeHTTP(rr, req)
		require.Equal(t, http.StatusOK, rr.Code)
	})
	t.Run("health does not need jwt", func(t *testing.T) {
		rr := httptest.NewRecorder()
		req, err := http.NewRequest(http.MethodGet, api.WebUrlPrefix+"health/logs", nil)
		require.NoError(t, err)
		testHandler.ServeHTTP(rr, req)
		require.Equal(t, http.StatusOK, rr.Code)
	})
}
