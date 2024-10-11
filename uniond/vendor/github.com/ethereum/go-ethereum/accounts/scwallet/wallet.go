// Copyright 2018 The go-ethereum Authors
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

package scwallet

import (
	"bytes"
	"context"
	"crypto/hmac"
	"crypto/sha256"
	"crypto/sha512"
	"encoding/asn1"
	"encoding/binary"
	"errors"
	"fmt"
	"math/big"
	"regexp"
	"sort"
	"strings"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/accounts"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/log"
	pcsc "github.com/gballet/go-libpcsclite"
	"github.com/status-im/keycard-go/derivationpath"
)

// ErrPairingPasswordNeeded is returned if opening the smart card requires pairing with a pairing
// password. In this case, the calling application should request user input to enter
// the pairing password and send it back.
var ErrPairingPasswordNeeded = errors.New("smartcard: pairing password needed")

// ErrPINNeeded is returned if opening the smart card requires a PIN code. In
// this case, the calling application should request user input to enter the PIN
// and send it back.
var ErrPINNeeded = errors.New("smartcard: pin needed")

// ErrPINUnblockNeeded is returned if opening the smart card requires a PIN code,
// but all PIN attempts have already been exhausted. In this case the calling
// application should request user input for the PUK and a new PIN code to set
// fo the card.
var ErrPINUnblockNeeded = errors.New("smartcard: pin unblock needed")

// ErrAlreadyOpen is returned if the smart card is attempted to be opened, but
// there is already a paired and unlocked session.
var ErrAlreadyOpen = errors.New("smartcard: already open")

// ErrPubkeyMismatch is returned if the public key recovered from a signature
// does not match the one expected by the user.
var ErrPubkeyMismatch = errors.New("smartcard: recovered public key mismatch")

var (
	appletAID = []byte{0xA0, 0x00, 0x00, 0x08, 0x04, 0x00, 0x01, 0x01, 0x01}
	// DerivationSignatureHash is used to derive the public key from the signature of this hash
	DerivationSignatureHash = sha256.Sum256(common.Hash{}.Bytes())
)

// List of APDU command-related constants
const (
	claISO7816  = 0
	claSCWallet = 0x80

	insSelect      = 0xA4
	insGetResponse = 0xC0
	sw1GetResponse = 0x61
	sw1Ok          = 0x90

	insVerifyPin  = 0x20
	insUnblockPin = 0x22
	insExportKey  = 0xC2
	insSign       = 0xC0
	insLoadKey    = 0xD0
	insDeriveKey  = 0xD1
	insStatus     = 0xF2
)

// List of ADPU command parameters
const (
	P1DeriveKeyFromMaster  = uint8(0x00)
	P1DeriveKeyFromParent  = uint8(0x01)
	P1DeriveKeyFromCurrent = uint8(0x10)
	statusP1WalletStatus   = uint8(0x00)
	statusP1Path           = uint8(0x01)
	signP1PrecomputedHash  = uint8(0x00)
	signP2OnlyBlock        = uint8(0x00)
	exportP1Any            = uint8(0x00)
	exportP2Pubkey         = uint8(0x01)
)

// Minimum time to wait between self derivation attempts, even it the user is
// requesting accounts like crazy.
const selfDeriveThrottling = time.Second

// Wallet represents a smartcard wallet instance.
type Wallet struct {
	Hub       *Hub   // A handle to the Hub that instantiated this wallet.
	PublicKey []byte // The wallet's public key (used for communication and identification, not signing!)

	lock    sync.Mutex // Lock that gates access to struct fields and communication with the card
	card    *pcsc.Card // A handle to the smartcard interface for the wallet.
	session *Session   // The secure communication session with the card
	log     log.Logger // Contextual logger to tag the base with its id

	deriveNextPaths []accounts.DerivationPath // Next derivation paths for account auto-discovery (multiple bases supported)
	deriveNextAddrs []common.Address          // Next derived account addresses for auto-discovery (multiple bases supported)
	deriveChain     ethereum.ChainStateReader // Blockchain state reader to discover used account with
	deriveReq       chan chan struct{}        // Channel to request a self-derivation on
	deriveQuit      chan chan error           // Channel to terminate the self-deriver with
}

// NewWallet constructs and returns a new Wallet instance.
func NewWallet(hub *Hub, card *pcsc.Card) *Wallet {
	wallet := &Wallet{
		Hub:  hub,
		card: card,
	}
	return wallet
}

