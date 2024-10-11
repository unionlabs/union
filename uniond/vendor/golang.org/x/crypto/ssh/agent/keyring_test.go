// Copyright 2015 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package agent

import "testing"

func addTestKey(t *testing.T, a Agent, keyName string) {
	err := a.Add(AddedKey{
		PrivateKey: testPrivateKeys[keyName],
		Comment:    keyName,
	})
	if err != nil {
		t.Fatalf("failed to add key %q: %v", keyName, err)
	}
}

func removeTestKey(t *testing.T, a Agent, keyName string) {
	err := a.Remove(testPublicKeys[keyName])
	if err != nil {
		t.Fatalf("failed to remove key %q: %v", keyName, err)
	}
}

func validateListedKeys(t *testing.T, a Agent, expectedKeys []string) {
	listedKeys, err := a.List()
	if err != nil {
		t.Fatalf("failed to list keys: %v", err)
		return
	}
	if len(listedKeys) != len(expectedKeys) {
		t.Fatalf("expeted %d key, got %d", len(expectedKeys), len(listedKeys))
		return
	}
	actualKeys := make(map[string]bool)
	for _, key := range listedKeys {
		actualKeys[key.Comment] = true
	}

	matchedKeys := make(map[string]bool)
	for _, expectedKey := range expectedKeys {
		if !actualKeys[expectedKey] {
			t.Fatalf("expected key %q, but was not found", expectedKey)
		} else {
			matchedKeys[expectedKey] = true
		}
	}

	for actualKey := range actualKeys {
		if !matchedKeys[actualKey] {
			t.Fatalf("key %q was found, but was not expected", actualKey)
		}
	}
}

func TestKeyringAddingAndRemoving(t *testing.T) {
	keyNames := []string{"dsa", "ecdsa", "rsa", "user"}

	// add all test private keys
	k := NewKeyring()
	for _, keyName := range keyNames {
		addTestKey(t, k, keyName)
	}
	validateListedKeys(t, k, keyNames)

	// remove a key in the middle
	keyToRemove := keyNames[1]
	keyNames = append(keyNames[:1], keyNames[2:]...)

	removeTestKey(t, k, keyToRemove)
	validateListedKeys(t, k, keyNames)

	// remove all keys
	err := k.RemoveAll()
	if err != nil {
		t.Fatalf("failed to remove all keys: %v", err)
	}
	validateListedKeys(t, k, []string{})
}

func TestAddDuplicateKey(t *testing.T) {
	keyNames := []string{"rsa", "user"}

	k := NewKeyring()
	for _, keyName := range keyNames {
		addTestKey(t, k, keyName)
	}
	validateListedKeys(t, k, keyNames)
	// Add the keys again.
	for _, keyName := range keyNames {
		addTestKey(t, k, keyName)
	}
	validateListedKeys(t, k, keyNames)
	// Add an existing key with an updated comment.
	keyName := keyNames[0]
	addedKey := AddedKey{
		PrivateKey: testPrivateKeys[keyName],
		Comment:    "comment updated",
	}
	err := k.Add(addedKey)
	if err != nil {
		t.Fatalf("failed to add key %q: %v", keyName, err)
	}
	// Check the that key is found and the comment was updated.
	keys, err := k.List()
	if err != nil {
		t.Fatalf("failed to list keys: %v", err)
	}
	if len(keys) != len(keyNames) {
		t.Fatalf("expected %d keys, got %d", len(keyNames), len(keys))
	}
	isFound := false
	for _, key := range keys {
		if key.Comment == addedKey.Comment {
			isFound = true
		}
	}
	if !isFound {
		t.Fatal("key with the updated comment not found")
	}
}
