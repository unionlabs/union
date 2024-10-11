package rpc

import (
	"fmt"
	"strconv"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/prysmaticlabs/prysm/v4/api/server"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
)

// local keymanager api
type ListKeystoresResponse struct {
	Data []*Keystore `json:"data"`
}

type Keystore struct {
	ValidatingPubkey string `json:"validating_pubkey"`
	DerivationPath   string `json:"derivation_path"`
}

type ImportKeystoresRequest struct {
	Keystores          []string `json:"keystores"`
	Passwords          []string `json:"passwords"`
	SlashingProtection string   `json:"slashing_protection"`
}

type ImportKeystoresResponse struct {
	Data []*keymanager.KeyStatus `json:"data"`
}

type DeleteKeystoresRequest struct {
	Pubkeys []string `json:"pubkeys"`
}

type DeleteKeystoresResponse struct {
	Data               []*keymanager.KeyStatus `json:"data"`
	SlashingProtection string                  `json:"slashing_protection"`
}

// voluntary exit keymanager api
type SetVoluntaryExitResponse struct {
	Data *shared.SignedVoluntaryExit `json:"data"`
}

// gas limit keymanager api
type GasLimitMetaData struct {
	Pubkey   string `json:"pubkey"`
	GasLimit string `json:"gas_limit"`
}

type GetGasLimitResponse struct {
	Data *GasLimitMetaData `json:"data"`
}

type SetGasLimitRequest struct {
	GasLimit string `json:"gas_limit"`
}

// remote keymanager api
type ListRemoteKeysResponse struct {
	Data []*RemoteKey `json:"data"`
}

type RemoteKey struct {
	Pubkey   string `json:"pubkey"`
	Url      string `json:"url"`
	Readonly bool   `json:"readonly"`
}

type ImportRemoteKeysRequest struct {
	RemoteKeys []*RemoteKey `json:"remote_keys"`
}

type DeleteRemoteKeysRequest struct {
	Pubkeys []string `json:"pubkeys"`
}

type RemoteKeysResponse struct {
	Data []*keymanager.KeyStatus `json:"data"`
}

// Fee Recipient keymanager api
type FeeRecipient struct {
	Pubkey     string `json:"pubkey"`
	Ethaddress string `json:"ethaddress"`
}

type GetFeeRecipientByPubkeyResponse struct {
	Data *FeeRecipient `json:"data"`
}

type SetFeeRecipientByPubkeyRequest struct {
	Ethaddress string `json:"ethaddress"`
}

type BeaconStatusResponse struct {
	BeaconNodeEndpoint     string     `json:"beacon_node_endpoint"`
	Connected              bool       `json:"connected"`
	Syncing                bool       `json:"syncing"`
	GenesisTime            string     `json:"genesis_time"`
	DepositContractAddress string     `json:"deposit_contract_address"`
	ChainHead              *ChainHead `json:"chain_head"`
}

// KeymanagerKind is a type of key manager for the wallet
type KeymanagerKind string

const (
	derivedKeymanagerKind    KeymanagerKind = "DERIVED"
	importedKeymanagerKind   KeymanagerKind = "IMPORTED"
	web3signerKeymanagerKind KeymanagerKind = "WEB3SIGNER"
)

type CreateWalletRequest struct {
	Keymanager       KeymanagerKind `json:"keymanager"`
	WalletPassword   string         `json:"wallet_password"`
	Mnemonic         string         `json:"mnemonic"`
	NumAccounts      uint64         `json:"num_accounts"`
	MnemonicLanguage string         `json:"mnemonic_language"`
}

type CreateWalletResponse struct {
	Wallet *WalletResponse `json:"wallet"`
}

type GenerateMnemonicResponse struct {
	Mnemonic string `json:"mnemonic"`
}

type WalletResponse struct {
	WalletPath     string         `json:"wallet_path"`
	KeymanagerKind KeymanagerKind `json:"keymanager_kind"`
}

type ValidateKeystoresRequest struct {
	Keystores         []string `json:"keystores"`
	KeystoresPassword string   `json:"keystores_password"`
}

type RecoverWalletRequest struct {
	Mnemonic         string `json:"mnemonic"`
	NumAccounts      uint64 `json:"num_accounts"`
	WalletPassword   string `json:"wallet_password"`
	Language         string `json:"language"`
	Mnemonic25ThWord string `json:"mnemonic25th_word"`
}

type ImportSlashingProtectionRequest struct {
	SlashingProtectionJson string `json:"slashing_protection_json"`
}

