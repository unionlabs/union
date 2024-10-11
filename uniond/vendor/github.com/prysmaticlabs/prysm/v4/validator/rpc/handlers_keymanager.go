package rpc

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	validatorServiceConfig "github.com/prysmaticlabs/prysm/v4/config/validator/service"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/validator"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	"github.com/prysmaticlabs/prysm/v4/validator/client"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/derived"
	slashingprotection "github.com/prysmaticlabs/prysm/v4/validator/slashing-protection-history"
	"github.com/prysmaticlabs/prysm/v4/validator/slashing-protection-history/format"
	"go.opencensus.io/trace"
	"google.golang.org/protobuf/types/known/emptypb"
)

// ListKeystores implements the standard validator key management API.
func (s *Server) ListKeystores(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.keymanagerAPI.ListKeystores")
	defer span.End()

	if s.validatorService == nil {
		httputil.HandleError(w, "Validator service not ready.", http.StatusServiceUnavailable)
		return
	}
	if !s.walletInitialized {
		httputil.HandleError(w, "Prysm Wallet not initialized. Please create a new wallet.", http.StatusServiceUnavailable)
		return
	}
	km, err := s.validatorService.Keymanager()
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	if s.wallet.KeymanagerKind() != keymanager.Derived && s.wallet.KeymanagerKind() != keymanager.Local {
		httputil.HandleError(w, errors.Wrap(err, "Prysm validator keys are not stored locally with this keymanager type").Error(), http.StatusInternalServerError)
		return
	}
	pubKeys, err := km.FetchValidatingPublicKeys(ctx)
	if err != nil {
		httputil.HandleError(w, errors.Wrap(err, "Could not retrieve keystores").Error(), http.StatusInternalServerError)
		return
	}
	keystoreResponse := make([]*Keystore, len(pubKeys))
	for i := 0; i < len(pubKeys); i++ {
		keystoreResponse[i] = &Keystore{
			ValidatingPubkey: hexutil.Encode(pubKeys[i][:]),
		}
		if s.wallet.KeymanagerKind() == keymanager.Derived {
			keystoreResponse[i].DerivationPath = fmt.Sprintf(derived.ValidatingKeyDerivationPathTemplate, i)
		}
	}
	response := &ListKeystoresResponse{
		Data: keystoreResponse,
	}
	httputil.WriteJson(w, response)
}

// ImportKeystores allows for importing keystores into Prysm with their slashing protection history.
func (s *Server) ImportKeystores(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.keymanagerAPI.ImportKeystores")
	defer span.End()

	if s.validatorService == nil {
		httputil.HandleError(w, "Validator service not ready.", http.StatusServiceUnavailable)
		return
	}
	if !s.walletInitialized {
		httputil.HandleError(w, "Prysm Wallet not initialized. Please create a new wallet.", http.StatusServiceUnavailable)
		return
	}
	km, err := s.validatorService.Keymanager()
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}

	var req ImportKeystoresRequest
	err = json.NewDecoder(r.Body).Decode(&req)
	switch {
	case err == io.EOF:
		httputil.HandleError(w, "No data submitted", http.StatusBadRequest)
		return
	case err != nil:
		httputil.HandleError(w, "Could not decode request body: "+err.Error(), http.StatusBadRequest)
		return
	}

	importer, ok := km.(keymanager.Importer)
	if !ok {
		statuses := make([]*keymanager.KeyStatus, len(req.Keystores))
		for i := 0; i < len(req.Keystores); i++ {
			statuses[i] = &keymanager.KeyStatus{
				Status:  keymanager.StatusError,
				Message: fmt.Sprintf("Keymanager kind %T cannot import local keys", km),
			}
		}
		httputil.WriteJson(w, &ImportKeystoresResponse{Data: statuses})
		return
	}
	if len(req.Keystores) == 0 {
		httputil.WriteJson(w, &ImportKeystoresResponse{})
		return
	}
	keystores := make([]*keymanager.Keystore, len(req.Keystores))
	for i := 0; i < len(req.Keystores); i++ {
		k := &keymanager.Keystore{}
		err = json.Unmarshal([]byte(req.Keystores[i]), k)
		if k.Description == "" && k.Name != "" {
			k.Description = k.Name
		}
		if err != nil {
			// we want to ignore unmarshal errors for now, the proper status is updated in importer.ImportKeystores
			k.Pubkey = "invalid format"
		}
		keystores[i] = k
	}
	if req.SlashingProtection != "" {
		if err := slashingprotection.ImportStandardProtectionJSON(
			ctx, s.valDB, bytes.NewBufferString(req.SlashingProtection),
		); err != nil {
			statuses := make([]*keymanager.KeyStatus, len(req.Keystores))
			for i := 0; i < len(req.Keystores); i++ {
				statuses[i] = &keymanager.KeyStatus{
					Status:  keymanager.StatusError,
					Message: fmt.Sprintf("could not import slashing protection: %v", err),
				}
			}
			httputil.WriteJson(w, &ImportKeystoresResponse{Data: statuses})
			return
		}
	}
	if len(req.Passwords) == 0 {
		req.Passwords = make([]string, len(req.Keystores))
	}

	// req.Passwords and req.Keystores are checked for 0 length in code above.
	if len(req.Passwords) > len(req.Keystores) {
		req.Passwords = req.Passwords[:len(req.Keystores)]
	} else if len(req.Passwords) < len(req.Keystores) {
		passwordList := make([]string, len(req.Keystores))
		copy(passwordList, req.Passwords)
		req.Passwords = passwordList
	}

	statuses, err := importer.ImportKeystores(ctx, keystores, req.Passwords)
	if err != nil {
		httputil.HandleError(w, errors.Wrap(err, "Could not import keystores").Error(), http.StatusInternalServerError)
		return
	}

	// If any of the keys imported had a slashing protection history before, we
	// stop marking them as deleted from our validator database.
	httputil.WriteJson(w, &ImportKeystoresResponse{Data: statuses})
}

