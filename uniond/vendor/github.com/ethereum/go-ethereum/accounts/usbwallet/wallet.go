// Copyright 2017 The go-ethereum Authors
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

// Package usbwallet implements support for USB hardware wallets.
package usbwallet

import (
	"context"
	"fmt"
	"io"
	"math/big"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/accounts"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/log"
	"github.com/karalabe/usb"
)

// Maximum time between wallet health checks to detect USB unplugs.
const heartbeatCycle = time.Second

// Minimum time to wait between self derivation attempts, even it the user is
// requesting accounts like crazy.
const selfDeriveThrottling = time.Second

// driver defines the vendor specific functionality hardware wallets instances
// must implement to allow using them with the wallet lifecycle management.
type driver interface {
	// Status returns a textual status to aid the user in the current state of the
	// wallet. It also returns an error indicating any failure the wallet might have
	// encountered.
	Status() (string, error)

	// Open initializes access to a wallet instance. The passphrase parameter may
	// or may not be used by the implementation of a particular wallet instance.
	Open(device io.ReadWriter, passphrase string) error

	// Close releases any resources held by an open wallet instance.
	Close() error

	// Heartbeat performs a sanity check against the hardware wallet to see if it
	// is still online and healthy.
	Heartbeat() error

	// Derive sends a derivation request to the USB device and returns the Ethereum
	// address located on that path.
	Derive(path accounts.DerivationPath) (common.Address, error)

	// SignTx sends the transaction to the USB device and waits for the user to confirm
	// or deny the transaction.
	SignTx(path accounts.DerivationPath, tx *types.Transaction, chainID *big.Int) (common.Address, *types.Transaction, error)

	SignTypedMessage(path accounts.DerivationPath, messageHash []byte, domainHash []byte) ([]byte, error)
}

// wallet represents the common functionality shared by all USB hardware
// wallets to prevent reimplementing the same complex maintenance mechanisms
// for different vendors.
type wallet struct {
	hub    *Hub          // USB hub scanning
	driver driver        // Hardware implementation of the low level device operations
	url    *accounts.URL // Textual URL uniquely identifying this wallet

	info   usb.DeviceInfo // Known USB device infos about the wallet
	device usb.Device     // USB device advertising itself as a hardware wallet

	accounts []accounts.Account                         // List of derive accounts pinned on the hardware wallet
	paths    map[common.Address]accounts.DerivationPath // Known derivation paths for signing operations

	deriveNextPaths []accounts.DerivationPath // Next derivation paths for account auto-discovery (multiple bases supported)
	deriveNextAddrs []common.Address          // Next derived account addresses for auto-discovery (multiple bases supported)
	deriveChain     ethereum.ChainStateReader // Blockchain state reader to discover used account with
	deriveReq       chan chan struct{}        // Channel to request a self-derivation on
	deriveQuit      chan chan error           // Channel to terminate the self-deriver with

	healthQuit chan chan error

	// Locking a hardware wallet is a bit special. Since hardware devices are lower
	// performing, any communication with them might take a non negligible amount of
	// time. Worse still, waiting for user confirmation can take arbitrarily long,
	// but exclusive communication must be upheld during. Locking the entire wallet
	// in the mean time however would stall any parts of the system that don't want
	// to communicate, just read some state (e.g. list the accounts).
	//
	// As such, a hardware wallet needs two locks to function correctly. A state
	// lock can be used to protect the wallet's software-side internal state, which
	// must not be held exclusively during hardware communication. A communication
	// lock can be used to achieve exclusive access to the device itself, this one
	// however should allow "skipping" waiting for operations that might want to
	// use the device, but can live without too (e.g. account self-derivation).
	//
	// Since we have two locks, it's important to know how to properly use them:
	//   - Communication requires the `device` to not change, so obtaining the
	//     commsLock should be done after having a stateLock.
	//   - Communication must not disable read access to the wallet state, so it
	//     must only ever hold a *read* lock to stateLock.
	commsLock chan struct{} // Mutex (buf=1) for the USB comms without keeping the state locked
	stateLock sync.RWMutex  // Protects read and write access to the wallet struct fields

	log log.Logger // Contextual logger to tag the base with its id
}

