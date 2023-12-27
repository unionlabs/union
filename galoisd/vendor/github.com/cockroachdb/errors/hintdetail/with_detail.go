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

type withDetail struct {
	cause  error
	detail string
}

var _ error = (*withDetail)(nil)
var _ ErrorDetailer = (*withDetail)(nil)
var _ fmt.Formatter = (*withDetail)(nil)
var _ errbase.Formatter = (*withDetail)(nil)

func (w *withDetail) ErrorDetail() string { return w.detail }
func (w *withDetail) Error() string       { return w.cause.Error() }
func (w *withDetail) Cause() error        { return w.cause }
func (w *withDetail) Unwrap() error       { return w.cause }

func (w *withDetail) Format(s fmt.State, verb rune) { errbase.FormatError(w, s, verb) }

func (w *withDetail) FormatError(p errbase.Printer) error {
	if p.Detail() {
		p.Print(w.detail)
	}
	return w.cause
}

func encodeWithDetail(_ context.Context, err error) (string, []string, proto.Message) {
	w := err.(*withDetail)
	return "", nil, &errorspb.StringPayload{Msg: w.detail}
}

func decodeWithDetail(
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
	return &withDetail{cause: cause, detail: m.Msg}
}

func init() {
	errbase.RegisterWrapperEncoder(errbase.GetTypeKey((*withDetail)(nil)), encodeWithDetail)
	errbase.RegisterWrapperDecoder(errbase.GetTypeKey((*withDetail)(nil)), decodeWithDetail)
}
