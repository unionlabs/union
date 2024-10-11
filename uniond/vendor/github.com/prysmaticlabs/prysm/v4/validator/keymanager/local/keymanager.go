package local

import (
	"context"
	"encoding/json"
	"fmt"
	"path/filepath"
	"strings"
	"sync"

	"github.com/google/uuid"
	"github.com/logrusorgru/aurora"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/async/event"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	validatorpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/validator-client"
	"github.com/prysmaticlabs/prysm/v4/runtime/interop"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/iface"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/petnames"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	keystorev4 "github.com/wealdtech/go-eth2-wallet-encryptor-keystorev4"
	"go.opencensus.io/trace"
)

var (
	lock              sync.RWMutex
	orderedPublicKeys = make([][fieldparams.BLSPubkeyLength]byte, 0)
	secretKeysCache   = make(map[[fieldparams.BLSPubkeyLength]byte]bls.SecretKey)
)

const (
	// KeystoreFileNameFormat exposes the filename the keystore should be formatted in.
	KeystoreFileNameFormat = "keystore-%d.json"
	// AccountsPath where all local keymanager keystores are kept.
	AccountsPath = "accounts"
	// AccountsKeystoreFileName exposes the name of the keystore file.
	AccountsKeystoreFileName = "all-accounts.keystore.json"
)

// Keymanager implementation for local keystores utilizing EIP-2335.
type Keymanager struct {
	wallet              iface.Wallet
	accountsStore       *accountStore
	accountsChangedFeed *event.Feed
}

// SetupConfig includes configuration values for initializing
// a keymanager, such as passwords, the wallet, and more.
type SetupConfig struct {
	Wallet           iface.Wallet
	ListenForChanges bool
}

// Defines a struct containing 1-to-1 corresponding
// private keys and public keys for Ethereum validators.
type accountStore struct {
	PrivateKeys [][]byte `json:"private_keys"`
	PublicKeys  [][]byte `json:"public_keys"`
}

// Copy creates a deep copy of accountStore
func (a *accountStore) Copy() *accountStore {
	storeCopy := &accountStore{}
	storeCopy.PrivateKeys = bytesutil.SafeCopy2dBytes(a.PrivateKeys)
	storeCopy.PublicKeys = bytesutil.SafeCopy2dBytes(a.PublicKeys)
	return storeCopy
}

// AccountsKeystoreRepresentation defines an internal Prysm representation
// of validator accounts, encrypted according to the EIP-2334 standard.
type AccountsKeystoreRepresentation struct {
	Crypto  map[string]interface{} `json:"crypto"`
	ID      string                 `json:"uuid"`
	Version uint                   `json:"version"`
	Name    string                 `json:"name"`
}

// ResetCaches for the keymanager.
func ResetCaches() {
	lock.Lock()
	orderedPublicKeys = make([][fieldparams.BLSPubkeyLength]byte, 0)
	secretKeysCache = make(map[[fieldparams.BLSPubkeyLength]byte]bls.SecretKey)
	lock.Unlock()
}

// NewKeymanager instantiates a new local keymanager from configuration options.
func NewKeymanager(ctx context.Context, cfg *SetupConfig) (*Keymanager, error) {
	k := &Keymanager{
		wallet:              cfg.Wallet,
		accountsStore:       &accountStore{},
		accountsChangedFeed: new(event.Feed),
	}

	if err := k.initializeAccountKeystore(ctx); err != nil {
		return nil, errors.Wrap(err, "failed to initialize account store")
	}

	if cfg.ListenForChanges {
		// We begin a goroutine to listen for file changes to our
		// all-accounts.keystore.json file in the wallet directory.
		go k.listenForAccountChanges(ctx)
	}
	return k, nil
}

// InteropKeymanagerConfig is used on validator launch to initialize the keymanager.
// InteropKeys are used for testing purposes.
type InteropKeymanagerConfig struct {
	Offset           uint64
	NumValidatorKeys uint64
}

// NewInteropKeymanager instantiates a new imported keymanager with the deterministically generated interop keys.
// InteropKeys are used for testing purposes.
func NewInteropKeymanager(_ context.Context, offset, numValidatorKeys uint64) (*Keymanager, error) {
	k := &Keymanager{
		accountsChangedFeed: new(event.Feed),
	}
	if numValidatorKeys == 0 {
		return k, nil
	}
	secretKeys, publicKeys, err := interop.DeterministicallyGenerateKeys(offset, numValidatorKeys)
	if err != nil {
		return nil, errors.Wrap(err, "could not generate interop keys")
	}
	lock.Lock()
	pubKeys := make([][fieldparams.BLSPubkeyLength]byte, numValidatorKeys)
	for i := uint64(0); i < numValidatorKeys; i++ {
		publicKey := bytesutil.ToBytes48(publicKeys[i].Marshal())
		pubKeys[i] = publicKey
		secretKeysCache[publicKey] = secretKeys[i]
	}
	orderedPublicKeys = pubKeys
	lock.Unlock()
	return k, nil
}

