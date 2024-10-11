package remote_web3signer

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"path/filepath"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/go-playground/validator/v10"
	"github.com/logrusorgru/aurora"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/async/event"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	validatorpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/validator-client"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/petnames"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/remote-web3signer/internal"
	web3signerv1 "github.com/prysmaticlabs/prysm/v4/validator/keymanager/remote-web3signer/v1"
	log "github.com/sirupsen/logrus"
)

// SetupConfig includes configuration values for initializing.
// a keymanager, such as passwords, the wallet, and more.
// Web3Signer contains one public keys option. Either through a URL or a static key list.
type SetupConfig struct {
	BaseEndpoint          string
	GenesisValidatorsRoot []byte

	// Either URL or keylist must be set.
	// If the URL is set, the keymanager will fetch the public keys from the URL.
	// caution: this option is susceptible to slashing if the web3signer's validator keys are shared across validators
	PublicKeysURL string

	// Either URL or keylist must be set.
	// a static list of public keys to be passed by the user to determine what accounts should sign.
	// This will provide a layer of safety against slashing if the web3signer is shared across validators.
	ProvidedPublicKeys [][48]byte
}

// Keymanager defines the web3signer keymanager.
type Keymanager struct {
	client                internal.HttpSignerClient
	genesisValidatorsRoot []byte
	publicKeysURL         string
	providedPublicKeys    [][48]byte
	accountsChangedFeed   *event.Feed
	validator             *validator.Validate
	publicKeysUrlCalled   bool
}

// NewKeymanager instantiates a new web3signer key manager.
func NewKeymanager(_ context.Context, cfg *SetupConfig) (*Keymanager, error) {
	if cfg.BaseEndpoint == "" || !bytesutil.IsValidRoot(cfg.GenesisValidatorsRoot) {
		return nil, fmt.Errorf("invalid setup config, one or more configs are empty: BaseEndpoint: %v, GenesisValidatorsRoot: %#x", cfg.BaseEndpoint, cfg.GenesisValidatorsRoot)
	}
	client, err := internal.NewApiClient(cfg.BaseEndpoint)
	if err != nil {
		return nil, errors.Wrap(err, "could not create apiClient")
	}
	return &Keymanager{
		client:                internal.HttpSignerClient(client),
		genesisValidatorsRoot: cfg.GenesisValidatorsRoot,
		accountsChangedFeed:   new(event.Feed),
		publicKeysURL:         cfg.PublicKeysURL,
		providedPublicKeys:    cfg.ProvidedPublicKeys,
		validator:             validator.New(),
		publicKeysUrlCalled:   false,
	}, nil
}

// FetchValidatingPublicKeys fetches the validating public keys
// from the remote server or from the provided keys if there are no existing public keys set
// or provides the existing keys in the keymanager.
func (km *Keymanager) FetchValidatingPublicKeys(ctx context.Context) ([][fieldparams.BLSPubkeyLength]byte, error) {
	if km.publicKeysURL != "" && !km.publicKeysUrlCalled {
		providedPublicKeys, err := km.client.GetPublicKeys(ctx, km.publicKeysURL)
		if err != nil {
			erroredResponsesTotal.Inc()
			return nil, errors.Wrap(err, fmt.Sprintf("could not get public keys from remote server url: %v", km.publicKeysURL))
		}
		// makes sure that if the public keys are deleted the validator does not call URL again.
		km.publicKeysUrlCalled = true
		km.providedPublicKeys = providedPublicKeys
	}
	return km.providedPublicKeys, nil
}

// Sign signs the message by using a remote web3signer server.
func (km *Keymanager) Sign(ctx context.Context, request *validatorpb.SignRequest) (bls.Signature, error) {
	signRequest, err := getSignRequestJson(ctx, km.validator, request, km.genesisValidatorsRoot)
	if err != nil {
		erroredResponsesTotal.Inc()
		return nil, err
	}

	signRequestsTotal.Inc()

	return km.client.Sign(ctx, hexutil.Encode(request.PublicKey), signRequest)
}