// transmit sends an APDU to the smartcard and receives and decodes the response.
// It automatically handles requests by the card to fetch the return data separately,
// and returns an error if the response status code is not success.
func transmit(card *pcsc.Card, command *commandAPDU) (*responseAPDU, error) {
	data, err := command.serialize()
	if err != nil {
		return nil, err
	}

	responseData, _, err := card.Transmit(data)
	if err != nil {
		return nil, err
	}

	response := new(responseAPDU)
	if err = response.deserialize(responseData); err != nil {
		return nil, err
	}

	// Are we being asked to fetch the response separately?
	if response.Sw1 == sw1GetResponse && (command.Cla != claISO7816 || command.Ins != insGetResponse) {
		return transmit(card, &commandAPDU{
			Cla:  claISO7816,
			Ins:  insGetResponse,
			P1:   0,
			P2:   0,
			Data: nil,
			Le:   response.Sw2,
		})
	}

	if response.Sw1 != sw1Ok {
		return nil, fmt.Errorf("unexpected insecure response status Cla=%#x, Ins=%#x, Sw=%#x%x", command.Cla, command.Ins, response.Sw1, response.Sw2)
	}

	return response, nil
}

// applicationInfo encodes information about the smartcard application - its
// instance UID and public key.
type applicationInfo struct {
	InstanceUID []byte `asn1:"tag:15"`
	PublicKey   []byte `asn1:"tag:0"`
}

// connect connects to the wallet application and establishes a secure channel with it.
// must be called before any other interaction with the wallet.
func (w *Wallet) connect() error {
	w.lock.Lock()
	defer w.lock.Unlock()

	appinfo, err := w.doselect()
	if err != nil {
		return err
	}

	channel, err := NewSecureChannelSession(w.card, appinfo.PublicKey)
	if err != nil {
		return err
	}

	w.PublicKey = appinfo.PublicKey
	w.log = log.New("url", w.URL())
	w.session = &Session{
		Wallet:  w,
		Channel: channel,
	}
	return nil
}

// doselect is an internal (unlocked) function to send a SELECT APDU to the card.
func (w *Wallet) doselect() (*applicationInfo, error) {
	response, err := transmit(w.card, &commandAPDU{
		Cla:  claISO7816,
		Ins:  insSelect,
		P1:   4,
		P2:   0,
		Data: appletAID,
	})
	if err != nil {
		return nil, err
	}

	appinfo := new(applicationInfo)
	if _, err := asn1.UnmarshalWithParams(response.Data, appinfo, "tag:4"); err != nil {
		return nil, err
	}
	return appinfo, nil
}

// ping checks the card's status and returns an error if unsuccessful.
func (w *Wallet) ping() error {
	w.lock.Lock()
	defer w.lock.Unlock()

	// We can't ping if not paired
	if !w.session.paired() {
		return nil
	}
	if _, err := w.session.walletStatus(); err != nil {
		return err
	}
	return nil
}

// release releases any resources held by an open wallet instance.
func (w *Wallet) release() error {
	if w.session != nil {
		return w.session.release()
	}
	return nil
}

// pair is an internal (unlocked) function for establishing a new pairing
// with the wallet.
func (w *Wallet) pair(puk []byte) error {
	if w.session.paired() {
		return errors.New("wallet already paired")
	}
	pairing, err := w.session.pair(puk)
	if err != nil {
		return err
	}
	if err = w.Hub.setPairing(w, &pairing); err != nil {
		return err
	}
	return w.session.authenticate(pairing)
}

// Unpair deletes an existing wallet pairing.
func (w *Wallet) Unpair(pin []byte) error {
	w.lock.Lock()
	defer w.lock.Unlock()

	if !w.session.paired() {
		return fmt.Errorf("wallet %x not paired", w.PublicKey)
	}
	if err := w.session.verifyPin(pin); err != nil {
		return fmt.Errorf("failed to verify pin: %s", err)
	}
	if err := w.session.unpair(); err != nil {
		return fmt.Errorf("failed to unpair: %s", err)
	}
	if err := w.Hub.setPairing(w, nil); err != nil {
		return err
	}
	return nil
}

// URL retrieves the canonical path under which this wallet is reachable. It is
// user by upper layers to define a sorting order over all wallets from multiple
// backends.
func (w *Wallet) URL() accounts.URL {
	return accounts.URL{
		Scheme: w.Hub.scheme,
		Path:   fmt.Sprintf("%x", w.PublicKey[1:5]), // Byte #0 isn't unique; 1:5 covers << 64K cards, bump to 1:9 for << 4M
	}
}

