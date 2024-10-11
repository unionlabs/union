package decoding

import (
	"errors"
	"fmt"
	"reflect"
	"sort"
	"strconv"
	"strings"
	"testing"

	"cosmossdk.io/schema"
	"cosmossdk.io/schema/appdata"
)

func TestMiddleware(t *testing.T) {
	tl := newTestFixture(t)
	listener, err := Middleware(tl.Listener, tl.resolver, MiddlewareOptions{})
	if err != nil {
		t.Fatal("unexpected error", err)
	}
	tl.setListener(listener)

	tl.bankMod.Mint("bob", "foo", 100)
	err = tl.bankMod.Send("bob", "alice", "foo", 50)
	if err != nil {
		t.Fatal("unexpected error", err)
	}

	tl.oneMod.SetValue("abc")

	expectedBank := []schema.StateObjectUpdate{
		{
			TypeName: "supply",
			Key:      []interface{}{"foo"},
			Value:    uint64(100),
		},
		{
			TypeName: "balances",
			Key:      []interface{}{"bob", "foo"},
			Value:    uint64(100),
		},
		{
			TypeName: "balances",
			Key:      []interface{}{"bob", "foo"},
			Value:    uint64(50),
		},
		{
			TypeName: "balances",
			Key:      []interface{}{"alice", "foo"},
			Value:    uint64(50),
		},
	}

	if !reflect.DeepEqual(tl.bankUpdates, expectedBank) {
		t.Fatalf("expected %v, got %v", expectedBank, tl.bankUpdates)
	}

	expectedOne := []schema.StateObjectUpdate{
		{TypeName: "item", Value: "abc"},
	}

	if !reflect.DeepEqual(tl.oneValueUpdates, expectedOne) {
		t.Fatalf("expected %v, got %v", expectedOne, tl.oneValueUpdates)
	}
}

func TestMiddleware_filtered(t *testing.T) {
	tl := newTestFixture(t)
	listener, err := Middleware(tl.Listener, tl.resolver, MiddlewareOptions{
		ModuleFilter: func(moduleName string) bool {
			return moduleName == "one" //nolint:goconst // adding constants for this would impede readability
		},
	})
	if err != nil {
		t.Fatal("unexpected error", err)
	}
	tl.setListener(listener)

	tl.bankMod.Mint("bob", "foo", 100)
	tl.oneMod.SetValue("abc")

	if len(tl.bankUpdates) != 0 {
		t.Fatalf("expected no bank updates")
	}

	expectedOne := []schema.StateObjectUpdate{
		{TypeName: "item", Value: "abc"},
	}

	if !reflect.DeepEqual(tl.oneValueUpdates, expectedOne) {
		t.Fatalf("expected %v, got %v", expectedOne, tl.oneValueUpdates)
	}
}

func TestSync(t *testing.T) {
	tl := newTestFixture(t)
	tl.bankMod.Mint("bob", "foo", 100)
	err := tl.bankMod.Send("bob", "alice", "foo", 50)
	if err != nil {
		t.Fatal("unexpected error", err)
	}

	tl.oneMod.SetValue("def")

	err = Sync(tl.Listener, tl.multiStore, tl.resolver, SyncOptions{})
	if err != nil {
		t.Fatal("unexpected error", err)
	}

	expected := []schema.StateObjectUpdate{
		{
			TypeName: "balances",
			Key:      []interface{}{"alice", "foo"},
			Value:    uint64(50),
		},
		{
			TypeName: "balances",
			Key:      []interface{}{"bob", "foo"},
			Value:    uint64(50),
		},
		{
			TypeName: "supply",
			Key:      []interface{}{"foo"},
			Value:    uint64(100),
		},
	}

	if !reflect.DeepEqual(tl.bankUpdates, expected) {
		t.Fatalf("expected %v, got %v", expected, tl.bankUpdates)
	}

	expectedOne := []schema.StateObjectUpdate{
		{TypeName: "item", Value: "def"},
	}

	if !reflect.DeepEqual(tl.oneValueUpdates, expectedOne) {
		t.Fatalf("expected %v, got %v", expectedOne, tl.oneValueUpdates)
	}
}