// getSignRequestJson returns a json request based on the SignRequest type.
func getSignRequestJson(ctx context.Context, validator *validator.Validate, request *validatorpb.SignRequest, genesisValidatorsRoot []byte) (internal.SignRequestJson, error) {
	if request == nil {
		return nil, errors.New("nil sign request provided")
	}
	if !bytesutil.IsValidRoot(genesisValidatorsRoot) {
		return nil, fmt.Errorf("invalid genesis validators root length, genesis root: %v", genesisValidatorsRoot)
	}
	switch request.Object.(type) {
	case *validatorpb.SignRequest_Block:
		return handleBlock(ctx, validator, request, genesisValidatorsRoot)
	case *validatorpb.SignRequest_AttestationData:
		return handleAttestationData(ctx, validator, request, genesisValidatorsRoot)
	case *validatorpb.SignRequest_AggregateAttestationAndProof:
		return handleAggregateAttestationAndProof(ctx, validator, request, genesisValidatorsRoot)
	case *validatorpb.SignRequest_Slot:
		return handleAggregationSlot(ctx, validator, request, genesisValidatorsRoot)
	case *validatorpb.SignRequest_BlockAltair:
		return handleBlockAltair(ctx, validator, request, genesisValidatorsRoot)
	case *validatorpb.SignRequest_BlockBellatrix:
		return handleBlockBellatrix(ctx, validator, request, genesisValidatorsRoot)
	case *validatorpb.SignRequest_BlindedBlockBellatrix:
		return handleBlindedBlockBellatrix(ctx, validator, request, genesisValidatorsRoot)
	case *validatorpb.SignRequest_BlockCapella:
		return handleBlockCapella(ctx, validator, request, genesisValidatorsRoot)
	case *validatorpb.SignRequest_BlindedBlockCapella:
		return handleBlindedBlockCapella(ctx, validator, request, genesisValidatorsRoot)
	case *validatorpb.SignRequest_BlockDeneb:
		return handleBlockDeneb(ctx, validator, request, genesisValidatorsRoot)
	case *validatorpb.SignRequest_BlindedBlockDeneb:
		return handleBlindedBlockDeneb(ctx, validator, request, genesisValidatorsRoot)
	// We do not support "DEPOSIT" type.
	/*
		case *validatorpb.:
		return "DEPOSIT", nil
	*/

	case *validatorpb.SignRequest_Epoch:
		// tech debt that prysm uses signing type epoch
		return handleRandaoReveal(ctx, validator, request, genesisValidatorsRoot)
	case *validatorpb.SignRequest_Exit:
		return handleVoluntaryExit(ctx, validator, request, genesisValidatorsRoot)
	case *validatorpb.SignRequest_SyncMessageBlockRoot:
		return handleSyncMessageBlockRoot(ctx, validator, request, genesisValidatorsRoot)
	case *validatorpb.SignRequest_SyncAggregatorSelectionData:
		return handleSyncAggregatorSelectionData(ctx, validator, request, genesisValidatorsRoot)
	case *validatorpb.SignRequest_ContributionAndProof:
		return handleContributionAndProof(ctx, validator, request, genesisValidatorsRoot)
	case *validatorpb.SignRequest_Registration:
		return handleRegistration(ctx, validator, request)
	default:
		return nil, fmt.Errorf("web3signer sign request type %T not supported", request.Object)
	}
}