// URL implements accounts.Wallet, returning the URL of the USB hardware device.
func (w *wallet) URL() accounts.URL {
	return *w.url // Immutable, no need for a lock
}

// Status implements accounts.Wallet, returning a custom status message from the
// underlying vendor-specific hardware wallet implementation.
func (w *wallet) Status() (string, error) {
	w.stateLock.RLock() // No device communication, state lock is enough
	defer w.stateLock.RUnlock()

	status, failure := w.driver.Status()
	if w.device == nil {
		return "Closed", failure
	}
	return status, failure
}

// Open implements accounts.Wallet, attempting to open a USB connection to the
// hardware wallet.
func (w *wallet) Open(passphrase string) error {
	w.stateLock.Lock() // State lock is enough since there's no connection yet at this point
	defer w.stateLock.Unlock()

	// If the device was already opened once, refuse to try again
	if w.paths != nil {
		return accounts.ErrWalletAlreadyOpen
	}
	// Make sure the actual device connection is done only once
	if w.device == nil {
		device, err := w.info.Open()
		if err != nil {
			return err
		}
		w.device = device
		w.commsLock = make(chan struct{}, 1)
		w.commsLock <- struct{}{} // Enable lock
	}
	// Delegate device initialization to the underlying driver
	if err := w.driver.Open(w.device, passphrase); err != nil {
		return err
	}
	// Connection successful, start life-cycle management
	w.paths = make(map[common.Address]accounts.DerivationPath)

	w.deriveReq = make(chan chan struct{})
	w.deriveQuit = make(chan chan error)
	w.healthQuit = make(chan chan error)

	go w.heartbeat()
	go w.selfDerive()

	// Notify anyone listening for wallet events that a new device is accessible
	go w.hub.updateFeed.Send(accounts.WalletEvent{Wallet: w, Kind: accounts.WalletOpened})

	return nil
}

// heartbeat is a health check loop for the USB wallets to periodically verify
// whether they are still present or if they malfunctioned.
func (w *wallet) heartbeat() {
	w.log.Debug("USB wallet health-check started")
	defer w.log.Debug("USB wallet health-check stopped")

	// Execute heartbeat checks until termination or error
	var (
		errc chan error
		err  error
	)
	for errc == nil && err == nil {
		// Wait until termination is requested or the heartbeat cycle arrives
		select {
		case errc = <-w.healthQuit:
			// Termination requested
			continue
		case <-time.After(heartbeatCycle):
			// Heartbeat time
		}
		// Execute a tiny data exchange to see responsiveness
		w.stateLock.RLock()
		if w.device == nil {
			// Terminated while waiting for the lock
			w.stateLock.RUnlock()
			continue
		}
		<-w.commsLock // Don't lock state while resolving version
		err = w.driver.Heartbeat()
		w.commsLock <- struct{}{}
		w.stateLock.RUnlock()

		if err != nil {
			w.stateLock.Lock() // Lock state to tear the wallet down
			w.close()
			w.stateLock.Unlock()
		}
		// Ignore non hardware related errors
		err = nil
	}
	// In case of error, wait for termination
	if err != nil {
		w.log.Debug("USB wallet health-check failed", "err", err)
		errc = <-w.healthQuit
	}
	errc <- err
}

// Close implements accounts.Wallet, closing the USB connection to the device.
func (w *wallet) Close() error {
	// Ensure the wallet was opened
	w.stateLock.RLock()
	hQuit, dQuit := w.healthQuit, w.deriveQuit
	w.stateLock.RUnlock()

	// Terminate the health checks
	var herr error
	if hQuit != nil {
		errc := make(chan error)
		hQuit <- errc
		herr = <-errc // Save for later, we *must* close the USB
	}
	// Terminate the self-derivations
	var derr error
	if dQuit != nil {
		errc := make(chan error)
		dQuit <- errc
		derr = <-errc // Save for later, we *must* close the USB
	}
	// Terminate the device connection
	w.stateLock.Lock()
	defer w.stateLock.Unlock()

	w.healthQuit = nil
	w.deriveQuit = nil
	w.deriveReq = nil

	if err := w.close(); err != nil {
		return err
	}
	if herr != nil {
		return herr
	}
	return derr
}

