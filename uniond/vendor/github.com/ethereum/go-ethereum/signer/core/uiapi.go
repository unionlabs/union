// Copyright 2019 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package core

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"math/big"
	"os"

	"github.com/ethereum/go-ethereum/accounts"
	"github.com/ethereum/go-ethereum/accounts/keystore"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/math"
	"github.com/ethereum/go-ethereum/crypto"
)

// SignerUIAPI implements methods Clef provides for a UI to query, in the bidirectional communication
// channel.
// This API is considered secure, since a request can only
// ever arrive from the UI -- and the UI is capable of approving any action, thus we can consider these
// requests pre-approved.
// NB: It's very important that these methods are not ever exposed on the external service
// registry.
type UIServerAPI struct {
	extApi *SignerAPI
	am     *accounts.Manager
}

// NewUIServerAPI creates a new UIServerAPI
func NewUIServerAPI(extapi *SignerAPI) *UIServerAPI {
	return &UIServerAPI{extapi, extapi.am}
}

// List available accounts. As opposed to the external API definition, this method delivers
// the full Account object and not only Address.
// Example call
// {"jsonrpc":"2.0","method":"clef_listAccounts","params":[], "id":4}
func (s *UIServerAPI) ListAccounts(ctx context.Context) ([]accounts.Account, error) {
	var accs []accounts.Account
	for _, wallet := range s.am.Wallets() {
		accs = append(accs, wallet.Accounts()...)
	}
	return accs, nil
}

// rawWallet is a JSON representation of an accounts.Wallet interface, with its
// data contents extracted into plain fields.
type rawWallet struct {
	URL      string             `json:"url"`
	Status   string             `json:"status"`
	Failure  string             `json:"failure,omitempty"`
	Accounts []accounts.Account `json:"accounts,omitempty"`
}

// ListWallets will return a list of wallets that clef manages
// Example call
// {"jsonrpc":"2.0","method":"clef_listWallets","params":[], "id":5}
func (s *UIServerAPI) ListWallets() []rawWallet {
	wallets := make([]rawWallet, 0) // return [] instead of nil if empty
	for _, wallet := range s.am.Wallets() {
		status, failure := wallet.Status()

		raw := rawWallet{
			URL:      wallet.URL().String(),
			Status:   status,
			Accounts: wallet.Accounts(),
		}
		if failure != nil {
			raw.Failure = failure.Error()
		}
		wallets = append(wallets, raw)
	}
	return wallets
}

// DeriveAccount requests a HD wallet to derive a new account, optionally pinning
// it for later reuse.
// Example call
// {"jsonrpc":"2.0","method":"clef_deriveAccount","params":["ledger://","m/44'/60'/0'", false], "id":6}
func (s *UIServerAPI) DeriveAccount(url string, path string, pin *bool) (accounts.Account, error) {
	wallet, err := s.am.Wallet(url)
	if err != nil {
		return accounts.Account{}, err
	}
	derivPath, err := accounts.ParseDerivationPath(path)
	if err != nil {
		return accounts.Account{}, err
	}
	if pin == nil {
		pin = new(bool)
	}
	return wallet.Derive(derivPath, *pin)
}

// fetchKeystore retrieves the encrypted keystore from the account manager.
func fetchKeystore(am *accounts.Manager) *keystore.KeyStore {
	ks := am.Backends(keystore.KeyStoreType)
	if len(ks) == 0 {
		return nil
	}
	return ks[0].(*keystore.KeyStore)
}

// ImportRawKey stores the given hex encoded ECDSA key into the key directory,
// encrypting it with the passphrase.
// Example call (should fail on password too short)
// {"jsonrpc":"2.0","method":"clef_importRawKey","params":["1111111111111111111111111111111111111111111111111111111111111111","test"], "id":6}
func (s *UIServerAPI) ImportRawKey(privkey string, password string) (accounts.Account, error) {
	key, err := crypto.HexToECDSA(privkey)
	if err != nil {
		return accounts.Account{}, err
	}
	if err := ValidatePasswordFormat(password); err != nil {
		return accounts.Account{}, fmt.Errorf("password requirements not met: %v", err)
	}
	// No error
	return fetchKeystore(s.am).ImportECDSA(key, password)
}

