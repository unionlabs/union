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

package buffer

import (
	"fmt"
	"testing"
	"unicode/utf8"

	m "github.com/cockroachdb/redact/internal/markers"
)

func (b *Buffer) checkInvariants(t *testing.T) {
	t.Helper()
	if !utf8.Valid(b.buf[:b.validUntil]) {
		t.Errorf("invalid utf-8 sequence in string: %q", string(b.buf[:b.validUntil]))
	}

	copy := *b
	copy.finalize()
	checkBalanced(t, string(copy.buf))
}

func checkBalanced(t *testing.T, s string) {
	t.Helper()
	open := false
	for i, c := range s {
		if c == m.Start {
			if open {
				t.Errorf("unexpected open marker at position %d: %q", i, s)
				return
			}
			open = true
		} else if c == m.End {
			if !open {
				t.Errorf("unexpected close marker at position %d: %q", i, s)
				return
			}
			open = false
		}
	}
	if open {
		t.Errorf("missing close marker at end of string: %q", s)
	}
}

func (b *Buffer) checkEqual(t *testing.T, expected string) {
	t.Helper()
	if actual := b.RedactableString(); string(actual) != expected {
		t.Errorf("redactable string expected:\n%s\ngot:\n%s", expected, actual)
	}
	if actual := b.RedactableBytes(); string(actual) != expected {
		t.Errorf("redactable bytes expected:\n%s\ngot:\n%s", expected, actual)
	}
}

func TestBufferUnsafeWrites(t *testing.T) {
	var b Buffer

	// Empty buffer is valid.
	b.checkInvariants(t)
	b.checkEqual(t, ``)

	// Adding empty stuff to a buffer keeps it valid and empty.
	b.WriteString("")
	b.checkInvariants(t)
	b.checkEqual(t, ``)

	b.Write(nil)
	b.checkInvariants(t)
	b.checkEqual(t, ``)

	// Simple unsafe string.
	b.WriteString("hello")
	b.checkInvariants(t)
	b.checkEqual(t, `‹hello›`)

	// Adding unsafe strings only adds a marker
	// at the end.
	b.WriteString("world")
	b.checkInvariants(t)
	b.checkEqual(t, `‹helloworld›`)

	// After a reset, the buffer is valid and empty.
	b.Reset()
	b.checkInvariants(t)
	b.checkEqual(t, ``)

	// Prepopulate some string.
	b.WriteString("hello")
	b.checkInvariants(t)

	// If we switch modes to safe and back to unsafe, the buffer
	// is still valid.
	b.SetMode(SafeRaw)
	b.checkInvariants(t)
	b.SetMode(UnsafeEscaped)
	b.checkInvariants(t)

	// Adding strings after a mode switch is valid.
	// The mode switch may have added markers.
	b.WriteString("world")
	b.checkInvariants(t)
	b.checkEqual(t, `‹helloworld›`)

	b.WriteString("\nuniverse")
	b.checkInvariants(t)
	b.checkEqual(t, `‹helloworld›
‹universe›`)

	// Newline characters as bytes or runes.
	b.Reset()
	b.WriteByte('a')
	b.WriteByte('\n')
	b.WriteByte('\n')
	b.WriteByte('b')
	b.WriteRune('\n')
	b.WriteRune('\n')
	b.WriteByte('c')
	b.WriteString("\n")
	b.WriteString("\n")
	b.WriteByte('d')
	b.checkEqual(t, "‹a›\n\n‹b›\n\n‹c›\n\n‹d›")

	// Valid runes.
	b.Reset()
	b.WriteRune('䬦')
	b.checkInvariants(t)
	b.checkEqual(t, `‹䬦›`)

	// Adding the marker runs in unsafe mode causes them to be escaped.
	b.Reset()
	b.WriteRune('‹')
	b.checkInvariants(t)
	b.checkEqual(t, `‹?›`)
	b.WriteRune('›')
	b.checkInvariants(t)
	b.checkEqual(t, `‹??›`)

	for _, seq := range [][]byte{m.StartBytes, m.EndBytes} {
		t.Run(fmt.Sprintf("%q", seq), func(t *testing.T) {
			// Using invalid utf-8 sequences that are the prefix to a marker
			// rune at the end of a string cause an escape marker to be
			// introduced afterwards.
			b.Reset()
			b.WriteByte(seq[0])
			b.checkInvariants(t)
			b.checkEqual(t, `‹?›`)

			b.Reset()
			b.Write(seq[:1])
			b.checkInvariants(t)
			b.checkEqual(t, "‹\342?›")

			b.Reset()
			b.WriteByte(seq[0])
			b.WriteByte(seq[1])
			b.checkInvariants(t)
			b.checkEqual(t, `‹??›`)

			b.Reset()
			b.Write(seq[:2])
			b.checkInvariants(t)
			b.checkEqual(t, "‹\342\200?›")

			// Invalid utf-8 sequences that are followed with
			// valid utf-8 bytes do not need an escape marker.
			b.Reset()
			b.Write(seq[:2])
			b.WriteString("hello")
			b.checkInvariants(t)
			b.checkEqual(t, "‹\342\200hello›")
		})
	}

	// Generic invalid utf-8 sequences.
	b.Reset()
	b.Write([]byte{0xe4, 0xac}) // prefix to 䬦
	b.checkInvariants(t)
	b.checkEqual(t, "‹\344\254?›")
	b.Reset()
	b.Write([]byte{0xe4, 0xac}) // prefix to 䬦
	b.WriteString("hello")
	b.checkInvariants(t)
	b.checkEqual(t, "‹\344\254hello›")
}

func (b *Buffer) resetWithMode(mode OutputMode) {
	b.Reset()
	b.SetMode(mode)
}

func TestBufferEscapedSafeWrites(t *testing.T) {
	var b Buffer

	b.SetMode(SafeEscaped)
	// Empty buffer is valid.
	b.checkInvariants(t)
	b.checkEqual(t, ``)

	// Adding empty stuff to a buffer keeps it valid and empty.
	b.WriteString("")
	b.checkInvariants(t)
	b.checkEqual(t, ``)

	b.Write(nil)
	b.checkInvariants(t)
	b.checkEqual(t, ``)

	// Simple safe string.
	b.WriteString("hello")
	b.checkInvariants(t)
	b.checkEqual(t, `hello`)

	// Adding safe strings.
	b.WriteString("world")
	b.checkInvariants(t)
	b.checkEqual(t, `helloworld`)

	// After a reset, the buffer is valid and empty.
	b.resetWithMode(SafeEscaped)
	b.checkInvariants(t)
	b.checkEqual(t, ``)

	// Prepopulate some string.
	b.WriteString("hello")
	b.checkInvariants(t)

	// If we switch modes to safe and back to unsafe, the buffer
	// is still valid.
	b.SetMode(UnsafeEscaped)
	b.checkInvariants(t)
	b.SetMode(SafeEscaped)
	b.checkInvariants(t)

	// Adding strings after a mode switch is valid.
	// The mode switch may have added markers.
	b.WriteString("world")
	b.checkInvariants(t)
	b.checkEqual(t, `helloworld`)

	b.WriteString("\nuniverse")
	b.checkInvariants(t)
	b.checkEqual(t, `helloworld
universe`)

	// Newline characters as bytes or runes.
	b.resetWithMode(SafeEscaped)
	b.WriteByte('a')
	b.WriteByte('\n')
	b.WriteByte('\n')
	b.WriteByte('b')
	b.WriteRune('\n')
	b.WriteRune('\n')
	b.WriteByte('c')
	b.WriteString("\n")
	b.WriteString("\n")
	b.WriteByte('d')
	b.checkEqual(t, "a\n\nb\n\nc\n\nd")

	// Valid runes.
	b.resetWithMode(SafeEscaped)
	b.WriteRune('䬦')
	b.checkInvariants(t)
	b.checkEqual(t, `䬦`)

	// Adding the marker runs in escaped safe mode causes them to be escaped.
	b.resetWithMode(SafeEscaped)
	b.WriteRune('‹')
	b.checkInvariants(t)
	b.checkEqual(t, `?`)
	b.WriteRune('›')
	b.checkInvariants(t)
	b.checkEqual(t, `??`)

	for _, seq := range [][]byte{m.StartBytes, m.EndBytes} {
		t.Run(fmt.Sprintf("%q", seq), func(t *testing.T) {
			// Using invalid utf-8 sequences that are the prefix to a marker
			// rune at the end of a string cause an escape marker to be
			// introduced afterwards.
			b.resetWithMode(SafeEscaped)
			b.WriteByte(seq[0])
			b.checkInvariants(t)
			b.checkEqual(t, "\342?")

			b.resetWithMode(SafeEscaped)
			b.Write(seq[:1])
			b.checkInvariants(t)
			b.checkEqual(t, "\342?")

			b.resetWithMode(SafeEscaped)
			b.WriteByte(seq[0])
			b.WriteByte(seq[1])
			b.checkInvariants(t)
			b.checkEqual(t, "\342\200?")

			b.resetWithMode(SafeEscaped)
			b.Write(seq[:2])
			b.checkInvariants(t)
			b.checkEqual(t, "\342\200?")

			// Invalid utf-8 sequences that are followed with
			// valid utf-8 bytes do not need an escape marker.
			b.resetWithMode(SafeEscaped)
			b.Write(seq[:2])
			b.WriteString("hello")
			b.checkInvariants(t)
			b.checkEqual(t, "\342\200hello")
		})
	}

	// Generic invalid utf-8 sequences.
	b.resetWithMode(SafeEscaped)
	b.Write([]byte{0xe4, 0xac}) // prefix to 䬦
	b.checkInvariants(t)
	b.checkEqual(t, "\344\254?")
	b.resetWithMode(SafeEscaped)
	b.Write([]byte{0xe4, 0xac}) // prefix to 䬦
	b.WriteString("hello")
	b.checkInvariants(t)
	b.checkEqual(t, "\344\254hello")
}

