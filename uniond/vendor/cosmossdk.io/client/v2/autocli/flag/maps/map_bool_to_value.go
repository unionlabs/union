package maps

import (
	"strconv"

	"github.com/spf13/pflag"
)

func newBoolToString[K bool, V string](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newBoolStringMap := newGenericMapValue(val, p)
	newBoolStringMap.Options = genericMapValueOptions[K, V]{
		genericType: "boolToString",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseBool(s)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			return V(s), nil
		},
	}
	return newBoolStringMap
}

func BoolToStringP(flagSet *pflag.FlagSet, name, shorthand string, value map[bool]string, usage string) *map[bool]string {
	p := make(map[bool]string)
	flagSet.VarP(newBoolToString(value, &p), name, shorthand, usage)
	return &p
}

func newBoolToInt32[K bool, V int32](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newBoolInt32Map := newGenericMapValue(val, p)
	newBoolInt32Map.Options = genericMapValueOptions[K, V]{
		genericType: "boolToInt32",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseBool(s)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseInt(s, 10, 32)
			return V(value), err
		},
	}
	return newBoolInt32Map
}

func BoolToInt32P(flagSet *pflag.FlagSet, name, shorthand string, value map[bool]int32, usage string) *map[bool]int32 {
	p := make(map[bool]int32)
	flagSet.VarP(newBoolToInt32(value, &p), name, shorthand, usage)
	return &p
}

func newBoolToInt64[K bool, V int64](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newBoolInt64Map := newGenericMapValue(val, p)
	newBoolInt64Map.Options = genericMapValueOptions[K, V]{
		genericType: "boolToInt64",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseBool(s)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseInt(s, 10, 64)
			return V(value), err
		},
	}
	return newBoolInt64Map
}

func BoolToInt64P(flagSet *pflag.FlagSet, name, shorthand string, value map[bool]int64, usage string) *map[bool]int64 {
	p := make(map[bool]int64)
	flagSet.VarP(newBoolToInt64(value, &p), name, shorthand, usage)
	return &p
}

func newBoolToUint32[K bool, V uint32](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newBoolUint32Map := newGenericMapValue(val, p)
	newBoolUint32Map.Options = genericMapValueOptions[K, V]{
		genericType: "boolToUint32",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseBool(s)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseUint(s, 10, 32)
			return V(value), err
		},
	}
	return newBoolUint32Map
}

func BoolToUint32P(flagSet *pflag.FlagSet, name, shorthand string, value map[bool]uint32, usage string) *map[bool]uint32 {
	p := make(map[bool]uint32)
	flagSet.VarP(newBoolToUint32(value, &p), name, shorthand, usage)
	return &p
}

func newBoolToUint64[K bool, V uint64](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newBoolUint64Map := newGenericMapValue(val, p)
	newBoolUint64Map.Options = genericMapValueOptions[K, V]{
		genericType: "boolToUint64",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseBool(s)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseUint(s, 10, 64)
			return V(value), err
		},
	}
	return newBoolUint64Map
}

func BoolToUint64P(flagSet *pflag.FlagSet, name, shorthand string, value map[bool]uint64, usage string) *map[bool]uint64 {
	p := make(map[bool]uint64)
	flagSet.VarP(newBoolToUint64(value, &p), name, shorthand, usage)
	return &p
}

func newBoolToBool[K, V bool](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newBoolBoolMap := newGenericMapValue(val, p)
	newBoolBoolMap.Options = genericMapValueOptions[K, V]{
		genericType: "boolToBool",
		keyParser: func(s string) (K, error) {
			value, err := strconv.ParseBool(s)
			return K(value), err
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseBool(s)
			return V(value), err
		},
	}
	return newBoolBoolMap
}

func BoolToBoolP(flagSet *pflag.FlagSet, name, shorthand string, value map[bool]bool, usage string) *map[bool]bool {
	p := make(map[bool]bool)
	flagSet.VarP(newBoolToBool(value, &p), name, shorthand, usage)
	return &p
}
