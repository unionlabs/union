package rpc

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"path/filepath"
	"strings"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/io/prompt"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/wallet"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/tyler-smith/go-bip39"
	"github.com/tyler-smith/go-bip39/wordlists"
	keystorev4 "github.com/wealdtech/go-eth2-wallet-encryptor-keystorev4"
	"go.opencensus.io/trace"
)

// CreateWallet via an API request, allowing a user to save a new
func (s *Server) CreateWallet(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.web.CreateWallet")
	defer span.End()

	var req CreateWalletRequest
	err := json.NewDecoder(r.Body).Decode(&req)
	switch {
	case err == io.EOF:
		httputil.HandleError(w, "No data submitted", http.StatusBadRequest)
		return
	case err != nil:
		httputil.HandleError(w, "Could not decode request body: "+err.Error(), http.StatusBadRequest)
		return
	}

	walletDir := s.walletDir
	exists, err := wallet.Exists(walletDir)
	if err != nil {
		httputil.HandleError(w, "Could not check for existing wallet: "+err.Error(), http.StatusInternalServerError)
		return
	}
	if exists {
		if err := s.initializeWallet(ctx, &wallet.Config{
			WalletDir:      walletDir,
			WalletPassword: req.WalletPassword,
		}); err != nil {
			httputil.HandleError(w, "Could not initialize wallet: "+err.Error(), http.StatusInternalServerError)
			return
		}
		keymanagerKind := importedKeymanagerKind
		switch s.wallet.KeymanagerKind() {
		case keymanager.Derived:
			keymanagerKind = derivedKeymanagerKind
		case keymanager.Web3Signer:
			keymanagerKind = web3signerKeymanagerKind
		}
		response := &CreateWalletResponse{
			Wallet: &WalletResponse{
				WalletPath:     walletDir,
				KeymanagerKind: keymanagerKind,
			},
		}
		httputil.WriteJson(w, response)
		return
	}
	if err := prompt.ValidatePasswordInput(req.WalletPassword); err != nil {
		httputil.HandleError(w, "Password too weak: "+err.Error(), http.StatusBadRequest)
		return
	}
	if req.Keymanager == importedKeymanagerKind {
		opts := []accounts.Option{
			accounts.WithWalletDir(walletDir),
			accounts.WithKeymanagerType(keymanager.Local),
			accounts.WithWalletPassword(req.WalletPassword),
			accounts.WithSkipMnemonicConfirm(true),
		}
		acc, err := accounts.NewCLIManager(opts...)
		if err != nil {
			httputil.HandleError(w, "Could not create CLI Manager: "+err.Error(), http.StatusInternalServerError)
			return
		}
		_, err = acc.WalletCreate(ctx)
		if err != nil {
			httputil.HandleError(w, "Could not create wallet: "+err.Error(), http.StatusInternalServerError)
			return
		}
		if err := s.initializeWallet(ctx, &wallet.Config{
			WalletDir:      walletDir,
			KeymanagerKind: keymanager.Local,
			WalletPassword: req.WalletPassword,
		}); err != nil {
			httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
			return
		}
		if err := writeWalletPasswordToDisk(walletDir, req.WalletPassword); err != nil {
			httputil.HandleError(w, "Could not write wallet password to disk: "+err.Error(), http.StatusInternalServerError)
			return
		}
		response := &CreateWalletResponse{
			Wallet: &WalletResponse{
				WalletPath:     walletDir,
				KeymanagerKind: importedKeymanagerKind,
			},
		}
		httputil.WriteJson(w, response)
		return
	}
	httputil.HandleError(w, fmt.Sprintf("Keymanager type %s create wallet not supported through web", req.Keymanager), http.StatusBadRequest)
}

// WalletConfig returns the wallet's configuration. If no wallet exists, we return an empty response.
func (s *Server) WalletConfig(w http.ResponseWriter, r *http.Request) {
	_, span := trace.StartSpan(r.Context(), "validator.web.WalletConfig")
	defer span.End()

	exists, err := wallet.Exists(s.walletDir)
	if err != nil {
		httputil.HandleError(w, wallet.CheckExistsErrMsg+": "+err.Error(), http.StatusInternalServerError)
		return
	}
	if !exists {
		// If no wallet is found, we simply return an empty response.
		httputil.WriteJson(w, &WalletResponse{})
		return
	}
	valid, err := wallet.IsValid(s.walletDir)
	if errors.Is(err, wallet.ErrNoWalletFound) {
		httputil.WriteJson(w, &WalletResponse{})
		return
	}
	if err != nil {
		httputil.HandleError(w, wallet.CheckValidityErrMsg+": "+err.Error(), http.StatusInternalServerError)
		return
	}
	if !valid {
		httputil.HandleError(w, wallet.InvalidWalletErrMsg, http.StatusInternalServerError)
		return
	}

	if s.wallet == nil || s.validatorService == nil {
		// If no wallet is found, we simply return an empty response.
		httputil.WriteJson(w, &WalletResponse{})
		return
	}
	var keymanagerKind KeymanagerKind
	switch s.wallet.KeymanagerKind() {
	case keymanager.Derived:
		keymanagerKind = derivedKeymanagerKind
	case keymanager.Local:
		keymanagerKind = importedKeymanagerKind
	case keymanager.Web3Signer:
		keymanagerKind = web3signerKeymanagerKind
	}
	httputil.WriteJson(w, &WalletResponse{
		WalletPath:     s.walletDir,
		KeymanagerKind: keymanagerKind,
	})
}