type ExportSlashingProtectionResponse struct {
	File string `json:"file"`
}

type BackupAccountsRequest struct {
	PublicKeys     []string `json:"public_keys"`
	BackupPassword string   `json:"backup_password"`
}

type VoluntaryExitRequest struct {
	PublicKeys []string `json:"public_keys"`
}

type BackupAccountsResponse struct {
	ZipFile string `json:"zip_file"`
}

type ListAccountsResponse struct {
	Accounts      []*Account `json:"accounts"`
	NextPageToken string     `json:"next_page_token"`
	TotalSize     int32      `json:"total_size"`
}

type Account struct {
	ValidatingPublicKey string `json:"validating_public_key"`
	AccountName         string `json:"account_name"`
	DepositTxData       string `json:"deposit_tx_data"`
	DerivationPath      string `json:"derivation_path"`
}

type VoluntaryExitResponse struct {
	ExitedKeys [][]byte `protobuf:"bytes,1,rep,name=exited_keys,json=exitedKeys,proto3" json:"exited_keys,omitempty"`
}

type InitializeAuthResponse struct {
	HasSignedUp bool `json:"has_signed_up"`
	HasWallet   bool `json:"has_wallet"`
}

type ValidatorPerformanceResponse struct {
	CurrentEffectiveBalances      []uint64 `json:"current_effective_balances"`
	InclusionSlots                []uint64 `json:"inclusion_slots"`
	InclusionDistances            []uint64 `json:"inclusion_distances"`
	CorrectlyVotedSource          []bool   `json:"correctly_voted_source"`
	CorrectlyVotedTarget          []bool   `json:"correctly_voted_target"`
	CorrectlyVotedHead            []bool   `json:"correctly_voted_head"`
	BalancesBeforeEpochTransition []uint64 `json:"balances_before_epoch_transition"`
	BalancesAfterEpochTransition  []uint64 `json:"balances_after_epoch_transition"`
	MissingValidators             []string `json:"missing_validators"`
	AverageActiveValidatorBalance float32  `json:"average_active_validator_balance"`
	PublicKeys                    []string `json:"public_keys"`
	InactivityScores              []uint64 `json:"inactivity_scores"`
}

func ValidatorPerformanceResponseFromConsensus(e *eth.ValidatorPerformanceResponse) *ValidatorPerformanceResponse {
	inclusionSlots := make([]uint64, len(e.InclusionSlots))
	for i, index := range e.InclusionSlots {
		inclusionSlots[i] = uint64(index)
	}
	inclusionDistances := make([]uint64, len(e.InclusionDistances))
	for i, index := range e.InclusionDistances {
		inclusionDistances[i] = uint64(index)
	}
	missingValidators := make([]string, len(e.MissingValidators))
	for i, key := range e.MissingValidators {
		missingValidators[i] = hexutil.Encode(key)
	}
	publicKeys := make([]string, len(e.PublicKeys))
	for i, key := range e.PublicKeys {
		publicKeys[i] = hexutil.Encode(key)
	}
	if len(e.CurrentEffectiveBalances) == 0 {
		e.CurrentEffectiveBalances = make([]uint64, 0)
	}
	if len(e.BalancesBeforeEpochTransition) == 0 {
		e.BalancesBeforeEpochTransition = make([]uint64, 0)
	}
	if len(e.BalancesAfterEpochTransition) == 0 {
		e.BalancesAfterEpochTransition = make([]uint64, 0)
	}
	if len(e.CorrectlyVotedSource) == 0 {
		e.CorrectlyVotedSource = make([]bool, 0)
	}
	if len(e.CorrectlyVotedTarget) == 0 {
		e.CorrectlyVotedTarget = make([]bool, 0)
	}
	if len(e.CorrectlyVotedHead) == 0 {
		e.CorrectlyVotedHead = make([]bool, 0)
	}
	if len(e.InactivityScores) == 0 {
		e.InactivityScores = make([]uint64, 0)
	}
	return &ValidatorPerformanceResponse{
		CurrentEffectiveBalances:      e.CurrentEffectiveBalances,
		InclusionSlots:                inclusionSlots,
		InclusionDistances:            inclusionDistances,
		CorrectlyVotedSource:          e.CorrectlyVotedSource,
		CorrectlyVotedTarget:          e.CorrectlyVotedTarget,
		CorrectlyVotedHead:            e.CorrectlyVotedHead,
		BalancesBeforeEpochTransition: e.BalancesBeforeEpochTransition,
		BalancesAfterEpochTransition:  e.BalancesAfterEpochTransition,
		MissingValidators:             missingValidators,
		AverageActiveValidatorBalance: e.AverageActiveValidatorBalance,
		PublicKeys:                    publicKeys,
		InactivityScores:              e.InactivityScores,
	}
}