func TestSync_filtered(t *testing.T) {
	tl := newTestFixture(t)
	tl.bankMod.Mint("bob", "foo", 100)
	tl.oneMod.SetValue("def")

	err := Sync(tl.Listener, tl.multiStore, tl.resolver, SyncOptions{
		ModuleFilter: func(moduleName string) bool {
			return moduleName == "one"
		},
	})
	if err != nil {
		t.Fatal("unexpected error", err)
	}

	if len(tl.bankUpdates) != 0 {
		t.Fatalf("expected no bank updates")
	}

	expectedOne := []schema.StateObjectUpdate{
		{TypeName: "item", Value: "def"},
	}

	if !reflect.DeepEqual(tl.oneValueUpdates, expectedOne) {
		t.Fatalf("expected %v, got %v", expectedOne, tl.oneValueUpdates)
	}
}

type testFixture struct {
	appdata.Listener
	bankUpdates     []schema.StateObjectUpdate
	oneValueUpdates []schema.StateObjectUpdate
	resolver        DecoderResolver
	multiStore      *testMultiStore
	bankMod         *exampleBankModule
	oneMod          *oneValueModule
}

func newTestFixture(t *testing.T) *testFixture {
	t.Helper()
	res := &testFixture{}
	res.Listener = appdata.Listener{
		InitializeModuleData: func(data appdata.ModuleInitializationData) error {
			var expected schema.ModuleSchema
			switch data.ModuleName {
			case "bank":
				expected = exampleBankSchema
			case "one":

				expected = oneValueModSchema
			default:
				t.Fatalf("unexpected module %s", data.ModuleName)
			}

			if !reflect.DeepEqual(data.Schema, expected) {
				t.Errorf("expected %v, got %v", expected, data.Schema)
			}
			return nil
		},
		OnObjectUpdate: func(data appdata.ObjectUpdateData) error {
			switch data.ModuleName {
			case "bank":
				res.bankUpdates = append(res.bankUpdates, data.Updates...)
			case "one":
				res.oneValueUpdates = append(res.oneValueUpdates, data.Updates...)
			default:
				t.Errorf("unexpected module %s", data.ModuleName)
			}
			return nil
		},
	}
	res.multiStore = newTestMultiStore()
	res.bankMod = &exampleBankModule{
		store: res.multiStore.newTestStore(t, "bank"),
	}
	res.oneMod = &oneValueModule{
		store: res.multiStore.newTestStore(t, "one"),
	}
	modSet := map[string]interface{}{
		"bank": res.bankMod,
		"one":  res.oneMod,
	}
	res.resolver = ModuleSetDecoderResolver(modSet)
	return res
}

func (f *testFixture) setListener(listener appdata.Listener) {
	f.bankMod.store.listener = listener
	f.oneMod.store.listener = listener
}

type testMultiStore struct {
	stores map[string]*testStore
}

type testStore struct {
	t        *testing.T
	modName  string
	store    map[string][]byte
	listener appdata.Listener
}

func newTestMultiStore() *testMultiStore {
	return &testMultiStore{
		stores: map[string]*testStore{},
	}
}

var _ SyncSource = &testMultiStore{}

