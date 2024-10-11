package mock

import (
	"context"
	"errors"
	"strings"
	"sync"
	"time"

	validatorserviceconfig "github.com/prysmaticlabs/prysm/v4/config/validator/service"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/iface"
	iface2 "github.com/prysmaticlabs/prysm/v4/validator/client/iface"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
)

// Wallet contains an in-memory, simulated wallet implementation.
type Wallet struct {
	InnerAccountsDir  string
	Directories       []string
	Files             map[string]map[string][]byte
	EncryptedSeedFile []byte
	AccountPasswords  map[string]string
	WalletPassword    string
	UnlockAccounts    bool
	lock              sync.RWMutex
	HasWriteFileError bool
}

// AccountNames --
func (w *Wallet) AccountNames() ([]string, error) {
	w.lock.RLock()
	defer w.lock.RUnlock()
	names := make([]string, 0)
	for name := range w.AccountPasswords {
		names = append(names, name)
	}
	return names, nil
}

// AccountsDir --
func (w *Wallet) AccountsDir() string {
	return w.InnerAccountsDir
}

// Exists --
func (w *Wallet) Exists() (bool, error) {
	return len(w.Directories) > 0, nil
}

// Password --
func (w *Wallet) Password() string {
	return w.WalletPassword
}

// WriteFileAtPath --
func (w *Wallet) WriteFileAtPath(_ context.Context, pathName, fileName string, data []byte) (bool, error) {
	w.lock.Lock()
	defer w.lock.Unlock()
	if w.HasWriteFileError {
		// reset the flag to not contaminate other tests
		w.HasWriteFileError = false
		return false, errors.New("could not write keystore file for accounts")
	}
	if w.Files[pathName] == nil {
		w.Files[pathName] = make(map[string][]byte)
	}
	w.Files[pathName][fileName] = data
	return true, nil
}

// ReadFileAtPath --
func (w *Wallet) ReadFileAtPath(_ context.Context, pathName, fileName string) ([]byte, error) {
	w.lock.RLock()
	defer w.lock.RUnlock()
	for f, v := range w.Files[pathName] {
		if strings.Contains(fileName, f) {
			return v, nil
		}
	}
	return nil, errors.New("no files found")
}

// InitializeKeymanager --
func (_ *Wallet) InitializeKeymanager(_ context.Context, _ iface.InitKeymanagerConfig) (keymanager.IKeymanager, error) {
	return nil, nil
}

type Validator struct {
	Km               keymanager.IKeymanager
	proposerSettings *validatorserviceconfig.ProposerSettings
}

func (_ *Validator) LogSyncCommitteeMessagesSubmitted() {}

func (_ *Validator) Done() {
	panic("implement me")
}

func (_ *Validator) WaitForChainStart(_ context.Context) error {
	panic("implement me")
}

func (_ *Validator) WaitForSync(_ context.Context) error {
	panic("implement me")
}

func (_ *Validator) WaitForActivation(_ context.Context, _ chan [][48]byte) error {
	panic("implement me")
}

func (_ *Validator) CanonicalHeadSlot(_ context.Context) (primitives.Slot, error) {
	panic("implement me")
}

func (_ *Validator) NextSlot() <-chan primitives.Slot {
	panic("implement me")
}

func (_ *Validator) SlotDeadline(_ primitives.Slot) time.Time {
	panic("implement me")
}

func (_ *Validator) LogValidatorGainsAndLosses(_ context.Context, _ primitives.Slot) error {
	panic("implement me")
}

func (_ *Validator) UpdateDuties(_ context.Context, _ primitives.Slot) error {
	panic("implement me")
}

func (_ *Validator) RolesAt(_ context.Context, _ primitives.Slot) (map[[48]byte][]iface2.ValidatorRole, error) {
	panic("implement me")
}

func (_ *Validator) SubmitAttestation(_ context.Context, _ primitives.Slot, _ [48]byte) {
	panic("implement me")
}

func (_ *Validator) ProposeBlock(_ context.Context, _ primitives.Slot, _ [48]byte) {
	panic("implement me")
}

func (_ *Validator) SubmitAggregateAndProof(_ context.Context, _ primitives.Slot, _ [48]byte) {
	panic("implement me")
}

func (_ *Validator) SubmitSyncCommitteeMessage(_ context.Context, _ primitives.Slot, _ [48]byte) {
	panic("implement me")
}

func (_ *Validator) SubmitSignedContributionAndProof(_ context.Context, _ primitives.Slot, _ [48]byte) {
	panic("implement me")
}

func (_ *Validator) LogAttestationsSubmitted() {
	panic("implement me")
}

func (_ *Validator) UpdateDomainDataCaches(_ context.Context, _ primitives.Slot) {
	panic("implement me")
}

func (_ *Validator) WaitForKeymanagerInitialization(_ context.Context) error {
	panic("implement me")
}

func (m *Validator) Keymanager() (keymanager.IKeymanager, error) {
	return m.Km, nil
}

func (_ *Validator) ReceiveSlots(_ context.Context, _ chan<- error) {
	panic("implement me")
}

func (_ *Validator) HandleKeyReload(_ context.Context, _ [][48]byte) (bool, error) {
	panic("implement me")
}

func (_ *Validator) CheckDoppelGanger(_ context.Context) error {
	panic("implement me")
}

// HasProposerSettings for mocking
func (*Validator) HasProposerSettings() bool {
	panic("implement me")
}

// PushProposerSettings for mocking
func (_ *Validator) PushProposerSettings(_ context.Context, _ keymanager.IKeymanager, _ primitives.Slot, _ time.Time) error {
	panic("implement me")
}

// SetPubKeyToValidatorIndexMap for mocking
func (_ *Validator) SetPubKeyToValidatorIndexMap(_ context.Context, _ keymanager.IKeymanager) error {
	panic("implement me")
}

// SignValidatorRegistrationRequest for mocking
func (_ *Validator) SignValidatorRegistrationRequest(_ context.Context, _ iface2.SigningFunc, _ *ethpb.ValidatorRegistrationV1) (*ethpb.SignedValidatorRegistrationV1, error) {
	panic("implement me")
}

// ProposerSettings for mocking
func (m *Validator) ProposerSettings() *validatorserviceconfig.ProposerSettings {
	return m.proposerSettings
}

// SetProposerSettings for mocking
func (m *Validator) SetProposerSettings(_ context.Context, settings *validatorserviceconfig.ProposerSettings) error {
	m.proposerSettings = settings
	return nil
}

func (_ *Validator) StartEventStream(_ context.Context) error {
	panic("implement me")
}

func (_ *Validator) EventStreamIsRunning() bool {
	panic("implement me")
}

func (_ *Validator) NodeIsHealthy(ctx context.Context) bool {
	panic("implement me")
}
