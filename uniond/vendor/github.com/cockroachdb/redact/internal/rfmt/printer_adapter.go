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

import i "github.com/cockroachdb/redact/interfaces"

// SafeString implements SafePrinter.
func (p *pp) SafeString(s i.SafeString) {
	defer p.startSafeOverride().restore()
	p.buf.WriteString(string(s))
}

// SafeInt implements SafePrinter.
func (p *pp) SafeInt(s i.SafeInt) {
	defer p.startSafeOverride().restore()
	p.fmtInteger(uint64(s), signed, 'd')
}

// SafeUint implements SafePrinter.
func (p *pp) SafeUint(s i.SafeUint) {
	defer p.startSafeOverride().restore()
	p.fmtInteger(uint64(s), unsigned, 'd')
}

// SafeFloat implements SafePrinter.
func (p *pp) SafeFloat(s i.SafeFloat) {
	defer p.startSafeOverride().restore()
	p.fmtFloat(float64(s), 64, 'v')
}

// SafeRune implements SafePrinter.
func (p *pp) SafeRune(r i.SafeRune) {
	defer p.startSafeOverride().restore()
	p.buf.WriteRune(rune(r))
}

func (p *pp) Print(args ...interface{}) {
	defer p.buf.SetMode(p.buf.GetMode())
	np := newPrinter()
	np.buf = p.buf
	np.doPrint(args)
	p.buf = np.buf
	np.buf = buffer{}
	np.free()
}

func (p *pp) Printf(format string, arg ...interface{}) {
	defer p.buf.SetMode(p.buf.GetMode())
	np := newPrinter()
	np.buf = p.buf
	np.doPrintf(format, arg)
	p.buf = np.buf
	np.buf = buffer{}
	np.free()
}

func (p *pp) UnsafeString(s string) {
	defer p.startUnsafe().restore()
	_, _ = p.buf.WriteString(s)
}

func (p *pp) UnsafeByte(bb byte) {
	defer p.startUnsafe().restore()
	_ = p.buf.WriteByte(bb)
}
func (p *pp) UnsafeBytes(bs []byte) {
	defer p.startUnsafe().restore()
	_, _ = p.buf.Write(bs)
}
func (p *pp) UnsafeRune(r rune) {
	defer p.startUnsafe().restore()
	_ = p.buf.WriteRune(r)
}
