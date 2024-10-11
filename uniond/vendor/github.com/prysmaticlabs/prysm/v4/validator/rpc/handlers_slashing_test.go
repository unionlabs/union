package rpc

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts"
	"github.com/prysmaticlabs/prysm/v4/validator/db/kv"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/prysmaticlabs/prysm/v4/validator/slashing-protection-history/format"
	mocks "github.com/prysmaticlabs/prysm/v4/validator/testing"
)

func TestImportSlashingProtection_Preconditions(t *testing.T) {
	ctx := context.Background()
	localWalletDir := setupWalletDir(t)
	defaultWalletPath = localWalletDir

	// Empty JSON.
	s := &Server{
		walletDir: defaultWalletPath,
	}

	request := &ImportSlashingProtectionRequest{
		SlashingProtectionJson: "",
	}
	var buf bytes.Buffer
	err := json.NewEncoder(&buf).Encode(request)
	require.NoError(t, err)

	req := httptest.NewRequest(http.MethodPost, "/v2/validator/slashing-protection/import", &buf)
	wr := httptest.NewRecorder()
	wr.Body = &bytes.Buffer{}
	// No validator DB provided.
	s.ImportSlashingProtection(wr, req)
	require.Equal(t, http.StatusInternalServerError, wr.Code)
	require.StringContains(t, "could not find validator database", wr.Body.String())

	// Create Wallet and add to server for more realistic testing.
	opts := []accounts.Option{
		accounts.WithWalletDir(defaultWalletPath),
		accounts.WithKeymanagerType(keymanager.Local),
		accounts.WithWalletPassword(strongPass),
		accounts.WithSkipMnemonicConfirm(true),
	}
	acc, err := accounts.NewCLIManager(opts...)
	require.NoError(t, err)
	w, err := acc.WalletCreate(ctx)
	require.NoError(t, err)
	s.wallet = w

	numValidators := 1
	// Create public keys for the mock validator DB.
	pubKeys, err := mocks.CreateRandomPubKeys(numValidators)
	require.NoError(t, err)

	// Create a validator database.
	validatorDB, err := kv.NewKVStore(ctx, defaultWalletPath, &kv.Config{
		PubKeys: pubKeys,
	})
	require.NoError(t, err)
	s.valDB = validatorDB

	// Have to close it after import is done otherwise it complains db is not open.
	defer func() {
		require.NoError(t, validatorDB.Close())
	}()

	// Test empty JSON.
	wr = httptest.NewRecorder()
	wr.Body = &bytes.Buffer{}
	s.ImportSlashingProtection(wr, req)
	require.Equal(t, http.StatusBadRequest, wr.Code)
	require.StringContains(t, "empty slashing_protection_json specified", wr.Body.String())

	// Generate mock slashing history.
	attestingHistory := make([][]*kv.AttestationRecord, 0)
	proposalHistory := make([]kv.ProposalHistoryForPubkey, len(pubKeys))
	for i := 0; i < len(pubKeys); i++ {
		proposalHistory[i].Proposals = make([]kv.Proposal, 0)
	}
	mockJSON, err := mocks.MockSlashingProtectionJSON(pubKeys, attestingHistory, proposalHistory)
	require.NoError(t, err)

	// JSON encode the protection JSON and save it in rpc req.
	encoded, err := json.Marshal(mockJSON)
	require.NoError(t, err)
	request.SlashingProtectionJson = string(encoded)
	err = json.NewEncoder(&buf).Encode(request)
	require.NoError(t, err)

	req = httptest.NewRequest(http.MethodPost, "/v2/validator/slashing-protection/import", &buf)
	wr = httptest.NewRecorder()
	wr.Body = &bytes.Buffer{}
	s.ImportSlashingProtection(wr, req)
	require.Equal(t, http.StatusOK, wr.Code)
}

