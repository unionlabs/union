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

package barriers

import (
	"context"
	"fmt"

	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/redact"
	"github.com/gogo/protobuf/proto"
)

// Handled swallows the provided error and hides it from the
// Cause()/Unwrap() interface, and thus the Is() facility that
// identifies causes. However, it retains it for the purpose of
// printing the error out (e.g. for troubleshooting). The error
// message is preserved in full.
//
// Detail is shown:
// - via `errors.GetSafeDetails()`, shows details from hidden error.
// - when formatting with `%+v`.
// - in Sentry reports.
func Handled(err error) error {
	if err == nil {
		return nil
	}
	return HandledWithSafeMessage(err, redact.Sprint(err))
}

// HandledWithMessage is like Handled except the message is overridden.
// This can be used e.g. to hide message details or to prevent
// downstream code to make assertions on the message's contents.
func HandledWithMessage(err error, msg string) error {
	if err == nil {
		return nil
	}
	return HandledWithSafeMessage(err, redact.Sprint(msg))
}

// HandledWithSafeMessage is like Handled except the message is overridden.
// This can be used e.g. to hide message details or to prevent
// downstream code to make assertions on the message's contents.
func HandledWithSafeMessage(err error, msg redact.RedactableString) error {
	return &barrierErr{maskedErr: err, smsg: msg}
}

// HandledWithMessagef is like HandledWithMessagef except the message
// is formatted.
func HandledWithMessagef(err error, format string, args ...interface{}) error {
	if err == nil {
		return nil
	}
	return &barrierErr{maskedErr: err, smsg: redact.Sprintf(format, args...)}
}

// barrierErr is a leaf error type. It encapsulates a chain of
// original causes, but these causes are hidden so that they inhibit
// matching via Is() and the Cause()/Unwrap() recursions.
type barrierErr struct {
	// Message for the barrier itself.
	// In the common case, the message from the masked error
	// is used as-is (see Handled() above) however it is
	// useful to cache it here since the masked error may
	// have a long chain of wrappers and its Error() call
	// may be expensive.
	smsg redact.RedactableString
	// Masked error chain.
	maskedErr error
}

var _ error = (*barrierErr)(nil)
var _ errbase.SafeDetailer = (*barrierErr)(nil)
var _ errbase.SafeFormatter = (*barrierErr)(nil)
var _ fmt.Formatter = (*barrierErr)(nil)

// barrierErr is an error.
func (e *barrierErr) Error() string { return e.smsg.StripMarkers() }

// SafeDetails reports the PII-free details from the masked error.
func (e *barrierErr) SafeDetails() []string {
	var details []string
	for err := e.maskedErr; err != nil; err = errbase.UnwrapOnce(err) {
		sd := errbase.GetSafeDetails(err)
		details = sd.Fill(details)
	}
	details = append(details, redact.Sprintf("masked error: %+v", e.maskedErr).Redact().StripMarkers())
	return details
}

// Printing a barrier reveals the details.
func (e *barrierErr) Format(s fmt.State, verb rune) { errbase.FormatError(e, s, verb) }

func (e *barrierErr) SafeFormatError(p errbase.Printer) (next error) {
	p.Print(e.smsg)
	if p.Detail() {
		p.Printf("-- cause hidden behind barrier\n%+v", e.maskedErr)
	}
	return nil
}

// A barrier error is encoded exactly.
func encodeBarrier(
	ctx context.Context, err error,
) (msg string, details []string, payload proto.Message) {
	e := err.(*barrierErr)
	enc := errbase.EncodeError(ctx, e.maskedErr)
	return string(e.smsg), e.SafeDetails(), &enc
}

// A barrier error is decoded exactly.
func decodeBarrier(ctx context.Context, msg string, _ []string, payload proto.Message) error {
	enc := payload.(*errbase.EncodedError)
	return &barrierErr{smsg: redact.RedactableString(msg), maskedErr: errbase.DecodeError(ctx, *enc)}
}

// Previous versions of barrier errors.
func decodeBarrierPrev(ctx context.Context, msg string, _ []string, payload proto.Message) error {
	enc := payload.(*errbase.EncodedError)
	return &barrierErr{smsg: redact.Sprint(msg), maskedErr: errbase.DecodeError(ctx, *enc)}
}

// barrierError is the "old" type name of barrierErr. We use a new
// name now to ensure a different decode function is used when
// importing barriers from the previous structure, where the
// message is not redactable.
type barrierError struct {
	msg       string
	maskedErr error
}

func (b *barrierError) Error() string { return "" }

func init() {
	errbase.RegisterLeafDecoder(errbase.GetTypeKey((*barrierError)(nil)), decodeBarrierPrev)
	tn := errbase.GetTypeKey((*barrierErr)(nil))
	errbase.RegisterLeafDecoder(tn, decodeBarrier)
	errbase.RegisterLeafEncoder(tn, encodeBarrier)
}
