package maps

import (
	"strconv"

	"github.com/spf13/pflag"
)

func newInt32ToString[K int32, V string](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newInt32StringMap := newGenericMapValue(val, p)
	newInt32StringMap.Options = genericMapValueOptions[K, V]{
		genericType: "int32ToString",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseInt(s, 10, 32)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			return V(s), nil
		},
	}
	return newInt32StringMap
}

func Int32ToStringP(flagSet *pflag.FlagSet, name, shorthand string, value map[int32]string, usage string) *map[int32]string {
	p := make(map[int32]string)
	flagSet.VarP(newInt32ToString(value, &p), name, shorthand, usage)
	return &p
}

func newInt32ToInt32[K, V int32](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newInt32Int32Map := newGenericMapValue(val, p)
	newInt32Int32Map.Options = genericMapValueOptions[K, V]{
		genericType: "int32ToInt32",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseInt(s, 10, 32)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseInt(s, 10, 32)
			return V(value), err
		},
	}
	return newInt32Int32Map
}

func Int32ToInt32P(flagSet *pflag.FlagSet, name, shorthand string, value map[int32]int32, usage string) *map[int32]int32 {
	p := make(map[int32]int32)
	flagSet.VarP(newInt32ToInt32(value, &p), name, shorthand, usage)
	return &p
}

func newInt32ToInt64[K int32, V int64](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newInt32Int64Map := newGenericMapValue(val, p)
	newInt32Int64Map.Options = genericMapValueOptions[K, V]{
		genericType: "int32ToInt64",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseInt(s, 10, 32)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseInt(s, 10, 64)
			return V(value), err
		},
	}
	return newInt32Int64Map
}

func Int32ToInt64P(flagSet *pflag.FlagSet, name, shorthand string, value map[int32]int64, usage string) *map[int32]int64 {
	p := make(map[int32]int64)
	flagSet.VarP(newInt32ToInt64(value, &p), name, shorthand, usage)
	return &p
}

func newInt32ToUint32[K int32, V uint32](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newInt32Uint32Map := newGenericMapValue(val, p)
	newInt32Uint32Map.Options = genericMapValueOptions[K, V]{
		genericType: "int32ToUint32",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseInt(s, 10, 32)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseUint(s, 10, 32)
			return V(value), err
		},
	}
	return newInt32Uint32Map
}

func Int32ToUint32P(flagSet *pflag.FlagSet, name, shorthand string, value map[int32]uint32, usage string) *map[int32]uint32 {
	p := make(map[int32]uint32)
	flagSet.VarP(newInt32ToUint32(value, &p), name, shorthand, usage)
	return &p
}

func newInt32ToUint64[K int32, V uint64](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newInt32Uint64Map := newGenericMapValue(val, p)
	newInt32Uint64Map.Options = genericMapValueOptions[K, V]{
		genericType: "int32ToUint64",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseInt(s, 10, 32)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseUint(s, 10, 64)
			return V(value), err
		},
	}
	return newInt32Uint64Map
}

func Int32ToUint64P(flagSet *pflag.FlagSet, name, shorthand string, value map[int32]uint64, usage string) *map[int32]uint64 {
	p := make(map[int32]uint64)
	flagSet.VarP(newInt32ToUint64(value, &p), name, shorthand, usage)
	return &p
}

func newInt32ToBool[K int32, V bool](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newInt32BoolMap := newGenericMapValue(val, p)
	newInt32BoolMap.Options = genericMapValueOptions[K, V]{
		genericType: "int32ToBool",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseInt(s, 10, 32)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseBool(s)
			return V(value), err
		},
	}
	return newInt32BoolMap
}

func Int32ToBoolP(flagSet *pflag.FlagSet, name, shorthand string, value map[int32]bool, usage string) *map[int32]bool {
	p := make(map[int32]bool)
	flagSet.VarP(newInt32ToBool(value, &p), name, shorthand, usage)
	return &p
}

func newStringToBool[K string, V bool](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newStringBoolMap := newGenericMapValue(val, p)
	newStringBoolMap.Options = genericMapValueOptions[K, V]{
		genericType: "stringToBool",
		keyParser: func(s string) (K, error) {
			return K(s), nil
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseBool(s)
			return V(value), err
		},
	}
	return newStringBoolMap
}

func StringToBoolP(flagSet *pflag.FlagSet, name, shorthand string, value map[string]bool, usage string) *map[string]bool {
	p := make(map[string]bool)
	flagSet.VarP(newStringToBool(value, &p), name, shorthand, usage)
	return &p
}