type ValidatorBalancesResponse struct {
	Epoch         uint64              `json:"epoch"`
	Balances      []*ValidatorBalance `json:"balances"`
	NextPageToken string              `json:"next_page_token"`
	TotalSize     int32               `json:"total_size,omitempty"`
}

type ValidatorBalance struct {
	PublicKey string `json:"public_key"`
	Index     uint64 `json:"index"`
	Balance   uint64 `json:"balance"`
	Status    string `json:"status"`
}

func ValidatorBalancesResponseFromConsensus(e *eth.ValidatorBalances) (*ValidatorBalancesResponse, error) {
	balances := make([]*ValidatorBalance, len(e.Balances))
	for i, balance := range e.Balances {
		balances[i] = &ValidatorBalance{
			PublicKey: hexutil.Encode(balance.PublicKey),
			Index:     uint64(balance.Index),
			Balance:   balance.Balance,
			Status:    balance.Status,
		}
	}
	return &ValidatorBalancesResponse{
		Epoch:         uint64(e.Epoch),
		Balances:      balances,
		NextPageToken: e.NextPageToken,
		TotalSize:     e.TotalSize,
	}, nil
}

type ValidatorsResponse struct {
	Epoch         uint64                `json:"epoch"`
	ValidatorList []*ValidatorContainer `json:"validator_list"`
	NextPageToken string                `json:"next_page_token"`
	TotalSize     int32                 `json:"total_size"`
}

type ValidatorContainer struct {
	Index     uint64     `json:"index"`
	Validator *Validator `json:"validator"`
}

type Validator struct {
	PublicKey                  string `json:"public_key,omitempty"`
	WithdrawalCredentials      string `json:"withdrawal_credentials"`
	EffectiveBalance           uint64 `json:"effective_balance"`
	Slashed                    bool   `json:"slashed"`
	ActivationEligibilityEpoch uint64 `json:"activation_eligibility_epoch"`
	ActivationEpoch            uint64 `json:"activation_epoch"`
	ExitEpoch                  uint64 `json:"exit_epoch"`
	WithdrawableEpoch          uint64 `json:"withdrawable_epoch"`
}

func ValidatorsResponseFromConsensus(e *eth.Validators) (*ValidatorsResponse, error) {
	validatorList := make([]*ValidatorContainer, len(e.ValidatorList))
	for i, validatorContainer := range e.ValidatorList {
		val := validatorContainer.Validator
		validatorList[i] = &ValidatorContainer{
			Index: uint64(validatorContainer.Index),
			Validator: &Validator{
				PublicKey:                  hexutil.Encode(val.PublicKey),
				WithdrawalCredentials:      hexutil.Encode(val.WithdrawalCredentials),
				EffectiveBalance:           val.EffectiveBalance,
				Slashed:                    val.Slashed,
				ActivationEligibilityEpoch: uint64(val.ActivationEligibilityEpoch),
				ActivationEpoch:            uint64(val.ActivationEpoch),
				ExitEpoch:                  uint64(val.ExitEpoch),
				WithdrawableEpoch:          uint64(val.WithdrawableEpoch),
			},
		}
	}
	return &ValidatorsResponse{
		Epoch:         uint64(e.Epoch),
		ValidatorList: validatorList,
		NextPageToken: e.NextPageToken,
		TotalSize:     e.TotalSize,
	}, nil
}

// ChainHead is the response for api endpoint /beacon/chainhead
type ChainHead struct {
	HeadSlot                   string `json:"head_slot"`
	HeadEpoch                  string `json:"head_epoch"`
	HeadBlockRoot              string `json:"head_block_root"`
	FinalizedSlot              string `json:"finalized_slot"`
	FinalizedEpoch             string `json:"finalized_epoch"`
	FinalizedBlockRoot         string `json:"finalized_block_root"`
	JustifiedSlot              string `json:"justified_slot"`
	JustifiedEpoch             string `json:"justified_epoch"`
	JustifiedBlockRoot         string `json:"justified_block_root"`
	PreviousJustifiedSlot      string `json:"previous_justified_slot"`
	PreviousJustifiedEpoch     string `json:"previous_justified_epoch"`
	PreviousJustifiedBlockRoot string `json:"previous_justified_block_root"`
	OptimisticStatus           bool   `json:"optimistic_status"`
}