func handleBlock(ctx context.Context, validator *validator.Validate, request *validatorpb.SignRequest, genesisValidatorsRoot []byte) ([]byte, error) {
	bockSignRequest, err := web3signerv1.GetBlockSignRequest(request, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	if err = validator.StructCtx(ctx, bockSignRequest); err != nil {
		return nil, err
	}
	blockSignRequestsTotal.Inc()
	return json.Marshal(bockSignRequest)
}

func handleAttestationData(ctx context.Context, validator *validator.Validate, request *validatorpb.SignRequest, genesisValidatorsRoot []byte) ([]byte, error) {
	attestationSignRequest, err := web3signerv1.GetAttestationSignRequest(request, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	if err = validator.StructCtx(ctx, attestationSignRequest); err != nil {
		return nil, err
	}
	attestationSignRequestsTotal.Inc()
	return json.Marshal(attestationSignRequest)
}

func handleAggregateAttestationAndProof(ctx context.Context, validator *validator.Validate, request *validatorpb.SignRequest, genesisValidatorsRoot []byte) ([]byte, error) {
	aggregateAndProofSignRequest, err := web3signerv1.GetAggregateAndProofSignRequest(request, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	if err = validator.StructCtx(ctx, aggregateAndProofSignRequest); err != nil {
		return nil, err
	}
	aggregateAndProofSignRequestsTotal.Inc()
	return json.Marshal(aggregateAndProofSignRequest)
}

func handleAggregationSlot(ctx context.Context, validator *validator.Validate, request *validatorpb.SignRequest, genesisValidatorsRoot []byte) ([]byte, error) {
	aggregationSlotSignRequest, err := web3signerv1.GetAggregationSlotSignRequest(request, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	if err = validator.StructCtx(ctx, aggregationSlotSignRequest); err != nil {
		return nil, err
	}
	aggregationSlotSignRequestsTotal.Inc()
	return json.Marshal(aggregationSlotSignRequest)
}

func handleBlockAltair(ctx context.Context, validator *validator.Validate, request *validatorpb.SignRequest, genesisValidatorsRoot []byte) ([]byte, error) {
	blockv2AltairSignRequest, err := web3signerv1.GetBlockAltairSignRequest(request, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	if err = validator.StructCtx(ctx, blockv2AltairSignRequest); err != nil {
		return nil, err
	}
	blockAltairSignRequestsTotal.Inc()
	return json.Marshal(blockv2AltairSignRequest)
}

func handleBlockBellatrix(ctx context.Context, validator *validator.Validate, request *validatorpb.SignRequest, genesisValidatorsRoot []byte) ([]byte, error) {
	blockv2BellatrixSignRequest, err := web3signerv1.GetBlockV2BlindedSignRequest(request, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	if err = validator.StructCtx(ctx, blockv2BellatrixSignRequest); err != nil {
		return nil, err
	}
	blockBellatrixSignRequestsTotal.Inc()
	return json.Marshal(blockv2BellatrixSignRequest)
}

func handleBlindedBlockBellatrix(ctx context.Context, validator *validator.Validate, request *validatorpb.SignRequest, genesisValidatorsRoot []byte) ([]byte, error) {
	blindedBlockv2SignRequest, err := web3signerv1.GetBlockV2BlindedSignRequest(request, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	if err = validator.StructCtx(ctx, blindedBlockv2SignRequest); err != nil {
		return nil, err
	}
	blindedBlockBellatrixSignRequestsTotal.Inc()
	return json.Marshal(blindedBlockv2SignRequest)
}

func handleBlockCapella(ctx context.Context, validator *validator.Validate, request *validatorpb.SignRequest, genesisValidatorsRoot []byte) ([]byte, error) {
	blockv2CapellaSignRequest, err := web3signerv1.GetBlockV2BlindedSignRequest(request, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	if err = validator.StructCtx(ctx, blockv2CapellaSignRequest); err != nil {
		return nil, err
	}
	blockCapellaSignRequestsTotal.Inc()
	return json.Marshal(blockv2CapellaSignRequest)
}

func handleBlindedBlockCapella(ctx context.Context, validator *validator.Validate, request *validatorpb.SignRequest, genesisValidatorsRoot []byte) ([]byte, error) {
	blindedBlockv2CapellaSignRequest, err := web3signerv1.GetBlockV2BlindedSignRequest(request, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	if err = validator.StructCtx(ctx, blindedBlockv2CapellaSignRequest); err != nil {
		return nil, err
	}
	blindedBlockCapellaSignRequestsTotal.Inc()
	return json.Marshal(blindedBlockv2CapellaSignRequest)
}

func handleBlockDeneb(ctx context.Context, validator *validator.Validate, request *validatorpb.SignRequest, genesisValidatorsRoot []byte) ([]byte, error) {
	blockv2DenebSignRequest, err := web3signerv1.GetBlockV2BlindedSignRequest(request, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	if err = validator.StructCtx(ctx, blockv2DenebSignRequest); err != nil {
		return nil, err
	}
	blockDenebSignRequestsTotal.Inc()
	return json.Marshal(blockv2DenebSignRequest)
}

func handleBlindedBlockDeneb(ctx context.Context, validator *validator.Validate, request *validatorpb.SignRequest, genesisValidatorsRoot []byte) ([]byte, error) {
	blindedBlockv2DenebSignRequest, err := web3signerv1.GetBlockV2BlindedSignRequest(request, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	if err = validator.StructCtx(ctx, blindedBlockv2DenebSignRequest); err != nil {
		return nil, err
	}
	blindedBlockDenebSignRequestsTotal.Inc()
	return json.Marshal(blindedBlockv2DenebSignRequest)
}

func handleRandaoReveal(ctx context.Context, validator *validator.Validate, request *validatorpb.SignRequest, genesisValidatorsRoot []byte) ([]byte, error) {
	randaoRevealSignRequest, err := web3signerv1.GetRandaoRevealSignRequest(request, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	if err = validator.StructCtx(ctx, randaoRevealSignRequest); err != nil {
		return nil, err
	}
	randaoRevealSignRequestsTotal.Inc()
	return json.Marshal(randaoRevealSignRequest)
}

func handleVoluntaryExit(ctx context.Context, validator *validator.Validate, request *validatorpb.SignRequest, genesisValidatorsRoot []byte) ([]byte, error) {
	voluntaryExitRequest, err := web3signerv1.GetVoluntaryExitSignRequest(request, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	if err = validator.StructCtx(ctx, voluntaryExitRequest); err != nil {
		return nil, err
	}
	voluntaryExitSignRequestsTotal.Inc()
	return json.Marshal(voluntaryExitRequest)
}

func handleSyncMessageBlockRoot(ctx context.Context, validator *validator.Validate, request *validatorpb.SignRequest, genesisValidatorsRoot []byte) ([]byte, error) {
	syncCommitteeMessageRequest, err := web3signerv1.GetSyncCommitteeMessageSignRequest(request, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	if err = validator.StructCtx(ctx, syncCommitteeMessageRequest); err != nil {
		return nil, err
	}
	syncCommitteeMessageSignRequestsTotal.Inc()
	return json.Marshal(syncCommitteeMessageRequest)
}

func handleSyncAggregatorSelectionData(ctx context.Context, validator *validator.Validate, request *validatorpb.SignRequest, genesisValidatorsRoot []byte) ([]byte, error) {
	syncCommitteeSelectionProofRequest, err := web3signerv1.GetSyncCommitteeSelectionProofSignRequest(request, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	if err = validator.StructCtx(ctx, syncCommitteeSelectionProofRequest); err != nil {
		return nil, err
	}
	syncCommitteeSelectionProofSignRequestsTotal.Inc()
	return json.Marshal(syncCommitteeSelectionProofRequest)
}

func handleContributionAndProof(ctx context.Context, validator *validator.Validate, request *validatorpb.SignRequest, genesisValidatorsRoot []byte) ([]byte, error) {
	contributionAndProofRequest, err := web3signerv1.GetSyncCommitteeContributionAndProofSignRequest(request, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	if err = validator.StructCtx(ctx, contributionAndProofRequest); err != nil {
		return nil, err
	}
	syncCommitteeContributionAndProofSignRequestsTotal.Inc()
	return json.Marshal(contributionAndProofRequest)
}

func handleRegistration(ctx context.Context, validator *validator.Validate, request *validatorpb.SignRequest) ([]byte, error) {
	validatorRegistrationRequest, err := web3signerv1.GetValidatorRegistrationSignRequest(request)
	if err != nil {
		return nil, err
	}
	if err = validator.StructCtx(ctx, validatorRegistrationRequest); err != nil {
		return nil, err
	}
	validatorRegistrationSignRequestsTotal.Inc()
	return json.Marshal(validatorRegistrationRequest)
}

// SubscribeAccountChanges returns the event subscription for changes to public keys.
func (km *Keymanager) SubscribeAccountChanges(pubKeysChan chan [][fieldparams.BLSPubkeyLength]byte) event.Subscription {
	return km.accountsChangedFeed.Subscribe(pubKeysChan)
}

// ExtractKeystores is not supported for the remote-web3signer keymanager type.
func (*Keymanager) ExtractKeystores(
	_ context.Context, _ []bls.PublicKey, _ string,
) ([]*keymanager.Keystore, error) {
	return nil, errors.New("extracting keys is not supported for a web3signer keymanager")
}

// DeleteKeystores is not supported for the remote-web3signer keymanager type.
func (km *Keymanager) DeleteKeystores(context.Context, [][]byte) ([]*keymanager.KeyStatus, error) {
	return nil, errors.New("Wrong wallet type: web3-signer. Only Imported or Derived wallets can delete accounts")
}

func (km *Keymanager) ListKeymanagerAccounts(ctx context.Context, cfg keymanager.ListKeymanagerAccountConfig) error {
	au := aurora.NewAurora(true)
	fmt.Printf("(keymanager kind) %s\n", au.BrightGreen("web3signer").Bold())
	fmt.Printf(
		"(configuration file path) %s\n",
		au.BrightGreen(filepath.Join(cfg.WalletAccountsDir, cfg.KeymanagerConfigFileName)).Bold(),
	)
	fmt.Println(" ")
	fmt.Printf("%s\n", au.BrightGreen("Setup Configuration").Bold())
	fmt.Println(" ")
	//TODO: add config options, may require refactor again
	validatingPubKeys, err := km.FetchValidatingPublicKeys(ctx)
	if err != nil {
		return errors.Wrap(err, "could not fetch validating public keys")
	}
	if len(validatingPubKeys) == 1 {
		fmt.Print("Showing 1 validator account\n")
	} else if len(validatingPubKeys) == 0 {
		fmt.Print("No accounts found\n")
		return nil
	} else {
		fmt.Printf("Showing %d validator accounts\n", len(validatingPubKeys))
	}
	DisplayRemotePublicKeys(validatingPubKeys)
	return nil
}

// DisplayRemotePublicKeys prints remote public keys to stdout.
func DisplayRemotePublicKeys(validatingPubKeys [][48]byte) {
	au := aurora.NewAurora(true)
	for i := 0; i < len(validatingPubKeys); i++ {
		fmt.Println("")
		fmt.Printf(
			"%s\n", au.BrightGreen(petnames.DeterministicName(validatingPubKeys[i][:], "-")).Bold(),
		)
		// Retrieve the validating key account metadata.
		fmt.Printf("%s %#x\n", au.BrightCyan("[validating public key]").Bold(), validatingPubKeys[i])
		fmt.Println(" ")
	}
}

// AddPublicKeys imports a list of public keys into the keymanager for web3signer use. Returns status with message.
func (km *Keymanager) AddPublicKeys(pubKeys []string) []*keymanager.KeyStatus {
	importedRemoteKeysStatuses := make([]*keymanager.KeyStatus, len(pubKeys))
	for i, pubkey := range pubKeys {
		found := false
		pubkeyBytes, err := hexutil.Decode(pubkey)
		if err != nil {
			importedRemoteKeysStatuses[i] = &keymanager.KeyStatus{
				Status:  keymanager.StatusError,
				Message: err.Error(),
			}
			continue
		}
		if len(pubkeyBytes) != fieldparams.BLSPubkeyLength {
			importedRemoteKeysStatuses[i] = &keymanager.KeyStatus{
				Status:  keymanager.StatusError,
				Message: fmt.Sprintf("pubkey byte length (%d) did not match bls pubkey byte length (%d)", len(pubkeyBytes), fieldparams.BLSPubkeyLength),
			}
			continue
		}
		for _, key := range km.providedPublicKeys {
			if bytes.Equal(key[:], pubkeyBytes) {
				found = true
				break
			}
		}
		if found {
			importedRemoteKeysStatuses[i] = &keymanager.KeyStatus{
				Status:  keymanager.StatusDuplicate,
				Message: fmt.Sprintf("Duplicate pubkey: %v, already in use", pubkey),
			}
			continue
		}
		km.providedPublicKeys = append(km.providedPublicKeys, bytesutil.ToBytes48(pubkeyBytes))
		importedRemoteKeysStatuses[i] = &keymanager.KeyStatus{
			Status:  keymanager.StatusImported,
			Message: fmt.Sprintf("Successfully added pubkey: %v", pubkey),
		}
		log.Debug("Added pubkey to keymanager for web3signer", "pubkey", pubkey)
	}
	km.accountsChangedFeed.Send(km.providedPublicKeys)
	return importedRemoteKeysStatuses
}

// DeletePublicKeys removes a list of public keys from the keymanager for web3signer use. Returns status with message.
func (km *Keymanager) DeletePublicKeys(pubKeys []string) []*keymanager.KeyStatus {
	deletedRemoteKeysStatuses := make([]*keymanager.KeyStatus, len(pubKeys))
	if len(km.providedPublicKeys) == 0 {
		for i := range deletedRemoteKeysStatuses {
			deletedRemoteKeysStatuses[i] = &keymanager.KeyStatus{
				Status:  keymanager.StatusNotFound,
				Message: "No pubkeys are set in validator",
			}
		}
		return deletedRemoteKeysStatuses
	}
	for i, pubkey := range pubKeys {
		for in, key := range km.providedPublicKeys {
			pubkeyBytes, err := hexutil.Decode(pubkey)
			if err != nil {
				deletedRemoteKeysStatuses[i] = &keymanager.KeyStatus{
					Status:  keymanager.StatusError,
					Message: err.Error(),
				}
				continue
			}
			if len(pubkeyBytes) != fieldparams.BLSPubkeyLength {
				deletedRemoteKeysStatuses[i] = &keymanager.KeyStatus{
					Status:  keymanager.StatusError,
					Message: fmt.Sprintf("pubkey byte length (%d) did not match bls pubkey byte length (%d)", len(pubkeyBytes), fieldparams.BLSPubkeyLength),
				}
				continue
			}
			if bytes.Equal(key[:], pubkeyBytes) {
				km.providedPublicKeys = append(km.providedPublicKeys[:in], km.providedPublicKeys[in+1:]...)
				deletedRemoteKeysStatuses[i] = &keymanager.KeyStatus{
					Status:  keymanager.StatusDeleted,
					Message: fmt.Sprintf("Successfully deleted pubkey: %v", pubkey),
				}
				log.Debug("Deleted pubkey from keymanager for web3signer", "pubkey", pubkey)
				break
			}
		}
		if deletedRemoteKeysStatuses[i] == nil {
			deletedRemoteKeysStatuses[i] = &keymanager.KeyStatus{
				Status:  keymanager.StatusNotFound,
				Message: fmt.Sprintf("Pubkey: %v not found", pubkey),
			}
		}
	}
	km.accountsChangedFeed.Send(km.providedPublicKeys)
	return deletedRemoteKeysStatuses
}