func TestBufferDelimiterElision(t *testing.T) {
	var b Buffer

	b.SetMode(SafeRaw)
	b.WriteString("safe‹unsafe›")

	b.checkInvariants(t)
	copy := b
	copy.checkEqual(t, `safe‹unsafe›`)

	b.SetMode(UnsafeEscaped)
	b.checkInvariants(t)
	copy = b
	copy.checkEqual(t, `safe‹unsafe›`)

	b.WriteByte('a')
	b.checkInvariants(t)
	copy = b
	copy.checkEqual(t, `safe‹unsafea›`)
}

func TestBufferRawSafeWrites(t *testing.T) {
	var b Buffer

	b.SetMode(SafeRaw)
	// Empty buffer is valid.
	b.checkInvariants(t)
	b.checkEqual(t, ``)

	// Adding empty stuff to a buffer keeps it valid and empty.
	b.WriteString("")
	b.checkInvariants(t)
	b.checkEqual(t, ``)

	b.Write(nil)
	b.checkInvariants(t)
	b.checkEqual(t, ``)

	// Simple safe string.
	b.WriteString("hello")
	b.checkInvariants(t)
	b.checkEqual(t, `hello`)

	// Adding safe strings.
	b.WriteString("world")
	b.checkInvariants(t)
	b.checkEqual(t, `helloworld`)

	// After a reset, the buffer is valid and empty.
	b.resetWithMode(SafeRaw)
	b.checkInvariants(t)
	b.checkEqual(t, ``)

	// Prepopulate some string.
	b.WriteString("hello")
	b.checkInvariants(t)

	// If we switch modes to safe and back to unsafe, the buffer
	// is still valid.
	b.SetMode(UnsafeEscaped)
	b.checkInvariants(t)
	b.SetMode(SafeRaw)
	b.checkInvariants(t)

	// Adding strings after a mode switch is valid.
	// The mode switch may have added markers.
	b.WriteString("world")
	b.checkInvariants(t)
	b.checkEqual(t, `helloworld`)

	b.WriteString("\nuniverse")
	b.checkInvariants(t)
	b.checkEqual(t, `helloworld
universe`)

	// Newline characters as bytes or runes.
	b.resetWithMode(SafeRaw)
	b.WriteByte('a')
	b.WriteByte('\n')
	b.WriteByte('\n')
	b.WriteByte('b')
	b.WriteRune('\n')
	b.WriteRune('\n')
	b.WriteByte('c')
	b.WriteString("\n")
	b.WriteString("\n")
	b.WriteByte('d')
	b.checkEqual(t, "a\n\nb\n\nc\n\nd")

	// Valid runes.
	b.resetWithMode(SafeRaw)
	b.WriteRune('䬦')
	b.checkInvariants(t)
	b.checkEqual(t, `䬦`)

	// Adding the marker runs in escaped safe mode causes them to be escaped.
	b.resetWithMode(SafeRaw)
	b.WriteRune('‹')
	// Cannot check invariants here because we have purposefully
	// a stray open marker.
	// b.checkInvariants(t)
	b.checkEqual(t, `‹`)
	b.WriteRune('›')
	b.checkInvariants(t)
	b.checkEqual(t, `‹›`)

	for _, seq := range [][]byte{m.StartBytes, m.EndBytes} {
		t.Run(fmt.Sprintf("%q", seq), func(t *testing.T) {
			// Using invalid utf-8 sequences that are the prefix to a marker
			// rune at the end of a string cause an escape marker to be
			// introduced afterwards.
			b.resetWithMode(SafeRaw)
			b.WriteByte(seq[0])
			b.checkInvariants(t)
			b.checkEqual(t, "\342")

			b.resetWithMode(SafeRaw)
			b.Write(seq[:1])
			b.checkInvariants(t)
			b.checkEqual(t, "\342")

			b.resetWithMode(SafeRaw)
			b.WriteByte(seq[0])
			b.WriteByte(seq[1])
			b.checkInvariants(t)
			b.checkEqual(t, "\342\200")

			b.resetWithMode(SafeRaw)
			b.Write(seq[:2])
			b.checkInvariants(t)
			b.checkEqual(t, "\342\200")

			// Invalid utf-8 sequences that are followed with
			// valid utf-8 bytes do not need an escape marker.
			b.resetWithMode(SafeRaw)
			b.Write(seq[:2])
			b.WriteString("hello")
			b.checkInvariants(t)
			b.checkEqual(t, "\342\200hello")
		})
	}

	// Generic invalid utf-8 sequences.
	b.resetWithMode(SafeRaw)
	b.Write([]byte{0xe4, 0xac}) // prefix to 䬦
	b.checkInvariants(t)
	b.checkEqual(t, "\344\254")
	b.resetWithMode(SafeRaw)
	b.Write([]byte{0xe4, 0xac}) // prefix to 䬦
	b.WriteString("hello")
	b.checkInvariants(t)
	b.checkEqual(t, "\344\254hello")
}

func Example_newlines() {
	var b Buffer
	b.SetMode(UnsafeEscaped)
	b.WriteString("a\n")
	fmt.Printf("%q\n", b.RedactableBytes())

	// Output:
	// "‹a›\n"
}

