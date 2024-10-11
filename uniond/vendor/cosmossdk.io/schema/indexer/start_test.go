package indexer

import (
	"context"
	"encoding/json"
	"reflect"
	"sync"
	"testing"

	"cosmossdk.io/schema/appdata"
)

func TestStart(t *testing.T) {
	ctx, cancelFn := context.WithCancel(context.Background())
	var test1CommitCalled, test2CommitCalled int
	Register("test1", Initializer{
		InitFunc: func(params InitParams) (InitResult, error) {
			if params.Config.Config.(testConfig).SomeParam != "foobar" {
				t.Fatalf("expected %q, got %q", "foobar", params.Config.Config.(testConfig).SomeParam)
			}
			return InitResult{
				Listener: appdata.Listener{
					Commit: func(data appdata.CommitData) (completionCallback func() error, err error) {
						test1CommitCalled++
						return nil, nil
					},
				},
			}, nil
		},
		ConfigType: testConfig{},
	})
	Register("test2", Initializer{
		InitFunc: func(params InitParams) (InitResult, error) {
			if params.Config.Config.(testConfig2).Foo != "bar" {
				t.Fatalf("expected %q, got %q", "bar", params.Config.Config.(testConfig2).Foo)
			}
			return InitResult{
				Listener: appdata.Listener{
					Commit: func(data appdata.CommitData) (completionCallback func() error, err error) {
						test2CommitCalled++
						return nil, nil
					},
				},
			}, nil
		},
		ConfigType: testConfig2{},
	})

	var wg sync.WaitGroup
	target, err := StartIndexing(IndexingOptions{
		Config: IndexingConfig{Target: map[string]Config{
			"t1": {Type: "test1", Config: testConfig{SomeParam: "foobar"}},
			"t2": {Type: "test2", Config: testConfig2{Foo: "bar"}},
		}},
		Resolver:      nil,
		SyncSource:    nil,
		Logger:        nil,
		Context:       ctx,
		AddressCodec:  nil,
		DoneWaitGroup: &wg,
	})
	if err != nil {
		t.Fatal(err)
	}

	const COMMIT_COUNT = 10
	for i := 0; i < COMMIT_COUNT; i++ {
		callCommit(t, target.Listener)
	}

	cancelFn()
	wg.Wait()

	if test1CommitCalled != COMMIT_COUNT {
		t.Fatalf("expected %d, got %d", COMMIT_COUNT, test1CommitCalled)
	}
	if test2CommitCalled != COMMIT_COUNT {
		t.Fatalf("expected %d, got %d", COMMIT_COUNT, test2CommitCalled)
	}
}

func callCommit(t *testing.T, listener appdata.Listener) {
	t.Helper()
	cb, err := listener.Commit(appdata.CommitData{})
	if err != nil {
		t.Fatal(err)
	}
	if cb != nil {
		err = cb()
		if err != nil {
			t.Fatal(err)
		}
	}
}

func TestUnmarshalIndexingConfig(t *testing.T) {
	cfg := &IndexingConfig{Target: map[string]Config{"target": {Type: "type"}}}
	jsonBz, err := json.Marshal(cfg)
	if err != nil {
		t.Fatal(err)
	}

	t.Run("json", func(t *testing.T) {
		res, err := unmarshalIndexingConfig(json.RawMessage(jsonBz))
		if err != nil {
			t.Fatal(err)
		}
		if !reflect.DeepEqual(res, cfg) {
			t.Fatalf("expected %v, got %v", cfg, res)
		}
	})

	t.Run("map", func(t *testing.T) {
		var m map[string]interface{}
		err := json.Unmarshal(jsonBz, &m)
		if err != nil {
			t.Fatal(err)
		}

		res, err := unmarshalIndexingConfig(m)
		if err != nil {
			t.Fatal(err)
		}
		if !reflect.DeepEqual(res, cfg) {
			t.Fatalf("expected %v, got %v", cfg, res)
		}
	})

	t.Run("ptr", func(t *testing.T) {
		res, err := unmarshalIndexingConfig(cfg)
		if err != nil {
			t.Fatal(err)
		}
		if res != cfg {
			t.Fatalf("expected %v, got %v", cfg, res)
		}
	})

	t.Run("struct", func(t *testing.T) {
		res, err := unmarshalIndexingConfig(*cfg)
		if err != nil {
			t.Fatal(err)
		}
		if !reflect.DeepEqual(res, cfg) {
			t.Fatalf("expected %v, got %v", cfg, res)
		}
	})
}

func TestUnmarshalIndexerConfig(t *testing.T) {
	t.Run("struct", func(t *testing.T) {
		cfg := testConfig{SomeParam: "foobar"}
		cfg2, err := unmarshalIndexerCustomConfig(cfg, testConfig{})
		if err != nil {
			t.Fatal(err)
		}
		if !reflect.DeepEqual(cfg, cfg2) {
			t.Fatalf("expected %v, got %v", cfg, cfg2)
		}
	})

	t.Run("ptr", func(t *testing.T) {
		cfg := &testConfig{SomeParam: "foobar"}
		cfg2, err := unmarshalIndexerCustomConfig(cfg, &testConfig{})
		if err != nil {
			t.Fatal(err)
		}
		if !reflect.DeepEqual(cfg, cfg2) {
			t.Fatalf("expected %v, got %v", cfg, cfg2)
		}
	})

	t.Run("map -> struct", func(t *testing.T) {
		cfg := testConfig{SomeParam: "foobar"}
		jzonBz, err := json.Marshal(cfg)
		if err != nil {
			t.Fatal(err)
		}
		var m map[string]interface{}
		err = json.Unmarshal(jzonBz, &m)
		if err != nil {
			t.Fatal(err)
		}
		cfg2, err := unmarshalIndexerCustomConfig(m, testConfig{})
		if err != nil {
			t.Fatal(err)
		}
		if !reflect.DeepEqual(cfg, cfg2) {
			t.Fatalf("expected %v, got %v", cfg, cfg2)
		}
	})

	t.Run("map -> ptr", func(t *testing.T) {
		cfg := &testConfig{SomeParam: "foobar"}
		jzonBz, err := json.Marshal(cfg)
		if err != nil {
			t.Fatal(err)
		}
		var m map[string]interface{}
		err = json.Unmarshal(jzonBz, &m)
		if err != nil {
			t.Fatal(err)
		}
		cfg2, err := unmarshalIndexerCustomConfig(m, &testConfig{})
		if err != nil {
			t.Fatal(err)
		}
		if !reflect.DeepEqual(cfg, cfg2) {
			t.Fatalf("expected %v, got %v", cfg, cfg2)
		}
	})
}

type testConfig struct {
	SomeParam string `json:"some_param"`
}

type testConfig2 struct {
	Foo string `json:"foo"`
}
