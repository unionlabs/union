package maps

import (
	"strconv"

	"github.com/spf13/pflag"
)

func newInt64ToString[K int64, V string](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newInt64StringMap := newGenericMapValue(val, p)
	newInt64StringMap.Options = genericMapValueOptions[K, V]{
		genericType: "int64ToString",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseInt(s, 10, 64)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			return V(s), nil
		},
	}
	return newInt64StringMap
}

func Int64ToStringP(flagSet *pflag.FlagSet, name, shorthand string, value map[int64]string, usage string) *map[int64]string {
	p := make(map[int64]string)
	flagSet.VarP(newInt64ToString(value, &p), name, shorthand, usage)
	return &p
}

func newInt64ToInt32[K int64, V int32](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newInt64Int32Map := newGenericMapValue(val, p)
	newInt64Int32Map.Options = genericMapValueOptions[K, V]{
		genericType: "int64ToInt32",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseInt(s, 10, 64)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseInt(s, 10, 32)
			return V(value), err
		},
	}
	return newInt64Int32Map
}

func Int64ToInt32P(flagSet *pflag.FlagSet, name, shorthand string, value map[int64]int32, usage string) *map[int64]int32 {
	p := make(map[int64]int32)
	flagSet.VarP(newInt64ToInt32(value, &p), name, shorthand, usage)
	return &p
}

func newInt64ToInt64[K, V int64](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newInt64Int64Map := newGenericMapValue(val, p)
	newInt64Int64Map.Options = genericMapValueOptions[K, V]{
		genericType: "int64ToInt64",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseInt(s, 10, 64)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseInt(s, 10, 64)
			return V(value), err
		},
	}
	return newInt64Int64Map
}

func Int64ToInt64P(flagSet *pflag.FlagSet, name, shorthand string, value map[int64]int64, usage string) *map[int64]int64 {
	p := make(map[int64]int64)
	flagSet.VarP(newInt64ToInt64(value, &p), name, shorthand, usage)
	return &p
}

func newInt64ToUint32[K int64, V uint32](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newInt64Uint32Map := newGenericMapValue(val, p)
	newInt64Uint32Map.Options = genericMapValueOptions[K, V]{
		genericType: "int64ToUint32",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseInt(s, 10, 64)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseUint(s, 10, 32)
			return V(value), err
		},
	}
	return newInt64Uint32Map
}

func Int64ToUint32P(flagSet *pflag.FlagSet, name, shorthand string, value map[int64]uint32, usage string) *map[int64]uint32 {
	p := make(map[int64]uint32)
	flagSet.VarP(newInt64ToUint32(value, &p), name, shorthand, usage)
	return &p
}

func newInt64ToUint64[K int64, V uint64](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newInt64Uint64Map := newGenericMapValue(val, p)
	newInt64Uint64Map.Options = genericMapValueOptions[K, V]{
		genericType: "int64ToUint64",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseInt(s, 10, 64)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseUint(s, 10, 64)
			return V(value), err
		},
	}
	return newInt64Uint64Map
}

func Int64ToUint64P(flagSet *pflag.FlagSet, name, shorthand string, value map[int64]uint64, usage string) *map[int64]uint64 {
	p := make(map[int64]uint64)
	flagSet.VarP(newInt64ToUint64(value, &p), name, shorthand, usage)
	return &p
}

func newInt64ToBool[K int64, V bool](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newInt64BoolMap := newGenericMapValue(val, p)
	newInt64BoolMap.Options = genericMapValueOptions[K, V]{
		genericType: "int64ToBool",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseInt(s, 10, 64)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseBool(s)
			return V(value), err
		},
	}
	return newInt64BoolMap
}

func Int64ToBoolP(flagSet *pflag.FlagSet, name, shorthand string, value map[int64]bool, usage string) *map[int64]bool {
	p := make(map[int64]bool)
	flagSet.VarP(newInt64ToBool(value, &p), name, shorthand, usage)
	return &p
}

func newStringToUint32[K string, V uint32](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newStringUintMap := newGenericMapValue(val, p)
	newStringUintMap.Options = genericMapValueOptions[K, V]{
		genericType: "stringToUint32",
		keyParser: func(s string) (K, error) {
			return K(s), nil
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseUint(s, 10, 32)
			return V(value), err
		},
	}
	return newStringUintMap
}

func StringToUint32P(flagSet *pflag.FlagSet, name, shorthand string, value map[string]uint32, usage string) *map[string]uint32 {
	p := make(map[string]uint32)
	flagSet.VarP(newStringToUint32(value, &p), name, shorthand, usage)
	return &p
}

func newStringToUint64[K string, V uint64](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newStringUint64Map := newGenericMapValue(val, p)
	newStringUint64Map.Options = genericMapValueOptions[K, V]{
		genericType: "stringToUint64",
		keyParser: func(s string) (K, error) {
			return K(s), nil
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseUint(s, 10, 64)
			return V(value), err
		},
	}
	return newStringUint64Map
}

func StringToUint64P(flagSet *pflag.FlagSet, name, shorthand string, value map[string]uint64, usage string) *map[string]uint64 {
	p := make(map[string]uint64)
	flagSet.VarP(newStringToUint64(value, &p), name, shorthand, usage)
	return &p
}
