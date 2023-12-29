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

package hintdetail

import (
	"context"
	"fmt"

	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/errorspb"
	"github.com/gogo/protobuf/proto"
)

type withHint struct {
	cause error
	hint  string
}

var _ error = (*withHint)(nil)
var _ ErrorHinter = (*withHint)(nil)
var _ fmt.Formatter = (*withHint)(nil)
var _ errbase.Formatter = (*withHint)(nil)

func (w *withHint) ErrorHint() string { return w.hint }
func (w *withHint) Error() string     { return w.cause.Error() }
func (w *withHint) Cause() error      { return w.cause }
func (w *withHint) Unwrap() error     { return w.cause }

func (w *withHint) Format(s fmt.State, verb rune) { errbase.FormatError(w, s, verb) }

func (w *withHint) FormatError(p errbase.Printer) error {
	if p.Detail() {
		p.Print(w.hint)
	}
	return w.cause
}

func encodeWithHint(_ context.Context, err error) (string, []string, proto.Message) {
	w := err.(*withHint)
	return "", nil, &errorspb.StringPayload{Msg: w.hint}
}

func decodeWithHint(
	_ context.Context, cause error, _ string, _ []string, payload proto.Message,
) error {
	m, ok := payload.(*errorspb.StringPayload)
	if !ok {
		// If this ever happens, this means some version of the library
		// (presumably future) changed the payload type, and we're
		// receiving this here. In this case, give up and let
		// DecodeError use the opaque type.
		return nil
	}
	return &withHint{cause: cause, hint: m.Msg}
}

func init() {
	errbase.RegisterWrapperEncoder(errbase.GetTypeKey((*withHint)(nil)), encodeWithHint)
	errbase.RegisterWrapperDecoder(errbase.GetTypeKey((*withHint)(nil)), decodeWithHint)
}
