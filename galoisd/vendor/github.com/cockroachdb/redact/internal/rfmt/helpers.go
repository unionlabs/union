// Copyright 2021 The Cockroach Authors.
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

package rfmt

import (
	"reflect"

	i "github.com/cockroachdb/redact/interfaces"
	b "github.com/cockroachdb/redact/internal/buffer"
	"github.com/cockroachdb/redact/internal/escape"
	m "github.com/cockroachdb/redact/internal/markers"
	rwrap "github.com/cockroachdb/redact/internal/redact"
)

type overrideMode int

const (
	noOverride overrideMode = iota
	// Consider unsafe values as safe.
	overrideSafe
	// Consider safe and pre-redactable values as unsafe.
	overrideUnsafe
)

func (p *pp) startUnsafe() restorer {
	prevMode := p.buf.GetMode()
	if p.override != overrideSafe {
		p.buf.SetMode(b.UnsafeEscaped)
	}
	return restorer{p, prevMode, p.override}
}

func (p *pp) startPreRedactable() restorer {
	prevMode := p.buf.GetMode()
	if p.override != overrideUnsafe {
		p.buf.SetMode(b.PreRedactable)
	}
	return restorer{p, prevMode, p.override}
}

func (p *pp) startSafeOverride() restorer {
	prevMode := p.buf.GetMode()
	prevOverride := p.override
	if p.override == noOverride {
		p.buf.SetMode(b.SafeEscaped)
		p.override = overrideSafe
	}
	return restorer{p, prevMode, prevOverride}
}

func (p *pp) startUnsafeOverride() restorer {
	prevMode := p.buf.GetMode()
	prevOverride := p.override
	if p.override == noOverride {
		p.buf.SetMode(b.UnsafeEscaped)
		p.override = overrideUnsafe
	}
	return restorer{p, prevMode, prevOverride}
}

type restorer struct {
	p            *pp
	prevMode     b.OutputMode
	prevOverride overrideMode
}

func (r restorer) restore() {
	r.p.buf.SetMode(r.prevMode)
	r.p.override = r.prevOverride
}

func (p *pp) handleSpecialValues(
	value reflect.Value, t reflect.Type, verb rune, depth int,
) (handled bool) {
	switch t {
	case safeWrapperType:
		handled = true
		defer p.startSafeOverride().restore()
		p.printValue(value.Field(0), verb, depth+1)

	case unsafeWrapperType:
		handled = true
		defer p.startUnsafeOverride().restore()
		p.printValue(value.Field(0), verb, depth+1)

	case redactableStringType:
		handled = true
		defer p.startPreRedactable().restore()
		p.buf.WriteString(value.String())

	case redactableBytesType:
		handled = true
		defer p.startPreRedactable().restore()
		p.buf.Write(value.Bytes())
	}

	return handled
}

// Sprintfn produces a RedactableString using the provided
// SafeFormat-alike function.
func Sprintfn(printer func(w i.SafePrinter)) m.RedactableString {
	p := newPrinter()
	printer(p)
	s := p.buf.TakeRedactableString()
	p.free()
	return s
}

// HelperForErrorf is a helper to implement a redaction-aware
// fmt.Errorf-compatible function in a different package. It formats
// the string according to the given format and arguments in the same
// way as Sprintf, but in addition to this if the format contains %w
// and an error object in the proper argument position it also returns
// that error object.
//
// Note: This function only works if an error redaction function
// has been injected with RegisterRedactErrorFn().
func HelperForErrorf(format string, args ...interface{}) (m.RedactableString, error) {
	p := newPrinter()
	p.wrapErrs = true
	p.doPrintf(format, args)
	e := p.wrappedErr
	s := p.buf.TakeRedactableString()
	p.free()
	return s, e
}

// EscapeBytes escapes markers inside the given byte slice and encloses
// the entire byte slice between redaction markers.
// EscapeBytes escapes markers inside the given byte slice and encloses
// the entire byte slice between redaction markers.
func EscapeBytes(s []byte) m.RedactableBytes {
	buf := make([]byte, 0, len(s)+len(m.StartS)+len(m.EndS))
	buf = append(buf, m.StartS...)
	start := len(buf)
	buf = append(buf, s...)
	buf = escape.InternalEscapeBytes(buf, start, true /* breakNewLine */, false /* strip */)
	buf = append(buf, m.EndS...)
	return m.RedactableBytes(buf)
}

var (
	unsafeWrapperType    = reflect.TypeOf(rwrap.UnsafeWrap{})
	safeWrapperType      = reflect.TypeOf(rwrap.SafeWrapper{})
	redactableStringType = reflect.TypeOf(m.RedactableString(""))
	redactableBytesType  = reflect.TypeOf(m.RedactableBytes{})
)
