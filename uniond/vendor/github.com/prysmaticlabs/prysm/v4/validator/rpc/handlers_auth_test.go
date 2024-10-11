package rpc

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
)

func TestInitialize(t *testing.T) {
	// Step 1: Create a temporary directory
	localWalletDir := setupWalletDir(t)

	// Step 2: Optionally create a temporary 'auth-token' file
	authTokenPath := filepath.Join(localWalletDir, AuthTokenFileName)
	_, err := os.Create(authTokenPath)
	require.NoError(t, err)

	// Create an instance of the Server with the temporary directory
	opts := []accounts.Option{
		accounts.WithWalletDir(localWalletDir),
		accounts.WithKeymanagerType(keymanager.Derived),
		accounts.WithWalletPassword(strongPass),
		accounts.WithSkipMnemonicConfirm(true),
	}
	acc, err := accounts.NewCLIManager(opts...)
	require.NoError(t, err)
	_, err = acc.WalletCreate(context.Background())
	require.NoError(t, err)
	server := &Server{walletDir: localWalletDir}

	// Step 4: Create an HTTP request and response recorder
	req := httptest.NewRequest(http.MethodGet, "/initialize", nil)
	w := httptest.NewRecorder()

	// Step 5: Call the Initialize function
	server.Initialize(w, req)

	// Step 6: Assert expectations
	result := w.Result()
	defer func() {
		err := result.Body.Close()
		require.NoError(t, err)
	}()

	var response InitializeAuthResponse
	err = json.NewDecoder(result.Body).Decode(&response)
	require.NoError(t, err)

	// Assert the expected response
	require.Equal(t, true, response.HasSignedUp)
	require.Equal(t, true, response.HasWallet)
}
