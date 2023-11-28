package maps

import (
	"strconv"

	"github.com/spf13/pflag"
)

func newUint32ToString[K uint32, V string](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newUint32StringMap := newGenericMapValue(val, p)
	newUint32StringMap.Options = genericMapValueOptions[K, V]{
		genericType: "uint32ToString",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseUint(s, 10, 32)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			return V(s), nil
		},
	}
	return newUint32StringMap
}

func Uint32ToStringP(flagSet *pflag.FlagSet, name, shorthand string, value map[uint32]string, usage string) *map[uint32]string {
	p := make(map[uint32]string)
	flagSet.VarP(newUint32ToString(value, &p), name, shorthand, usage)
	return &p
}

func newUint32ToInt32[K uint32, V int32](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newUint32Int32Map := newGenericMapValue(val, p)
	newUint32Int32Map.Options = genericMapValueOptions[K, V]{
		genericType: "uint32ToInt32",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseUint(s, 10, 32)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseInt(s, 10, 32)
			return V(value), err
		},
	}
	return newUint32Int32Map
}

func Uint32ToInt32P(flagSet *pflag.FlagSet, name, shorthand string, value map[uint32]int32, usage string) *map[uint32]int32 {
	p := make(map[uint32]int32)
	flagSet.VarP(newUint32ToInt32(value, &p), name, shorthand, usage)
	return &p
}

func newUint32ToInt64[K uint32, V int64](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newUint32Int64Map := newGenericMapValue(val, p)
	newUint32Int64Map.Options = genericMapValueOptions[K, V]{
		genericType: "uint32ToInt64",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseUint(s, 10, 32)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseInt(s, 10, 64)
			return V(value), err
		},
	}
	return newUint32Int64Map
}

func Uint32ToInt64P(flagSet *pflag.FlagSet, name, shorthand string, value map[uint32]int64, usage string) *map[uint32]int64 {
	p := make(map[uint32]int64)
	flagSet.VarP(newUint32ToInt64(value, &p), name, shorthand, usage)
	return &p
}

func newUint32ToUint32[K, V uint32](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newUint32Uint32Map := newGenericMapValue(val, p)
	newUint32Uint32Map.Options = genericMapValueOptions[K, V]{
		genericType: "uint32ToUint32",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseUint(s, 10, 32)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseUint(s, 10, 32)
			return V(value), err
		},
	}
	return newUint32Uint32Map
}

func Uint32ToUint32P(flagSet *pflag.FlagSet, name, shorthand string, value map[uint32]uint32, usage string) *map[uint32]uint32 {
	p := make(map[uint32]uint32)
	flagSet.VarP(newUint32ToUint32(value, &p), name, shorthand, usage)
	return &p
}

func newUint32ToUint64[K uint32, V uint64](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newUint32Uint64Map := newGenericMapValue(val, p)
	newUint32Uint64Map.Options = genericMapValueOptions[K, V]{
		genericType: "uint32ToUint64",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseUint(s, 10, 32)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseUint(s, 10, 64)
			return V(value), err
		},
	}
	return newUint32Uint64Map
}

func Uint32ToUint64P(flagSet *pflag.FlagSet, name, shorthand string, value map[uint32]uint64, usage string) *map[uint32]uint64 {
	p := make(map[uint32]uint64)
	flagSet.VarP(newUint32ToUint64(value, &p), name, shorthand, usage)
	return &p
}

func newUint32ToBool[K uint32, V bool](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newUint32BoolMap := newGenericMapValue(val, p)
	newUint32BoolMap.Options = genericMapValueOptions[K, V]{
		genericType: "uint32ToBool",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseUint(s, 10, 32)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseBool(s)
			return V(value), err
		},
	}
	return newUint32BoolMap
}

func Uint32ToBoolP(flagSet *pflag.FlagSet, name, shorthand string, value map[uint32]bool, usage string) *map[uint32]bool {
	p := make(map[uint32]bool)
	flagSet.VarP(newUint32ToBool(value, &p), name, shorthand, usage)
	return &p
}