// SubscribeAccountChanges creates an event subscription for a channel
// to listen for public key changes at runtime, such as when new validator accounts
// are imported into the keymanager while the validator process is running.
func (km *Keymanager) SubscribeAccountChanges(pubKeysChan chan [][fieldparams.BLSPubkeyLength]byte) event.Subscription {
	return km.accountsChangedFeed.Subscribe(pubKeysChan)
}

// ValidatingAccountNames for a local keymanager.
func (_ *Keymanager) ValidatingAccountNames() ([]string, error) {
	lock.RLock()
	names := make([]string, len(orderedPublicKeys))
	for i, pubKey := range orderedPublicKeys {
		names[i] = petnames.DeterministicName(bytesutil.FromBytes48(pubKey), "-")
	}
	lock.RUnlock()
	return names, nil
}

// Initialize public and secret key caches that are used to speed up the functions
// FetchValidatingPublicKeys and Sign
func (km *Keymanager) initializeKeysCachesFromKeystore() error {
	lock.Lock()
	defer lock.Unlock()
	count := len(km.accountsStore.PrivateKeys)
	orderedPublicKeys = make([][fieldparams.BLSPubkeyLength]byte, count)
	secretKeysCache = make(map[[fieldparams.BLSPubkeyLength]byte]bls.SecretKey, count)
	for i, publicKey := range km.accountsStore.PublicKeys {
		publicKey48 := bytesutil.ToBytes48(publicKey)
		orderedPublicKeys[i] = publicKey48
		secretKey, err := bls.SecretKeyFromBytes(km.accountsStore.PrivateKeys[i])
		if err != nil {
			return errors.Wrap(err, "failed to initialize keys caches from account keystore")
		}
		secretKeysCache[publicKey48] = secretKey
	}
	return nil
}

// FetchValidatingPublicKeys fetches the list of active public keys from the local account keystores.
func (_ *Keymanager) FetchValidatingPublicKeys(ctx context.Context) ([][fieldparams.BLSPubkeyLength]byte, error) {
	_, span := trace.StartSpan(ctx, "keymanager.FetchValidatingPublicKeys")
	defer span.End()

	lock.RLock()
	keys := orderedPublicKeys
	result := make([][fieldparams.BLSPubkeyLength]byte, len(keys))
	copy(result, keys)
	lock.RUnlock()
	return result, nil
}

// FetchValidatingPrivateKeys fetches the list of private keys from the secret keys cache
func (km *Keymanager) FetchValidatingPrivateKeys(ctx context.Context) ([][32]byte, error) {
	lock.RLock()
	defer lock.RUnlock()
	privKeys := make([][32]byte, len(secretKeysCache))
	pubKeys, err := km.FetchValidatingPublicKeys(ctx)
	if err != nil {
		return nil, errors.Wrap(err, "could not retrieve public keys")
	}
	for i, pk := range pubKeys {
		seckey, ok := secretKeysCache[pk]
		if !ok {
			return nil, errors.New("Could not fetch private key")
		}
		privKeys[i] = bytesutil.ToBytes32(seckey.Marshal())
	}
	return privKeys, nil
}

// Sign signs a message using a validator key.
func (_ *Keymanager) Sign(ctx context.Context, req *validatorpb.SignRequest) (bls.Signature, error) {
	publicKey := req.PublicKey
	if publicKey == nil {
		return nil, errors.New("nil public key in request")
	}
	lock.RLock()
	secretKey, ok := secretKeysCache[bytesutil.ToBytes48(publicKey)]
	lock.RUnlock()
	if !ok {
		return nil, errors.New("no signing key found in keys cache")
	}
	return secretKey.Sign(req.SigningRoot), nil
}