// OpenWallet initiates a hardware wallet opening procedure, establishing a USB
// connection and attempting to authenticate via the provided passphrase. Note,
// the method may return an extra challenge requiring a second open (e.g. the
// Trezor PIN matrix challenge).
// Example
// {"jsonrpc":"2.0","method":"clef_openWallet","params":["ledger://",""], "id":6}
func (s *UIServerAPI) OpenWallet(url string, passphrase *string) error {
	wallet, err := s.am.Wallet(url)
	if err != nil {
		return err
	}
	pass := ""
	if passphrase != nil {
		pass = *passphrase
	}
	return wallet.Open(pass)
}

// ChainId returns the chainid in use for Eip-155 replay protection
// Example call
// {"jsonrpc":"2.0","method":"clef_chainId","params":[], "id":8}
func (s *UIServerAPI) ChainId() math.HexOrDecimal64 {
	return (math.HexOrDecimal64)(s.extApi.chainID.Uint64())
}

// SetChainId sets the chain id to use when signing transactions.
// Example call to set Ropsten:
// {"jsonrpc":"2.0","method":"clef_setChainId","params":["3"], "id":8}
func (s *UIServerAPI) SetChainId(id math.HexOrDecimal64) math.HexOrDecimal64 {
	s.extApi.chainID = new(big.Int).SetUint64(uint64(id))
	return s.ChainId()
}

// Export returns encrypted private key associated with the given address in web3 keystore format.
// Example
// {"jsonrpc":"2.0","method":"clef_export","params":["0x19e7e376e7c213b7e7e7e46cc70a5dd086daff2a"], "id":4}
func (s *UIServerAPI) Export(ctx context.Context, addr common.Address) (json.RawMessage, error) {
	// Look up the wallet containing the requested signer
	wallet, err := s.am.Find(accounts.Account{Address: addr})
	if err != nil {
		return nil, err
	}
	if wallet.URL().Scheme != keystore.KeyStoreScheme {
		return nil, errors.New("account is not a keystore-account")
	}
	return os.ReadFile(wallet.URL().Path)
}

// Import tries to import the given keyJSON in the local keystore. The keyJSON data is expected to be
// in web3 keystore format. It will decrypt the keyJSON with the given passphrase and on successful
// decryption it will encrypt the key with the given newPassphrase and store it in the keystore.
// Example (the address in question has privkey `11...11`):
// {"jsonrpc":"2.0","method":"clef_import","params":[{"address":"19e7e376e7c213b7e7e7e46cc70a5dd086daff2a","crypto":{"cipher":"aes-128-ctr","ciphertext":"33e4cd3756091d037862bb7295e9552424a391a6e003272180a455ca2a9fb332","cipherparams":{"iv":"b54b263e8f89c42bb219b6279fba5cce"},"kdf":"scrypt","kdfparams":{"dklen":32,"n":262144,"p":1,"r":8,"salt":"e4ca94644fd30569c1b1afbbc851729953c92637b7fe4bb9840bbb31ffbc64a5"},"mac":"f4092a445c2b21c0ef34f17c9cd0d873702b2869ec5df4439a0c2505823217e7"},"id":"216c7eac-e8c1-49af-a215-fa0036f29141","version":3},"test","yaddayadda"], "id":4}
func (api *UIServerAPI) Import(ctx context.Context, keyJSON json.RawMessage, oldPassphrase, newPassphrase string) (accounts.Account, error) {
	be := api.am.Backends(keystore.KeyStoreType)

	if len(be) == 0 {
		return accounts.Account{}, errors.New("password based accounts not supported")
	}
	if err := ValidatePasswordFormat(newPassphrase); err != nil {
		return accounts.Account{}, fmt.Errorf("password requirements not met: %v", err)
	}
	return be[0].(*keystore.KeyStore).Import(keyJSON, oldPassphrase, newPassphrase)
}

// New creates a new password protected Account. The private key is protected with
// the given password. Users are responsible to backup the private key that is stored
// in the keystore location that was specified when this API was created.
// This method is the same as New on the external API, the difference being that
// this implementation does not ask for confirmation, since it's initiated by
// the user
func (api *UIServerAPI) New(ctx context.Context) (common.Address, error) {
	return api.extApi.newAccount()
}

// Other methods to be added, not yet implemented are:
// - Ruleset interaction: add rules, attest rulefiles
// - Store metadata about accounts, e.g. naming of accounts
