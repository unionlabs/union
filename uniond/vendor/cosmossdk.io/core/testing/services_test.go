package coretesting

import (
	"context"
	"testing"
)

func TestKVStoreService(t *testing.T) {
	ctx := Context()
	svc1 := KVStoreService(ctx, "bank")

	// must panic
	t.Run("must panic on invalid ctx", func(t *testing.T) {
		defer func() {
			if r := recover(); r == nil {
				t.Errorf("expected panic, but got none")
			}
		}()

		svc1.OpenKVStore(context.Background())
	})

	t.Run("success", func(t *testing.T) {
		kv := svc1.OpenKVStore(ctx)
		err := kv.Set([]byte("key"), []byte("value"))
		if err != nil {
			t.Errorf("failed to set value: %v", err)
		}

		value, err := kv.Get([]byte("key"))
		if err != nil {
			t.Errorf("failed to get value: %v", err)
		}

		if string(value) != "value" {
			t.Errorf("expected value 'value', but got '%s'", string(value))
		}
	})

	t.Run("contains module name", func(t *testing.T) {
		KVStoreService(ctx, "auth")
		_, ok := unwrap(ctx).stores["auth"]
		if !ok {
			t.Errorf("expected store 'auth' to exist, but it doesn't")
		}
	})
}
