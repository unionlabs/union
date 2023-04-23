//go:build !go1.12
// +build !go1.12

package genopenapi

import "strings"

func fieldName(k string) string {
	return strings.Replace(strings.Title(k), "-", "_", -1)
}