// Status returns a textual status to aid the user in the current state of the
// wallet. It also returns an error indicating any failure the wallet might have
// encountered.
func (w *Wallet) Status() (string, error) {
	w.lock.Lock()
	defer w.lock.Unlock()

	// If the card is not paired, we can only wait
	if !w.session.paired() {
		return "Unpaired, waiting for pairing password", nil
	}
	// Yay, we have an encrypted session, retrieve the actual status
	status, err := w.session.walletStatus()
	if err != nil {
		return fmt.Sprintf("Failed: %v", err), err
	}
	switch {
	case !w.session.verified && status.PinRetryCount == 0 && status.PukRetryCount == 0:
		return "Bricked, waiting for full wipe", nil
	case !w.session.verified && status.PinRetryCount == 0:
		return fmt.Sprintf("Blocked, waiting for PUK (%d attempts left) and new PIN", status.PukRetryCount), nil
	case !w.session.verified:
		return fmt.Sprintf("Locked, waiting for PIN (%d attempts left)", status.PinRetryCount), nil
	case !status.Initialized:
		return "Empty, waiting for initialization", nil
	default:
		return "Online", nil
	}
}

// Open initializes access to a wallet instance. It is not meant to unlock or
// decrypt account keys, rather simply to establish a connection to hardware
// wallets and/or to access derivation seeds.
//
// The passphrase parameter may or may not be used by the implementation of a
// particular wallet instance. The reason there is no passwordless open method
// is to strive towards a uniform wallet handling, oblivious to the different
// backend providers.
//
// Please note, if you open a wallet, you must close it to release any allocated
// resources (especially important when working with hardware wallets).
func (w *Wallet) Open(passphrase string) error {
	w.lock.Lock()
	defer w.lock.Unlock()

	// If the session is already open, bail out
	if w.session.verified {
		return ErrAlreadyOpen
	}
	// If the smart card is not yet paired, attempt to do so either from a previous
	// pairing key or form the supplied PUK code.
	if !w.session.paired() {
		// If a previous pairing exists, only ever try to use that
		if pairing := w.Hub.pairing(w); pairing != nil {
			if err := w.session.authenticate(*pairing); err != nil {
				return fmt.Errorf("failed to authenticate card %x: %s", w.PublicKey[:4], err)
			}
			// Pairing still ok, fall through to PIN checks
		} else {
			// If no passphrase was supplied, request the PUK from the user
			if passphrase == "" {
				return ErrPairingPasswordNeeded
			}
			// Attempt to pair the smart card with the user supplied PUK
			if err := w.pair([]byte(passphrase)); err != nil {
				return err
			}
			// Pairing succeeded, fall through to PIN checks. This will of course fail,
			// but we can't return ErrPINNeeded directly here because we don't know whether
			// a PIN check or a PIN reset is needed.
			passphrase = ""
		}
	}
	// The smart card was successfully paired, retrieve its status to check whether
	// PIN verification or unblocking is needed.
	status, err := w.session.walletStatus()
	if err != nil {
		return err
	}
	// Request the appropriate next authentication data, or use the one supplied
	switch {
	case passphrase == "" && status.PinRetryCount > 0:
		return ErrPINNeeded
	case passphrase == "":
		return ErrPINUnblockNeeded
	case status.PinRetryCount > 0:
		if !regexp.MustCompile(`^[0-9]{6,}$`).MatchString(passphrase) {
			w.log.Error("PIN needs to be at least 6 digits")
			return ErrPINNeeded
		}
		if err := w.session.verifyPin([]byte(passphrase)); err != nil {
			return err
		}
	default:
		if !regexp.MustCompile(`^[0-9]{12,}$`).MatchString(passphrase) {
			w.log.Error("PUK needs to be at least 12 digits")
			return ErrPINUnblockNeeded
		}
		if err := w.session.unblockPin([]byte(passphrase)); err != nil {
			return err
		}
	}
	// Smart card paired and unlocked, initialize and register
	w.deriveReq = make(chan chan struct{})
	w.deriveQuit = make(chan chan error)

	go w.selfDerive()

	// Notify anyone listening for wallet events that a new device is accessible
	go w.Hub.updateFeed.Send(accounts.WalletEvent{Wallet: w, Kind: accounts.WalletOpened})

	return nil
}

// Close stops and closes the wallet, freeing any resources.
func (w *Wallet) Close() error {
	// Ensure the wallet was opened
	w.lock.Lock()
	dQuit := w.deriveQuit
	w.lock.Unlock()

	// Terminate the self-derivations
	var derr error
	if dQuit != nil {
		errc := make(chan error)
		dQuit <- errc
		derr = <-errc // Save for later, we *must* close the USB
	}
	// Terminate the device connection
	w.lock.Lock()
	defer w.lock.Unlock()

	w.deriveQuit = nil
	w.deriveReq = nil

	if err := w.release(); err != nil {
		return err
	}
	return derr
}