// DeleteKeystores allows for deleting specified public keys from Prysm.
func (s *Server) DeleteKeystores(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.keymanagerAPI.DeleteKeystores")
	defer span.End()

	if s.validatorService == nil {
		httputil.HandleError(w, "Validator service not ready.", http.StatusServiceUnavailable)
		return
	}
	if !s.walletInitialized {
		httputil.HandleError(w, "Prysm Wallet not initialized. Please create a new wallet.", http.StatusServiceUnavailable)
		return
	}
	km, err := s.validatorService.Keymanager()
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	var req DeleteKeystoresRequest
	err = json.NewDecoder(r.Body).Decode(&req)
	switch {
	case err == io.EOF:
		httputil.HandleError(w, "No data submitted", http.StatusBadRequest)
		return
	case err != nil:
		httputil.HandleError(w, "Could not decode request body: "+err.Error(), http.StatusBadRequest)
		return
	}

	if len(req.Pubkeys) == 0 {
		httputil.WriteJson(w, &DeleteKeystoresResponse{Data: make([]*keymanager.KeyStatus, 0)})
		return
	}
	deleter, ok := km.(keymanager.Deleter)
	if !ok {
		sts := make([]*keymanager.KeyStatus, len(req.Pubkeys))
		for i := 0; i < len(req.Pubkeys); i++ {
			sts[i] = &keymanager.KeyStatus{
				Status:  keymanager.StatusError,
				Message: fmt.Sprintf("Keymanager kind %T cannot delete local keys", km),
			}
		}
		httputil.WriteJson(w, &DeleteKeystoresResponse{Data: sts})
		return
	}
	bytePubKeys := make([][]byte, len(req.Pubkeys))
	for i, pubkey := range req.Pubkeys {
		key, ok := shared.ValidateHex(w, fmt.Sprintf("pubkeys[%d]", i), pubkey, fieldparams.BLSPubkeyLength)
		if !ok {
			return
		}
		bytePubKeys[i] = key
	}
	statuses, err := deleter.DeleteKeystores(ctx, bytePubKeys)
	if err != nil {
		httputil.HandleError(w, errors.Wrap(err, "Could not delete keys").Error(), http.StatusInternalServerError)
		return
	}

	statuses, err = s.transformDeletedKeysStatuses(ctx, bytePubKeys, statuses)
	if err != nil {
		httputil.HandleError(w, errors.Wrap(err, "Could not transform deleted keys statuses").Error(), http.StatusInternalServerError)
		return
	}

	exportedHistory, err := s.slashingProtectionHistoryForDeletedKeys(ctx, bytePubKeys, statuses)
	if err != nil {
		log.WithError(err).Warn("Could not get slashing protection history for deleted keys")
		sts := make([]*keymanager.KeyStatus, len(req.Pubkeys))
		for i := 0; i < len(req.Pubkeys); i++ {
			sts[i] = &keymanager.KeyStatus{
				Status:  keymanager.StatusError,
				Message: "Could not export slashing protection history as existing non duplicate keys were deleted",
			}
		}
		httputil.WriteJson(w, &DeleteKeystoresResponse{Data: sts})
		return
	}
	jsonHist, err := json.Marshal(exportedHistory)
	if err != nil {
		httputil.HandleError(w, errors.Wrap(err, "Could not JSON marshal slashing protection history").Error(), http.StatusInternalServerError)
		return
	}

	response := &DeleteKeystoresResponse{
		Data:               statuses,
		SlashingProtection: string(jsonHist),
	}
	httputil.WriteJson(w, response)
}

