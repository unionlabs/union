package maps

import (
	"strconv"

	"github.com/spf13/pflag"
)

func newStringToInt32[K string, V int32](val map[K]V, p *map[K]V) *genericMapValue[K, V] {
	newStringIntMap := newGenericMapValue(val, p)
	newStringIntMap.Options = genericMapValueOptions[K, V]{
		genericType: "stringToInt32",
		keyParser: func(s string) (K, error) {
			return K(s), nil
		},
		valueParser: func(s string) (V, error) {
			value, err := strconv.ParseInt(s, 10, 32)
			return V(value), err
		},
	}
	return newStringIntMap
}

func StringToInt32P(flagSet *pflag.FlagSet, name, shorthand string, value map[string]int32, usage string) *map[string]int32 {
	p := make(map[string]int32)
	flagSet.VarP(newStringToInt32(value, &p), name, shorthand, usage)
	return &p
}