// selfDerive is an account derivation loop that upon request attempts to find
// new non-zero accounts.
func (w *Wallet) selfDerive() {
	w.log.Debug("Smart card wallet self-derivation started")
	defer w.log.Debug("Smart card wallet self-derivation stopped")

	// Execute self-derivations until termination or error
	var (
		reqc chan struct{}
		errc chan error
		err  error
	)
	for errc == nil && err == nil {
		// Wait until either derivation or termination is requested
		select {
		case errc = <-w.deriveQuit:
			// Termination requested
			continue
		case reqc = <-w.deriveReq:
			// Account discovery requested
		}
		// Derivation needs a chain and device access, skip if either unavailable
		w.lock.Lock()
		if w.session == nil || w.deriveChain == nil {
			w.lock.Unlock()
			reqc <- struct{}{}
			continue
		}
		pairing := w.Hub.pairing(w)

		// Device lock obtained, derive the next batch of accounts
		var (
			paths   []accounts.DerivationPath
			nextAcc accounts.Account

			nextPaths = append([]accounts.DerivationPath{}, w.deriveNextPaths...)
			nextAddrs = append([]common.Address{}, w.deriveNextAddrs...)

			context = context.Background()
		)
		for i := 0; i < len(nextAddrs); i++ {
			for empty := false; !empty; {
				// Retrieve the next derived Ethereum account
				if nextAddrs[i] == (common.Address{}) {
					if nextAcc, err = w.session.derive(nextPaths[i]); err != nil {
						w.log.Warn("Smartcard wallet account derivation failed", "err", err)
						break
					}
					nextAddrs[i] = nextAcc.Address
				}
				// Check the account's status against the current chain state
				var (
					balance *big.Int
					nonce   uint64
				)
				balance, err = w.deriveChain.BalanceAt(context, nextAddrs[i], nil)
				if err != nil {
					w.log.Warn("Smartcard wallet balance retrieval failed", "err", err)
					break
				}
				nonce, err = w.deriveChain.NonceAt(context, nextAddrs[i], nil)
				if err != nil {
					w.log.Warn("Smartcard wallet nonce retrieval failed", "err", err)
					break
				}
				// If the next account is empty, stop self-derivation, but add for the last base path
				if balance.Sign() == 0 && nonce == 0 {
					empty = true
					if i < len(nextAddrs)-1 {
						break
					}
				}
				// We've just self-derived a new account, start tracking it locally
				path := make(accounts.DerivationPath, len(nextPaths[i]))
				copy(path[:], nextPaths[i][:])
				paths = append(paths, path)

				// Display a log message to the user for new (or previously empty accounts)
				if _, known := pairing.Accounts[nextAddrs[i]]; !known || !empty || nextAddrs[i] != w.deriveNextAddrs[i] {
					w.log.Info("Smartcard wallet discovered new account", "address", nextAddrs[i], "path", path, "balance", balance, "nonce", nonce)
				}
				pairing.Accounts[nextAddrs[i]] = path

				// Fetch the next potential account
				if !empty {
					nextAddrs[i] = common.Address{}
					nextPaths[i][len(nextPaths[i])-1]++
				}
			}
		}
		// If there are new accounts, write them out
		if len(paths) > 0 {
			err = w.Hub.setPairing(w, pairing)
		}
		// Shift the self-derivation forward
		w.deriveNextAddrs = nextAddrs
		w.deriveNextPaths = nextPaths

		// Self derivation complete, release device lock
		w.lock.Unlock()

		// Notify the user of termination and loop after a bit of time (to avoid trashing)
		reqc <- struct{}{}
		if err == nil {
			select {
			case errc = <-w.deriveQuit:
				// Termination requested, abort
			case <-time.After(selfDeriveThrottling):
				// Waited enough, willing to self-derive again
			}
		}
	}
	// In case of error, wait for termination
	if err != nil {
		w.log.Debug("Smartcard wallet self-derivation failed", "err", err)
		errc = <-w.deriveQuit
	}
	errc <- err
}

// Accounts retrieves the list of signing accounts the wallet is currently aware
// of. For hierarchical deterministic wallets, the list will not be exhaustive,
// rather only contain the accounts explicitly pinned during account derivation.
func (w *Wallet) Accounts() []accounts.Account {
	// Attempt self-derivation if it's running
	reqc := make(chan struct{}, 1)
	select {
	case w.deriveReq <- reqc:
		// Self-derivation request accepted, wait for it
		<-reqc
	default:
		// Self-derivation offline, throttled or busy, skip
	}

	w.lock.Lock()
	defer w.lock.Unlock()

	if pairing := w.Hub.pairing(w); pairing != nil {
		ret := make([]accounts.Account, 0, len(pairing.Accounts))
		for address, path := range pairing.Accounts {
			ret = append(ret, w.makeAccount(address, path))
		}
		sort.Sort(accounts.AccountsByURL(ret))
		return ret
	}
	return nil
}

