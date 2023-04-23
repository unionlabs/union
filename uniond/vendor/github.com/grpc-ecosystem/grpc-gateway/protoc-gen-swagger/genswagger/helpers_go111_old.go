//+build !go1.12

package genswagger

import "strings"

func fieldName(k string) string {
	return strings.Replace(strings.Title(k), "-", "_", -1)
}
