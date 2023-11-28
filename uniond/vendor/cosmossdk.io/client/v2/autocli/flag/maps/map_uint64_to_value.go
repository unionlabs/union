package maps

import (
	"strconv"

	"github.com/spf13/pflag"
)

func newUint64ToString[K uint64, V string](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newUint64StringMap := newGenericMapValue(val, p)
	newUint64StringMap.Options = genericMapValueOptions[K, V]{
		genericType: "uint64ToString",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseUint(s, 10, 64)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			return V(s), nil
		},
	}
	return newUint64StringMap
}

func Uint64ToStringP(flagSet *pflag.FlagSet, name, shorthand string, value map[uint64]string, usage string) *map[uint64]string {
	p := make(map[uint64]string)
	flagSet.VarP(newUint64ToString(value, &p), name, shorthand, usage)
	return &p
}

func newUint64ToInt32[K uint64, V int32](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newUint64Int32Map := newGenericMapValue(val, p)
	newUint64Int32Map.Options = genericMapValueOptions[K, V]{
		genericType: "uint64ToInt32",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseUint(s, 10, 64)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseInt(s, 10, 32)
			return V(value), err
		},
	}
	return newUint64Int32Map
}

func Uint64ToInt32P(flagSet *pflag.FlagSet, name, shorthand string, value map[uint64]int32, usage string) *map[uint64]int32 {
	p := make(map[uint64]int32)
	flagSet.VarP(newUint64ToInt32(value, &p), name, shorthand, usage)
	return &p
}

func newUint64ToInt64[K uint64, V int64](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newUint64Int64Map := newGenericMapValue(val, p)
	newUint64Int64Map.Options = genericMapValueOptions[K, V]{
		genericType: "uint64ToInt64",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseUint(s, 10, 64)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseInt(s, 10, 64)
			return V(value), err
		},
	}
	return newUint64Int64Map
}

func Uint64ToInt64P(flagSet *pflag.FlagSet, name, shorthand string, value map[uint64]int64, usage string) *map[uint64]int64 {
	p := make(map[uint64]int64)
	flagSet.VarP(newUint64ToInt64(value, &p), name, shorthand, usage)
	return &p
}

func newUint64ToUint32[K uint64, V uint32](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newUint64Uint32Map := newGenericMapValue(val, p)
	newUint64Uint32Map.Options = genericMapValueOptions[K, V]{
		genericType: "uint64ToUint32",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseUint(s, 10, 64)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseUint(s, 10, 32)
			return V(value), err
		},
	}
	return newUint64Uint32Map
}

func Uint64ToUint32P(flagSet *pflag.FlagSet, name, shorthand string, value map[uint64]uint32, usage string) *map[uint64]uint32 {
	p := make(map[uint64]uint32)
	flagSet.VarP(newUint64ToUint32(value, &p), name, shorthand, usage)
	return &p
}

func newUint64ToUint64[K, V uint64](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newUint64Uint64Map := newGenericMapValue(val, p)
	newUint64Uint64Map.Options = genericMapValueOptions[K, V]{
		genericType: "uint64ToUint64",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseUint(s, 10, 64)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseUint(s, 10, 64)
			return V(value), err
		},
	}
	return newUint64Uint64Map
}

func Uint64ToUint64P(flagSet *pflag.FlagSet, name, shorthand string, value map[uint64]uint64, usage string) *map[uint64]uint64 {
	p := make(map[uint64]uint64)
	flagSet.VarP(newUint64ToUint64(value, &p), name, shorthand, usage)
	return &p
}

func newUint64ToBool[K uint64, V bool](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newUint64BoolMap := newGenericMapValue(val, p)
	newUint64BoolMap.Options = genericMapValueOptions[K, V]{
		genericType: "uint64ToBool",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseUint(s, 10, 64)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseBool(s)
			return V(value), err
		},
	}
	return newUint64BoolMap
}

func Uint64ToBoolP(flagSet *pflag.FlagSet, name, shorthand string, value map[uint64]bool, usage string) *map[uint64]bool {
	p := make(map[uint64]bool)
	flagSet.VarP(newUint64ToBool(value, &p), name, shorthand, usage)
	return &p
}
