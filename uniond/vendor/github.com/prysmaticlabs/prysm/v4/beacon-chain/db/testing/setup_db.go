// Package testing allows for spinning up a real bolt-db
// instance for unit tests throughout the Prysm repo.
package testing

import (
	"context"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/iface"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/kv"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/slasherkv"
)

// SetupDB instantiates and returns database backed by key value store.
func SetupDB(t testing.TB) db.Database {
	s, err := kv.NewKVStore(context.Background(), t.TempDir())
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() {
		if err := s.Close(); err != nil {
			t.Fatalf("failed to close database: %v", err)
		}
	})
	return s
}

// SetupSlasherDB --
func SetupSlasherDB(t testing.TB) iface.SlasherDatabase {
	s, err := slasherkv.NewKVStore(context.Background(), t.TempDir())
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() {
		if err := s.Close(); err != nil {
			t.Fatalf("failed to close database: %v", err)
		}
	})
	return s
}