// close is the internal wallet closer that terminates the USB connection and
// resets all the fields to their defaults.
//
// Note, close assumes the state lock is held!
func (w *wallet) close() error {
	// Allow duplicate closes, especially for health-check failures
	if w.device == nil {
		return nil
	}
	// Close the device, clear everything, then return
	w.device.Close()
	w.device = nil

	w.accounts, w.paths = nil, nil
	return w.driver.Close()
}

// Accounts implements accounts.Wallet, returning the list of accounts pinned to
// the USB hardware wallet. If self-derivation was enabled, the account list is
// periodically expanded based on current chain state.
func (w *wallet) Accounts() []accounts.Account {
	// Attempt self-derivation if it's running
	reqc := make(chan struct{}, 1)
	select {
	case w.deriveReq <- reqc:
		// Self-derivation request accepted, wait for it
		<-reqc
	default:
		// Self-derivation offline, throttled or busy, skip
	}
	// Return whatever account list we ended up with
	w.stateLock.RLock()
	defer w.stateLock.RUnlock()

	cpy := make([]accounts.Account, len(w.accounts))
	copy(cpy, w.accounts)
	return cpy
}

// selfDerive is an account derivation loop that upon request attempts to find
// new non-zero accounts.
func (w *wallet) selfDerive() {
	w.log.Debug("USB wallet self-derivation started")
	defer w.log.Debug("USB wallet self-derivation stopped")

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
		w.stateLock.RLock()
		if w.device == nil || w.deriveChain == nil {
			w.stateLock.RUnlock()
			reqc <- struct{}{}
			continue
		}
		select {
		case <-w.commsLock:
		default:
			w.stateLock.RUnlock()
			reqc <- struct{}{}
			continue
		}
		// Device lock obtained, derive the next batch of accounts
		var (
			accs  []accounts.Account
			paths []accounts.DerivationPath

			nextPaths = append([]accounts.DerivationPath{}, w.deriveNextPaths...)
			nextAddrs = append([]common.Address{}, w.deriveNextAddrs...)

			context = context.Background()
		)
		for i := 0; i < len(nextAddrs); i++ {
			for empty := false; !empty; {
				// Retrieve the next derived Ethereum account
				if nextAddrs[i] == (common.Address{}) {
					if nextAddrs[i], err = w.driver.Derive(nextPaths[i]); err != nil {
						w.log.Warn("USB wallet account derivation failed", "err", err)
						break
					}
				}
				// Check the account's status against the current chain state
				var (
					balance *big.Int
					nonce   uint64
				)
				balance, err = w.deriveChain.BalanceAt(context, nextAddrs[i], nil)
				if err != nil {
					w.log.Warn("USB wallet balance retrieval failed", "err", err)
					break
				}
				nonce, err = w.deriveChain.NonceAt(context, nextAddrs[i], nil)
				if err != nil {
					w.log.Warn("USB wallet nonce retrieval failed", "err", err)
					break
				}
				// We've just self-derived a new account, start tracking it locally
				// unless the account was empty.
				path := make(accounts.DerivationPath, len(nextPaths[i]))
				copy(path[:], nextPaths[i][:])
				if balance.Sign() == 0 && nonce == 0 {
					empty = true
					// If it indeed was empty, make a log output for it anyway. In the case
					// of legacy-ledger, the first account on the legacy-path will
					// be shown to the user, even if we don't actively track it
					if i < len(nextAddrs)-1 {
						w.log.Info("Skipping tracking first account on legacy path, use personal.deriveAccount(<url>,<path>, false) to track",
							"path", path, "address", nextAddrs[i])
						break
					}
				}
				paths = append(paths, path)
				account := accounts.Account{
					Address: nextAddrs[i],
					URL:     accounts.URL{Scheme: w.url.Scheme, Path: fmt.Sprintf("%s/%s", w.url.Path, path)},
				}
				accs = append(accs, account)

				// Display a log message to the user for new (or previously empty accounts)
				if _, known := w.paths[nextAddrs[i]]; !known || (!empty && nextAddrs[i] == w.deriveNextAddrs[i]) {
					w.log.Info("USB wallet discovered new account", "address", nextAddrs[i], "path", path, "balance", balance, "nonce", nonce)
				}
				// Fetch the next potential account
				if !empty {
					nextAddrs[i] = common.Address{}
					nextPaths[i][len(nextPaths[i])-1]++
				}
			}
		}
		// Self derivation complete, release device lock
		w.commsLock <- struct{}{}
		w.stateLock.RUnlock()

		// Insert any accounts successfully derived
		w.stateLock.Lock()
		for i := 0; i < len(accs); i++ {
			if _, ok := w.paths[accs[i].Address]; !ok {
				w.accounts = append(w.accounts, accs[i])
				w.paths[accs[i].Address] = paths[i]
			}
		}
		// Shift the self-derivation forward
		// TODO(karalabe): don't overwrite changes from wallet.SelfDerive
		w.deriveNextAddrs = nextAddrs
		w.deriveNextPaths = nextPaths
		w.stateLock.Unlock()

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
		w.log.Debug("USB wallet self-derivation failed", "err", err)
		errc = <-w.deriveQuit
	}
	errc <- err
}