func (w *Wallet) makeAccount(address common.Address, path accounts.DerivationPath) accounts.Account {
	return accounts.Account{
		Address: address,
		URL: accounts.URL{
			Scheme: w.Hub.scheme,
			Path:   fmt.Sprintf("%x/%s", w.PublicKey[1:3], path.String()),
		},
	}
}

// Contains returns whether an account is part of this particular wallet or not.
func (w *Wallet) Contains(account accounts.Account) bool {
	if pairing := w.Hub.pairing(w); pairing != nil {
		_, ok := pairing.Accounts[account.Address]
		return ok
	}
	return false
}

// Initialize installs a keypair generated from the provided key into the wallet.
func (w *Wallet) Initialize(seed []byte) error {
	go w.selfDerive()
	// DO NOT lock at this stage, as the initialize
	// function relies on Status()
	return w.session.initialize(seed)
}

// Derive attempts to explicitly derive a hierarchical deterministic account at
// the specified derivation path. If requested, the derived account will be added
// to the wallet's tracked account list.
func (w *Wallet) Derive(path accounts.DerivationPath, pin bool) (accounts.Account, error) {
	w.lock.Lock()
	defer w.lock.Unlock()

	account, err := w.session.derive(path)
	if err != nil {
		return accounts.Account{}, err
	}

	if pin {
		pairing := w.Hub.pairing(w)
		pairing.Accounts[account.Address] = path
		if err := w.Hub.setPairing(w, pairing); err != nil {
			return accounts.Account{}, err
		}
	}

	return account, nil
}

// SelfDerive sets a base account derivation path from which the wallet attempts
// to discover non zero accounts and automatically add them to list of tracked
// accounts.
//
// Note, self derivation will increment the last component of the specified path
// opposed to descending into a child path to allow discovering accounts starting
// from non zero components.
//
// Some hardware wallets switched derivation paths through their evolution, so
// this method supports providing multiple bases to discover old user accounts
// too. Only the last base will be used to derive the next empty account.
//
// You can disable automatic account discovery by calling SelfDerive with a nil
// chain state reader.
func (w *Wallet) SelfDerive(bases []accounts.DerivationPath, chain ethereum.ChainStateReader) {
	w.lock.Lock()
	defer w.lock.Unlock()

	w.deriveNextPaths = make([]accounts.DerivationPath, len(bases))
	for i, base := range bases {
		w.deriveNextPaths[i] = make(accounts.DerivationPath, len(base))
		copy(w.deriveNextPaths[i][:], base[:])
	}
	w.deriveNextAddrs = make([]common.Address, len(bases))
	w.deriveChain = chain
}

// SignData requests the wallet to sign the hash of the given data.
//
// It looks up the account specified either solely via its address contained within,
// or optionally with the aid of any location metadata from the embedded URL field.
//
// If the wallet requires additional authentication to sign the request (e.g.
// a password to decrypt the account, or a PIN code o verify the transaction),
// an AuthNeededError instance will be returned, containing infos for the user
// about which fields or actions are needed. The user may retry by providing
// the needed details via SignDataWithPassphrase, or by other means (e.g. unlock
// the account in a keystore).
func (w *Wallet) SignData(account accounts.Account, mimeType string, data []byte) ([]byte, error) {
	return w.signHash(account, crypto.Keccak256(data))
}

func (w *Wallet) signHash(account accounts.Account, hash []byte) ([]byte, error) {
	w.lock.Lock()
	defer w.lock.Unlock()

	path, err := w.findAccountPath(account)
	if err != nil {
		return nil, err
	}

	return w.session.sign(path, hash)
}

// SignTx requests the wallet to sign the given transaction.
//
// It looks up the account specified either solely via its address contained within,
// or optionally with the aid of any location metadata from the embedded URL field.
//
// If the wallet requires additional authentication to sign the request (e.g.
// a password to decrypt the account, or a PIN code o verify the transaction),
// an AuthNeededError instance will be returned, containing infos for the user
// about which fields or actions are needed. The user may retry by providing
// the needed details via SignTxWithPassphrase, or by other means (e.g. unlock
// the account in a keystore).
func (w *Wallet) SignTx(account accounts.Account, tx *types.Transaction, chainID *big.Int) (*types.Transaction, error) {
	signer := types.LatestSignerForChainID(chainID)
	hash := signer.Hash(tx)
	sig, err := w.signHash(account, hash[:])
	if err != nil {
		return nil, err
	}
	return tx.WithSignature(signer, sig)
}