func (km *Keymanager) initializeAccountKeystore(ctx context.Context) error {
	encoded, err := km.wallet.ReadFileAtPath(ctx, AccountsPath, AccountsKeystoreFileName)
	if err != nil && strings.Contains(err.Error(), "no files found") {
		// If there are no keys to initialize at all, just exit.
		return nil
	} else if err != nil {
		return errors.Wrapf(err, "could not read keystore file for accounts %s", AccountsKeystoreFileName)
	}
	keystoreFile := &AccountsKeystoreRepresentation{}
	if err := json.Unmarshal(encoded, keystoreFile); err != nil {
		return errors.Wrapf(err, "could not decode keystore file for accounts %s", AccountsKeystoreFileName)
	}
	// We extract the validator signing private key from the keystore
	// by utilizing the password and initialize a new BLS secret key from
	// its raw bytes.
	password := km.wallet.Password()
	decryptor := keystorev4.New()
	enc, err := decryptor.Decrypt(keystoreFile.Crypto, password)
	if err != nil && strings.Contains(err.Error(), keymanager.IncorrectPasswordErrMsg) {
		return errors.Wrap(err, "wrong password for wallet entered")
	} else if err != nil {
		return errors.Wrap(err, "could not decrypt keystore")
	}

	store := &accountStore{}
	if err := json.Unmarshal(enc, store); err != nil {
		return err
	}
	if len(store.PublicKeys) != len(store.PrivateKeys) {
		return errors.New("unequal number of public keys and private keys")
	}
	if len(store.PublicKeys) == 0 {
		return nil
	}
	km.accountsStore = store
	err = km.initializeKeysCachesFromKeystore()
	if err != nil {
		return errors.Wrap(err, "failed to initialize keys caches")
	}
	return err
}

// CreateAccountsKeystore creates a new keystore holding the provided keys.
func (km *Keymanager) CreateAccountsKeystore(ctx context.Context, privateKeys [][]byte, publicKeys [][]byte) (*AccountsKeystoreRepresentation, error) {
	if err := km.CreateOrUpdateInMemoryAccountsStore(ctx, privateKeys, publicKeys); err != nil {
		return nil, err
	}
	return CreateAccountsKeystoreRepresentation(ctx, km.accountsStore, km.wallet.Password())
}

// SaveStoreAndReInitialize saves the store to disk and re-initializes the account keystore from file
func (km *Keymanager) SaveStoreAndReInitialize(ctx context.Context, store *accountStore) error {
	// Save the copy to disk
	accountsKeystore, err := CreateAccountsKeystoreRepresentation(ctx, store, km.wallet.Password())
	if err != nil {
		return err
	}
	encodedAccounts, err := json.MarshalIndent(accountsKeystore, "", "\t")
	if err != nil {
		return err
	}

	existedPreviously, err := km.wallet.WriteFileAtPath(ctx, AccountsPath, AccountsKeystoreFileName, encodedAccounts)
	if err != nil {
		return err
	}

	if existedPreviously {
		// Reinitialize account store and cache
		// This will update the in-memory information instead of reading from the file itself for safety concerns
		km.accountsStore = store
		err = km.initializeKeysCachesFromKeystore()
		if err != nil {
			return errors.Wrap(err, "failed to initialize keys caches")
		}

		return nil
	}

	// manually reload the account from the keystore the first time
	km.reloadAccountsFromKeystoreFile(filepath.Join(km.wallet.AccountsDir(), AccountsPath, AccountsKeystoreFileName))
	// listen to account changes of the new file
	go km.listenForAccountChanges(ctx)
	return nil
}

// CreateAccountsKeystoreRepresentation is a pure function that takes an accountStore and wallet password and returns the encrypted formatted json version for local writing.
func CreateAccountsKeystoreRepresentation(
	_ context.Context,
	store *accountStore,
	walletPW string,
) (*AccountsKeystoreRepresentation, error) {
	encryptor := keystorev4.New()
	id, err := uuid.NewRandom()
	if err != nil {
		return nil, err
	}
	encodedStore, err := json.MarshalIndent(store, "", "\t")
	if err != nil {
		return nil, err
	}
	cryptoFields, err := encryptor.Encrypt(encodedStore, walletPW)
	if err != nil {
		return nil, errors.Wrap(err, "could not encrypt accounts")
	}
	return &AccountsKeystoreRepresentation{
		Crypto:  cryptoFields,
		ID:      id.String(),
		Version: encryptor.Version(),
		Name:    encryptor.Name(),
	}, nil
}

// CreateEmptyKeyStoreRepresentationForNewWallet creates a placeholder accounts keystore for a new Prysm Local Wallet.
func CreateEmptyKeyStoreRepresentationForNewWallet(ctx context.Context, walletPassword string) (*AccountsKeystoreRepresentation, error) {
	// make sure everything is clean when creating this.
	ResetCaches()
	return CreateAccountsKeystoreRepresentation(ctx, &accountStore{}, walletPassword)
}

