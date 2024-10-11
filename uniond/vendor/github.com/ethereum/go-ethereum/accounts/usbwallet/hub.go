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

package usbwallet

import (
	"errors"
	"runtime"
	"sync"
	"sync/atomic"
	"time"

	"github.com/ethereum/go-ethereum/accounts"
	"github.com/ethereum/go-ethereum/event"
	"github.com/ethereum/go-ethereum/log"
	"github.com/karalabe/usb"
)

// LedgerScheme is the protocol scheme prefixing account and wallet URLs.
const LedgerScheme = "ledger"

// TrezorScheme is the protocol scheme prefixing account and wallet URLs.
const TrezorScheme = "trezor"

// refreshCycle is the maximum time between wallet refreshes (if USB hotplug
// notifications don't work).
const refreshCycle = time.Second

// refreshThrottling is the minimum time between wallet refreshes to avoid USB
// trashing.
const refreshThrottling = 500 * time.Millisecond

// Hub is a accounts.Backend that can find and handle generic USB hardware wallets.
type Hub struct {
	scheme     string                  // Protocol scheme prefixing account and wallet URLs.
	vendorID   uint16                  // USB vendor identifier used for device discovery
	productIDs []uint16                // USB product identifiers used for device discovery
	usageID    uint16                  // USB usage page identifier used for macOS device discovery
	endpointID int                     // USB endpoint identifier used for non-macOS device discovery
	makeDriver func(log.Logger) driver // Factory method to construct a vendor specific driver

	refreshed   time.Time               // Time instance when the list of wallets was last refreshed
	wallets     []accounts.Wallet       // List of USB wallet devices currently tracking
	updateFeed  event.Feed              // Event feed to notify wallet additions/removals
	updateScope event.SubscriptionScope // Subscription scope tracking current live listeners
	updating    bool                    // Whether the event notification loop is running

	quit chan chan error

	stateLock sync.RWMutex // Protects the internals of the hub from racey access

	// TODO(karalabe): remove if hotplug lands on Windows
	commsPend int           // Number of operations blocking enumeration
	commsLock sync.Mutex    // Lock protecting the pending counter and enumeration
	enumFails atomic.Uint32 // Number of times enumeration has failed
}

// NewLedgerHub creates a new hardware wallet manager for Ledger devices.
func NewLedgerHub() (*Hub, error) {
	return newHub(LedgerScheme, 0x2c97, []uint16{

		// Device definitions taken from
		// https://github.com/LedgerHQ/ledger-live/blob/38012bc8899e0f07149ea9cfe7e64b2c146bc92b/libs/ledgerjs/packages/devices/src/index.ts

		// Original product IDs
		0x0000, /* Ledger Blue */
		0x0001, /* Ledger Nano S */
		0x0004, /* Ledger Nano X */
		0x0005, /* Ledger Nano S Plus */
		0x0006, /* Ledger Nano FTS */

		0x0015, /* HID + U2F + WebUSB Ledger Blue */
		0x1015, /* HID + U2F + WebUSB Ledger Nano S */
		0x4015, /* HID + U2F + WebUSB Ledger Nano X */
		0x5015, /* HID + U2F + WebUSB Ledger Nano S Plus */
		0x6015, /* HID + U2F + WebUSB Ledger Nano FTS */

		0x0011, /* HID + WebUSB Ledger Blue */
		0x1011, /* HID + WebUSB Ledger Nano S */
		0x4011, /* HID + WebUSB Ledger Nano X */
		0x5011, /* HID + WebUSB Ledger Nano S Plus */
		0x6011, /* HID + WebUSB Ledger Nano FTS */
	}, 0xffa0, 0, newLedgerDriver)
}

// NewTrezorHubWithHID creates a new hardware wallet manager for Trezor devices.
func NewTrezorHubWithHID() (*Hub, error) {
	return newHub(TrezorScheme, 0x534c, []uint16{0x0001 /* Trezor HID */}, 0xff00, 0, newTrezorDriver)
}

// NewTrezorHubWithWebUSB creates a new hardware wallet manager for Trezor devices with
// firmware version > 1.8.0
func NewTrezorHubWithWebUSB() (*Hub, error) {
	return newHub(TrezorScheme, 0x1209, []uint16{0x53c1 /* Trezor WebUSB */}, 0xffff /* No usage id on webusb, don't match unset (0) */, 0, newTrezorDriver)
}

// newHub creates a new hardware wallet manager for generic USB devices.
func newHub(scheme string, vendorID uint16, productIDs []uint16, usageID uint16, endpointID int, makeDriver func(log.Logger) driver) (*Hub, error) {
	if !usb.Supported() {
		return nil, errors.New("unsupported platform")
	}
	hub := &Hub{
		scheme:     scheme,
		vendorID:   vendorID,
		productIDs: productIDs,
		usageID:    usageID,
		endpointID: endpointID,
		makeDriver: makeDriver,
		quit:       make(chan chan error),
	}
	hub.refreshWallets()
	return hub, nil
}

// Wallets implements accounts.Backend, returning all the currently tracked USB
// devices that appear to be hardware wallets.
func (hub *Hub) Wallets() []accounts.Wallet {
	// Make sure the list of wallets is up to date
	hub.refreshWallets()

	hub.stateLock.RLock()
	defer hub.stateLock.RUnlock()

	cpy := make([]accounts.Wallet, len(hub.wallets))
	copy(cpy, hub.wallets)
	return cpy
}