// SignDataWithPassphrase requests the wallet to sign the given hash with the
// given passphrase as extra authentication information.
//
// It looks up the account specified either solely via its address contained within,
// or optionally with the aid of any location metadata from the embedded URL field.
func (w *Wallet) SignDataWithPassphrase(account accounts.Account, passphrase, mimeType string, data []byte) ([]byte, error) {
	return w.signHashWithPassphrase(account, passphrase, crypto.Keccak256(data))
}

func (w *Wallet) signHashWithPassphrase(account accounts.Account, passphrase string, hash []byte) ([]byte, error) {
	if !w.session.verified {
		if err := w.Open(passphrase); err != nil {
			return nil, err
		}
	}

	return w.signHash(account, hash)
}

// SignText requests the wallet to sign the hash of a given piece of data, prefixed
// by the Ethereum prefix scheme
// It looks up the account specified either solely via its address contained within,
// or optionally with the aid of any location metadata from the embedded URL field.
//
// If the wallet requires additional authentication to sign the request (e.g.
// a password to decrypt the account, or a PIN code o verify the transaction),
// an AuthNeededError instance will be returned, containing infos for the user
// about which fields or actions are needed. The user may retry by providing
// the needed details via SignHashWithPassphrase, or by other means (e.g. unlock
// the account in a keystore).
func (w *Wallet) SignText(account accounts.Account, text []byte) ([]byte, error) {
	return w.signHash(account, accounts.TextHash(text))
}

// SignTextWithPassphrase implements accounts.Wallet, attempting to sign the
// given hash with the given account using passphrase as extra authentication
func (w *Wallet) SignTextWithPassphrase(account accounts.Account, passphrase string, text []byte) ([]byte, error) {
	return w.signHashWithPassphrase(account, passphrase, crypto.Keccak256(accounts.TextHash(text)))
}

// SignTxWithPassphrase requests the wallet to sign the given transaction, with the
// given passphrase as extra authentication information.
//
// It looks up the account specified either solely via its address contained within,
// or optionally with the aid of any location metadata from the embedded URL field.
func (w *Wallet) SignTxWithPassphrase(account accounts.Account, passphrase string, tx *types.Transaction, chainID *big.Int) (*types.Transaction, error) {
	if !w.session.verified {
		if err := w.Open(passphrase); err != nil {
			return nil, err
		}
	}
	return w.SignTx(account, tx, chainID)
}

// findAccountPath returns the derivation path for the provided account.
// It first checks for the address in the list of pinned accounts, and if it is
// not found, attempts to parse the derivation path from the account's URL.
func (w *Wallet) findAccountPath(account accounts.Account) (accounts.DerivationPath, error) {
	pairing := w.Hub.pairing(w)
	if path, ok := pairing.Accounts[account.Address]; ok {
		return path, nil
	}

	// Look for the path in the URL
	if account.URL.Scheme != w.Hub.scheme {
		return nil, fmt.Errorf("scheme %s does not match wallet scheme %s", account.URL.Scheme, w.Hub.scheme)
	}

	parts := strings.SplitN(account.URL.Path, "/", 2)
	if len(parts) != 2 {
		return nil, fmt.Errorf("invalid URL format: %s", account.URL)
	}

	if parts[0] != fmt.Sprintf("%x", w.PublicKey[1:3]) {
		return nil, fmt.Errorf("URL %s is not for this wallet", account.URL)
	}

	return accounts.ParseDerivationPath(parts[1])
}

// Session represents a secured communication session with the wallet.
type Session struct {
	Wallet   *Wallet               // A handle to the wallet that opened the session
	Channel  *SecureChannelSession // A secure channel for encrypted messages
	verified bool                  // Whether the pin has been verified in this session.
}

// pair establishes a new pairing over this channel, using the provided secret.
func (s *Session) pair(secret []byte) (smartcardPairing, error) {
	err := s.Channel.Pair(secret)
	if err != nil {
		return smartcardPairing{}, err
	}

	return smartcardPairing{
		PublicKey:    s.Wallet.PublicKey,
		PairingIndex: s.Channel.PairingIndex,
		PairingKey:   s.Channel.PairingKey,
		Accounts:     make(map[common.Address]accounts.DerivationPath),
	}, nil
}

// unpair deletes an existing pairing.
func (s *Session) unpair() error {
	if !s.verified {
		return errors.New("unpair requires that the PIN be verified")
	}
	return s.Channel.Unpair()
}

