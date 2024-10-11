// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.19 && !go1.20

package slogtest

import (
	"errors"
	"strings"
)

func errorsJoin(errs ...error) error {
	var b strings.Builder
	for _, err := range errs {
		if err != nil {
			b.WriteString(err.Error())
			b.WriteByte('\n')
		}
	}
	s := b.String()
	if len(s) == 0 {
		return nil
	}
	return errors.New(s)
}