// refreshWallets scans the USB devices attached to the machine and updates the
// list of wallets based on the found devices.
func (hub *Hub) refreshWallets() {
	// Don't scan the USB like crazy it the user fetches wallets in a loop
	hub.stateLock.RLock()
	elapsed := time.Since(hub.refreshed)
	hub.stateLock.RUnlock()

	if elapsed < refreshThrottling {
		return
	}
	// If USB enumeration is continually failing, don't keep trying indefinitely
	if hub.enumFails.Load() > 2 {
		return
	}
	// Retrieve the current list of USB wallet devices
	var devices []usb.DeviceInfo

	if runtime.GOOS == "linux" {
		// hidapi on Linux opens the device during enumeration to retrieve some infos,
		// breaking the Ledger protocol if that is waiting for user confirmation. This
		// is a bug acknowledged at Ledger, but it won't be fixed on old devices so we
		// need to prevent concurrent comms ourselves. The more elegant solution would
		// be to ditch enumeration in favor of hotplug events, but that don't work yet
		// on Windows so if we need to hack it anyway, this is more elegant for now.
		hub.commsLock.Lock()
		if hub.commsPend > 0 { // A confirmation is pending, don't refresh
			hub.commsLock.Unlock()
			return
		}
	}
	infos, err := usb.Enumerate(hub.vendorID, 0)
	if err != nil {
		failcount := hub.enumFails.Add(1)
		if runtime.GOOS == "linux" {
			// See rationale before the enumeration why this is needed and only on Linux.
			hub.commsLock.Unlock()
		}
		log.Error("Failed to enumerate USB devices", "hub", hub.scheme,
			"vendor", hub.vendorID, "failcount", failcount, "err", err)
		return
	}
	hub.enumFails.Store(0)

	for _, info := range infos {
		for _, id := range hub.productIDs {
			// Windows and Macos use UsageID matching, Linux uses Interface matching
			if info.ProductID == id && (info.UsagePage == hub.usageID || info.Interface == hub.endpointID) {
				devices = append(devices, info)
				break
			}
		}
	}
	if runtime.GOOS == "linux" {
		// See rationale before the enumeration why this is needed and only on Linux.
		hub.commsLock.Unlock()
	}
	// Transform the current list of wallets into the new one
	hub.stateLock.Lock()

	var (
		wallets = make([]accounts.Wallet, 0, len(devices))
		events  []accounts.WalletEvent
	)

	for _, device := range devices {
		url := accounts.URL{Scheme: hub.scheme, Path: device.Path}

		// Drop wallets in front of the next device or those that failed for some reason
		for len(hub.wallets) > 0 {
			// Abort if we're past the current device and found an operational one
			_, failure := hub.wallets[0].Status()
			if hub.wallets[0].URL().Cmp(url) >= 0 || failure == nil {
				break
			}
			// Drop the stale and failed devices
			events = append(events, accounts.WalletEvent{Wallet: hub.wallets[0], Kind: accounts.WalletDropped})
			hub.wallets = hub.wallets[1:]
		}
		// If there are no more wallets or the device is before the next, wrap new wallet
		if len(hub.wallets) == 0 || hub.wallets[0].URL().Cmp(url) > 0 {
			logger := log.New("url", url)
			wallet := &wallet{hub: hub, driver: hub.makeDriver(logger), url: &url, info: device, log: logger}

			events = append(events, accounts.WalletEvent{Wallet: wallet, Kind: accounts.WalletArrived})
			wallets = append(wallets, wallet)
			continue
		}
		// If the device is the same as the first wallet, keep it
		if hub.wallets[0].URL().Cmp(url) == 0 {
			wallets = append(wallets, hub.wallets[0])
			hub.wallets = hub.wallets[1:]
			continue
		}
	}
	// Drop any leftover wallets and set the new batch
	for _, wallet := range hub.wallets {
		events = append(events, accounts.WalletEvent{Wallet: wallet, Kind: accounts.WalletDropped})
	}
	hub.refreshed = time.Now()
	hub.wallets = wallets
	hub.stateLock.Unlock()

	// Fire all wallet events and return
	for _, event := range events {
		hub.updateFeed.Send(event)
	}
}

// Subscribe implements accounts.Backend, creating an async subscription to
// receive notifications on the addition or removal of USB wallets.
func (hub *Hub) Subscribe(sink chan<- accounts.WalletEvent) event.Subscription {
	// We need the mutex to reliably start/stop the update loop
	hub.stateLock.Lock()
	defer hub.stateLock.Unlock()

	// Subscribe the caller and track the subscriber count
	sub := hub.updateScope.Track(hub.updateFeed.Subscribe(sink))

	// Subscribers require an active notification loop, start it
	if !hub.updating {
		hub.updating = true
		go hub.updater()
	}
	return sub
}

// updater is responsible for maintaining an up-to-date list of wallets managed
// by the USB hub, and for firing wallet addition/removal events.
func (hub *Hub) updater() {
	for {
		// TODO: Wait for a USB hotplug event (not supported yet) or a refresh timeout
		// <-hub.changes
		time.Sleep(refreshCycle)

		// Run the wallet refresher
		hub.refreshWallets()

		// If all our subscribers left, stop the updater
		hub.stateLock.Lock()
		if hub.updateScope.Count() == 0 {
			hub.updating = false
			hub.stateLock.Unlock()
			return
		}
		hub.stateLock.Unlock()
	}
}