// RecoverWallet via an API request, allowing a user to recover a derived wallet.
// Generate the seed from the mnemonic + language + 25th passphrase(optional).
// Create N validator keystores from the seed specified by req.NumAccounts.
// Set the wallet password to req.WalletPassword, then create the wallet from
// the provided Mnemonic and return CreateWalletResponse.
// DEPRECATED: this endpoint will be removed to improve the safety and security of interacting with wallets
func (s *Server) RecoverWallet(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "validator.web.RecoverWallet")
	defer span.End()

	var req RecoverWalletRequest
	err := json.NewDecoder(r.Body).Decode(&req)
	switch {
	case err == io.EOF:
		httputil.HandleError(w, "No data submitted", http.StatusBadRequest)
		return
	case err != nil:
		httputil.HandleError(w, "Could not decode request body: "+err.Error(), http.StatusBadRequest)
		return
	}

	numAccounts := int(req.NumAccounts)
	if numAccounts == 0 {
		httputil.HandleError(w, "Must create at least 1 validator account", http.StatusBadRequest)
		return
	}

	// Check validate mnemonic with chosen language
	language := strings.ToLower(req.Language)
	allowedLanguages := map[string][]string{
		"chinese_simplified":  wordlists.ChineseSimplified,
		"chinese_traditional": wordlists.ChineseTraditional,
		"czech":               wordlists.Czech,
		"english":             wordlists.English,
		"french":              wordlists.French,
		"japanese":            wordlists.Japanese,
		"korean":              wordlists.Korean,
		"italian":             wordlists.Italian,
		"spanish":             wordlists.Spanish,
	}
	if _, ok := allowedLanguages[language]; !ok {
		httputil.HandleError(w, "input not in the list of supported languages", http.StatusBadRequest)
		return
	}
	bip39.SetWordList(allowedLanguages[language])
	mnemonic := req.Mnemonic
	if err := accounts.ValidateMnemonic(mnemonic); err != nil {
		httputil.HandleError(w, "invalid mnemonic in request: "+err.Error(), http.StatusBadRequest)
		return
	}

	// Check it is not whitespace-only (empty is valid)
	if req.Mnemonic25ThWord != "" && strings.TrimSpace(req.Mnemonic25ThWord) == "" {
		httputil.HandleError(w, "mnemonic 25th word cannot be empty", http.StatusBadRequest)
		return
	}

	// Web UI is structured to only write to the default wallet directory
	// accounts.Recoverwallet checks if wallet already exists.
	walletDir := s.walletDir

	// Web UI should check the new and confirmed password are equal.
	walletPassword := req.WalletPassword
	if err := prompt.ValidatePasswordInput(walletPassword); err != nil {
		httputil.HandleError(w, "password did not pass validation: "+err.Error(), http.StatusBadRequest)
		return
	}

	opts := []accounts.Option{
		accounts.WithWalletDir(walletDir),
		accounts.WithWalletPassword(walletPassword),
		accounts.WithMnemonic(mnemonic),
		accounts.WithMnemonic25thWord(req.Mnemonic25ThWord),
		accounts.WithNumAccounts(numAccounts),
	}
	acc, err := accounts.NewCLIManager(opts...)
	if err != nil {
		httputil.HandleError(w, "Could not create CLI Manager: "+err.Error(), http.StatusInternalServerError)
		return
	}
	if _, err := acc.WalletRecover(ctx); err != nil {
		httputil.HandleError(w, "Failed to recover wallet: "+err.Error(), http.StatusInternalServerError)
		return
	}
	if err := s.initializeWallet(ctx, &wallet.Config{
		WalletDir:      walletDir,
		KeymanagerKind: keymanager.Derived,
		WalletPassword: walletPassword,
	}); err != nil {
		httputil.HandleError(w, "Failed to initialize wallet: "+err.Error(), http.StatusInternalServerError)
		return
	}
	if err := writeWalletPasswordToDisk(walletDir, walletPassword); err != nil {
		httputil.HandleError(w, "Could not write wallet password to disk: "+err.Error(), http.StatusInternalServerError)
		return
	}
	httputil.WriteJson(w, &CreateWalletResponse{
		Wallet: &WalletResponse{
			WalletPath:     walletDir,
			KeymanagerKind: derivedKeymanagerKind,
		},
	})
}