// For a list of deleted keystore statuses, we check if any NOT_FOUND status actually
// has a corresponding public key in the database. In this case, we transform the status
// to NOT_ACTIVE, as we do have slashing protection history for it and should not mark it
// as NOT_FOUND when returning a response to the caller.
func (s *Server) transformDeletedKeysStatuses(
	ctx context.Context, pubKeys [][]byte, statuses []*keymanager.KeyStatus,
) ([]*keymanager.KeyStatus, error) {
	pubKeysInDB, err := s.publicKeysInDB(ctx)
	if err != nil {
		return nil, errors.Wrap(err, "could not get public keys from DB")
	}
	if len(pubKeysInDB) > 0 {
		for i := 0; i < len(pubKeys); i++ {
			keyExistsInDB := pubKeysInDB[bytesutil.ToBytes48(pubKeys[i])]
			if keyExistsInDB && statuses[i].Status == keymanager.StatusNotFound {
				statuses[i].Status = keymanager.StatusNotActive
			}
		}
	}
	return statuses, nil
}

// Gets a map of all public keys in the database, useful for O(1) lookups.
func (s *Server) publicKeysInDB(ctx context.Context) (map[[fieldparams.BLSPubkeyLength]byte]bool, error) {
	pubKeysInDB := make(map[[fieldparams.BLSPubkeyLength]byte]bool)
	attestedPublicKeys, err := s.valDB.AttestedPublicKeys(ctx)
	if err != nil {
		return nil, fmt.Errorf("could not get attested public keys from DB: %v", err)
	}
	proposedPublicKeys, err := s.valDB.ProposedPublicKeys(ctx)
	if err != nil {
		return nil, fmt.Errorf("could not get proposed public keys from DB: %v", err)
	}
	for _, pk := range append(attestedPublicKeys, proposedPublicKeys...) {
		pubKeysInDB[pk] = true
	}
	return pubKeysInDB, nil
}

// Exports slashing protection data for a list of DELETED or NOT_ACTIVE keys only to be used
// as part of the DeleteKeystores endpoint.
func (s *Server) slashingProtectionHistoryForDeletedKeys(
	ctx context.Context, pubKeys [][]byte, statuses []*keymanager.KeyStatus,
) (*format.EIPSlashingProtectionFormat, error) {
	// We select the keys that were DELETED or NOT_ACTIVE from the previous action
	// and use that to filter our slashing protection export.
	filteredKeys := make([][]byte, 0, len(pubKeys))
	for i, pk := range pubKeys {
		if statuses[i].Status == keymanager.StatusDeleted ||
			statuses[i].Status == keymanager.StatusNotActive {
			filteredKeys = append(filteredKeys, pk)
		}
	}
	return slashingprotection.ExportStandardProtectionJSON(ctx, s.valDB, filteredKeys...)
}

