//Copyright 2018 Improbable. All Rights Reserved.
// See LICENSE for licensing terms.

package grpcweb

import (
	"net/http"
	"strings"
)

// gRPC-Web spec says that must use lower-case header/trailer names.
// See "HTTP wire protocols" section in
// https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-WEB.md#protocol-differences-vs-grpc-over-http2
type trailer struct {
	http.Header
}

func (t trailer) Add(key, value string) {
	key = strings.ToLower(key)
	t.Header[key] = append(t.Header[key], value)
}

func (t trailer) Get(key string) string {
	if t.Header == nil {
		return ""
	}
	v := t.Header[key]
	if len(v) == 0 {
		return ""
	}
	return v[0]
}
