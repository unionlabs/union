package rpc

import (
	"archive/zip"
	"bytes"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strconv"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/api/pagination"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/cmd"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/petnames"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/derived"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/local"
	"go.opencensus.io/trace"
)

// ListAccounts allows retrieval of validating keys and their petnames
// for a user's wallet via RPC.
func (s *Server) ListAccounts(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.web.accounts.ListAccounts")
	defer span.End()
	if s.validatorService == nil {
		httputil.HandleError(w, "Validator service not ready.", http.StatusServiceUnavailable)
		return
	}
	if !s.walletInitialized {
		httputil.HandleError(w, "Prysm Wallet not initialized. Please create a new wallet.", http.StatusServiceUnavailable)
		return
	}
	pageSize := r.URL.Query().Get("page_size")
	var ps int64
	if pageSize != "" {
		psi, err := strconv.ParseInt(pageSize, 10, 32)
		if err != nil {
			httputil.HandleError(w, errors.Wrap(err, "Failed to parse page_size").Error(), http.StatusBadRequest)
			return
		}
		ps = psi
	}
	pageToken := r.URL.Query().Get("page_token")
	publicKeys := r.URL.Query()["public_keys"]
	pubkeys := make([][]byte, len(publicKeys))
	for i, key := range publicKeys {
		k, ok := shared.ValidateHex(w, fmt.Sprintf("PublicKeys[%d]", i), key, fieldparams.BLSPubkeyLength)
		if !ok {
			return
		}
		pubkeys[i] = bytesutil.SafeCopyBytes(k)
	}
	if int(ps) > cmd.Get().MaxRPCPageSize {
		httputil.HandleError(w, fmt.Sprintf("Requested page size %d can not be greater than max size %d",
			ps, cmd.Get().MaxRPCPageSize), http.StatusBadRequest)
		return
	}
	km, err := s.validatorService.Keymanager()
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	keys, err := km.FetchValidatingPublicKeys(ctx)
	if err != nil {
		httputil.HandleError(w, errors.Errorf("Could not retrieve public keys: %v", err).Error(), http.StatusInternalServerError)
		return
	}
	accs := make([]*Account, len(keys))
	for i := 0; i < len(keys); i++ {
		accs[i] = &Account{
			ValidatingPublicKey: hexutil.Encode(keys[i][:]),
			AccountName:         petnames.DeterministicName(keys[i][:], "-"),
		}
		if s.wallet.KeymanagerKind() == keymanager.Derived {
			accs[i].DerivationPath = fmt.Sprintf(derived.ValidatingKeyDerivationPathTemplate, i)
		}
	}
	if r.URL.Query().Get("all") == "true" {
		httputil.WriteJson(w, &ListAccountsResponse{
			Accounts:      accs,
			TotalSize:     int32(len(keys)),
			NextPageToken: "",
		})
		return
	}
	start, end, nextPageToken, err := pagination.StartAndEndPage(pageToken, int(ps), len(keys))
	if err != nil {
		httputil.HandleError(w, fmt.Errorf("Could not paginate results: %v",
			err).Error(), http.StatusInternalServerError)
		return
	}
	httputil.WriteJson(w, &ListAccountsResponse{
		Accounts:      accs[start:end],
		TotalSize:     int32(len(keys)),
		NextPageToken: nextPageToken,
	})
}