// SetVoluntaryExit creates a signed voluntary exit message and returns a VoluntaryExit object.
func (s *Server) SetVoluntaryExit(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.keymanagerAPI.SetVoluntaryExit")
	defer span.End()

	if s.validatorService == nil {
		httputil.HandleError(w, "Validator service not ready", http.StatusServiceUnavailable)
		return
	}

	if !s.walletInitialized {
		httputil.HandleError(w, "No wallet found", http.StatusServiceUnavailable)
		return
	}

	km, err := s.validatorService.Keymanager()
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	_, pubkey, ok := shared.HexFromRoute(w, r, "pubkey", fieldparams.BLSPubkeyLength)
	if !ok {
		return
	}
	rawEpoch, e, ok := shared.UintFromQuery(w, r, "epoch", false)
	if !ok {
		return
	}
	epoch := primitives.Epoch(e)

	if rawEpoch == "" {
		genesisResponse, err := s.beaconNodeClient.GetGenesis(ctx, &emptypb.Empty{})
		if err != nil {
			httputil.HandleError(w, errors.Wrap(err, "Failed to get genesis time").Error(), http.StatusInternalServerError)
			return
		}
		currentEpoch, err := client.CurrentEpoch(genesisResponse.GenesisTime)
		if err != nil {
			httputil.HandleError(w, errors.Wrap(err, "Failed to get current epoch").Error(), http.StatusInternalServerError)
			return
		}
		epoch = currentEpoch
	}
	sve, err := client.CreateSignedVoluntaryExit(
		ctx,
		s.beaconNodeValidatorClient,
		km.Sign,
		pubkey,
		epoch,
	)
	if err != nil {
		httputil.HandleError(w, errors.Wrap(err, "Could not create voluntary exit").Error(), http.StatusInternalServerError)
		return
	}

	response := &SetVoluntaryExitResponse{
		Data: &shared.SignedVoluntaryExit{
			Message: &shared.VoluntaryExit{
				Epoch:          fmt.Sprintf("%d", sve.Exit.Epoch),
				ValidatorIndex: fmt.Sprintf("%d", sve.Exit.ValidatorIndex),
			},
			Signature: hexutil.Encode(sve.Signature),
		},
	}
	httputil.WriteJson(w, response)
}

// ListRemoteKeys returns a list of all public keys defined for web3signer keymanager type.
func (s *Server) ListRemoteKeys(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.keymanagerAPI.ListRemoteKeys")
	defer span.End()

	if s.validatorService == nil {
		httputil.HandleError(w, "Validator service not ready.", http.StatusServiceUnavailable)
		return
	}
	if !s.walletInitialized {
		httputil.HandleError(w, "Prysm Wallet not initialized. Please create a new wallet.", http.StatusServiceUnavailable)
		return
	}
	km, err := s.validatorService.Keymanager()
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	if s.wallet.KeymanagerKind() != keymanager.Web3Signer {
		httputil.HandleError(w, "Prysm Wallet is not of type Web3Signer. Please execute validator client with web3signer flags.", http.StatusInternalServerError)
		return
	}
	pubKeys, err := km.FetchValidatingPublicKeys(ctx)
	if err != nil {
		httputil.HandleError(w, errors.Errorf("Could not retrieve public keys: %v", err).Error(), http.StatusInternalServerError)
		return
	}
	keystoreResponse := make([]*RemoteKey, len(pubKeys))
	for i := 0; i < len(pubKeys); i++ {
		keystoreResponse[i] = &RemoteKey{
			Pubkey:   hexutil.Encode(pubKeys[i][:]),
			Url:      s.validatorService.Web3SignerConfig.BaseEndpoint,
			Readonly: true,
		}
	}

	response := &ListRemoteKeysResponse{
		Data: keystoreResponse,
	}
	httputil.WriteJson(w, response)
}

