package appdata

import (
	"context"
	"reflect"
	"testing"
)

func TestBatch(t *testing.T) {
	l, got := batchListener()

	if err := l.SendPacket(testBatch); err != nil {
		t.Error(err)
	}

	if !reflect.DeepEqual(*got, testBatch) {
		t.Errorf("got %v, expected %v", *got, testBatch)
	}
}

var testBatch = PacketBatch{
	ModuleInitializationData{},
	StartBlockData{},
	TxData{},
	EventData{},
	KVPairData{},
	ObjectUpdateData{},
}

func batchListener() (Listener, *PacketBatch) {
	got := new(PacketBatch)
	l := Listener{
		InitializeModuleData: func(m ModuleInitializationData) error {
			*got = append(*got, m)
			return nil
		},
		StartBlock: func(b StartBlockData) error {
			*got = append(*got, b)
			return nil
		},
		OnTx: func(t TxData) error {
			*got = append(*got, t)
			return nil
		},
		OnEvent: func(e EventData) error {
			*got = append(*got, e)
			return nil
		},
		OnKVPair: func(k KVPairData) error {
			*got = append(*got, k)
			return nil
		},
		OnObjectUpdate: func(o ObjectUpdateData) error {
			*got = append(*got, o)
			return nil
		},
	}

	return l, got
}

func TestBatchAsync(t *testing.T) {
	l, got := batchListener()

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	l = AsyncListenerMux(AsyncListenerOptions{Context: ctx}, l)

	if err := l.SendPacket(testBatch); err != nil {
		t.Error(err)
	}

	// commit to synchronize
	cb, err := l.Commit(CommitData{})
	if err != nil {
		t.Error(err)
	}
	if err := cb(); err != nil {
		t.Error(err)
	}

	if !reflect.DeepEqual(*got, testBatch) {
		t.Errorf("got %v, expected %v", *got, testBatch)
	}
}
