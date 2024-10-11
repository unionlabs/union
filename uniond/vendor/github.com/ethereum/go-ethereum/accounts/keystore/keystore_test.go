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

package keystore

import (
	"math/rand"
	"os"
	"runtime"
	"strings"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/accounts"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/event"
	"golang.org/x/exp/slices"
)

var testSigData = make([]byte, 32)

func TestKeyStore(t *testing.T) {
	dir, ks := tmpKeyStore(t, true)

	a, err := ks.NewAccount("foo")
	if err != nil {
		t.Fatal(err)
	}
	if !strings.HasPrefix(a.URL.Path, dir) {
		t.Errorf("account file %s doesn't have dir prefix", a.URL)
	}
	stat, err := os.Stat(a.URL.Path)
	if err != nil {
		t.Fatalf("account file %s doesn't exist (%v)", a.URL, err)
	}
	if runtime.GOOS != "windows" && stat.Mode() != 0600 {
		t.Fatalf("account file has wrong mode: got %o, want %o", stat.Mode(), 0600)
	}
	if !ks.HasAddress(a.Address) {
		t.Errorf("HasAccount(%x) should've returned true", a.Address)
	}
	if err := ks.Update(a, "foo", "bar"); err != nil {
		t.Errorf("Update error: %v", err)
	}
	if err := ks.Delete(a, "bar"); err != nil {
		t.Errorf("Delete error: %v", err)
	}
	if common.FileExist(a.URL.Path) {
		t.Errorf("account file %s should be gone after Delete", a.URL)
	}
	if ks.HasAddress(a.Address) {
		t.Errorf("HasAccount(%x) should've returned true after Delete", a.Address)
	}
}

func TestSign(t *testing.T) {
	_, ks := tmpKeyStore(t, true)

	pass := "" // not used but required by API
	a1, err := ks.NewAccount(pass)
	if err != nil {
		t.Fatal(err)
	}
	if err := ks.Unlock(a1, ""); err != nil {
		t.Fatal(err)
	}
	if _, err := ks.SignHash(accounts.Account{Address: a1.Address}, testSigData); err != nil {
		t.Fatal(err)
	}
}

func TestSignWithPassphrase(t *testing.T) {
	_, ks := tmpKeyStore(t, true)

	pass := "passwd"
	acc, err := ks.NewAccount(pass)
	if err != nil {
		t.Fatal(err)
	}

	if _, unlocked := ks.unlocked[acc.Address]; unlocked {
		t.Fatal("expected account to be locked")
	}

	_, err = ks.SignHashWithPassphrase(acc, pass, testSigData)
	if err != nil {
		t.Fatal(err)
	}

	if _, unlocked := ks.unlocked[acc.Address]; unlocked {
		t.Fatal("expected account to be locked")
	}

	if _, err = ks.SignHashWithPassphrase(acc, "invalid passwd", testSigData); err == nil {
		t.Fatal("expected SignHashWithPassphrase to fail with invalid password")
	}
}

func TestTimedUnlock(t *testing.T) {
	t.Parallel()
	_, ks := tmpKeyStore(t, true)

	pass := "foo"
	a1, err := ks.NewAccount(pass)
	if err != nil {
		t.Fatal(err)
	}

	// Signing without passphrase fails because account is locked
	_, err = ks.SignHash(accounts.Account{Address: a1.Address}, testSigData)
	if err != ErrLocked {
		t.Fatal("Signing should've failed with ErrLocked before unlocking, got ", err)
	}

	// Signing with passphrase works
	if err = ks.TimedUnlock(a1, pass, 100*time.Millisecond); err != nil {
		t.Fatal(err)
	}

	// Signing without passphrase works because account is temp unlocked
	_, err = ks.SignHash(accounts.Account{Address: a1.Address}, testSigData)
	if err != nil {
		t.Fatal("Signing shouldn't return an error after unlocking, got ", err)
	}

	// Signing fails again after automatic locking
	time.Sleep(250 * time.Millisecond)
	_, err = ks.SignHash(accounts.Account{Address: a1.Address}, testSigData)
	if err != ErrLocked {
		t.Fatal("Signing should've failed with ErrLocked timeout expired, got ", err)
	}
}

func TestOverrideUnlock(t *testing.T) {
	t.Parallel()
	_, ks := tmpKeyStore(t, false)

	pass := "foo"
	a1, err := ks.NewAccount(pass)
	if err != nil {
		t.Fatal(err)
	}

	// Unlock indefinitely.
	if err = ks.TimedUnlock(a1, pass, 5*time.Minute); err != nil {
		t.Fatal(err)
	}

	// Signing without passphrase works because account is temp unlocked
	_, err = ks.SignHash(accounts.Account{Address: a1.Address}, testSigData)
	if err != nil {
		t.Fatal("Signing shouldn't return an error after unlocking, got ", err)
	}

	// reset unlock to a shorter period, invalidates the previous unlock
	if err = ks.TimedUnlock(a1, pass, 100*time.Millisecond); err != nil {
		t.Fatal(err)
	}

	// Signing without passphrase still works because account is temp unlocked
	_, err = ks.SignHash(accounts.Account{Address: a1.Address}, testSigData)
	if err != nil {
		t.Fatal("Signing shouldn't return an error after unlocking, got ", err)
	}

	// Signing fails again after automatic locking
	time.Sleep(250 * time.Millisecond)
	_, err = ks.SignHash(accounts.Account{Address: a1.Address}, testSigData)
	if err != ErrLocked {
		t.Fatal("Signing should've failed with ErrLocked timeout expired, got ", err)
	}
}