// ImportRemoteKeys imports a list of public keys defined for web3signer keymanager type.
func (s *Server) ImportRemoteKeys(w http.ResponseWriter, r *http.Request) {
	_, span := trace.StartSpan(r.Context(), "validator.keymanagerAPI.ImportRemoteKeys")
	defer span.End()

	if s.validatorService == nil {
		httputil.HandleError(w, "Validator service not ready.", http.StatusServiceUnavailable)
		return
	}
	if !s.walletInitialized {
		httputil.HandleError(w, "Prysm Wallet not initialized. Please create a new wallet.", http.StatusServiceUnavailable)
		return
	}
	km, err := s.validatorService.Keymanager()
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	if s.wallet.KeymanagerKind() != keymanager.Web3Signer {
		httputil.HandleError(w, "Prysm Wallet is not of type Web3Signer. Please execute validator client with web3signer flags.", http.StatusInternalServerError)
		return
	}

	var req ImportRemoteKeysRequest
	err = json.NewDecoder(r.Body).Decode(&req)
	switch {
	case err == io.EOF:
		httputil.HandleError(w, "No data submitted", http.StatusBadRequest)
		return
	case err != nil:
		httputil.HandleError(w, "Could not decode request body: "+err.Error(), http.StatusBadRequest)
		return
	}

	adder, ok := km.(keymanager.PublicKeyAdder)
	if !ok {
		statuses := make([]*keymanager.KeyStatus, len(req.RemoteKeys))
		for i := 0; i < len(req.RemoteKeys); i++ {
			statuses[i] = &keymanager.KeyStatus{
				Status:  keymanager.StatusError,
				Message: "Keymanager kind cannot import public keys for web3signer keymanager type.",
			}
		}
		httputil.WriteJson(w, &RemoteKeysResponse{Data: statuses})
		return
	}

	remoteKeys := make([]string, len(req.RemoteKeys))
	isUrlUsed := false
	for i, obj := range req.RemoteKeys {
		remoteKeys[i] = obj.Pubkey
		if obj.Url != "" {
			isUrlUsed = true
		}
	}
	if isUrlUsed {
		log.Warnf("Setting web3signer base url for IMPORTED keys is not supported. Prysm only uses the url from --validators-external-signer-url flag for web3signerKeymanagerKind.")
	}

	httputil.WriteJson(w, &RemoteKeysResponse{Data: adder.AddPublicKeys(remoteKeys)})
}