func ChainHeadResponseFromConsensus(e *eth.ChainHead) *ChainHead {
	return &ChainHead{
		HeadSlot:                   fmt.Sprintf("%d", e.HeadSlot),
		HeadEpoch:                  fmt.Sprintf("%d", e.HeadEpoch),
		HeadBlockRoot:              hexutil.Encode(e.HeadBlockRoot),
		FinalizedSlot:              fmt.Sprintf("%d", e.FinalizedSlot),
		FinalizedEpoch:             fmt.Sprintf("%d", e.FinalizedEpoch),
		FinalizedBlockRoot:         hexutil.Encode(e.FinalizedBlockRoot),
		JustifiedSlot:              fmt.Sprintf("%d", e.JustifiedSlot),
		JustifiedEpoch:             fmt.Sprintf("%d", e.JustifiedEpoch),
		JustifiedBlockRoot:         hexutil.Encode(e.JustifiedBlockRoot),
		PreviousJustifiedSlot:      fmt.Sprintf("%d", e.PreviousJustifiedSlot),
		PreviousJustifiedEpoch:     fmt.Sprintf("%d", e.PreviousJustifiedEpoch),
		PreviousJustifiedBlockRoot: hexutil.Encode(e.PreviousJustifiedBlockRoot),
		OptimisticStatus:           e.OptimisticStatus,
	}
}

func (m *ChainHead) ToConsensus() (*eth.ChainHead, error) {
	headSlot, err := strconv.ParseUint(m.HeadSlot, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "HeadSlot")
	}
	headEpoch, err := strconv.ParseUint(m.HeadEpoch, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "HeadEpoch")
	}
	headBlockRoot, err := bytesutil.DecodeHexWithLength(m.HeadBlockRoot, fieldparams.RootLength)
	if err != nil {
		return nil, server.NewDecodeError(err, "HeadBlockRoot")
	}
	finalizedSlot, err := strconv.ParseUint(m.FinalizedSlot, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "FinalizedSlot")
	}
	finalizedEpoch, err := strconv.ParseUint(m.FinalizedEpoch, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "FinalizedEpoch")
	}
	finalizedBlockRoot, err := bytesutil.DecodeHexWithLength(m.FinalizedBlockRoot, fieldparams.RootLength)
	if err != nil {
		return nil, server.NewDecodeError(err, "FinalizedBlockRoot")
	}
	justifiedSlot, err := strconv.ParseUint(m.JustifiedSlot, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "JustifiedSlot")
	}
	justifiedEpoch, err := strconv.ParseUint(m.JustifiedEpoch, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "JustifiedEpoch")
	}
	justifiedBlockRoot, err := bytesutil.DecodeHexWithLength(m.JustifiedBlockRoot, fieldparams.RootLength)
	if err != nil {
		return nil, server.NewDecodeError(err, "JustifiedBlockRoot")
	}
	previousjustifiedSlot, err := strconv.ParseUint(m.PreviousJustifiedSlot, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "PreviousJustifiedSlot")
	}
	previousjustifiedEpoch, err := strconv.ParseUint(m.PreviousJustifiedEpoch, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "PreviousJustifiedEpoch")
	}
	previousjustifiedBlockRoot, err := bytesutil.DecodeHexWithLength(m.PreviousJustifiedBlockRoot, fieldparams.RootLength)
	if err != nil {
		return nil, server.NewDecodeError(err, "PreviousJustifiedBlockRoot")
	}
	return &eth.ChainHead{
		HeadSlot:                   primitives.Slot(headSlot),
		HeadEpoch:                  primitives.Epoch(headEpoch),
		HeadBlockRoot:              headBlockRoot,
		FinalizedSlot:              primitives.Slot(finalizedSlot),
		FinalizedEpoch:             primitives.Epoch(finalizedEpoch),
		FinalizedBlockRoot:         finalizedBlockRoot,
		JustifiedSlot:              primitives.Slot(justifiedSlot),
		JustifiedEpoch:             primitives.Epoch(justifiedEpoch),
		JustifiedBlockRoot:         justifiedBlockRoot,
		PreviousJustifiedSlot:      primitives.Slot(previousjustifiedSlot),
		PreviousJustifiedEpoch:     primitives.Epoch(previousjustifiedEpoch),
		PreviousJustifiedBlockRoot: previousjustifiedBlockRoot,
		OptimisticStatus:           m.OptimisticStatus,
	}, nil
}