// verifyPin unlocks a wallet with the provided pin.
func (s *Session) verifyPin(pin []byte) error {
	if _, err := s.Channel.transmitEncrypted(claSCWallet, insVerifyPin, 0, 0, pin); err != nil {
		return err
	}
	s.verified = true
	return nil
}

// unblockPin unblocks a wallet with the provided puk and resets the pin to the
// new one specified.
func (s *Session) unblockPin(pukpin []byte) error {
	if _, err := s.Channel.transmitEncrypted(claSCWallet, insUnblockPin, 0, 0, pukpin); err != nil {
		return err
	}
	s.verified = true
	return nil
}

// release releases resources associated with the channel.
func (s *Session) release() error {
	return s.Wallet.card.Disconnect(pcsc.LeaveCard)
}

// paired returns true if a valid pairing exists.
func (s *Session) paired() bool {
	return s.Channel.PairingKey != nil
}

// authenticate uses an existing pairing to establish a secure channel.
func (s *Session) authenticate(pairing smartcardPairing) error {
	if !bytes.Equal(s.Wallet.PublicKey, pairing.PublicKey) {
		return fmt.Errorf("cannot pair using another wallet's pairing; %x != %x", s.Wallet.PublicKey, pairing.PublicKey)
	}
	s.Channel.PairingKey = pairing.PairingKey
	s.Channel.PairingIndex = pairing.PairingIndex
	return s.Channel.Open()
}

// walletStatus describes a smartcard wallet's status information.
type walletStatus struct {
	PinRetryCount int  // Number of remaining PIN retries
	PukRetryCount int  // Number of remaining PUK retries
	Initialized   bool // Whether the card has been initialized with a private key
}

// walletStatus fetches the wallet's status from the card.
func (s *Session) walletStatus() (*walletStatus, error) {
	response, err := s.Channel.transmitEncrypted(claSCWallet, insStatus, statusP1WalletStatus, 0, nil)
	if err != nil {
		return nil, err
	}

	status := new(walletStatus)
	if _, err := asn1.UnmarshalWithParams(response.Data, status, "tag:3"); err != nil {
		return nil, err
	}
	return status, nil
}

// derivationPath fetches the wallet's current derivation path from the card.
//
//lint:ignore U1000 needs to be added to the console interface
func (s *Session) derivationPath() (accounts.DerivationPath, error) {
	response, err := s.Channel.transmitEncrypted(claSCWallet, insStatus, statusP1Path, 0, nil)
	if err != nil {
		return nil, err
	}
	buf := bytes.NewReader(response.Data)
	path := make(accounts.DerivationPath, len(response.Data)/4)
	return path, binary.Read(buf, binary.BigEndian, &path)
}

// initializeData contains data needed to initialize the smartcard wallet.
type initializeData struct {
	PublicKey  []byte `asn1:"tag:0"`
	PrivateKey []byte `asn1:"tag:1"`
	ChainCode  []byte `asn1:"tag:2"`
}

// initialize initializes the card with new key data.
func (s *Session) initialize(seed []byte) error {
	// Check that the wallet isn't currently initialized,
	// otherwise the key would be overwritten.
	status, err := s.Wallet.Status()
	if err != nil {
		return err
	}
	if status == "Online" {
		return errors.New("card is already initialized, cowardly refusing to proceed")
	}

	s.Wallet.lock.Lock()
	defer s.Wallet.lock.Unlock()

	// HMAC the seed to produce the private key and chain code
	mac := hmac.New(sha512.New, []byte("Bitcoin seed"))
	mac.Write(seed)
	seed = mac.Sum(nil)

	key, err := crypto.ToECDSA(seed[:32])
	if err != nil {
		return err
	}

	id := initializeData{}
	id.PublicKey = crypto.FromECDSAPub(&key.PublicKey)
	id.PrivateKey = seed[:32]
	id.ChainCode = seed[32:]
	data, err := asn1.Marshal(id)
	if err != nil {
		return err
	}

	// Nasty hack to force the top-level struct tag to be context-specific
	data[0] = 0xA1

	_, err = s.Channel.transmitEncrypted(claSCWallet, insLoadKey, 0x02, 0, data)
	return err
}

