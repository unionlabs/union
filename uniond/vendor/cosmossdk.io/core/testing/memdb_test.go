package coretesting

import (
	"bytes"
	"fmt"
	"testing"

	"cosmossdk.io/core/store"
)

func TestMemDB(t *testing.T) {
	var db store.KVStore = NewMemKV()

	key, value := []byte("key"), []byte("value")
	if err := db.Set(key, value); err != nil {
		t.Errorf("Error setting value: %s", err)
	}
	val, err := db.Get(key)
	if err != nil {
		t.Errorf("Error getting value: %s", err)
	}
	if !bytes.Equal(value, val) {
		t.Errorf("Expected value %s, got %s", value, val)
	}
	if err := db.Delete(key); err != nil {
		t.Errorf("Error deleting value: %s", err)
	}
	has, err := db.Has(key)
	if err != nil {
		t.Errorf("Error checking if key exists: %s", err)
	}
	if has {
		t.Errorf("Expected key to be deleted, but it still exists")
	}

	// test iter
	makeKey := func(i int) []byte {
		return []byte(fmt.Sprintf("key_%d", i))
	}
	for i := 0; i < 10; i++ {
		if err := db.Set(makeKey(i), makeKey(i)); err != nil {
			t.Errorf("Error setting value: %s", err)
		}
	}

	iter, err := db.Iterator(nil, nil)
	if err != nil {
		t.Errorf("Error creating iterator: %s", err)
	}
	key = iter.Key()
	value = iter.Value()
	if !bytes.Equal(makeKey(0), key) {
		t.Errorf("Expected key %s, got %s", makeKey(0), key)
	}
	if !bytes.Equal(makeKey(0), value) {
		t.Errorf("Expected value %s, got %s", makeKey(0), value)
	}
	if err := iter.Error(); err != nil {
		t.Errorf("Iterator error: %s", err)
	}
	iter.Next()
	key, value = iter.Key(), iter.Value()
	if !bytes.Equal(makeKey(1), key) {
		t.Errorf("Expected key %s, got %s", makeKey(1), key)
	}
	if !bytes.Equal(makeKey(1), value) {
		t.Errorf("Expected value %s, got %s", makeKey(1), value)
	}
	if err := iter.Close(); err != nil {
		t.Errorf("Error closing iterator: %s", err)
	}

	// test reverse iter
	iter, err = db.ReverseIterator(nil, nil)
	if err != nil {
		t.Errorf("Error creating reverse iterator: %s", err)
	}
	key = iter.Key()
	value = iter.Value()
	if !bytes.Equal(makeKey(9), key) {
		t.Errorf("Expected key %s, got %s", makeKey(9), key)
	}
	if !bytes.Equal(makeKey(9), value) {
		t.Errorf("Expected value %s, got %s", makeKey(9), value)
	}
	if err := iter.Error(); err != nil {
		t.Errorf("Iterator error: %s", err)
	}
	iter.Next()
	key, value = iter.Key(), iter.Value()
	if !bytes.Equal(makeKey(8), key) {
		t.Errorf("Expected key %s, got %s", makeKey(8), key)
	}
	if !bytes.Equal(makeKey(8), value) {
		t.Errorf("Expected value %s, got %s", makeKey(8), value)
	}
	if err := iter.Close(); err != nil {
		t.Errorf("Error closing iterator: %s", err)
	}
}