// ValidateKeystores checks whether a set of EIP-2335 keystores in the request
// can indeed be decrypted using a password in the request. If there is no issue,
// we return an empty response with no error. If the password is incorrect for a single keystore,
// we return an appropriate error.
func (*Server) ValidateKeystores(w http.ResponseWriter, r *http.Request) {
	_, span := trace.StartSpan(r.Context(), "validator.web.ValidateKeystores")
	defer span.End()

	var req ValidateKeystoresRequest
	err := json.NewDecoder(r.Body).Decode(&req)
	switch {
	case err == io.EOF:
		httputil.HandleError(w, "No data submitted", http.StatusBadRequest)
		return
	case err != nil:
		httputil.HandleError(w, "Could not decode request body: "+err.Error(), http.StatusBadRequest)
		return
	}

	if req.KeystoresPassword == "" {
		httputil.HandleError(w, "Password required for keystores", http.StatusBadRequest)
		return
	}
	// Needs to unmarshal the keystores from the requests.
	if req.Keystores == nil || len(req.Keystores) < 1 {
		httputil.HandleError(w, "No keystores included in request", http.StatusBadRequest)
		return
	}
	decryptor := keystorev4.New()
	for i := 0; i < len(req.Keystores); i++ {
		encoded := req.Keystores[i]
		keystore := &keymanager.Keystore{}
		if err := json.Unmarshal([]byte(encoded), &keystore); err != nil {
			httputil.HandleError(w, "Not a valid EIP-2335 keystore JSON file: "+err.Error(), http.StatusBadRequest)
			return
		}
		if keystore.Description == "" && keystore.Name != "" {
			keystore.Description = keystore.Name
		}
		if _, err := decryptor.Decrypt(keystore.Crypto, req.KeystoresPassword); err != nil {
			doesNotDecrypt := strings.Contains(err.Error(), keymanager.IncorrectPasswordErrMsg)
			if doesNotDecrypt {
				httputil.HandleError(w, fmt.Sprintf("Password for keystore with public key %s is incorrect. "+
					"Prysm web only supports importing batches of keystores with the same password for all of them",
					keystore.Pubkey), http.StatusBadRequest)
				return
			} else {
				httputil.HandleError(w, "Unexpected error decrypting keystore: "+err.Error(), http.StatusInternalServerError)
				return
			}
		}
	}
}

// Initialize a wallet and send it over a global feed.
func (s *Server) initializeWallet(ctx context.Context, cfg *wallet.Config) error {
	// We first ensure the user has a wallet.
	exists, err := wallet.Exists(cfg.WalletDir)
	if err != nil {
		return errors.Wrap(err, wallet.CheckExistsErrMsg)
	}
	if !exists {
		return wallet.ErrNoWalletFound
	}
	valid, err := wallet.IsValid(cfg.WalletDir)
	if errors.Is(err, wallet.ErrNoWalletFound) {
		return wallet.ErrNoWalletFound
	}
	if err != nil {
		return errors.Wrap(err, wallet.CheckValidityErrMsg)
	}
	if !valid {
		return errors.New(wallet.InvalidWalletErrMsg)
	}

	// We fire an event with the opened wallet over
	// a global feed signifying wallet initialization.
	w, err := wallet.OpenWallet(ctx, &wallet.Config{
		WalletDir:      cfg.WalletDir,
		WalletPassword: cfg.WalletPassword,
	})
	if err != nil {
		return errors.Wrap(err, "could not open wallet")
	}

	s.walletInitialized = true
	s.wallet = w
	s.walletDir = cfg.WalletDir

	s.walletInitializedFeed.Send(w)

	return nil
}

func writeWalletPasswordToDisk(walletDir, password string) error {
	if !features.Get().WriteWalletPasswordOnWebOnboarding {
		return nil
	}
	passwordFilePath := filepath.Join(walletDir, wallet.DefaultWalletPasswordFile)
	if file.Exists(passwordFilePath) {
		return fmt.Errorf("cannot write wallet password file as it already exists %s", passwordFilePath)
	}
	return file.WriteFile(passwordFilePath, []byte(password))
}
