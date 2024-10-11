// Copyright 2019 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build !cgo

package main

import (
	"crypto/x509"
	"errors"
)

func loadSystemRoots() (*x509.CertPool, error) {
	return nil, errors.New("can't load system roots: cgo not enabled")
}
