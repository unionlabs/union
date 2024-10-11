package appdata

import (
	"fmt"
	"testing"
)

func TestListenerMux(t *testing.T) {
	t.Run("empty", func(t *testing.T) {
		listener := ListenerMux(Listener{}, Listener{})

		if listener.InitializeModuleData != nil {
			t.Error("expected nil")
		}
		if listener.StartBlock != nil {
			t.Error("expected nil")
		}
		if listener.OnTx != nil {
			t.Error("expected nil")
		}
		if listener.OnEvent != nil {
			t.Error("expected nil")
		}
		if listener.OnKVPair != nil {
			t.Error("expected nil")
		}
		if listener.OnObjectUpdate != nil {
			t.Error("expected nil")
		}
		if listener.Commit != nil {
			t.Error("expected nil")
		}
	})

	t.Run("all called once", func(t *testing.T) {
		var calls []string
		onCall := func(name string, i int, _ Packet) {
			calls = append(calls, fmt.Sprintf("%s %d", name, i))
		}

		res := ListenerMux(callCollector(1, onCall), callCollector(2, onCall))

		completeCb := callAllCallbacksOnces(t, res)
		if completeCb != nil {
			if err := completeCb(); err != nil {
				t.Fatal(err)
			}
		}

		checkExpectedCallOrder(t, calls, []string{
			"InitializeModuleData 1",
			"InitializeModuleData 2",
			"StartBlock 1",
			"StartBlock 2",
			"OnTx 1",
			"OnTx 2",
			"OnEvent 1",
			"OnEvent 2",
			"OnKVPair 1",
			"OnKVPair 2",
			"OnObjectUpdate 1",
			"OnObjectUpdate 2",
			"Commit 1",
			"Commit 2",
		})
	})
}

func callAllCallbacksOnces(t *testing.T, listener Listener) (completeCb func() error) {
	t.Helper()
	if err := listener.InitializeModuleData(ModuleInitializationData{}); err != nil {
		t.Error(err)
	}
	if err := listener.StartBlock(StartBlockData{}); err != nil {
		t.Error(err)
	}
	if err := listener.OnTx(TxData{}); err != nil {
		t.Error(err)
	}
	if err := listener.OnEvent(EventData{}); err != nil {
		t.Error(err)
	}
	if err := listener.OnKVPair(KVPairData{}); err != nil {
		t.Error(err)
	}
	if err := listener.OnObjectUpdate(ObjectUpdateData{}); err != nil {
		t.Error(err)
	}
	var err error
	completeCb, err = listener.Commit(CommitData{})
	if err != nil {
		t.Error(err)
	}
	return completeCb
}

func callCollector(i int, onCall func(string, int, Packet)) Listener {
	return Listener{
		InitializeModuleData: func(ModuleInitializationData) error {
			onCall("InitializeModuleData", i, nil)
			return nil
		},
		StartBlock: func(StartBlockData) error {
			onCall("StartBlock", i, nil)
			return nil
		},
		OnTx: func(TxData) error {
			onCall("OnTx", i, nil)
			return nil
		},
		OnEvent: func(EventData) error {
			onCall("OnEvent", i, nil)
			return nil
		},
		OnKVPair: func(KVPairData) error {
			onCall("OnKVPair", i, nil)
			return nil
		},
		OnObjectUpdate: func(ObjectUpdateData) error {
			onCall("OnObjectUpdate", i, nil)
			return nil
		},
		Commit: func(data CommitData) (completionCallback func() error, err error) {
			onCall("Commit", i, nil)
			return nil, nil
		},
	}
}

func checkExpectedCallOrder(t *testing.T, actual, expected []string) {
	t.Helper()
	if len(actual) != len(expected) {
		t.Fatalf("expected %d calls, got %d", len(expected), len(actual))
	}

	for i := range actual {
		if actual[i] != expected[i] {
			t.Errorf("expected %q, got %q", expected[i], actual[i])
		}
	}
}
