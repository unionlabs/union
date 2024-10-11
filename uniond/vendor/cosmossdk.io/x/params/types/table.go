package types

import (
	"reflect"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

type attribute struct {
	ty  reflect.Type
	vfn ValueValidatorFn
}

// KeyTable subspaces appropriate type for each parameter key
type KeyTable struct {
	m map[string]attribute
}

// IsOnePerModuleType implements depinject.OnePerModuleType
func (KeyTable) IsOnePerModuleType() {}

func NewKeyTable(pairs ...ParamSetPair) KeyTable {
	keyTable := KeyTable{
		m: make(map[string]attribute),
	}

	for _, psp := range pairs {
		keyTable = keyTable.RegisterType(psp)
	}

	return keyTable
}

// RegisterType registers a single ParamSetPair (key-type pair) in a KeyTable.
func (t KeyTable) RegisterType(psp ParamSetPair) KeyTable {
	if len(psp.Key) == 0 {
		panic("cannot register ParamSetPair with an parameter empty key")
	}
	if !sdk.IsAlphaNumeric(string(psp.Key)) {
		panic("cannot register ParamSetPair with a non-alphanumeric parameter key")
	}
	if psp.ValidatorFn == nil {
		panic("cannot register ParamSetPair without a value validation function")
	}

	keystr := string(psp.Key)
	if _, ok := t.m[keystr]; ok {
		panic("duplicate parameter key")
	}

	rty := reflect.TypeOf(psp.Value)

	// indirect rty if it is a pointer
	for rty.Kind() == reflect.Ptr {
		rty = rty.Elem()
	}

	t.m[keystr] = attribute{
		vfn: psp.ValidatorFn,
		ty:  rty,
	}

	return t
}

// RegisterParamSet registers multiple ParamSetPairs from a ParamSet in a KeyTable.
func (t KeyTable) RegisterParamSet(ps ParamSet) KeyTable {
	for _, psp := range ps.ParamSetPairs() {
		t = t.RegisterType(psp)
	}
	return t
}

func (t KeyTable) maxKeyLength() (res int) {
	for k := range t.m {
		l := len(k)
		if l > res {
			res = l
		}
	}

	return
}