// This test should fail under -race if signing races the expiration goroutine.
func TestSignRace(t *testing.T) {
	t.Parallel()
	_, ks := tmpKeyStore(t, false)

	// Create a test account.
	a1, err := ks.NewAccount("")
	if err != nil {
		t.Fatal("could not create the test account", err)
	}

	if err := ks.TimedUnlock(a1, "", 15*time.Millisecond); err != nil {
		t.Fatal("could not unlock the test account", err)
	}
	end := time.Now().Add(500 * time.Millisecond)
	for time.Now().Before(end) {
		if _, err := ks.SignHash(accounts.Account{Address: a1.Address}, testSigData); err == ErrLocked {
			return
		} else if err != nil {
			t.Errorf("Sign error: %v", err)
			return
		}
		time.Sleep(1 * time.Millisecond)
	}
	t.Errorf("Account did not lock within the timeout")
}

// waitForKsUpdating waits until the updating-status of the ks reaches the
// desired wantStatus.
// It waits for a maximum time of maxTime, and returns false if it does not
// finish in time
func waitForKsUpdating(t *testing.T, ks *KeyStore, wantStatus bool, maxTime time.Duration) bool {
	t.Helper()
	// Wait max 250 ms, then return false
	for t0 := time.Now(); time.Since(t0) < maxTime; {
		if ks.isUpdating() == wantStatus {
			return true
		}
		time.Sleep(25 * time.Millisecond)
	}
	return false
}

// Tests that the wallet notifier loop starts and stops correctly based on the
// addition and removal of wallet event subscriptions.
func TestWalletNotifierLifecycle(t *testing.T) {
	t.Parallel()
	// Create a temporary keystore to test with
	_, ks := tmpKeyStore(t, false)

	// Ensure that the notification updater is not running yet
	time.Sleep(250 * time.Millisecond)

	if ks.isUpdating() {
		t.Errorf("wallet notifier running without subscribers")
	}
	// Subscribe to the wallet feed and ensure the updater boots up
	updates := make(chan accounts.WalletEvent)

	subs := make([]event.Subscription, 2)
	for i := 0; i < len(subs); i++ {
		// Create a new subscription
		subs[i] = ks.Subscribe(updates)
		if !waitForKsUpdating(t, ks, true, 250*time.Millisecond) {
			t.Errorf("sub %d: wallet notifier not running after subscription", i)
		}
	}
	// Close all but one sub
	for i := 0; i < len(subs)-1; i++ {
		// Close an existing subscription
		subs[i].Unsubscribe()
	}
	// Check that it is still running
	time.Sleep(250 * time.Millisecond)

	if !ks.isUpdating() {
		t.Fatal("event notifier stopped prematurely")
	}
	// Unsubscribe the last one and ensure the updater terminates eventually.
	subs[len(subs)-1].Unsubscribe()
	if !waitForKsUpdating(t, ks, false, 4*time.Second) {
		t.Errorf("wallet notifier didn't terminate after unsubscribe")
	}
}

type walletEvent struct {
	accounts.WalletEvent
	a accounts.Account
}

// Tests that wallet notifications and correctly fired when accounts are added
// or deleted from the keystore.
func TestWalletNotifications(t *testing.T) {
	_, ks := tmpKeyStore(t, false)

	// Subscribe to the wallet feed and collect events.
	var (
		events  []walletEvent
		updates = make(chan accounts.WalletEvent)
		sub     = ks.Subscribe(updates)
	)
	defer sub.Unsubscribe()
	go func() {
		for {
			select {
			case ev := <-updates:
				events = append(events, walletEvent{ev, ev.Wallet.Accounts()[0]})
			case <-sub.Err():
				close(updates)
				return
			}
		}
	}()

	// Randomly add and remove accounts.
	var (
		live       = make(map[common.Address]accounts.Account)
		wantEvents []walletEvent
	)
	for i := 0; i < 1024; i++ {
		if create := len(live) == 0 || rand.Int()%4 > 0; create {
			// Add a new account and ensure wallet notifications arrives
			account, err := ks.NewAccount("")
			if err != nil {
				t.Fatalf("failed to create test account: %v", err)
			}
			live[account.Address] = account
			wantEvents = append(wantEvents, walletEvent{accounts.WalletEvent{Kind: accounts.WalletArrived}, account})
		} else {
			// Delete a random account.
			var account accounts.Account
			for _, a := range live {
				account = a
				break
			}
			if err := ks.Delete(account, ""); err != nil {
				t.Fatalf("failed to delete test account: %v", err)
			}
			delete(live, account.Address)
			wantEvents = append(wantEvents, walletEvent{accounts.WalletEvent{Kind: accounts.WalletDropped}, account})
		}
	}

	// Shut down the event collector and check events.
	sub.Unsubscribe()
	for ev := range updates {
		events = append(events, walletEvent{ev, ev.Wallet.Accounts()[0]})
	}
	checkAccounts(t, live, ks.Wallets())
	checkEvents(t, wantEvents, events)
}

