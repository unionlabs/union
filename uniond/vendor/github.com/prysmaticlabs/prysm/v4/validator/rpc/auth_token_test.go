package rpc

import (
	"bytes"
	"context"
	"encoding/hex"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/golang-jwt/jwt/v4"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"
)

func setupWalletDir(t testing.TB) string {
	walletDir := filepath.Join(t.TempDir(), "wallet")
	require.NoError(t, os.MkdirAll(walletDir, os.ModePerm))
	return walletDir
}

func TestServer_AuthenticateUsingExistingToken(t *testing.T) {
	// Initializing for the first time, there is no auth token file in
	// the wallet directory, so we generate a jwt token and secret from scratch.
	srv := &Server{}
	walletDir := setupWalletDir(t)
	token, err := srv.initializeAuthToken(walletDir)
	require.NoError(t, err)
	require.Equal(t, true, len(srv.jwtSecret) > 0)

	unaryInfo := &grpc.UnaryServerInfo{
		FullMethod: "Proto.CreateWallet",
	}
	unaryHandler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return nil, nil
	}
	ctxMD := map[string][]string{
		"authorization": {"Bearer " + token},
	}
	ctx := context.Background()
	ctx = metadata.NewIncomingContext(ctx, ctxMD)
	_, err = srv.JWTInterceptor()(ctx, "xyz", unaryInfo, unaryHandler)
	require.NoError(t, err)

	// Next up, we make the same request but reinitialize the server and we should still
	// pass with the same auth token.
	srv = &Server{}
	_, err = srv.initializeAuthToken(walletDir)
	require.NoError(t, err)
	require.Equal(t, true, len(srv.jwtSecret) > 0)
	_, err = srv.JWTInterceptor()(ctx, "xyz", unaryInfo, unaryHandler)
	require.NoError(t, err)
}

func TestServer_RefreshJWTSecretOnFileChange(t *testing.T) {
	// Initializing for the first time, there is no auth token file in
	// the wallet directory, so we generate a jwt token and secret from scratch.
	srv := &Server{}
	walletDir := setupWalletDir(t)
	_, err := srv.initializeAuthToken(walletDir)
	require.NoError(t, err)
	currentSecret := srv.jwtSecret
	require.Equal(t, true, len(currentSecret) > 0)

	authTokenPath := filepath.Join(walletDir, AuthTokenFileName)

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	go srv.refreshAuthTokenFromFileChanges(ctx, authTokenPath)

	// Wait for service to be ready.
	time.Sleep(time.Millisecond * 250)

	// Update the auth token file with a new secret.
	require.NoError(t, CreateAuthToken(walletDir, "localhost:7500"))

	// The service should have picked up the file change and set the jwt secret to the new one.
	time.Sleep(time.Millisecond * 500)
	newSecret := srv.jwtSecret
	require.Equal(t, true, len(newSecret) > 0)
	require.Equal(t, true, !bytes.Equal(currentSecret, newSecret))
	err = os.Remove(AuthTokenFileName)
	require.NoError(t, err)
}

func Test_initializeAuthToken(t *testing.T) {
	// Initializing for the first time, there is no auth token file in
	// the wallet directory, so we generate a jwt token and secret from scratch.
	srv := &Server{}
	walletDir := setupWalletDir(t)
	token, err := srv.initializeAuthToken(walletDir)
	require.NoError(t, err)
	require.Equal(t, true, len(srv.jwtSecret) > 0)

	// Initializing second time, we generate something from the initial file.
	srv2 := &Server{}
	token2, err := srv2.initializeAuthToken(walletDir)
	require.NoError(t, err)
	require.Equal(t, true, bytes.Equal(srv.jwtSecret, srv2.jwtSecret))
	require.Equal(t, token, token2)

	// Deleting the auth token and re-initializing means we create a jwt token
	// and secret from scratch again.
	srv3 := &Server{}
	walletDir = setupWalletDir(t)
	token3, err := srv3.initializeAuthToken(walletDir)
	require.NoError(t, err)
	require.Equal(t, true, len(srv.jwtSecret) > 0)
	require.NotEqual(t, token, token3)
}

// "createTokenString" now uses jwt.RegisteredClaims instead of jwt.StandardClaims (deprecated),
// make sure empty jwt.RegisteredClaims and empty jwt.StandardClaims generates the same token.
func Test_UseRegisteredClaimInsteadOfStandClaims(t *testing.T) {
	jwtsecret, err := hex.DecodeString("12345678900123456789abcdeffedcba")
	require.NoError(t, err)
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, jwt.StandardClaims{}) // jwt.StandardClaims is deprecated
	wantedTokenString, err := token.SignedString(jwtsecret)
	require.NoError(t, err)

	gotTokenString, err := createTokenString(jwtsecret)
	require.NoError(t, err)

	if wantedTokenString != gotTokenString {
		t.Errorf("%s != %s", wantedTokenString, gotTokenString)
	}
}