func Example_mixed_writes() {
	testCases := []struct {
		startMode, endMode OutputMode
	}{
		{UnsafeEscaped, SafeEscaped},
		{SafeEscaped, UnsafeEscaped},
		{UnsafeEscaped, SafeRaw},
		{SafeRaw, UnsafeEscaped},
		{SafeEscaped, SafeRaw},
		{SafeRaw, SafeEscaped},
	}

	doWritesFn := []func(*Buffer){
		// Noop
		func(b *Buffer) {},
		func(b *Buffer) { b.WriteByte('a') },
		func(b *Buffer) { b.WriteRune('a') },
		func(b *Buffer) { b.WriteString("bc") },
		func(b *Buffer) { b.Write([]byte("bc")) },
		func(b *Buffer) { b.WriteString("safe‹unsafe›") },
		func(b *Buffer) { b.WriteByte('\n') },
		func(b *Buffer) { b.WriteRune('\n') },
		func(b *Buffer) { b.WriteString("\nx\ny\n") },
		func(b *Buffer) { b.Write([]byte("\nx\ny\n")) },
	}

	for i, tc := range testCases {
		if i > 0 {
			fmt.Println()
		}
		fmt.Println(tc.startMode, " -> ", tc.endMode)
		for j, doWrites := range doWritesFn {
			var b Buffer
			b.SetMode(tc.startMode)
			doWrites(&b)
			for k, doWrites2 := range doWritesFn {
				copy := b.clone()
				copy.SetMode(tc.endMode)
				doWrites2(copy)
				var b2 Buffer
				b2.SetMode(tc.endMode)
				doWrites2(&b2)
				fmt.Printf("fn%d+fn%d: %q   (%q + %q)\n", j, k, copy.RedactableString(), b.RedactableString(), b2.RedactableString())
			}
		}
	}

	// Output:
	// 0  ->  1
	// fn0+fn0: ""   ("" + "")
	// fn0+fn1: "a"   ("" + "a")
	// fn0+fn2: "a"   ("" + "a")
	// fn0+fn3: "bc"   ("" + "bc")
	// fn0+fn4: "bc"   ("" + "bc")
	// fn0+fn5: "safe?unsafe?"   ("" + "safe?unsafe?")
	// fn0+fn6: "\n"   ("" + "\n")
	// fn0+fn7: "\n"   ("" + "\n")
	// fn0+fn8: "\nx\ny\n"   ("" + "\nx\ny\n")
	// fn0+fn9: "\nx\ny\n"   ("" + "\nx\ny\n")
	// fn1+fn0: "‹a›"   ("‹a›" + "")
	// fn1+fn1: "‹a›a"   ("‹a›" + "a")
	// fn1+fn2: "‹a›a"   ("‹a›" + "a")
	// fn1+fn3: "‹a›bc"   ("‹a›" + "bc")
	// fn1+fn4: "‹a›bc"   ("‹a›" + "bc")
	// fn1+fn5: "‹a›safe?unsafe?"   ("‹a›" + "safe?unsafe?")
	// fn1+fn6: "‹a›\n"   ("‹a›" + "\n")
	// fn1+fn7: "‹a›\n"   ("‹a›" + "\n")
	// fn1+fn8: "‹a›\nx\ny\n"   ("‹a›" + "\nx\ny\n")
	// fn1+fn9: "‹a›\nx\ny\n"   ("‹a›" + "\nx\ny\n")
	// fn2+fn0: "‹a›"   ("‹a›" + "")
	// fn2+fn1: "‹a›a"   ("‹a›" + "a")
	// fn2+fn2: "‹a›a"   ("‹a›" + "a")
	// fn2+fn3: "‹a›bc"   ("‹a›" + "bc")
	// fn2+fn4: "‹a›bc"   ("‹a›" + "bc")
	// fn2+fn5: "‹a›safe?unsafe?"   ("‹a›" + "safe?unsafe?")
	// fn2+fn6: "‹a›\n"   ("‹a›" + "\n")
	// fn2+fn7: "‹a›\n"   ("‹a›" + "\n")
	// fn2+fn8: "‹a›\nx\ny\n"   ("‹a›" + "\nx\ny\n")
	// fn2+fn9: "‹a›\nx\ny\n"   ("‹a›" + "\nx\ny\n")
	// fn3+fn0: "‹bc›"   ("‹bc›" + "")
	// fn3+fn1: "‹bc›a"   ("‹bc›" + "a")
	// fn3+fn2: "‹bc›a"   ("‹bc›" + "a")
	// fn3+fn3: "‹bc›bc"   ("‹bc›" + "bc")
	// fn3+fn4: "‹bc›bc"   ("‹bc›" + "bc")
	// fn3+fn5: "‹bc›safe?unsafe?"   ("‹bc›" + "safe?unsafe?")
	// fn3+fn6: "‹bc›\n"   ("‹bc›" + "\n")
	// fn3+fn7: "‹bc›\n"   ("‹bc›" + "\n")
	// fn3+fn8: "‹bc›\nx\ny\n"   ("‹bc›" + "\nx\ny\n")
	// fn3+fn9: "‹bc›\nx\ny\n"   ("‹bc›" + "\nx\ny\n")
	// fn4+fn0: "‹bc›"   ("‹bc›" + "")
	// fn4+fn1: "‹bc›a"   ("‹bc›" + "a")
	// fn4+fn2: "‹bc›a"   ("‹bc›" + "a")
	// fn4+fn3: "‹bc›bc"   ("‹bc›" + "bc")
	// fn4+fn4: "‹bc›bc"   ("‹bc›" + "bc")
	// fn4+fn5: "‹bc›safe?unsafe?"   ("‹bc›" + "safe?unsafe?")
	// fn4+fn6: "‹bc›\n"   ("‹bc›" + "\n")
	// fn4+fn7: "‹bc›\n"   ("‹bc›" + "\n")
	// fn4+fn8: "‹bc›\nx\ny\n"   ("‹bc›" + "\nx\ny\n")
	// fn4+fn9: "‹bc›\nx\ny\n"   ("‹bc›" + "\nx\ny\n")
	// fn5+fn0: "‹safe?unsafe?›"   ("‹safe?unsafe?›" + "")
	// fn5+fn1: "‹safe?unsafe?›a"   ("‹safe?unsafe?›" + "a")
	// fn5+fn2: "‹safe?unsafe?›a"   ("‹safe?unsafe?›" + "a")
	// fn5+fn3: "‹safe?unsafe?›bc"   ("‹safe?unsafe?›" + "bc")
	// fn5+fn4: "‹safe?unsafe?›bc"   ("‹safe?unsafe?›" + "bc")
	// fn5+fn5: "‹safe?unsafe?›safe?unsafe?"   ("‹safe?unsafe?›" + "safe?unsafe?")
	// fn5+fn6: "‹safe?unsafe?›\n"   ("‹safe?unsafe?›" + "\n")
	// fn5+fn7: "‹safe?unsafe?›\n"   ("‹safe?unsafe?›" + "\n")
	// fn5+fn8: "‹safe?unsafe?›\nx\ny\n"   ("‹safe?unsafe?›" + "\nx\ny\n")
	// fn5+fn9: "‹safe?unsafe?›\nx\ny\n"   ("‹safe?unsafe?›" + "\nx\ny\n")
	// fn6+fn0: "\n"   ("\n" + "")
	// fn6+fn1: "\na"   ("\n" + "a")
	// fn6+fn2: "\na"   ("\n" + "a")
	// fn6+fn3: "\nbc"   ("\n" + "bc")
	// fn6+fn4: "\nbc"   ("\n" + "bc")
	// fn6+fn5: "\nsafe?unsafe?"   ("\n" + "safe?unsafe?")
	// fn6+fn6: "\n\n"   ("\n" + "\n")
	// fn6+fn7: "\n\n"   ("\n" + "\n")
	// fn6+fn8: "\n\nx\ny\n"   ("\n" + "\nx\ny\n")
	// fn6+fn9: "\n\nx\ny\n"   ("\n" + "\nx\ny\n")
	// fn7+fn0: "\n"   ("\n" + "")
	// fn7+fn1: "\na"   ("\n" + "a")
	// fn7+fn2: "\na"   ("\n" + "a")
	// fn7+fn3: "\nbc"   ("\n" + "bc")
	// fn7+fn4: "\nbc"   ("\n" + "bc")
	// fn7+fn5: "\nsafe?unsafe?"   ("\n" + "safe?unsafe?")
	// fn7+fn6: "\n\n"   ("\n" + "\n")
	// fn7+fn7: "\n\n"   ("\n" + "\n")
	// fn7+fn8: "\n\nx\ny\n"   ("\n" + "\nx\ny\n")
	// fn7+fn9: "\n\nx\ny\n"   ("\n" + "\nx\ny\n")
	// fn8+fn0: "\n‹x›\n‹y›\n"   ("\n‹x›\n‹y›\n" + "")
	// fn8+fn1: "\n‹x›\n‹y›\na"   ("\n‹x›\n‹y›\n" + "a")
	// fn8+fn2: "\n‹x›\n‹y›\na"   ("\n‹x›\n‹y›\n" + "a")
	// fn8+fn3: "\n‹x›\n‹y›\nbc"   ("\n‹x›\n‹y›\n" + "bc")
	// fn8+fn4: "\n‹x›\n‹y›\nbc"   ("\n‹x›\n‹y›\n" + "bc")
	// fn8+fn5: "\n‹x›\n‹y›\nsafe?unsafe?"   ("\n‹x›\n‹y›\n" + "safe?unsafe?")
	// fn8+fn6: "\n‹x›\n‹y›\n\n"   ("\n‹x›\n‹y›\n" + "\n")
	// fn8+fn7: "\n‹x›\n‹y›\n\n"   ("\n‹x›\n‹y›\n" + "\n")
	// fn8+fn8: "\n‹x›\n‹y›\n\nx\ny\n"   ("\n‹x›\n‹y›\n" + "\nx\ny\n")
	// fn8+fn9: "\n‹x›\n‹y›\n\nx\ny\n"   ("\n‹x›\n‹y›\n" + "\nx\ny\n")
	// fn9+fn0: "\n‹x›\n‹y›\n"   ("\n‹x›\n‹y›\n" + "")
	// fn9+fn1: "\n‹x›\n‹y›\na"   ("\n‹x›\n‹y›\n" + "a")
	// fn9+fn2: "\n‹x›\n‹y›\na"   ("\n‹x›\n‹y›\n" + "a")
	// fn9+fn3: "\n‹x›\n‹y›\nbc"   ("\n‹x›\n‹y›\n" + "bc")
	// fn9+fn4: "\n‹x›\n‹y›\nbc"   ("\n‹x›\n‹y›\n" + "bc")
	// fn9+fn5: "\n‹x›\n‹y›\nsafe?unsafe?"   ("\n‹x›\n‹y›\n" + "safe?unsafe?")
	// fn9+fn6: "\n‹x›\n‹y›\n\n"   ("\n‹x›\n‹y›\n" + "\n")
	// fn9+fn7: "\n‹x›\n‹y›\n\n"   ("\n‹x›\n‹y›\n" + "\n")
	// fn9+fn8: "\n‹x›\n‹y›\n\nx\ny\n"   ("\n‹x›\n‹y›\n" + "\nx\ny\n")
	// fn9+fn9: "\n‹x›\n‹y›\n\nx\ny\n"   ("\n‹x›\n‹y›\n" + "\nx\ny\n")
	//
	// 1  ->  0
	// fn0+fn0: ""   ("" + "")
	// fn0+fn1: "‹a›"   ("" + "‹a›")
	// fn0+fn2: "‹a›"   ("" + "‹a›")
	// fn0+fn3: "‹bc›"   ("" + "‹bc›")
	// fn0+fn4: "‹bc›"   ("" + "‹bc›")
	// fn0+fn5: "‹safe?unsafe?›"   ("" + "‹safe?unsafe?›")
	// fn0+fn6: "\n"   ("" + "\n")
	// fn0+fn7: "\n"   ("" + "\n")
	// fn0+fn8: "\n‹x›\n‹y›\n"   ("" + "\n‹x›\n‹y›\n")
	// fn0+fn9: "\n‹x›\n‹y›\n"   ("" + "\n‹x›\n‹y›\n")
	// fn1+fn0: "a"   ("a" + "")
	// fn1+fn1: "a‹a›"   ("a" + "‹a›")
	// fn1+fn2: "a‹a›"   ("a" + "‹a›")
	// fn1+fn3: "a‹bc›"   ("a" + "‹bc›")
	// fn1+fn4: "a‹bc›"   ("a" + "‹bc›")
	// fn1+fn5: "a‹safe?unsafe?›"   ("a" + "‹safe?unsafe?›")
	// fn1+fn6: "a\n"   ("a" + "\n")
	// fn1+fn7: "a\n"   ("a" + "\n")
	// fn1+fn8: "a\n‹x›\n‹y›\n"   ("a" + "\n‹x›\n‹y›\n")
	// fn1+fn9: "a\n‹x›\n‹y›\n"   ("a" + "\n‹x›\n‹y›\n")
	// fn2+fn0: "a"   ("a" + "")
	// fn2+fn1: "a‹a›"   ("a" + "‹a›")
	// fn2+fn2: "a‹a›"   ("a" + "‹a›")
	// fn2+fn3: "a‹bc›"   ("a" + "‹bc›")
	// fn2+fn4: "a‹bc›"   ("a" + "‹bc›")
	// fn2+fn5: "a‹safe?unsafe?›"   ("a" + "‹safe?unsafe?›")
	// fn2+fn6: "a\n"   ("a" + "\n")
	// fn2+fn7: "a\n"   ("a" + "\n")
	// fn2+fn8: "a\n‹x›\n‹y›\n"   ("a" + "\n‹x›\n‹y›\n")
	// fn2+fn9: "a\n‹x›\n‹y›\n"   ("a" + "\n‹x›\n‹y›\n")
	// fn3+fn0: "bc"   ("bc" + "")
	// fn3+fn1: "bc‹a›"   ("bc" + "‹a›")
	// fn3+fn2: "bc‹a›"   ("bc" + "‹a›")
	// fn3+fn3: "bc‹bc›"   ("bc" + "‹bc›")
	// fn3+fn4: "bc‹bc›"   ("bc" + "‹bc›")
	// fn3+fn5: "bc‹safe?unsafe?›"   ("bc" + "‹safe?unsafe?›")
	// fn3+fn6: "bc\n"   ("bc" + "\n")
	// fn3+fn7: "bc\n"   ("bc" + "\n")
	// fn3+fn8: "bc\n‹x›\n‹y›\n"   ("bc" + "\n‹x›\n‹y›\n")
	// fn3+fn9: "bc\n‹x›\n‹y›\n"   ("bc" + "\n‹x›\n‹y›\n")
	// fn4+fn0: "bc"   ("bc" + "")
	// fn4+fn1: "bc‹a›"   ("bc" + "‹a›")
	// fn4+fn2: "bc‹a›"   ("bc" + "‹a›")
	// fn4+fn3: "bc‹bc›"   ("bc" + "‹bc›")
	// fn4+fn4: "bc‹bc›"   ("bc" + "‹bc›")
	// fn4+fn5: "bc‹safe?unsafe?›"   ("bc" + "‹safe?unsafe?›")
	// fn4+fn6: "bc\n"   ("bc" + "\n")
	// fn4+fn7: "bc\n"   ("bc" + "\n")
	// fn4+fn8: "bc\n‹x›\n‹y›\n"   ("bc" + "\n‹x›\n‹y›\n")
	// fn4+fn9: "bc\n‹x›\n‹y›\n"   ("bc" + "\n‹x›\n‹y›\n")
	// fn5+fn0: "safe?unsafe?"   ("safe?unsafe?" + "")
	// fn5+fn1: "safe?unsafe?‹a›"   ("safe?unsafe?" + "‹a›")
	// fn5+fn2: "safe?unsafe?‹a›"   ("safe?unsafe?" + "‹a›")
	// fn5+fn3: "safe?unsafe?‹bc›"   ("safe?unsafe?" + "‹bc›")
	// fn5+fn4: "safe?unsafe?‹bc›"   ("safe?unsafe?" + "‹bc›")
	// fn5+fn5: "safe?unsafe?‹safe?unsafe?›"   ("safe?unsafe?" + "‹safe?unsafe?›")
	// fn5+fn6: "safe?unsafe?\n"   ("safe?unsafe?" + "\n")
	// fn5+fn7: "safe?unsafe?\n"   ("safe?unsafe?" + "\n")
	// fn5+fn8: "safe?unsafe?\n‹x›\n‹y›\n"   ("safe?unsafe?" + "\n‹x›\n‹y›\n")
	// fn5+fn9: "safe?unsafe?\n‹x›\n‹y›\n"   ("safe?unsafe?" + "\n‹x›\n‹y›\n")
	// fn6+fn0: "\n"   ("\n" + "")
	// fn6+fn1: "\n‹a›"   ("\n" + "‹a›")
	// fn6+fn2: "\n‹a›"   ("\n" + "‹a›")
	// fn6+fn3: "\n‹bc›"   ("\n" + "‹bc›")
	// fn6+fn4: "\n‹bc›"   ("\n" + "‹bc›")
	// fn6+fn5: "\n‹safe?unsafe?›"   ("\n" + "‹safe?unsafe?›")
	// fn6+fn6: "\n\n"   ("\n" + "\n")
	// fn6+fn7: "\n\n"   ("\n" + "\n")
	// fn6+fn8: "\n\n‹x›\n‹y›\n"   ("\n" + "\n‹x›\n‹y›\n")
	// fn6+fn9: "\n\n‹x›\n‹y›\n"   ("\n" + "\n‹x›\n‹y›\n")
	// fn7+fn0: "\n"   ("\n" + "")
	// fn7+fn1: "\n‹a›"   ("\n" + "‹a›")
	// fn7+fn2: "\n‹a›"   ("\n" + "‹a›")
	// fn7+fn3: "\n‹bc›"   ("\n" + "‹bc›")
	// fn7+fn4: "\n‹bc›"   ("\n" + "‹bc›")
	// fn7+fn5: "\n‹safe?unsafe?›"   ("\n" + "‹safe?unsafe?›")
	// fn7+fn6: "\n\n"   ("\n" + "\n")
	// fn7+fn7: "\n\n"   ("\n" + "\n")
	// fn7+fn8: "\n\n‹x›\n‹y›\n"   ("\n" + "\n‹x›\n‹y›\n")
	// fn7+fn9: "\n\n‹x›\n‹y›\n"   ("\n" + "\n‹x›\n‹y›\n")
	// fn8+fn0: "\nx\ny\n"   ("\nx\ny\n" + "")
	// fn8+fn1: "\nx\ny\n‹a›"   ("\nx\ny\n" + "‹a›")
	// fn8+fn2: "\nx\ny\n‹a›"   ("\nx\ny\n" + "‹a›")
	// fn8+fn3: "\nx\ny\n‹bc›"   ("\nx\ny\n" + "‹bc›")
	// fn8+fn4: "\nx\ny\n‹bc›"   ("\nx\ny\n" + "‹bc›")
	// fn8+fn5: "\nx\ny\n‹safe?unsafe?›"   ("\nx\ny\n" + "‹safe?unsafe?›")
	// fn8+fn6: "\nx\ny\n\n"   ("\nx\ny\n" + "\n")
	// fn8+fn7: "\nx\ny\n\n"   ("\nx\ny\n" + "\n")
	// fn8+fn8: "\nx\ny\n\n‹x›\n‹y›\n"   ("\nx\ny\n" + "\n‹x›\n‹y›\n")
	// fn8+fn9: "\nx\ny\n\n‹x›\n‹y›\n"   ("\nx\ny\n" + "\n‹x›\n‹y›\n")
	// fn9+fn0: "\nx\ny\n"   ("\nx\ny\n" + "")
	// fn9+fn1: "\nx\ny\n‹a›"   ("\nx\ny\n" + "‹a›")
	// fn9+fn2: "\nx\ny\n‹a›"   ("\nx\ny\n" + "‹a›")
	// fn9+fn3: "\nx\ny\n‹bc›"   ("\nx\ny\n" + "‹bc›")
	// fn9+fn4: "\nx\ny\n‹bc›"   ("\nx\ny\n" + "‹bc›")
	// fn9+fn5: "\nx\ny\n‹safe?unsafe?›"   ("\nx\ny\n" + "‹safe?unsafe?›")
	// fn9+fn6: "\nx\ny\n\n"   ("\nx\ny\n" + "\n")
	// fn9+fn7: "\nx\ny\n\n"   ("\nx\ny\n" + "\n")
	// fn9+fn8: "\nx\ny\n\n‹x›\n‹y›\n"   ("\nx\ny\n" + "\n‹x›\n‹y›\n")
	// fn9+fn9: "\nx\ny\n\n‹x›\n‹y›\n"   ("\nx\ny\n" + "\n‹x›\n‹y›\n")
	//
	// 0  ->  2
	// fn0+fn0: ""   ("" + "")
	// fn0+fn1: "a"   ("" + "a")
	// fn0+fn2: "a"   ("" + "a")
	// fn0+fn3: "bc"   ("" + "bc")
	// fn0+fn4: "bc"   ("" + "bc")
	// fn0+fn5: "safe‹unsafe›"   ("" + "safe‹unsafe›")
	// fn0+fn6: "\n"   ("" + "\n")
	// fn0+fn7: "\n"   ("" + "\n")
	// fn0+fn8: "\nx\ny\n"   ("" + "\nx\ny\n")
	// fn0+fn9: "\nx\ny\n"   ("" + "\nx\ny\n")
	// fn1+fn0: "‹a›"   ("‹a›" + "")
	// fn1+fn1: "‹a›a"   ("‹a›" + "a")
	// fn1+fn2: "‹a›a"   ("‹a›" + "a")
	// fn1+fn3: "‹a›bc"   ("‹a›" + "bc")
	// fn1+fn4: "‹a›bc"   ("‹a›" + "bc")
	// fn1+fn5: "‹a›safe‹unsafe›"   ("‹a›" + "safe‹unsafe›")
	// fn1+fn6: "‹a›\n"   ("‹a›" + "\n")
	// fn1+fn7: "‹a›\n"   ("‹a›" + "\n")
	// fn1+fn8: "‹a›\nx\ny\n"   ("‹a›" + "\nx\ny\n")
	// fn1+fn9: "‹a›\nx\ny\n"   ("‹a›" + "\nx\ny\n")
	// fn2+fn0: "‹a›"   ("‹a›" + "")
	// fn2+fn1: "‹a›a"   ("‹a›" + "a")
	// fn2+fn2: "‹a›a"   ("‹a›" + "a")
	// fn2+fn3: "‹a›bc"   ("‹a›" + "bc")
	// fn2+fn4: "‹a›bc"   ("‹a›" + "bc")
	// fn2+fn5: "‹a›safe‹unsafe›"   ("‹a›" + "safe‹unsafe›")
	// fn2+fn6: "‹a›\n"   ("‹a›" + "\n")
	// fn2+fn7: "‹a›\n"   ("‹a›" + "\n")
	// fn2+fn8: "‹a›\nx\ny\n"   ("‹a›" + "\nx\ny\n")
	// fn2+fn9: "‹a›\nx\ny\n"   ("‹a›" + "\nx\ny\n")
	// fn3+fn0: "‹bc›"   ("‹bc›" + "")
	// fn3+fn1: "‹bc›a"   ("‹bc›" + "a")
	// fn3+fn2: "‹bc›a"   ("‹bc›" + "a")
	// fn3+fn3: "‹bc›bc"   ("‹bc›" + "bc")
	// fn3+fn4: "‹bc›bc"   ("‹bc›" + "bc")
	// fn3+fn5: "‹bc›safe‹unsafe›"   ("‹bc›" + "safe‹unsafe›")
	// fn3+fn6: "‹bc›\n"   ("‹bc›" + "\n")
	// fn3+fn7: "‹bc›\n"   ("‹bc›" + "\n")
	// fn3+fn8: "‹bc›\nx\ny\n"   ("‹bc›" + "\nx\ny\n")
	// fn3+fn9: "‹bc›\nx\ny\n"   ("‹bc›" + "\nx\ny\n")
	// fn4+fn0: "‹bc›"   ("‹bc›" + "")
	// fn4+fn1: "‹bc›a"   ("‹bc›" + "a")
	// fn4+fn2: "‹bc›a"   ("‹bc›" + "a")
	// fn4+fn3: "‹bc›bc"   ("‹bc›" + "bc")
	// fn4+fn4: "‹bc›bc"   ("‹bc›" + "bc")
	// fn4+fn5: "‹bc›safe‹unsafe›"   ("‹bc›" + "safe‹unsafe›")
	// fn4+fn6: "‹bc›\n"   ("‹bc›" + "\n")
	// fn4+fn7: "‹bc›\n"   ("‹bc›" + "\n")
	// fn4+fn8: "‹bc›\nx\ny\n"   ("‹bc›" + "\nx\ny\n")
	// fn4+fn9: "‹bc›\nx\ny\n"   ("‹bc›" + "\nx\ny\n")
	// fn5+fn0: "‹safe?unsafe?›"   ("‹safe?unsafe?›" + "")
	// fn5+fn1: "‹safe?unsafe?›a"   ("‹safe?unsafe?›" + "a")
	// fn5+fn2: "‹safe?unsafe?›a"   ("‹safe?unsafe?›" + "a")
	// fn5+fn3: "‹safe?unsafe?›bc"   ("‹safe?unsafe?›" + "bc")
	// fn5+fn4: "‹safe?unsafe?›bc"   ("‹safe?unsafe?›" + "bc")
	// fn5+fn5: "‹safe?unsafe?›safe‹unsafe›"   ("‹safe?unsafe?›" + "safe‹unsafe›")
	// fn5+fn6: "‹safe?unsafe?›\n"   ("‹safe?unsafe?›" + "\n")
	// fn5+fn7: "‹safe?unsafe?›\n"   ("‹safe?unsafe?›" + "\n")
	// fn5+fn8: "‹safe?unsafe?›\nx\ny\n"   ("‹safe?unsafe?›" + "\nx\ny\n")
	// fn5+fn9: "‹safe?unsafe?›\nx\ny\n"   ("‹safe?unsafe?›" + "\nx\ny\n")
	// fn6+fn0: "\n"   ("\n" + "")
	// fn6+fn1: "\na"   ("\n" + "a")
	// fn6+fn2: "\na"   ("\n" + "a")
	// fn6+fn3: "\nbc"   ("\n" + "bc")
	// fn6+fn4: "\nbc"   ("\n" + "bc")
	// fn6+fn5: "\nsafe‹unsafe›"   ("\n" + "safe‹unsafe›")
	// fn6+fn6: "\n\n"   ("\n" + "\n")
	// fn6+fn7: "\n\n"   ("\n" + "\n")
	// fn6+fn8: "\n\nx\ny\n"   ("\n" + "\nx\ny\n")
	// fn6+fn9: "\n\nx\ny\n"   ("\n" + "\nx\ny\n")
	// fn7+fn0: "\n"   ("\n" + "")
	// fn7+fn1: "\na"   ("\n" + "a")
	// fn7+fn2: "\na"   ("\n" + "a")
	// fn7+fn3: "\nbc"   ("\n" + "bc")
	// fn7+fn4: "\nbc"   ("\n" + "bc")
	// fn7+fn5: "\nsafe‹unsafe›"   ("\n" + "safe‹unsafe›")
	// fn7+fn6: "\n\n"   ("\n" + "\n")
	// fn7+fn7: "\n\n"   ("\n" + "\n")
	// fn7+fn8: "\n\nx\ny\n"   ("\n" + "\nx\ny\n")
	// fn7+fn9: "\n\nx\ny\n"   ("\n" + "\nx\ny\n")
	// fn8+fn0: "\n‹x›\n‹y›\n"   ("\n‹x›\n‹y›\n" + "")
	// fn8+fn1: "\n‹x›\n‹y›\na"   ("\n‹x›\n‹y›\n" + "a")
	// fn8+fn2: "\n‹x›\n‹y›\na"   ("\n‹x›\n‹y›\n" + "a")
	// fn8+fn3: "\n‹x›\n‹y›\nbc"   ("\n‹x›\n‹y›\n" + "bc")
	// fn8+fn4: "\n‹x›\n‹y›\nbc"   ("\n‹x›\n‹y›\n" + "bc")
	// fn8+fn5: "\n‹x›\n‹y›\nsafe‹unsafe›"   ("\n‹x›\n‹y›\n" + "safe‹unsafe›")
	// fn8+fn6: "\n‹x›\n‹y›\n\n"   ("\n‹x›\n‹y›\n" + "\n")
	// fn8+fn7: "\n‹x›\n‹y›\n\n"   ("\n‹x›\n‹y›\n" + "\n")
	// fn8+fn8: "\n‹x›\n‹y›\n\nx\ny\n"   ("\n‹x›\n‹y›\n" + "\nx\ny\n")
	// fn8+fn9: "\n‹x›\n‹y›\n\nx\ny\n"   ("\n‹x›\n‹y›\n" + "\nx\ny\n")
	// fn9+fn0: "\n‹x›\n‹y›\n"   ("\n‹x›\n‹y›\n" + "")
	// fn9+fn1: "\n‹x›\n‹y›\na"   ("\n‹x›\n‹y›\n" + "a")
	// fn9+fn2: "\n‹x›\n‹y›\na"   ("\n‹x›\n‹y›\n" + "a")
	// fn9+fn3: "\n‹x›\n‹y›\nbc"   ("\n‹x›\n‹y›\n" + "bc")
	// fn9+fn4: "\n‹x›\n‹y›\nbc"   ("\n‹x›\n‹y›\n" + "bc")
	// fn9+fn5: "\n‹x›\n‹y›\nsafe‹unsafe›"   ("\n‹x›\n‹y›\n" + "safe‹unsafe›")
	// fn9+fn6: "\n‹x›\n‹y›\n\n"   ("\n‹x›\n‹y›\n" + "\n")
	// fn9+fn7: "\n‹x›\n‹y›\n\n"   ("\n‹x›\n‹y›\n" + "\n")
	// fn9+fn8: "\n‹x›\n‹y›\n\nx\ny\n"   ("\n‹x›\n‹y›\n" + "\nx\ny\n")
	// fn9+fn9: "\n‹x›\n‹y›\n\nx\ny\n"   ("\n‹x›\n‹y›\n" + "\nx\ny\n")
	//
	// 2  ->  0
	// fn0+fn0: ""   ("" + "")
	// fn0+fn1: "‹a›"   ("" + "‹a›")
	// fn0+fn2: "‹a›"   ("" + "‹a›")
	// fn0+fn3: "‹bc›"   ("" + "‹bc›")
	// fn0+fn4: "‹bc›"   ("" + "‹bc›")
	// fn0+fn5: "‹safe?unsafe?›"   ("" + "‹safe?unsafe?›")
	// fn0+fn6: "\n"   ("" + "\n")
	// fn0+fn7: "\n"   ("" + "\n")
	// fn0+fn8: "\n‹x›\n‹y›\n"   ("" + "\n‹x›\n‹y›\n")
	// fn0+fn9: "\n‹x›\n‹y›\n"   ("" + "\n‹x›\n‹y›\n")
	// fn1+fn0: "a"   ("a" + "")
	// fn1+fn1: "a‹a›"   ("a" + "‹a›")
	// fn1+fn2: "a‹a›"   ("a" + "‹a›")
	// fn1+fn3: "a‹bc›"   ("a" + "‹bc›")
	// fn1+fn4: "a‹bc›"   ("a" + "‹bc›")
	// fn1+fn5: "a‹safe?unsafe?›"   ("a" + "‹safe?unsafe?›")
	// fn1+fn6: "a\n"   ("a" + "\n")
	// fn1+fn7: "a\n"   ("a" + "\n")
	// fn1+fn8: "a\n‹x›\n‹y›\n"   ("a" + "\n‹x›\n‹y›\n")
	// fn1+fn9: "a\n‹x›\n‹y›\n"   ("a" + "\n‹x›\n‹y›\n")
	// fn2+fn0: "a"   ("a" + "")
	// fn2+fn1: "a‹a›"   ("a" + "‹a›")
	// fn2+fn2: "a‹a›"   ("a" + "‹a›")
	// fn2+fn3: "a‹bc›"   ("a" + "‹bc›")
	// fn2+fn4: "a‹bc›"   ("a" + "‹bc›")
	// fn2+fn5: "a‹safe?unsafe?›"   ("a" + "‹safe?unsafe?›")
	// fn2+fn6: "a\n"   ("a" + "\n")
	// fn2+fn7: "a\n"   ("a" + "\n")
	// fn2+fn8: "a\n‹x›\n‹y›\n"   ("a" + "\n‹x›\n‹y›\n")
	// fn2+fn9: "a\n‹x›\n‹y›\n"   ("a" + "\n‹x›\n‹y›\n")
	// fn3+fn0: "bc"   ("bc" + "")
	// fn3+fn1: "bc‹a›"   ("bc" + "‹a›")
	// fn3+fn2: "bc‹a›"   ("bc" + "‹a›")
	// fn3+fn3: "bc‹bc›"   ("bc" + "‹bc›")
	// fn3+fn4: "bc‹bc›"   ("bc" + "‹bc›")
	// fn3+fn5: "bc‹safe?unsafe?›"   ("bc" + "‹safe?unsafe?›")
	// fn3+fn6: "bc\n"   ("bc" + "\n")
	// fn3+fn7: "bc\n"   ("bc" + "\n")
	// fn3+fn8: "bc\n‹x›\n‹y›\n"   ("bc" + "\n‹x›\n‹y›\n")
	// fn3+fn9: "bc\n‹x›\n‹y›\n"   ("bc" + "\n‹x›\n‹y›\n")
	// fn4+fn0: "bc"   ("bc" + "")
	// fn4+fn1: "bc‹a›"   ("bc" + "‹a›")
	// fn4+fn2: "bc‹a›"   ("bc" + "‹a›")
	// fn4+fn3: "bc‹bc›"   ("bc" + "‹bc›")
	// fn4+fn4: "bc‹bc›"   ("bc" + "‹bc›")
	// fn4+fn5: "bc‹safe?unsafe?›"   ("bc" + "‹safe?unsafe?›")
	// fn4+fn6: "bc\n"   ("bc" + "\n")
	// fn4+fn7: "bc\n"   ("bc" + "\n")
	// fn4+fn8: "bc\n‹x›\n‹y›\n"   ("bc" + "\n‹x›\n‹y›\n")
	// fn4+fn9: "bc\n‹x›\n‹y›\n"   ("bc" + "\n‹x›\n‹y›\n")
	// fn5+fn0: "safe‹unsafe›"   ("safe‹unsafe›" + "")
	// fn5+fn1: "safe‹unsafea›"   ("safe‹unsafe›" + "‹a›")
	// fn5+fn2: "safe‹unsafea›"   ("safe‹unsafe›" + "‹a›")
	// fn5+fn3: "safe‹unsafebc›"   ("safe‹unsafe›" + "‹bc›")
	// fn5+fn4: "safe‹unsafebc›"   ("safe‹unsafe›" + "‹bc›")
	// fn5+fn5: "safe‹unsafesafe?unsafe?›"   ("safe‹unsafe›" + "‹safe?unsafe?›")
	// fn5+fn6: "safe‹unsafe›\n"   ("safe‹unsafe›" + "\n")
	// fn5+fn7: "safe‹unsafe›\n"   ("safe‹unsafe›" + "\n")
	// fn5+fn8: "safe‹unsafe›\n‹x›\n‹y›\n"   ("safe‹unsafe›" + "\n‹x›\n‹y›\n")
	// fn5+fn9: "safe‹unsafe›\n‹x›\n‹y›\n"   ("safe‹unsafe›" + "\n‹x›\n‹y›\n")
	// fn6+fn0: "\n"   ("\n" + "")
	// fn6+fn1: "\n‹a›"   ("\n" + "‹a›")
	// fn6+fn2: "\n‹a›"   ("\n" + "‹a›")
	// fn6+fn3: "\n‹bc›"   ("\n" + "‹bc›")
	// fn6+fn4: "\n‹bc›"   ("\n" + "‹bc›")
	// fn6+fn5: "\n‹safe?unsafe?›"   ("\n" + "‹safe?unsafe?›")
	// fn6+fn6: "\n\n"   ("\n" + "\n")
	// fn6+fn7: "\n\n"   ("\n" + "\n")
	// fn6+fn8: "\n\n‹x›\n‹y›\n"   ("\n" + "\n‹x›\n‹y›\n")
	// fn6+fn9: "\n\n‹x›\n‹y›\n"   ("\n" + "\n‹x›\n‹y›\n")
	// fn7+fn0: "\n"   ("\n" + "")
	// fn7+fn1: "\n‹a›"   ("\n" + "‹a›")
	// fn7+fn2: "\n‹a›"   ("\n" + "‹a›")
	// fn7+fn3: "\n‹bc›"   ("\n" + "‹bc›")
	// fn7+fn4: "\n‹bc›"   ("\n" + "‹bc›")
	// fn7+fn5: "\n‹safe?unsafe?›"   ("\n" + "‹safe?unsafe?›")
	// fn7+fn6: "\n\n"   ("\n" + "\n")
	// fn7+fn7: "\n\n"   ("\n" + "\n")
	// fn7+fn8: "\n\n‹x›\n‹y›\n"   ("\n" + "\n‹x›\n‹y›\n")
	// fn7+fn9: "\n\n‹x›\n‹y›\n"   ("\n" + "\n‹x›\n‹y›\n")
	// fn8+fn0: "\nx\ny\n"   ("\nx\ny\n" + "")
	// fn8+fn1: "\nx\ny\n‹a›"   ("\nx\ny\n" + "‹a›")
	// fn8+fn2: "\nx\ny\n‹a›"   ("\nx\ny\n" + "‹a›")
	// fn8+fn3: "\nx\ny\n‹bc›"   ("\nx\ny\n" + "‹bc›")
	// fn8+fn4: "\nx\ny\n‹bc›"   ("\nx\ny\n" + "‹bc›")
	// fn8+fn5: "\nx\ny\n‹safe?unsafe?›"   ("\nx\ny\n" + "‹safe?unsafe?›")
	// fn8+fn6: "\nx\ny\n\n"   ("\nx\ny\n" + "\n")
	// fn8+fn7: "\nx\ny\n\n"   ("\nx\ny\n" + "\n")
	// fn8+fn8: "\nx\ny\n\n‹x›\n‹y›\n"   ("\nx\ny\n" + "\n‹x›\n‹y›\n")
	// fn8+fn9: "\nx\ny\n\n‹x›\n‹y›\n"   ("\nx\ny\n" + "\n‹x›\n‹y›\n")
	// fn9+fn0: "\nx\ny\n"   ("\nx\ny\n" + "")
	// fn9+fn1: "\nx\ny\n‹a›"   ("\nx\ny\n" + "‹a›")
	// fn9+fn2: "\nx\ny\n‹a›"   ("\nx\ny\n" + "‹a›")
	// fn9+fn3: "\nx\ny\n‹bc›"   ("\nx\ny\n" + "‹bc›")
	// fn9+fn4: "\nx\ny\n‹bc›"   ("\nx\ny\n" + "‹bc›")
	// fn9+fn5: "\nx\ny\n‹safe?unsafe?›"   ("\nx\ny\n" + "‹safe?unsafe?›")
	// fn9+fn6: "\nx\ny\n\n"   ("\nx\ny\n" + "\n")
	// fn9+fn7: "\nx\ny\n\n"   ("\nx\ny\n" + "\n")
	// fn9+fn8: "\nx\ny\n\n‹x›\n‹y›\n"   ("\nx\ny\n" + "\n‹x›\n‹y›\n")
	// fn9+fn9: "\nx\ny\n\n‹x›\n‹y›\n"   ("\nx\ny\n" + "\n‹x›\n‹y›\n")
	//
	// 1  ->  2
	// fn0+fn0: ""   ("" + "")
	// fn0+fn1: "a"   ("" + "a")
	// fn0+fn2: "a"   ("" + "a")
	// fn0+fn3: "bc"   ("" + "bc")
	// fn0+fn4: "bc"   ("" + "bc")
	// fn0+fn5: "safe‹unsafe›"   ("" + "safe‹unsafe›")
	// fn0+fn6: "\n"   ("" + "\n")
	// fn0+fn7: "\n"   ("" + "\n")
	// fn0+fn8: "\nx\ny\n"   ("" + "\nx\ny\n")
	// fn0+fn9: "\nx\ny\n"   ("" + "\nx\ny\n")
	// fn1+fn0: "a"   ("a" + "")
	// fn1+fn1: "aa"   ("a" + "a")
	// fn1+fn2: "aa"   ("a" + "a")
	// fn1+fn3: "abc"   ("a" + "bc")
	// fn1+fn4: "abc"   ("a" + "bc")
	// fn1+fn5: "asafe‹unsafe›"   ("a" + "safe‹unsafe›")
	// fn1+fn6: "a\n"   ("a" + "\n")
	// fn1+fn7: "a\n"   ("a" + "\n")
	// fn1+fn8: "a\nx\ny\n"   ("a" + "\nx\ny\n")
	// fn1+fn9: "a\nx\ny\n"   ("a" + "\nx\ny\n")
	// fn2+fn0: "a"   ("a" + "")
	// fn2+fn1: "aa"   ("a" + "a")
	// fn2+fn2: "aa"   ("a" + "a")
	// fn2+fn3: "abc"   ("a" + "bc")
	// fn2+fn4: "abc"   ("a" + "bc")
	// fn2+fn5: "asafe‹unsafe›"   ("a" + "safe‹unsafe›")
	// fn2+fn6: "a\n"   ("a" + "\n")
	// fn2+fn7: "a\n"   ("a" + "\n")
	// fn2+fn8: "a\nx\ny\n"   ("a" + "\nx\ny\n")
	// fn2+fn9: "a\nx\ny\n"   ("a" + "\nx\ny\n")
	// fn3+fn0: "bc"   ("bc" + "")
	// fn3+fn1: "bca"   ("bc" + "a")
	// fn3+fn2: "bca"   ("bc" + "a")
	// fn3+fn3: "bcbc"   ("bc" + "bc")
	// fn3+fn4: "bcbc"   ("bc" + "bc")
	// fn3+fn5: "bcsafe‹unsafe›"   ("bc" + "safe‹unsafe›")
	// fn3+fn6: "bc\n"   ("bc" + "\n")
	// fn3+fn7: "bc\n"   ("bc" + "\n")
	// fn3+fn8: "bc\nx\ny\n"   ("bc" + "\nx\ny\n")
	// fn3+fn9: "bc\nx\ny\n"   ("bc" + "\nx\ny\n")
	// fn4+fn0: "bc"   ("bc" + "")
	// fn4+fn1: "bca"   ("bc" + "a")
	// fn4+fn2: "bca"   ("bc" + "a")
	// fn4+fn3: "bcbc"   ("bc" + "bc")
	// fn4+fn4: "bcbc"   ("bc" + "bc")
	// fn4+fn5: "bcsafe‹unsafe›"   ("bc" + "safe‹unsafe›")
	// fn4+fn6: "bc\n"   ("bc" + "\n")
	// fn4+fn7: "bc\n"   ("bc" + "\n")
	// fn4+fn8: "bc\nx\ny\n"   ("bc" + "\nx\ny\n")
	// fn4+fn9: "bc\nx\ny\n"   ("bc" + "\nx\ny\n")
	// fn5+fn0: "safe?unsafe?"   ("safe?unsafe?" + "")
	// fn5+fn1: "safe?unsafe?a"   ("safe?unsafe?" + "a")
	// fn5+fn2: "safe?unsafe?a"   ("safe?unsafe?" + "a")
	// fn5+fn3: "safe?unsafe?bc"   ("safe?unsafe?" + "bc")
	// fn5+fn4: "safe?unsafe?bc"   ("safe?unsafe?" + "bc")
	// fn5+fn5: "safe?unsafe?safe‹unsafe›"   ("safe?unsafe?" + "safe‹unsafe›")
	// fn5+fn6: "safe?unsafe?\n"   ("safe?unsafe?" + "\n")
	// fn5+fn7: "safe?unsafe?\n"   ("safe?unsafe?" + "\n")
	// fn5+fn8: "safe?unsafe?\nx\ny\n"   ("safe?unsafe?" + "\nx\ny\n")
	// fn5+fn9: "safe?unsafe?\nx\ny\n"   ("safe?unsafe?" + "\nx\ny\n")
	// fn6+fn0: "\n"   ("\n" + "")
	// fn6+fn1: "\na"   ("\n" + "a")
	// fn6+fn2: "\na"   ("\n" + "a")
	// fn6+fn3: "\nbc"   ("\n" + "bc")
	// fn6+fn4: "\nbc"   ("\n" + "bc")
	// fn6+fn5: "\nsafe‹unsafe›"   ("\n" + "safe‹unsafe›")
	// fn6+fn6: "\n\n"   ("\n" + "\n")
	// fn6+fn7: "\n\n"   ("\n" + "\n")
	// fn6+fn8: "\n\nx\ny\n"   ("\n" + "\nx\ny\n")
	// fn6+fn9: "\n\nx\ny\n"   ("\n" + "\nx\ny\n")
	// fn7+fn0: "\n"   ("\n" + "")
	// fn7+fn1: "\na"   ("\n" + "a")
	// fn7+fn2: "\na"   ("\n" + "a")
	// fn7+fn3: "\nbc"   ("\n" + "bc")
	// fn7+fn4: "\nbc"   ("\n" + "bc")
	// fn7+fn5: "\nsafe‹unsafe›"   ("\n" + "safe‹unsafe›")
	// fn7+fn6: "\n\n"   ("\n" + "\n")
	// fn7+fn7: "\n\n"   ("\n" + "\n")
	// fn7+fn8: "\n\nx\ny\n"   ("\n" + "\nx\ny\n")
	// fn7+fn9: "\n\nx\ny\n"   ("\n" + "\nx\ny\n")
	// fn8+fn0: "\nx\ny\n"   ("\nx\ny\n" + "")
	// fn8+fn1: "\nx\ny\na"   ("\nx\ny\n" + "a")
	// fn8+fn2: "\nx\ny\na"   ("\nx\ny\n" + "a")
	// fn8+fn3: "\nx\ny\nbc"   ("\nx\ny\n" + "bc")
	// fn8+fn4: "\nx\ny\nbc"   ("\nx\ny\n" + "bc")
	// fn8+fn5: "\nx\ny\nsafe‹unsafe›"   ("\nx\ny\n" + "safe‹unsafe›")
	// fn8+fn6: "\nx\ny\n\n"   ("\nx\ny\n" + "\n")
	// fn8+fn7: "\nx\ny\n\n"   ("\nx\ny\n" + "\n")
	// fn8+fn8: "\nx\ny\n\nx\ny\n"   ("\nx\ny\n" + "\nx\ny\n")
	// fn8+fn9: "\nx\ny\n\nx\ny\n"   ("\nx\ny\n" + "\nx\ny\n")
	// fn9+fn0: "\nx\ny\n"   ("\nx\ny\n" + "")
	// fn9+fn1: "\nx\ny\na"   ("\nx\ny\n" + "a")
	// fn9+fn2: "\nx\ny\na"   ("\nx\ny\n" + "a")
	// fn9+fn3: "\nx\ny\nbc"   ("\nx\ny\n" + "bc")
	// fn9+fn4: "\nx\ny\nbc"   ("\nx\ny\n" + "bc")
	// fn9+fn5: "\nx\ny\nsafe‹unsafe›"   ("\nx\ny\n" + "safe‹unsafe›")
	// fn9+fn6: "\nx\ny\n\n"   ("\nx\ny\n" + "\n")
	// fn9+fn7: "\nx\ny\n\n"   ("\nx\ny\n" + "\n")
	// fn9+fn8: "\nx\ny\n\nx\ny\n"   ("\nx\ny\n" + "\nx\ny\n")
	// fn9+fn9: "\nx\ny\n\nx\ny\n"   ("\nx\ny\n" + "\nx\ny\n")
	//
	// 2  ->  1
	// fn0+fn0: ""   ("" + "")
	// fn0+fn1: "a"   ("" + "a")
	// fn0+fn2: "a"   ("" + "a")
	// fn0+fn3: "bc"   ("" + "bc")
	// fn0+fn4: "bc"   ("" + "bc")
	// fn0+fn5: "safe?unsafe?"   ("" + "safe?unsafe?")
	// fn0+fn6: "\n"   ("" + "\n")
	// fn0+fn7: "\n"   ("" + "\n")
	// fn0+fn8: "\nx\ny\n"   ("" + "\nx\ny\n")
	// fn0+fn9: "\nx\ny\n"   ("" + "\nx\ny\n")
	// fn1+fn0: "a"   ("a" + "")
	// fn1+fn1: "aa"   ("a" + "a")
	// fn1+fn2: "aa"   ("a" + "a")
	// fn1+fn3: "abc"   ("a" + "bc")
	// fn1+fn4: "abc"   ("a" + "bc")
	// fn1+fn5: "asafe?unsafe?"   ("a" + "safe?unsafe?")
	// fn1+fn6: "a\n"   ("a" + "\n")
	// fn1+fn7: "a\n"   ("a" + "\n")
	// fn1+fn8: "a\nx\ny\n"   ("a" + "\nx\ny\n")
	// fn1+fn9: "a\nx\ny\n"   ("a" + "\nx\ny\n")
	// fn2+fn0: "a"   ("a" + "")
	// fn2+fn1: "aa"   ("a" + "a")
	// fn2+fn2: "aa"   ("a" + "a")
	// fn2+fn3: "abc"   ("a" + "bc")
	// fn2+fn4: "abc"   ("a" + "bc")
	// fn2+fn5: "asafe?unsafe?"   ("a" + "safe?unsafe?")
	// fn2+fn6: "a\n"   ("a" + "\n")
	// fn2+fn7: "a\n"   ("a" + "\n")
	// fn2+fn8: "a\nx\ny\n"   ("a" + "\nx\ny\n")
	// fn2+fn9: "a\nx\ny\n"   ("a" + "\nx\ny\n")
	// fn3+fn0: "bc"   ("bc" + "")
	// fn3+fn1: "bca"   ("bc" + "a")
	// fn3+fn2: "bca"   ("bc" + "a")
	// fn3+fn3: "bcbc"   ("bc" + "bc")
	// fn3+fn4: "bcbc"   ("bc" + "bc")
	// fn3+fn5: "bcsafe?unsafe?"   ("bc" + "safe?unsafe?")
	// fn3+fn6: "bc\n"   ("bc" + "\n")
	// fn3+fn7: "bc\n"   ("bc" + "\n")
	// fn3+fn8: "bc\nx\ny\n"   ("bc" + "\nx\ny\n")
	// fn3+fn9: "bc\nx\ny\n"   ("bc" + "\nx\ny\n")
	// fn4+fn0: "bc"   ("bc" + "")
	// fn4+fn1: "bca"   ("bc" + "a")
	// fn4+fn2: "bca"   ("bc" + "a")
	// fn4+fn3: "bcbc"   ("bc" + "bc")
	// fn4+fn4: "bcbc"   ("bc" + "bc")
	// fn4+fn5: "bcsafe?unsafe?"   ("bc" + "safe?unsafe?")
	// fn4+fn6: "bc\n"   ("bc" + "\n")
	// fn4+fn7: "bc\n"   ("bc" + "\n")
	// fn4+fn8: "bc\nx\ny\n"   ("bc" + "\nx\ny\n")
	// fn4+fn9: "bc\nx\ny\n"   ("bc" + "\nx\ny\n")
	// fn5+fn0: "safe‹unsafe›"   ("safe‹unsafe›" + "")
	// fn5+fn1: "safe‹unsafe›a"   ("safe‹unsafe›" + "a")
	// fn5+fn2: "safe‹unsafe›a"   ("safe‹unsafe›" + "a")
	// fn5+fn3: "safe‹unsafe›bc"   ("safe‹unsafe›" + "bc")
	// fn5+fn4: "safe‹unsafe›bc"   ("safe‹unsafe›" + "bc")
	// fn5+fn5: "safe‹unsafe›safe?unsafe?"   ("safe‹unsafe›" + "safe?unsafe?")
	// fn5+fn6: "safe‹unsafe›\n"   ("safe‹unsafe›" + "\n")
	// fn5+fn7: "safe‹unsafe›\n"   ("safe‹unsafe›" + "\n")
	// fn5+fn8: "safe‹unsafe›\nx\ny\n"   ("safe‹unsafe›" + "\nx\ny\n")
	// fn5+fn9: "safe‹unsafe›\nx\ny\n"   ("safe‹unsafe›" + "\nx\ny\n")
	// fn6+fn0: "\n"   ("\n" + "")
	// fn6+fn1: "\na"   ("\n" + "a")
	// fn6+fn2: "\na"   ("\n" + "a")
	// fn6+fn3: "\nbc"   ("\n" + "bc")
	// fn6+fn4: "\nbc"   ("\n" + "bc")
	// fn6+fn5: "\nsafe?unsafe?"   ("\n" + "safe?unsafe?")
	// fn6+fn6: "\n\n"   ("\n" + "\n")
	// fn6+fn7: "\n\n"   ("\n" + "\n")
	// fn6+fn8: "\n\nx\ny\n"   ("\n" + "\nx\ny\n")
	// fn6+fn9: "\n\nx\ny\n"   ("\n" + "\nx\ny\n")
	// fn7+fn0: "\n"   ("\n" + "")
	// fn7+fn1: "\na"   ("\n" + "a")
	// fn7+fn2: "\na"   ("\n" + "a")
	// fn7+fn3: "\nbc"   ("\n" + "bc")
	// fn7+fn4: "\nbc"   ("\n" + "bc")
	// fn7+fn5: "\nsafe?unsafe?"   ("\n" + "safe?unsafe?")
	// fn7+fn6: "\n\n"   ("\n" + "\n")
	// fn7+fn7: "\n\n"   ("\n" + "\n")
	// fn7+fn8: "\n\nx\ny\n"   ("\n" + "\nx\ny\n")
	// fn7+fn9: "\n\nx\ny\n"   ("\n" + "\nx\ny\n")
	// fn8+fn0: "\nx\ny\n"   ("\nx\ny\n" + "")
	// fn8+fn1: "\nx\ny\na"   ("\nx\ny\n" + "a")
	// fn8+fn2: "\nx\ny\na"   ("\nx\ny\n" + "a")
	// fn8+fn3: "\nx\ny\nbc"   ("\nx\ny\n" + "bc")
	// fn8+fn4: "\nx\ny\nbc"   ("\nx\ny\n" + "bc")
	// fn8+fn5: "\nx\ny\nsafe?unsafe?"   ("\nx\ny\n" + "safe?unsafe?")
	// fn8+fn6: "\nx\ny\n\n"   ("\nx\ny\n" + "\n")
	// fn8+fn7: "\nx\ny\n\n"   ("\nx\ny\n" + "\n")
	// fn8+fn8: "\nx\ny\n\nx\ny\n"   ("\nx\ny\n" + "\nx\ny\n")
	// fn8+fn9: "\nx\ny\n\nx\ny\n"   ("\nx\ny\n" + "\nx\ny\n")
	// fn9+fn0: "\nx\ny\n"   ("\nx\ny\n" + "")
	// fn9+fn1: "\nx\ny\na"   ("\nx\ny\n" + "a")
	// fn9+fn2: "\nx\ny\na"   ("\nx\ny\n" + "a")
	// fn9+fn3: "\nx\ny\nbc"   ("\nx\ny\n" + "bc")
	// fn9+fn4: "\nx\ny\nbc"   ("\nx\ny\n" + "bc")
	// fn9+fn5: "\nx\ny\nsafe?unsafe?"   ("\nx\ny\n" + "safe?unsafe?")
	// fn9+fn6: "\nx\ny\n\n"   ("\nx\ny\n" + "\n")
	// fn9+fn7: "\nx\ny\n\n"   ("\nx\ny\n" + "\n")
	// fn9+fn8: "\nx\ny\n\nx\ny\n"   ("\nx\ny\n" + "\nx\ny\n")
	// fn9+fn9: "\nx\ny\n\nx\ny\n"   ("\nx\ny\n" + "\nx\ny\n")
}

// This test checks that moving mode from SafeEscaped to UnsafeEscaped
// does not erase safe spaces written previously.
func TestBufferPreserveSafeSpacesWhenSwitchingToUnsafe(t *testing.T) {
	var b Buffer

	b.SetMode(SafeEscaped)
	b.WriteRune('o')
	b.WriteRune('1')
	b.WriteRune(' ')
	t.Logf("%+v", b)
	b.SetMode(UnsafeEscaped)
	t.Logf("%+v", b)
	if expected, actual := m.RedactableString(`o1 `), b.RedactableString(); expected != actual {
		t.Errorf("expected %q, got %q", expected, actual)
	}
}