// TestImportExport tests the import functionality of a keystore.
func TestImportECDSA(t *testing.T) {
	_, ks := tmpKeyStore(t, true)
	key, err := crypto.GenerateKey()
	if err != nil {
		t.Fatalf("failed to generate key: %v", key)
	}
	if _, err = ks.ImportECDSA(key, "old"); err != nil {
		t.Errorf("importing failed: %v", err)
	}
	if _, err = ks.ImportECDSA(key, "old"); err == nil {
		t.Errorf("importing same key twice succeeded")
	}
	if _, err = ks.ImportECDSA(key, "new"); err == nil {
		t.Errorf("importing same key twice succeeded")
	}
}

// TestImportECDSA tests the import and export functionality of a keystore.
func TestImportExport(t *testing.T) {
	_, ks := tmpKeyStore(t, true)
	acc, err := ks.NewAccount("old")
	if err != nil {
		t.Fatalf("failed to create account: %v", acc)
	}
	json, err := ks.Export(acc, "old", "new")
	if err != nil {
		t.Fatalf("failed to export account: %v", acc)
	}
	_, ks2 := tmpKeyStore(t, true)
	if _, err = ks2.Import(json, "old", "old"); err == nil {
		t.Errorf("importing with invalid password succeeded")
	}
	acc2, err := ks2.Import(json, "new", "new")
	if err != nil {
		t.Errorf("importing failed: %v", err)
	}
	if acc.Address != acc2.Address {
		t.Error("imported account does not match exported account")
	}
	if _, err = ks2.Import(json, "new", "new"); err == nil {
		t.Errorf("importing a key twice succeeded")
	}
}

// TestImportRace tests the keystore on races.
// This test should fail under -race if importing races.
func TestImportRace(t *testing.T) {
	_, ks := tmpKeyStore(t, true)
	acc, err := ks.NewAccount("old")
	if err != nil {
		t.Fatalf("failed to create account: %v", acc)
	}
	json, err := ks.Export(acc, "old", "new")
	if err != nil {
		t.Fatalf("failed to export account: %v", acc)
	}
	_, ks2 := tmpKeyStore(t, true)
	var atom atomic.Uint32
	var wg sync.WaitGroup
	wg.Add(2)
	for i := 0; i < 2; i++ {
		go func() {
			defer wg.Done()
			if _, err := ks2.Import(json, "new", "new"); err != nil {
				atom.Add(1)
			}
		}()
	}
	wg.Wait()
	if atom.Load() != 1 {
		t.Errorf("Import is racy")
	}
}

// checkAccounts checks that all known live accounts are present in the wallet list.
func checkAccounts(t *testing.T, live map[common.Address]accounts.Account, wallets []accounts.Wallet) {
	if len(live) != len(wallets) {
		t.Errorf("wallet list doesn't match required accounts: have %d, want %d", len(wallets), len(live))
		return
	}
	liveList := make([]accounts.Account, 0, len(live))
	for _, account := range live {
		liveList = append(liveList, account)
	}
	slices.SortFunc(liveList, byURL)
	for j, wallet := range wallets {
		if accs := wallet.Accounts(); len(accs) != 1 {
			t.Errorf("wallet %d: contains invalid number of accounts: have %d, want 1", j, len(accs))
		} else if accs[0] != liveList[j] {
			t.Errorf("wallet %d: account mismatch: have %v, want %v", j, accs[0], liveList[j])
		}
	}
}

// checkEvents checks that all events in 'want' are present in 'have'. Events may be present multiple times.
func checkEvents(t *testing.T, want []walletEvent, have []walletEvent) {
	for _, wantEv := range want {
		nmatch := 0
		for ; len(have) > 0; nmatch++ {
			if have[0].Kind != wantEv.Kind || have[0].a != wantEv.a {
				break
			}
			have = have[1:]
		}
		if nmatch == 0 {
			t.Fatalf("can't find event with Kind=%v for %x", wantEv.Kind, wantEv.a.Address)
		}
	}
}

func tmpKeyStore(t *testing.T, encrypted bool) (string, *KeyStore) {
	d := t.TempDir()
	newKs := NewPlaintextKeyStore
	if encrypted {
		newKs = func(kd string) *KeyStore { return NewKeyStore(kd, veryLightScryptN, veryLightScryptP) }
	}
	return d, newKs(d)
}
