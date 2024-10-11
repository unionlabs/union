package indexes

import (
	"context"

	"cosmossdk.io/core/store"
	"cosmossdk.io/core/testing"
)

func deps() (store.KVStoreService, context.Context) {
	ctx := coretesting.Context()
	kv := coretesting.KVStoreService(ctx, "test")
	return kv, ctx
}

type company struct {
	City string
	Vat  uint64
}