// BackupAccounts creates a zip file containing EIP-2335 keystores for the user's
// specified public keys by encrypting them with the specified password.
func (s *Server) BackupAccounts(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.web.accounts.ListAccounts")
	defer span.End()
	if s.validatorService == nil {
		httputil.HandleError(w, "Validator service not ready.", http.StatusServiceUnavailable)
		return
	}
	if !s.walletInitialized {
		httputil.HandleError(w, "Prysm Wallet not initialized. Please create a new wallet.", http.StatusServiceUnavailable)
		return
	}

	var req BackupAccountsRequest
	err := json.NewDecoder(r.Body).Decode(&req)
	switch {
	case err == io.EOF:
		httputil.HandleError(w, "No data submitted", http.StatusBadRequest)
		return
	case err != nil:
		httputil.HandleError(w, "Could not decode request body: "+err.Error(), http.StatusBadRequest)
		return
	}

	if req.PublicKeys == nil || len(req.PublicKeys) < 1 {
		httputil.HandleError(w, "No public keys specified to backup", http.StatusBadRequest)
		return
	}
	if req.BackupPassword == "" {
		httputil.HandleError(w, "Backup password cannot be empty", http.StatusBadRequest)
		return
	}

	km, err := s.validatorService.Keymanager()
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	pubKeys := make([]bls.PublicKey, len(req.PublicKeys))
	for i, key := range req.PublicKeys {
		byteskey, ok := shared.ValidateHex(w, "pubkey", key, fieldparams.BLSPubkeyLength)
		if !ok {
			return
		}
		pubKey, err := bls.PublicKeyFromBytes(byteskey)
		if err != nil {
			httputil.HandleError(w, errors.Wrap(err, fmt.Sprintf("%s Not a valid BLS public key", key)).Error(), http.StatusBadRequest)
			return
		}
		pubKeys[i] = pubKey
	}

	var keystoresToBackup []*keymanager.Keystore
	switch km := km.(type) {
	case *local.Keymanager:
		keystoresToBackup, err = km.ExtractKeystores(ctx, pubKeys, req.BackupPassword)
		if err != nil {
			httputil.HandleError(w, errors.Wrap(err, "Could not backup accounts for local keymanager").Error(), http.StatusInternalServerError)
			return
		}
	case *derived.Keymanager:
		keystoresToBackup, err = km.ExtractKeystores(ctx, pubKeys, req.BackupPassword)
		if err != nil {
			httputil.HandleError(w, errors.Wrap(err, "Could not backup accounts for derived keymanager").Error(), http.StatusInternalServerError)
			return
		}
	default:
		httputil.HandleError(w, "Only HD or IMPORTED wallets can backup accounts", http.StatusBadRequest)
		return
	}
	if len(keystoresToBackup) == 0 {
		httputil.HandleError(w, "No keystores to backup", http.StatusBadRequest)
		return
	}

	buf := new(bytes.Buffer)
	writer := zip.NewWriter(buf)
	for i, k := range keystoresToBackup {
		encodedFile, err := json.MarshalIndent(k, "", "\t")
		if err != nil {
			if err := writer.Close(); err != nil {
				log.WithError(err).Error("Could not close zip file after writing")
			}
			httputil.HandleError(w, "could not marshal keystore to JSON file", http.StatusInternalServerError)
			return
		}
		f, err := writer.Create(fmt.Sprintf("keystore-%d.json", i))
		if err != nil {
			if err := writer.Close(); err != nil {
				log.WithError(err).Error("Could not close zip file after writing")
			}
			httputil.HandleError(w, "Could not write keystore file to zip", http.StatusInternalServerError)
			return
		}
		if _, err = f.Write(encodedFile); err != nil {
			if err := writer.Close(); err != nil {
				log.WithError(err).Error("Could not close zip file after writing")
			}
			httputil.HandleError(w, "Could not write keystore file contents", http.StatusBadRequest)
			return
		}
	}
	if err := writer.Close(); err != nil {
		log.WithError(err).Error("Could not close zip file after writing")
	}
	httputil.WriteJson(w, &BackupAccountsResponse{
		ZipFile: base64.StdEncoding.EncodeToString(buf.Bytes()), // convert to base64 string for processing
	})
}

// VoluntaryExit performs a voluntary exit for the validator keys specified in a request.
func (s *Server) VoluntaryExit(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.web.accounts.VoluntaryExit")
	defer span.End()
	if s.validatorService == nil {
		httputil.HandleError(w, "Validator service not ready.", http.StatusServiceUnavailable)
		return
	}
	if !s.walletInitialized {
		httputil.HandleError(w, "Prysm Wallet not initialized. Please create a new wallet.", http.StatusServiceUnavailable)
		return
	}
	var req VoluntaryExitRequest
	err := json.NewDecoder(r.Body).Decode(&req)
	switch {
	case err == io.EOF:
		httputil.HandleError(w, "No data submitted", http.StatusBadRequest)
		return
	case err != nil:
		httputil.HandleError(w, "Could not decode request body: "+err.Error(), http.StatusBadRequest)
		return
	}
	if len(req.PublicKeys) == 0 {
		httputil.HandleError(w, "No public keys specified to delete", http.StatusBadRequest)
		return
	}
	km, err := s.validatorService.Keymanager()
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	pubKeys := make([][]byte, len(req.PublicKeys))
	for i, key := range req.PublicKeys {
		byteskey, ok := shared.ValidateHex(w, "pubkey", key, fieldparams.BLSPubkeyLength)
		if !ok {
			return
		}
		pubKeys[i] = byteskey
	}
	cfg := accounts.PerformExitCfg{
		ValidatorClient:  s.beaconNodeValidatorClient,
		NodeClient:       s.beaconNodeClient,
		Keymanager:       km,
		RawPubKeys:       pubKeys,
		FormattedPubKeys: req.PublicKeys,
	}
	rawExitedKeys, _, err := accounts.PerformVoluntaryExit(ctx, cfg)
	if err != nil {
		httputil.HandleError(w, errors.Wrap(err, "Could not perform voluntary exit").Error(), http.StatusInternalServerError)
		return
	}
	httputil.WriteJson(w, &VoluntaryExitResponse{
		ExitedKeys: rawExitedKeys,
	})
}