// Contains implements accounts.Wallet, returning whether a particular account is
// or is not pinned into this wallet instance. Although we could attempt to resolve
// unpinned accounts, that would be an non-negligible hardware operation.
func (w *wallet) Contains(account accounts.Account) bool {
	w.stateLock.RLock()
	defer w.stateLock.RUnlock()

	_, exists := w.paths[account.Address]
	return exists
}

// Derive implements accounts.Wallet, deriving a new account at the specific
// derivation path. If pin is set to true, the account will be added to the list
// of tracked accounts.
func (w *wallet) Derive(path accounts.DerivationPath, pin bool) (accounts.Account, error) {
	// Try to derive the actual account and update its URL if successful
	w.stateLock.RLock() // Avoid device disappearing during derivation

	if w.device == nil {
		w.stateLock.RUnlock()
		return accounts.Account{}, accounts.ErrWalletClosed
	}
	<-w.commsLock // Avoid concurrent hardware access
	address, err := w.driver.Derive(path)
	w.commsLock <- struct{}{}

	w.stateLock.RUnlock()

	// If an error occurred or no pinning was requested, return
	if err != nil {
		return accounts.Account{}, err
	}
	account := accounts.Account{
		Address: address,
		URL:     accounts.URL{Scheme: w.url.Scheme, Path: fmt.Sprintf("%s/%s", w.url.Path, path)},
	}
	if !pin {
		return account, nil
	}
	// Pinning needs to modify the state
	w.stateLock.Lock()
	defer w.stateLock.Unlock()

	if _, ok := w.paths[address]; !ok {
		w.accounts = append(w.accounts, account)
		w.paths[address] = make(accounts.DerivationPath, len(path))
		copy(w.paths[address], path)
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
func (w *wallet) SelfDerive(bases []accounts.DerivationPath, chain ethereum.ChainStateReader) {
	w.stateLock.Lock()
	defer w.stateLock.Unlock()

	w.deriveNextPaths = make([]accounts.DerivationPath, len(bases))
	for i, base := range bases {
		w.deriveNextPaths[i] = make(accounts.DerivationPath, len(base))
		copy(w.deriveNextPaths[i][:], base[:])
	}
	w.deriveNextAddrs = make([]common.Address, len(bases))
	w.deriveChain = chain
}

// signHash implements accounts.Wallet, however signing arbitrary data is not
// supported for hardware wallets, so this method will always return an error.
func (w *wallet) signHash(account accounts.Account, hash []byte) ([]byte, error) {
	return nil, accounts.ErrNotSupported
}

// SignData signs keccak256(data). The mimetype parameter describes the type of data being signed
func (w *wallet) SignData(account accounts.Account, mimeType string, data []byte) ([]byte, error) {
	// Unless we are doing 712 signing, simply dispatch to signHash
	if !(mimeType == accounts.MimetypeTypedData && len(data) == 66 && data[0] == 0x19 && data[1] == 0x01) {
		return w.signHash(account, crypto.Keccak256(data))
	}

	// dispatch to 712 signing if the mimetype is TypedData and the format matches
	w.stateLock.RLock() // Comms have own mutex, this is for the state fields
	defer w.stateLock.RUnlock()

	// If the wallet is closed, abort
	if w.device == nil {
		return nil, accounts.ErrWalletClosed
	}
	// Make sure the requested account is contained within
	path, ok := w.paths[account.Address]
	if !ok {
		return nil, accounts.ErrUnknownAccount
	}
	// All infos gathered and metadata checks out, request signing
	<-w.commsLock
	defer func() { w.commsLock <- struct{}{} }()

	// Ensure the device isn't screwed with while user confirmation is pending
	// TODO(karalabe): remove if hotplug lands on Windows
	w.hub.commsLock.Lock()
	w.hub.commsPend++
	w.hub.commsLock.Unlock()

	defer func() {
		w.hub.commsLock.Lock()
		w.hub.commsPend--
		w.hub.commsLock.Unlock()
	}()
	// Sign the transaction
	signature, err := w.driver.SignTypedMessage(path, data[2:34], data[34:66])
	if err != nil {
		return nil, err
	}
	return signature, nil
}

// SignDataWithPassphrase implements accounts.Wallet, attempting to sign the given
// data with the given account using passphrase as extra authentication.
// Since USB wallets don't rely on passphrases, these are silently ignored.
func (w *wallet) SignDataWithPassphrase(account accounts.Account, passphrase, mimeType string, data []byte) ([]byte, error) {
	return w.SignData(account, mimeType, data)
}

func (w *wallet) SignText(account accounts.Account, text []byte) ([]byte, error) {
	return w.signHash(account, accounts.TextHash(text))
}

// SignTx implements accounts.Wallet. It sends the transaction over to the Ledger
// wallet to request a confirmation from the user. It returns either the signed
// transaction or a failure if the user denied the transaction.
//
// Note, if the version of the Ethereum application running on the Ledger wallet is
// too old to sign EIP-155 transactions, but such is requested nonetheless, an error
// will be returned opposed to silently signing in Homestead mode.
func (w *wallet) SignTx(account accounts.Account, tx *types.Transaction, chainID *big.Int) (*types.Transaction, error) {
	w.stateLock.RLock() // Comms have own mutex, this is for the state fields
	defer w.stateLock.RUnlock()

	// If the wallet is closed, abort
	if w.device == nil {
		return nil, accounts.ErrWalletClosed
	}
	// Make sure the requested account is contained within
	path, ok := w.paths[account.Address]
	if !ok {
		return nil, accounts.ErrUnknownAccount
	}
	// All infos gathered and metadata checks out, request signing
	<-w.commsLock
	defer func() { w.commsLock <- struct{}{} }()

	// Ensure the device isn't screwed with while user confirmation is pending
	// TODO(karalabe): remove if hotplug lands on Windows
	w.hub.commsLock.Lock()
	w.hub.commsPend++
	w.hub.commsLock.Unlock()

	defer func() {
		w.hub.commsLock.Lock()
		w.hub.commsPend--
		w.hub.commsLock.Unlock()
	}()
	// Sign the transaction and verify the sender to avoid hardware fault surprises
	sender, signed, err := w.driver.SignTx(path, tx, chainID)
	if err != nil {
		return nil, err
	}
	if sender != account.Address {
		return nil, fmt.Errorf("signer mismatch: expected %s, got %s", account.Address.Hex(), sender.Hex())
	}
	return signed, nil
}

// SignTextWithPassphrase implements accounts.Wallet, however signing arbitrary
// data is not supported for Ledger wallets, so this method will always return
// an error.
func (w *wallet) SignTextWithPassphrase(account accounts.Account, passphrase string, text []byte) ([]byte, error) {
	return w.SignText(account, accounts.TextHash(text))
}

// SignTxWithPassphrase implements accounts.Wallet, attempting to sign the given
// transaction with the given account using passphrase as extra authentication.
// Since USB wallets don't rely on passphrases, these are silently ignored.
func (w *wallet) SignTxWithPassphrase(account accounts.Account, passphrase string, tx *types.Transaction, chainID *big.Int) (*types.Transaction, error) {
	return w.SignTx(account, tx, chainID)
}