func TestExportSlashingProtection_Preconditions(t *testing.T) {
	ctx := context.Background()
	localWalletDir := setupWalletDir(t)
	defaultWalletPath = localWalletDir

	s := &Server{
		walletDir: defaultWalletPath,
	}
	req := httptest.NewRequest(http.MethodGet, "/v2/validator/slashing-protection/export", nil)
	wr := httptest.NewRecorder()
	wr.Body = &bytes.Buffer{}
	// No validator DB provided.
	s.ExportSlashingProtection(wr, req)
	require.Equal(t, http.StatusInternalServerError, wr.Code)
	require.StringContains(t, "could not find validator database", wr.Body.String())

	numValidators := 10
	// Create public keys for the mock validator DB.
	pubKeys, err := mocks.CreateRandomPubKeys(numValidators)
	require.NoError(t, err)

	// We create a validator database.
	validatorDB, err := kv.NewKVStore(ctx, defaultWalletPath, &kv.Config{
		PubKeys: pubKeys,
	})
	require.NoError(t, err)
	s.valDB = validatorDB

	// Have to close it after export is done otherwise it complains db is not open.
	defer func() {
		require.NoError(t, validatorDB.Close())
	}()
	genesisValidatorsRoot := [32]byte{1}
	err = validatorDB.SaveGenesisValidatorsRoot(ctx, genesisValidatorsRoot[:])
	require.NoError(t, err)
	wr = httptest.NewRecorder()
	wr.Body = &bytes.Buffer{}
	s.ExportSlashingProtection(wr, req)
	require.Equal(t, http.StatusOK, wr.Code)
}

func TestImportExportSlashingProtection_RoundTrip(t *testing.T) {
	ctx := context.Background()
	localWalletDir := setupWalletDir(t)
	defaultWalletPath = localWalletDir

	s := &Server{
		walletDir: defaultWalletPath,
	}

	numValidators := 10
	// Create public keys for the mock validator DB.
	pubKeys, err := mocks.CreateRandomPubKeys(numValidators)
	require.NoError(t, err)

	// Create a validator database.
	validatorDB, err := kv.NewKVStore(ctx, defaultWalletPath, &kv.Config{
		PubKeys: pubKeys,
	})
	require.NoError(t, err)
	s.valDB = validatorDB

	// Have to close it after import is done otherwise it complains db is not open.
	defer func() {
		require.NoError(t, validatorDB.Close())
	}()

	// Generate mock slashing history.
	attestingHistory := make([][]*kv.AttestationRecord, 0)
	proposalHistory := make([]kv.ProposalHistoryForPubkey, len(pubKeys))
	for i := 0; i < len(pubKeys); i++ {
		proposalHistory[i].Proposals = make([]kv.Proposal, 0)
	}
	mockJSON, err := mocks.MockSlashingProtectionJSON(pubKeys, attestingHistory, proposalHistory)
	require.NoError(t, err)

	// JSON encode the protection JSON and save it in rpc req.
	encoded, err := json.Marshal(mockJSON)
	require.NoError(t, err)
	request := &ImportSlashingProtectionRequest{
		SlashingProtectionJson: string(encoded),
	}
	var buf bytes.Buffer
	err = json.NewEncoder(&buf).Encode(request)
	require.NoError(t, err)

	req := httptest.NewRequest(http.MethodPost, "/v2/validator/slashing-protection/import", &buf)
	wr := httptest.NewRecorder()
	s.ImportSlashingProtection(wr, req)

	req = httptest.NewRequest(http.MethodGet, "/v2/validator/slashing-protection/export", nil)
	wr = httptest.NewRecorder()
	s.ExportSlashingProtection(wr, req)
	require.Equal(t, http.StatusOK, wr.Code)
	resp := &ExportSlashingProtectionResponse{}
	require.NoError(t, json.Unmarshal(wr.Body.Bytes(), resp))
	// Attempt to read the exported data and convert from string to EIP-3076.
	enc := []byte(resp.File)

	receivedJSON := &format.EIPSlashingProtectionFormat{}
	err = json.Unmarshal(enc, receivedJSON)
	require.NoError(t, err)

	require.DeepEqual(t, mockJSON.Metadata, receivedJSON.Metadata)
}
