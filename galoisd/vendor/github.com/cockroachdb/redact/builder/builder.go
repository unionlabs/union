// Copyright 2020 The Cockroach Authors.
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

package builder

import (
	"fmt"
	"io"

	i "github.com/cockroachdb/redact/interfaces"
	ib "github.com/cockroachdb/redact/internal/buffer"
	ifmt "github.com/cockroachdb/redact/internal/rfmt"
)

// StringBuilder accumulates strings with optional redaction markers.
//
// It implements io.Writer but marks direct writes as redactable.
// To distinguish safe and unsafe bits, it also implements the SafeWriter
// interface.
type StringBuilder struct {
	ib.Buffer
}

var _ fmt.Stringer = StringBuilder{}
var _ fmt.Stringer = (*StringBuilder)(nil)

// SafeFormat implements SafeFormatter.
func (b StringBuilder) SafeFormat(p i.SafePrinter, _ rune) {
	// We only support the %v / %s natural print here.
	// Go supports other formatting verbs for strings: %x/%X/%q.
	//
	// We don't do this here, keeping in mind that the output
	// of a SafeFormat must remain a redactable string.
	//
	// %x/%X cannot be implemented because they would turn redaction
	//       markers into hex codes, and the entire result string would
	//       appear safe for reporting, which would break the semantics
	//       of this package.
	//
	// %q    cannot be implemented because it replaces non-ASCII characters
	//       with numeric unicode escapes, which breaks redaction
	//       markers too.
	p.Print(b.RedactableString())
}

var _ i.SafeFormatter = StringBuilder{}
var _ i.SafeFormatter = (*StringBuilder)(nil)

// StringBuilder implements io.Writer.
// Direct Write() calls are considered unsafe.
var _ io.Writer = (*StringBuilder)(nil)

// Write implements the io.Writer interface.
func (b *StringBuilder) Write(s []byte) (int, error) {
	b.SetMode(ib.UnsafeEscaped)
	return b.Buffer.Write(s)
}

// WriteString shadows the ib.Buffer.WriteString method.
func (b *StringBuilder) WriteString(s string) (int, error) {
	b.SetMode(ib.UnsafeEscaped)
	return b.Buffer.WriteString(s)
}

// WriteByte shadows the ib.Buffer.WriteByte method.
func (b *StringBuilder) WriteByte(c byte) error {
	b.SetMode(ib.UnsafeEscaped)
	return b.Buffer.WriteByte(c)
}

// WriteRune shadows the ib.Buffer.WriteRune method.
func (b *StringBuilder) WriteRune(r rune) error {
	b.SetMode(ib.UnsafeEscaped)
	return b.Buffer.WriteRune(r)
}

// StringBuilder implements SafeWriter.
var _ i.SafeWriter = (*StringBuilder)(nil)

// Print is part of the SafeWriter interface.
func (b *StringBuilder) Print(args ...interface{}) {
	b.SetMode(ib.PreRedactable)
	_, _ = ifmt.Fprint(&b.Buffer, args...)
}

// Printf is part of the SafeWriter interface.
func (b *StringBuilder) Printf(format string, args ...interface{}) {
	b.SetMode(ib.PreRedactable)
	_, _ = ifmt.Fprintf(&b.Buffer, format, args...)
}

// SafeString is part of the SafeWriter interface.
func (b *StringBuilder) SafeString(s i.SafeString) {
	b.SetMode(ib.SafeEscaped)
	_, _ = b.Buffer.WriteString(string(s))
}

// SafeInt is part of the SafeWriter interface.
func (b *StringBuilder) SafeInt(s i.SafeInt) {
	b.SetMode(ib.SafeEscaped)
	_, _ = ifmt.Fprintf(&b.Buffer, "%d", s)
}

// SafeUint is part of the SafeWriter interface.
func (b *StringBuilder) SafeUint(s i.SafeUint) {
	b.SetMode(ib.SafeEscaped)
	_, _ = ifmt.Fprintf(&b.Buffer, "%d", s)
}

// SafeFloat is part of the SafeWriter interface.
func (b *StringBuilder) SafeFloat(s i.SafeFloat) {
	b.SetMode(ib.SafeEscaped)
	_, _ = ifmt.Fprintf(&b.Buffer, "%v", s)
}

// SafeRune is part of the SafeWriter interface.
func (b *StringBuilder) SafeRune(s i.SafeRune) {
	b.SetMode(ib.SafeEscaped)
	_ = b.Buffer.WriteRune(rune(s))
}

// UnsafeString is part of the SafeWriter interface.
func (b *StringBuilder) UnsafeString(s string) {
	b.SetMode(ib.UnsafeEscaped)
	_, _ = b.Buffer.WriteString(s)
}

// UnsafeRune is part of the SafeWriter interface.
func (b *StringBuilder) UnsafeRune(s rune) {
	b.SetMode(ib.UnsafeEscaped)
	_ = b.Buffer.WriteRune(s)
}

// UnsafeByte is part of the SafeWriter interface.
func (b *StringBuilder) UnsafeByte(s byte) {
	b.SetMode(ib.UnsafeEscaped)
	_ = b.Buffer.WriteByte(s)
}

// UnsafeBytes is part of the SafeWriter interface.
func (b *StringBuilder) UnsafeBytes(s []byte) {
	b.SetMode(ib.UnsafeEscaped)
	_, _ = b.Buffer.Write(s)
}
