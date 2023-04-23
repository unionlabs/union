//Copyright 2018 Improbable. All Rights Reserved.
// See LICENSE for licensing terms.

package grpcweb

import (
	"net/http"
	"strings"
)

// replacer is the function that replaces the key and the slice of strings
// per the header item. This function returns false as the last return value
// if neither the key nor the slice were replaced.
type replacer func(key string, vv []string) (string, []string, bool)

// copyOptions acts as a storage for copyHeader options.
type copyOptions struct {
	skipKeys  map[string]bool
	replacers []replacer
}

// copyOption is the option type to pass to copyHeader function.
type copyOption func(*copyOptions)

// skipKeys returns an option to skip specified keys when copying headers
// with copyHeader function. Key matching in the source header is
// case-insensitive.
func skipKeys(keys ...string) copyOption {
	return func(opts *copyOptions) {
		if opts.skipKeys == nil {
			opts.skipKeys = make(map[string]bool)
		}
		for _, k := range keys {
			// normalize the key
			opts.skipKeys[strings.ToLower(k)] = true
		}
	}
}

// replaceInVals returns an option to replace old substring with new substring
// in header values keyed with key. Key matching in the header is
// case-insensitive.
func replaceInVals(key, old, new string) copyOption {
	return func(opts *copyOptions) {
		opts.replacers = append(
			opts.replacers,
			func(k string, vv []string) (string, []string, bool) {
				if strings.ToLower(key) == strings.ToLower(k) {
					vv2 := make([]string, 0, len(vv))
					for _, v := range vv {
						vv2 = append(
							vv2,
							strings.Replace(v, old, new, 1),
						)
					}
					return k, vv2, true
				}
				return "", nil, false
			},
		)
	}
}

// replaceInKeys returns an option to replace an old substring with a new
// substring in header keys.
func replaceInKeys(old, new string) copyOption {
	return func(opts *copyOptions) {
		opts.replacers = append(
			opts.replacers,
			func(k string, vv []string) (string, []string, bool) {
				if strings.Contains(k, old) {
					return strings.Replace(k, old, new, 1), vv, true
				}
				return "", nil, false
			},
		)
	}
}

// keyCase returns an option to unconditionally modify the case of the
// destination header keys with function fn. Typically fn can be
// strings.ToLower, strings.ToUpper, http.CanonicalHeaderKey
func keyCase(fn func(string) string) copyOption {
	return func(opts *copyOptions) {
		opts.replacers = append(
			opts.replacers,
			func(k string, vv []string) (string, []string, bool) {
				return fn(k), vv, true
			},
		)
	}
}

// keyTrim returns an option to unconditionally trim the keys of the
// destination header with function fn. Typically fn can be
// strings.Trim, strings.TrimLeft/TrimRight, strings.TrimPrefix/TrimSuffix
func keyTrim(fn func(string, string) string, cut string) copyOption {
	return func(opts *copyOptions) {
		opts.replacers = append(
			opts.replacers,
			func(k string, vv []string) (string, []string, bool) {
				return fn(k, cut), vv, true
			},
		)
	}
}

// copyHeader copies src to dst header. This function does not uses http.Header
// methods internally, so header keys are copied as is. If any key normalization
// is required, use keyCase option.
func copyHeader(
	dst, src http.Header,
	opts ...copyOption,
) {
	options := new(copyOptions)
	for _, opt := range opts {
		opt(options)
	}

	for k, vv := range src {
		if options.skipKeys[strings.ToLower(k)] {
			continue
		}
		for _, r := range options.replacers {
			if k2, vv2, ok := r(k, vv); ok {
				k, vv = k2, vv2
			}
		}
		dst[k] = vv
	}
}

// headerKeys returns a slice of strings representing the keys in the header h.
func headerKeys(h http.Header) []string {
	keys := make([]string, 0, len(h))
	for k := range h {
		keys = append(keys, k)
	}
	return keys
}