func (ms *testMultiStore) IterateAllKVPairs(moduleName string, fn func(key, value []byte) error) error {
	s, ok := ms.stores[moduleName]
	if !ok {
		return fmt.Errorf("don't have state for module %s", moduleName)
	}

	var keys []string
	for key := range s.store {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	for _, key := range keys {
		err := fn([]byte(key), s.store[key])
		if err != nil {
			return err
		}
	}
	return nil
}

func (ms *testMultiStore) newTestStore(t *testing.T, modName string) *testStore {
	t.Helper()
	s := &testStore{
		t:       t,
		modName: modName,
		store:   map[string][]byte{},
	}
	ms.stores[modName] = s
	return s
}

func (t testStore) Get(key []byte) []byte {
	return t.store[string(key)]
}

func (t testStore) GetUInt64(key []byte) uint64 {
	bz := t.store[string(key)]
	if len(bz) == 0 {
		return 0
	}
	x, err := strconv.ParseUint(string(bz), 10, 64)
	if err != nil {
		t.t.Fatalf("unexpected error: %v", err)
	}
	return x
}

func (t testStore) Set(key, value []byte) {
	if t.listener.OnKVPair != nil {
		err := t.listener.OnKVPair(appdata.KVPairData{Updates: []appdata.ActorKVPairUpdate{
			{
				Actor: []byte(t.modName),
				StateChanges: []schema.KVPairUpdate{
					{
						Key:   key,
						Value: value,
					},
				},
			},
		}})
		if err != nil {
			t.t.Fatalf("unexpected error: %v", err)
		}
	}
	t.store[string(key)] = value
}

func (t testStore) SetUInt64(key []byte, value uint64) {
	t.Set(key, []byte(strconv.FormatUint(value, 10)))
}

type exampleBankModule struct {
	store *testStore
}

func (e exampleBankModule) Mint(acct, denom string, amount uint64) {
	key := supplyKey(denom)
	e.store.SetUInt64(key, e.store.GetUInt64(key)+amount)
	e.addBalance(acct, denom, amount)
}

func (e exampleBankModule) Send(from, to, denom string, amount uint64) error {
	err := e.subBalance(from, denom, amount)
	if err != nil {
		return nil
	}
	e.addBalance(to, denom, amount)
	return nil
}

func (e exampleBankModule) GetBalance(acct, denom string) uint64 {
	return e.store.GetUInt64(balanceKey(acct, denom))
}

func (e exampleBankModule) GetSupply(denom string) uint64 {
	return e.store.GetUInt64(supplyKey(denom))
}

func balanceKey(acct, denom string) []byte {
	return []byte(fmt.Sprintf("balance/%s/%s", acct, denom))
}

func supplyKey(denom string) []byte {
	return []byte(fmt.Sprintf("supply/%s", denom))
}

func (e exampleBankModule) addBalance(acct, denom string, amount uint64) {
	key := balanceKey(acct, denom)
	e.store.SetUInt64(key, e.store.GetUInt64(key)+amount)
}

func (e exampleBankModule) subBalance(acct, denom string, amount uint64) error {
	key := balanceKey(acct, denom)
	cur := e.store.GetUInt64(key)
	if cur < amount {
		return errors.New("insufficient balance")
	}
	e.store.SetUInt64(key, cur-amount)
	return nil
}

func init() {
	var err error
	exampleBankSchema, err = schema.CompileModuleSchema(schema.StateObjectType{
		Name: "balances",
		KeyFields: []schema.Field{
			{
				Name: "account",
				Kind: schema.StringKind,
			},
			{
				Name: "denom",
				Kind: schema.StringKind,
			},
		},
		ValueFields: []schema.Field{
			{
				Name: "amount",
				Kind: schema.Uint64Kind,
			},
		},
	})
	if err != nil {
		panic(err)
	}
}

var exampleBankSchema schema.ModuleSchema

func (e exampleBankModule) ModuleCodec() (schema.ModuleCodec, error) {
	return schema.ModuleCodec{
		Schema: exampleBankSchema,
		KVDecoder: func(update schema.KVPairUpdate) ([]schema.StateObjectUpdate, error) {
			key := string(update.Key)
			value, err := strconv.ParseUint(string(update.Value), 10, 64)
			if err != nil {
				return nil, err
			}
			if strings.HasPrefix(key, "balance/") {
				parts := strings.Split(key, "/")
				return []schema.StateObjectUpdate{{
					TypeName: "balances",
					Key:      []interface{}{parts[1], parts[2]},
					Value:    value,
				}}, nil
			} else if strings.HasPrefix(key, "supply/") {
				parts := strings.Split(key, "/")
				return []schema.StateObjectUpdate{{
					TypeName: "supply",
					Key:      []interface{}{parts[1]},
					Value:    value,
				}}, nil
			} else {
				return nil, fmt.Errorf("unexpected key: %s", key)
			}
		},
	}, nil
}

var _ schema.HasModuleCodec = exampleBankModule{}

type oneValueModule struct {
	store *testStore
}

func init() {
	var err error
	oneValueModSchema, err = schema.CompileModuleSchema(schema.StateObjectType{
		Name: "item",
		ValueFields: []schema.Field{
			{Name: "value", Kind: schema.StringKind},
		},
	})
	if err != nil {
		panic(err)
	}
}

var oneValueModSchema schema.ModuleSchema

func (i oneValueModule) ModuleCodec() (schema.ModuleCodec, error) {
	return schema.ModuleCodec{
		Schema: oneValueModSchema,
		KVDecoder: func(update schema.KVPairUpdate) ([]schema.StateObjectUpdate, error) {
			if string(update.Key) != "key" {
				return nil, fmt.Errorf("unexpected key: %v", update.Key)
			}
			return []schema.StateObjectUpdate{
				{TypeName: "item", Value: string(update.Value)},
			}, nil
		},
	}, nil
}

func (i oneValueModule) SetValue(x string) {
	i.store.Set([]byte("key"), []byte(x))
}

var _ schema.HasModuleCodec = oneValueModule{}
