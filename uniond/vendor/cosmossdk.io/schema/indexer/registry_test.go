package indexer

import "testing"

func TestRegister(t *testing.T) {
	Register("test", Initializer{
		InitFunc: func(params InitParams) (InitResult, error) {
			return InitResult{}, nil
		},
	})

	if _, ok := indexerRegistry["test"]; !ok {
		t.Fatalf("expected to find indexer")
	}

	if _, ok := indexerRegistry["test2"]; ok {
		t.Fatalf("expected not to find indexer")
	}

	defer func() {
		if r := recover(); r == nil {
			t.Fatalf("expected to panic")
		}
	}()
	Register("test", Initializer{
		InitFunc: func(params InitParams) (InitResult, error) {
			return InitResult{}, nil
		},
	})
}
