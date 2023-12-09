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

package errbase

import (
	"fmt"

	"github.com/cockroachdb/errors/errorspb"
	"github.com/cockroachdb/redact"
)

// opaqueLeaf is used when receiving an unknown leaf type.
// Its important property is that if it is communicated
// back to some network system that _does_ know about
// the type, the original object can be restored.
type opaqueLeaf struct {
	msg     string
	details errorspb.EncodedErrorDetails
}

// opaqueLeafCauses is used when receiving an unknown multi-cause
// wrapper type. Its important property is that if it is communicated
// back to some network system that _does_ know about the type, the
// original object can be restored. We encode multi-cause errors as
// leaf nodes over the network, in order to support backwards
// compatibility with existing single-cause wrapper messages.
//
// This struct *must* be initialized with a non-nil causes value in
// order to comply with go stdlib expectations for `Unwrap()`.
type opaqueLeafCauses struct {
	opaqueLeaf
	causes []error
}

var _ error = (*opaqueLeaf)(nil)
var _ SafeDetailer = (*opaqueLeaf)(nil)
var _ fmt.Formatter = (*opaqueLeaf)(nil)
var _ SafeFormatter = (*opaqueLeaf)(nil)

var _ error = (*opaqueLeafCauses)(nil)
var _ SafeDetailer = (*opaqueLeafCauses)(nil)
var _ fmt.Formatter = (*opaqueLeafCauses)(nil)
var _ SafeFormatter = (*opaqueLeafCauses)(nil)

// opaqueWrapper is used when receiving an unknown wrapper type.
// Its important property is that if it is communicated
// back to some network system that _does_ know about
// the type, the original object can be restored.
type opaqueWrapper struct {
	cause       error
	prefix      string
	details     errorspb.EncodedErrorDetails
	messageType MessageType
}

var _ error = (*opaqueWrapper)(nil)
var _ SafeDetailer = (*opaqueWrapper)(nil)
var _ fmt.Formatter = (*opaqueWrapper)(nil)
var _ SafeFormatter = (*opaqueWrapper)(nil)

func (e *opaqueLeaf) Error() string { return e.msg }

func (e *opaqueWrapper) Error() string {
	if e.messageType == FullMessage {
		return e.prefix
	}
	if e.prefix == "" {
		return e.cause.Error()
	}
	return fmt.Sprintf("%s: %s", e.prefix, e.cause)
}

// the opaque wrapper is a wrapper.
func (e *opaqueWrapper) Cause() error  { return e.cause }
func (e *opaqueWrapper) Unwrap() error { return e.cause }

func (e *opaqueLeaf) SafeDetails() []string    { return e.details.ReportablePayload }
func (e *opaqueWrapper) SafeDetails() []string { return e.details.ReportablePayload }

func (e *opaqueLeaf) Format(s fmt.State, verb rune)       { FormatError(e, s, verb) }
func (e *opaqueLeafCauses) Format(s fmt.State, verb rune) { FormatError(e, s, verb) }
func (e *opaqueWrapper) Format(s fmt.State, verb rune)    { FormatError(e, s, verb) }

// opaqueLeafCauses is a multi-cause wrapper
func (e *opaqueLeafCauses) Unwrap() []error { return e.causes }

func (e *opaqueLeaf) SafeFormatError(p Printer) (next error) {
	p.Print(e.msg)
	if p.Detail() {
		p.Printf("\n(opaque error leaf)")
		p.Printf("\ntype name: %s", redact.Safe(e.details.OriginalTypeName))
		for i, d := range e.details.ReportablePayload {
			p.Printf("\nreportable %d:\n%s", redact.Safe(i), redact.Safe(d))
		}
		if e.details.FullDetails != nil {
			p.Printf("\npayload type: %s", redact.Safe(e.details.FullDetails.TypeUrl))
		}
	}
	return nil
}

func (e *opaqueWrapper) SafeFormatError(p Printer) (next error) {
	if len(e.prefix) > 0 {
		// We use the condition if len(msg) > 0 because
		// otherwise an empty string would cause a "redactable
		// empty string" to be emitted (something that looks like "<>")
		// and the error formatting code only cleanly elides
		// the prefix properly if the output string is completely empty.
		p.Print(e.prefix)
	}
	if p.Detail() {
		p.Printf("\n(opaque error wrapper)")
		p.Printf("\ntype name: %s", redact.Safe(e.details.OriginalTypeName))
		for i, d := range e.details.ReportablePayload {
			p.Printf("\nreportable %d:\n%s", redact.Safe(i), redact.Safe(d))
		}
		if e.details.FullDetails != nil {
			p.Printf("\npayload type: %s", redact.Safe(e.details.FullDetails.TypeUrl))
		}
	}
	if e.messageType == FullMessage {
		return nil
	}
	return e.cause
}
