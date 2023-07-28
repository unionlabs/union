package dotenv

import (
	"strings"

	"github.com/spf13/cast"
)

// flattenAndMergeMap recursively flattens the given map into a new map
// Code is based on the function with the same name in tha main package.
// TODO: move it to a common place
func flattenAndMergeMap(shadow map[string]interface{}, m map[string]interface{}, prefix string, delimiter string) map[string]interface{} {
	if shadow != nil && prefix != "" && shadow[prefix] != nil {
		// prefix is shadowed => nothing more to flatten
		return shadow
	}
	if shadow == nil {
		shadow = make(map[string]interface{})
	}

	var m2 map[string]interface{}
	if prefix != "" {
		prefix += delimiter
	}
	for k, val := range m {
		fullKey := prefix + k
		switch val.(type) {
		case map[string]interface{}:
			m2 = val.(map[string]interface{})
		case map[interface{}]interface{}:
			m2 = cast.ToStringMap(val)
		default:
			// immediate value
			shadow[strings.ToLower(fullKey)] = val
			continue
		}
		// recursively merge to shadow map
		shadow = flattenAndMergeMap(shadow, m2, fullKey, delimiter)
	}
	return shadow
}
