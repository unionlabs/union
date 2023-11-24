// Copyright 2019 The Cockroach Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License.

package join

import (
	"context"
	"fmt"

	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/redact"
	"github.com/gogo/protobuf/proto"
)

// Join returns an error that wraps the given errors.
// Any nil error values are discarded.
// Join returns nil if errs contains no non-nil values.
// The error formats as the concatenation of the strings obtained
// by calling the Error method of each element of errs, with a newline
// between each string.
func Join(errs ...error) error {
	n := 0
	for _, err := range errs {
		if err != nil {
			n++
		}
	}
	if n == 0 {
		return nil
	}
	e := &joinError{
		errs: make([]error, 0, n),
	}
	for _, err := range errs {
		if err != nil {
			e.errs = append(e.errs, err)
		}
	}
	return e
}

type joinError struct {
	errs []error
}

var _ error = (*joinError)(nil)
var _ fmt.Formatter = (*joinError)(nil)
var _ errbase.SafeFormatter = (*joinError)(nil)

func (e *joinError) Error() string {
	return redact.Sprint(e).StripMarkers()
}

func (e *joinError) Unwrap() []error {
	return e.errs
}

func (e *joinError) SafeFormatError(p errbase.Printer) error {
	for i, err := range e.errs {
		if i > 0 {
			p.Print("\n")
		}
		p.Print(err)
	}
	return nil
}

func (e *joinError) Format(s fmt.State, verb rune) {
	errbase.FormatError(e, s, verb)
}

func init() {
	errbase.RegisterMultiCauseEncoder(
		errbase.GetTypeKey(&joinError{}),
		func(
			ctx context.Context,
			err error,
		) (msg string, safeDetails []string, payload proto.Message) {
			return "", nil, nil
		},
	)
	errbase.RegisterMultiCauseDecoder(
		errbase.GetTypeKey(&joinError{}),
		func(
			ctx context.Context,
			causes []error,
			msgPrefix string,
			safeDetails []string,
			payload proto.Message,
		) error {
			return Join(causes...)
		},
	)
}