// derive derives a new HD key path on the card.
func (s *Session) derive(path accounts.DerivationPath) (accounts.Account, error) {
	startingPoint, path, err := derivationpath.Decode(path.String())
	if err != nil {
		return accounts.Account{}, err
	}

	var p1 uint8
	switch startingPoint {
	case derivationpath.StartingPointMaster:
		p1 = P1DeriveKeyFromMaster
	case derivationpath.StartingPointParent:
		p1 = P1DeriveKeyFromParent
	case derivationpath.StartingPointCurrent:
		p1 = P1DeriveKeyFromCurrent
	default:
		return accounts.Account{}, fmt.Errorf("invalid startingPoint %d", startingPoint)
	}

	data := new(bytes.Buffer)
	for _, segment := range path {
		if err := binary.Write(data, binary.BigEndian, segment); err != nil {
			return accounts.Account{}, err
		}
	}

	_, err = s.Channel.transmitEncrypted(claSCWallet, insDeriveKey, p1, 0, data.Bytes())
	if err != nil {
		return accounts.Account{}, err
	}

	response, err := s.Channel.transmitEncrypted(claSCWallet, insSign, 0, 0, DerivationSignatureHash[:])
	if err != nil {
		return accounts.Account{}, err
	}

	sigdata := new(signatureData)
	if _, err := asn1.UnmarshalWithParams(response.Data, sigdata, "tag:0"); err != nil {
		return accounts.Account{}, err
	}
	rbytes, sbytes := sigdata.Signature.R.Bytes(), sigdata.Signature.S.Bytes()
	sig := make([]byte, 65)
	copy(sig[32-len(rbytes):32], rbytes)
	copy(sig[64-len(sbytes):64], sbytes)

	if err := confirmPublicKey(sig, sigdata.PublicKey); err != nil {
		return accounts.Account{}, err
	}
	pub, err := crypto.UnmarshalPubkey(sigdata.PublicKey)
	if err != nil {
		return accounts.Account{}, err
	}
	return s.Wallet.makeAccount(crypto.PubkeyToAddress(*pub), path), nil
}

// keyExport contains information on an exported keypair.
//
//lint:ignore U1000 needs to be added to the console interface
type keyExport struct {
	PublicKey  []byte `asn1:"tag:0"`
	PrivateKey []byte `asn1:"tag:1,optional"`
}

// publicKey returns the public key for the current derivation path.
//
//lint:ignore U1000 needs to be added to the console interface
func (s *Session) publicKey() ([]byte, error) {
	response, err := s.Channel.transmitEncrypted(claSCWallet, insExportKey, exportP1Any, exportP2Pubkey, nil)
	if err != nil {
		return nil, err
	}
	keys := new(keyExport)
	if _, err := asn1.UnmarshalWithParams(response.Data, keys, "tag:1"); err != nil {
		return nil, err
	}
	return keys.PublicKey, nil
}

// signatureData contains information on a signature - the signature itself and
// the corresponding public key.
type signatureData struct {
	PublicKey []byte `asn1:"tag:0"`
	Signature struct {
		R *big.Int
		S *big.Int
	}
}

// sign asks the card to sign a message, and returns a valid signature after
// recovering the v value.
func (s *Session) sign(path accounts.DerivationPath, hash []byte) ([]byte, error) {
	startTime := time.Now()
	_, err := s.derive(path)
	if err != nil {
		return nil, err
	}
	deriveTime := time.Now()

	response, err := s.Channel.transmitEncrypted(claSCWallet, insSign, signP1PrecomputedHash, signP2OnlyBlock, hash)
	if err != nil {
		return nil, err
	}
	sigdata := new(signatureData)
	if _, err := asn1.UnmarshalWithParams(response.Data, sigdata, "tag:0"); err != nil {
		return nil, err
	}
	// Serialize the signature
	rbytes, sbytes := sigdata.Signature.R.Bytes(), sigdata.Signature.S.Bytes()
	sig := make([]byte, 65)
	copy(sig[32-len(rbytes):32], rbytes)
	copy(sig[64-len(sbytes):64], sbytes)

	// Recover the V value.
	sig, err = makeRecoverableSignature(hash, sig, sigdata.PublicKey)
	if err != nil {
		return nil, err
	}
	log.Debug("Signed using smartcard", "deriveTime", deriveTime.Sub(startTime), "signingTime", time.Since(deriveTime))

	return sig, nil
}

// confirmPublicKey confirms that the given signature belongs to the specified key.
func confirmPublicKey(sig, pubkey []byte) error {
	_, err := makeRecoverableSignature(DerivationSignatureHash[:], sig, pubkey)
	return err
}

// makeRecoverableSignature uses a signature and an expected public key to
// recover the v value and produce a recoverable signature.
func makeRecoverableSignature(hash, sig, expectedPubkey []byte) ([]byte, error) {
	var libraryError error
	for v := 0; v < 2; v++ {
		sig[64] = byte(v)
		if pubkey, err := crypto.Ecrecover(hash, sig); err == nil {
			if bytes.Equal(pubkey, expectedPubkey) {
				return sig, nil
			}
		} else {
			libraryError = err
		}
	}
	if libraryError != nil {
		return nil, libraryError
	}
	return nil, ErrPubkeyMismatch
}
