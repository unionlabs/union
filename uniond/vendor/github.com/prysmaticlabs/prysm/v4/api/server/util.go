package server

import (
	"net/url"
	"strings"
)

// NormalizeQueryValues replaces comma-separated values with individual values
func NormalizeQueryValues(queryParams url.Values) {
	for key, vals := range queryParams {
		splitVals := make([]string, 0)
		for _, v := range vals {
			splitVals = append(splitVals, strings.Split(v, ",")...)
		}
		queryParams[key] = splitVals
	}
}