// DeleteRemoteKeys deletes a list of public keys defined for web3signer keymanager type.
func (s *Server) DeleteRemoteKeys(w http.ResponseWriter, r *http.Request) {
	_, span := trace.StartSpan(r.Context(), "validator.keymanagerAPI.DeleteRemoteKeys")
	defer span.End()

	if s.validatorService == nil {
		httputil.HandleError(w, "Validator service not ready.", http.StatusServiceUnavailable)
		return
	}
	if !s.walletInitialized {
		httputil.HandleError(w, "Prysm Wallet not initialized. Please create a new wallet.", http.StatusServiceUnavailable)
		return
	}
	km, err := s.validatorService.Keymanager()
	if err != nil {
		httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	if s.wallet.KeymanagerKind() != keymanager.Web3Signer {
		httputil.HandleError(w, "Prysm Wallet is not of type Web3Signer. Please execute validator client with web3signer flags.", http.StatusInternalServerError)
		return
	}

	var req DeleteRemoteKeysRequest
	err = json.NewDecoder(r.Body).Decode(&req)
	switch {
	case err == io.EOF:
		httputil.HandleError(w, "No data submitted", http.StatusBadRequest)
		return
	case err != nil:
		httputil.HandleError(w, "Could not decode request body: "+err.Error(), http.StatusBadRequest)
		return
	}

	deleter, ok := km.(keymanager.PublicKeyDeleter)
	if !ok {
		statuses := make([]*keymanager.KeyStatus, len(req.Pubkeys))
		for i := 0; i < len(req.Pubkeys); i++ {
			statuses[i] = &keymanager.KeyStatus{
				Status:  keymanager.StatusError,
				Message: "Keymanager kind cannot delete public keys for web3signer keymanager type.",
			}
		}
		httputil.WriteJson(w, &RemoteKeysResponse{Data: statuses})
		return
	}

	httputil.WriteJson(w, RemoteKeysResponse{Data: deleter.DeletePublicKeys(req.Pubkeys)})
}

// ListFeeRecipientByPubkey returns the public key to eth address mapping object to the end user.
func (s *Server) ListFeeRecipientByPubkey(w http.ResponseWriter, r *http.Request) {
	_, span := trace.StartSpan(r.Context(), "validator.keymanagerAPI.ListFeeRecipientByPubkey")
	defer span.End()

	if s.validatorService == nil {
		httputil.HandleError(w, "Validator service not ready.", http.StatusServiceUnavailable)
		return
	}
	rawPubkey, pubkey, ok := shared.HexFromRoute(w, r, "pubkey", fieldparams.BLSPubkeyLength)
	if !ok {
		return
	}
	finalResp := &GetFeeRecipientByPubkeyResponse{
		Data: &FeeRecipient{
			Pubkey: rawPubkey,
		},
	}

	proposerSettings := s.validatorService.ProposerSettings()

	// If fee recipient is defined for this specific pubkey in proposer configuration, use it
	if proposerSettings != nil && proposerSettings.ProposeConfig != nil {
		proposerOption, found := proposerSettings.ProposeConfig[bytesutil.ToBytes48(pubkey)]

		if found && proposerOption.FeeRecipientConfig != nil {
			finalResp.Data.Ethaddress = proposerOption.FeeRecipientConfig.FeeRecipient.String()
			httputil.WriteJson(w, finalResp)
			return
		}
	}

	// If fee recipient is defined in default configuration, use it
	if proposerSettings != nil && proposerSettings.DefaultConfig != nil && proposerSettings.DefaultConfig.FeeRecipientConfig != nil {
		finalResp.Data.Ethaddress = proposerSettings.DefaultConfig.FeeRecipientConfig.FeeRecipient.String()
		httputil.WriteJson(w, finalResp)
		return
	}

	httputil.HandleError(w, "No fee recipient set", http.StatusBadRequest)
}

// SetFeeRecipientByPubkey updates the eth address mapped to the public key.
func (s *Server) SetFeeRecipientByPubkey(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.keymanagerAPI.SetFeeRecipientByPubkey")
	defer span.End()

	if s.validatorService == nil {
		httputil.HandleError(w, "Validator service not ready.", http.StatusServiceUnavailable)
		return
	}
	_, pubkey, ok := shared.HexFromRoute(w, r, "pubkey", fieldparams.BLSPubkeyLength)
	if !ok {
		return
	}

	var req SetFeeRecipientByPubkeyRequest
	err := json.NewDecoder(r.Body).Decode(&req)
	switch {
	case err == io.EOF:
		httputil.HandleError(w, "No data submitted", http.StatusBadRequest)
		return
	case err != nil:
		httputil.HandleError(w, "Could not decode request body: "+err.Error(), http.StatusBadRequest)
		return
	}
	ethAddress, valid := shared.ValidateHex(w, "ethaddress", req.Ethaddress, fieldparams.FeeRecipientLength)
	if !valid {
		return
	}
	feeRecipient := common.BytesToAddress(ethAddress)
	settings := s.validatorService.ProposerSettings()
	switch {
	case settings == nil:
		settings = &validatorServiceConfig.ProposerSettings{
			ProposeConfig: map[[fieldparams.BLSPubkeyLength]byte]*validatorServiceConfig.ProposerOption{
				bytesutil.ToBytes48(pubkey): {
					FeeRecipientConfig: &validatorServiceConfig.FeeRecipientConfig{
						FeeRecipient: feeRecipient,
					},
					BuilderConfig: nil,
				},
			},
			DefaultConfig: nil,
		}
	case settings.ProposeConfig == nil:
		var builderConfig *validatorServiceConfig.BuilderConfig
		if settings.DefaultConfig != nil && settings.DefaultConfig.BuilderConfig != nil {
			builderConfig = settings.DefaultConfig.BuilderConfig.Clone()
		}
		settings.ProposeConfig = map[[fieldparams.BLSPubkeyLength]byte]*validatorServiceConfig.ProposerOption{
			bytesutil.ToBytes48(pubkey): {
				FeeRecipientConfig: &validatorServiceConfig.FeeRecipientConfig{
					FeeRecipient: feeRecipient,
				},
				BuilderConfig: builderConfig,
			},
		}
	default:
		proposerOption, found := settings.ProposeConfig[bytesutil.ToBytes48(pubkey)]
		if found && proposerOption != nil {
			proposerOption.FeeRecipientConfig = &validatorServiceConfig.FeeRecipientConfig{
				FeeRecipient: feeRecipient,
			}
		} else {
			var builderConfig = &validatorServiceConfig.BuilderConfig{}
			if settings.DefaultConfig != nil && settings.DefaultConfig.BuilderConfig != nil {
				builderConfig = settings.DefaultConfig.BuilderConfig.Clone()
			}
			settings.ProposeConfig[bytesutil.ToBytes48(pubkey)] = &validatorServiceConfig.ProposerOption{
				FeeRecipientConfig: &validatorServiceConfig.FeeRecipientConfig{
					FeeRecipient: feeRecipient,
				},
				BuilderConfig: builderConfig,
			}
		}
	}
	// save the settings
	if err := s.validatorService.SetProposerSettings(ctx, settings); err != nil {
		httputil.HandleError(w, "Could not set proposer settings: "+err.Error(), http.StatusInternalServerError)
		return
	}
	// override the 200 success with 202 according to the specs
	w.WriteHeader(http.StatusAccepted)
}

// DeleteFeeRecipientByPubkey updates the eth address mapped to the public key to the default fee recipient listed
func (s *Server) DeleteFeeRecipientByPubkey(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.keymanagerAPI.DeleteFeeRecipientByPubkey")
	defer span.End()

	if s.validatorService == nil {
		httputil.HandleError(w, "Validator service not ready.", http.StatusServiceUnavailable)
		return
	}
	_, pubkey, ok := shared.HexFromRoute(w, r, "pubkey", fieldparams.BLSPubkeyLength)
	if !ok {
		return
	}

	settings := s.validatorService.ProposerSettings()
	if settings != nil && settings.ProposeConfig != nil {
		proposerOption, found := settings.ProposeConfig[bytesutil.ToBytes48(pubkey)]
		if found {
			proposerOption.FeeRecipientConfig = nil
		}
	}

	// save the settings
	if err := s.validatorService.SetProposerSettings(ctx, settings); err != nil {
		httputil.HandleError(w, "Could not set proposer settings: "+err.Error(), http.StatusInternalServerError)
		return
	}

	// override the 200 success with 204 according to the specs
	w.WriteHeader(http.StatusNoContent)
}

// GetGasLimit returns the gas limit measured in gwei defined for the custom mev builder by public key
func (s *Server) GetGasLimit(w http.ResponseWriter, r *http.Request) {
	_, span := trace.StartSpan(r.Context(), "validator.keymanagerAPI.GetGasLimit")
	defer span.End()

	if s.validatorService == nil {
		httputil.HandleError(w, "Validator service not ready", http.StatusServiceUnavailable)
		return
	}

	rawPubkey, pubkey, ok := shared.HexFromRoute(w, r, "pubkey", fieldparams.BLSPubkeyLength)
	if !ok {
		return
	}

	resp := &GetGasLimitResponse{
		Data: &GasLimitMetaData{
			Pubkey:   rawPubkey,
			GasLimit: fmt.Sprintf("%d", params.BeaconConfig().DefaultBuilderGasLimit),
		},
	}
	settings := s.validatorService.ProposerSettings()
	if settings != nil {
		proposerOption, found := settings.ProposeConfig[bytesutil.ToBytes48(pubkey)]
		if found {
			if proposerOption.BuilderConfig != nil {
				resp.Data.GasLimit = fmt.Sprintf("%d", proposerOption.BuilderConfig.GasLimit)
			}
		} else if settings.DefaultConfig != nil && settings.DefaultConfig.BuilderConfig != nil {
			resp.Data.GasLimit = fmt.Sprintf("%d", s.validatorService.ProposerSettings().DefaultConfig.BuilderConfig.GasLimit)
		}
	}
	httputil.WriteJson(w, resp)
}

// SetGasLimit updates the gas limit by public key
func (s *Server) SetGasLimit(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.keymanagerAPI.SetGasLimit")
	defer span.End()

	if s.validatorService == nil {
		httputil.HandleError(w, "Validator service not ready", http.StatusServiceUnavailable)
		return
	}
	_, pubkey, ok := shared.HexFromRoute(w, r, "pubkey", fieldparams.BLSPubkeyLength)
	if !ok {
		return
	}

	var req SetGasLimitRequest
	err := json.NewDecoder(r.Body).Decode(&req)
	switch {
	case err == io.EOF:
		httputil.HandleError(w, "No data submitted", http.StatusBadRequest)
		return
	case err != nil:
		httputil.HandleError(w, "Could not decode request body: "+err.Error(), http.StatusBadRequest)
		return
	}

	gasLimit, valid := shared.ValidateUint(w, "gas_limit", req.GasLimit)
	if !valid {
		return
	}

	settings := s.validatorService.ProposerSettings()
	if settings == nil {
		httputil.HandleError(w, "No proposer settings were found to update", http.StatusInternalServerError)
		return
	} else if settings.ProposeConfig == nil {
		if settings.DefaultConfig == nil || settings.DefaultConfig.BuilderConfig == nil || !settings.DefaultConfig.BuilderConfig.Enabled {
			httputil.HandleError(w, "Gas limit changes only apply when builder is enabled", http.StatusInternalServerError)
			return
		}
		settings.ProposeConfig = make(map[[fieldparams.BLSPubkeyLength]byte]*validatorServiceConfig.ProposerOption)
		option := settings.DefaultConfig.Clone()
		option.BuilderConfig.GasLimit = validator.Uint64(gasLimit)
		settings.ProposeConfig[bytesutil.ToBytes48(pubkey)] = option
	} else {
		proposerOption, found := settings.ProposeConfig[bytesutil.ToBytes48(pubkey)]
		if found {
			if proposerOption.BuilderConfig == nil || !proposerOption.BuilderConfig.Enabled {
				httputil.HandleError(w, "Gas limit changes only apply when builder is enabled", http.StatusInternalServerError)
				return
			} else {
				proposerOption.BuilderConfig.GasLimit = validator.Uint64(gasLimit)
			}
		} else {
			if settings.DefaultConfig == nil {
				httputil.HandleError(w, "Gas limit changes only apply when builder is enabled", http.StatusInternalServerError)
				return
			}
			option := settings.DefaultConfig.Clone()
			option.BuilderConfig.GasLimit = validator.Uint64(gasLimit)
			settings.ProposeConfig[bytesutil.ToBytes48(pubkey)] = option
		}
	}
	// save the settings
	if err := s.validatorService.SetProposerSettings(ctx, settings); err != nil {
		httputil.HandleError(w, "Could not set proposer settings: "+err.Error(), http.StatusInternalServerError)
		return
	}
	w.WriteHeader(http.StatusAccepted)
}

// DeleteGasLimit deletes the gas limit by public key
func (s *Server) DeleteGasLimit(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.keymanagerAPI.DeleteGasLimit")
	defer span.End()

	if s.validatorService == nil {
		httputil.HandleError(w, "Validator service not ready", http.StatusServiceUnavailable)
		return
	}
	rawPubkey, pubkey, ok := shared.HexFromRoute(w, r, "pubkey", fieldparams.BLSPubkeyLength)
	if !ok {
		return
	}

	proposerSettings := s.validatorService.ProposerSettings()
	if proposerSettings != nil && proposerSettings.ProposeConfig != nil {
		proposerOption, found := proposerSettings.ProposeConfig[bytesutil.ToBytes48(pubkey)]
		if found && proposerOption.BuilderConfig != nil {
			// If proposerSettings has default value, use it.
			if proposerSettings.DefaultConfig != nil && proposerSettings.DefaultConfig.BuilderConfig != nil {
				proposerOption.BuilderConfig.GasLimit = proposerSettings.DefaultConfig.BuilderConfig.GasLimit
			} else {
				// Fallback to using global default.
				proposerOption.BuilderConfig.GasLimit = validator.Uint64(params.BeaconConfig().DefaultBuilderGasLimit)
			}
			// save the settings
			if err := s.validatorService.SetProposerSettings(ctx, proposerSettings); err != nil {
				httputil.HandleError(w, "Could not set proposer settings: "+err.Error(), http.StatusBadRequest)
				return
			}
			// Successfully deleted gas limit (reset to proposer config default or global default).
			// Return with success http code "204".
			w.WriteHeader(http.StatusNoContent)
			return
		}
	}
	// Otherwise, either no proposerOption is found for the pubkey or proposerOption.BuilderConfig is not enabled at all,
	// we respond "not found".
	httputil.HandleError(w, fmt.Sprintf("No gas limit found for pubkey %q", rawPubkey), http.StatusNotFound)
}