// CreateOrUpdateInMemoryAccountsStore will set or update the local accounts store and update the local cache.
// This function DOES NOT save the accounts store to disk.
func (km *Keymanager) CreateOrUpdateInMemoryAccountsStore(_ context.Context, privateKeys, publicKeys [][]byte) error {
	if len(privateKeys) != len(publicKeys) {
		return fmt.Errorf(
			"number of private keys and public keys is not equal: %d != %d", len(privateKeys), len(publicKeys),
		)
	}
	if km.accountsStore == nil {
		km.accountsStore = &accountStore{
			PrivateKeys: privateKeys,
			PublicKeys:  publicKeys,
		}
	} else {
		updateAccountsStoreKeys(km.accountsStore, privateKeys, publicKeys)
	}
	err := km.initializeKeysCachesFromKeystore()
	if err != nil {
		return errors.Wrap(err, "failed to initialize keys caches")
	}
	return nil
}

func updateAccountsStoreKeys(store *accountStore, privateKeys, publicKeys [][]byte) {
	existingPubKeys := make(map[string]bool)
	existingPrivKeys := make(map[string]bool)
	for i := 0; i < len(store.PrivateKeys); i++ {
		existingPrivKeys[string(store.PrivateKeys[i])] = true
		existingPubKeys[string(store.PublicKeys[i])] = true
	}
	// We append to the accounts store keys only
	// if the private/secret key do not already exist, to prevent duplicates.
	for i := 0; i < len(privateKeys); i++ {
		sk := privateKeys[i]
		pk := publicKeys[i]
		_, privKeyExists := existingPrivKeys[string(sk)]
		_, pubKeyExists := existingPubKeys[string(pk)]
		if privKeyExists || pubKeyExists {
			continue
		}
		store.PublicKeys = append(store.PublicKeys, pk)
		store.PrivateKeys = append(store.PrivateKeys, sk)
	}
}

func (km *Keymanager) ListKeymanagerAccounts(ctx context.Context, cfg keymanager.ListKeymanagerAccountConfig) error {
	au := aurora.NewAurora(true)
	// We initialize the wallet's keymanager.
	accountNames, err := km.ValidatingAccountNames()
	if err != nil {
		return errors.Wrap(err, "could not fetch account names")
	}
	numAccounts := au.BrightYellow(len(accountNames))
	fmt.Printf("(keymanager kind) %s\n", au.BrightGreen("local wallet").Bold())
	fmt.Println("")
	if len(accountNames) == 1 {
		fmt.Printf("Showing %d validator account\n", numAccounts)
	} else {
		fmt.Printf("Showing %d validator accounts\n", numAccounts)
	}
	fmt.Println(
		au.BrightRed("View the eth1 deposit transaction data for your accounts " +
			"by running `validator accounts list --show-deposit-data`"),
	)

	pubKeys, err := km.FetchValidatingPublicKeys(ctx)
	if err != nil {
		return errors.Wrap(err, "could not fetch validating public keys")
	}
	var privateKeys [][32]byte
	if cfg.ShowPrivateKeys {
		privateKeys, err = km.FetchValidatingPrivateKeys(ctx)
		if err != nil {
			return errors.Wrap(err, "could not fetch private keys")
		}
	}
	for i := 0; i < len(accountNames); i++ {
		fmt.Println("")
		fmt.Printf("%s | %s\n", au.BrightBlue(fmt.Sprintf("Account %d", i)).Bold(), au.BrightGreen(accountNames[i]).Bold())
		fmt.Printf("%s %#x\n", au.BrightMagenta("[validating public key]").Bold(), pubKeys[i])
		if cfg.ShowPrivateKeys {
			if len(privateKeys) > i {
				fmt.Printf("%s %#x\n", au.BrightRed("[validating private key]").Bold(), privateKeys[i])
			}
		}
		if !cfg.ShowDepositData {
			continue
		}
		fmt.Printf(
			"%s\n",
			au.BrightRed("If you imported your account coming from the eth2 launchpad, you will find your "+
				"deposit_data.json in the eth2.0-deposit-cli's validator_keys folder"),
		)
		fmt.Println("")
	}
	fmt.Println("")
	return nil
}

func CreatePrintoutOfKeys(keys [][]byte) string {
	var keysStr string
	for i, k := range keys {
		if i != 0 {
			keysStr += "," // Add a comma before each key except the first one
		}
		keysStr += fmt.Sprintf("%#x", bytesutil.Trunc(k))
	}
	return keysStr
}
