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
	"testing"

	m "github.com/cockroachdb/redact/internal/markers"
	w "github.com/cockroachdb/redact/internal/redact"
	ifmt "github.com/cockroachdb/redact/internal/rfmt"
)

func TestBuilder(t *testing.T) {
	var b StringBuilder

	fmt.Fprint(&b, "unsafe")
	b.SafeRune('\n')

	b.Print("unsafe")
	b.SafeRune('\n')

	b.Print(w.Safe("safe"))
	b.SafeRune('\n')

	b.Printf("safe")
	b.SafeRune('\n')

	b.Printf("hello %v %v", w.Safe("safe"), "unsafe")
	b.SafeRune('\n')

	b.SafeString("safe\n")

	b.SafeInt(123)
	b.SafeUint(456)
	b.SafeFloat(3.14)

	b.SafeRune('S')
	b.SafeRune('\n')

	b.UnsafeString("unsafe")
	b.SafeRune('\n')
	b.UnsafeString("unsafe" + m.StartS[:1])
	b.SafeRune('\n')

	b.UnsafeRune('U')
	b.SafeRune('\n')

	b.UnsafeByte('U')
	b.SafeRune('\n')

	b.UnsafeByte(m.StartS[0])
	b.SafeRune('\n')

	b.UnsafeBytes([]byte("UUU"))
	b.SafeRune('\n')

	actualR := b.RedactableString()
	const expectedR = `‹unsafe›
‹unsafe›
safe
safe
hello safe ‹unsafe›
safe
1234563.14S
‹unsafe›
‹unsafe` + "\342" + `?›
‹U›
‹U›
‹?›
‹UUU›
`
	if actualR != expectedR {
		t.Errorf("expected:\n%s\n\ngot:\n%s", expectedR, actualR)
	}
	if actualB := b.RedactableBytes(); string(actualB) != expectedR {
		t.Errorf("expected:\n%s\n\ngot:\n%s", expectedR, actualB)
	}

	if actualR2 := ifmt.Sprint(&b); actualR2 != expectedR {
		t.Errorf("expected:\n%s\n\ngot:\n%s", expectedR, actualR2)
	}

	actual := b.String()
	const expected = `unsafe
unsafe
safe
safe
hello safe unsafe
safe
1234563.14S
unsafe
unsafe` + "\342" + `?
U
U
?
UUU
`
	if actual != expected {
		t.Errorf("expected:\n%s\n\ngot:\n%s", expected, actual)
	}

	if actual := fmt.Sprintf("%v", b); actual != expected {
		t.Errorf("expected:\n%s\n\ngot:\n%s", expected, actual)
	}
}

func TestMixedWrites(t *testing.T) {
	t.Run("oneline", func(t *testing.T) {
		var b StringBuilder
		b.SafeString("safe")
		b.WriteString("unsafe")
		b.SafeString("")
		b.WriteByte('U')
		b.SafeString("")
		b.WriteRune('U')
		actual := b.RedactableString()
		const expected = "safe‹unsafeUU›"
		if actual != expected {
			t.Errorf("expected:\n%s\n\ngot:\n%s", expected, actual)
		}
		const expectedWithoutMarkers = "safeunsafeUU"
		if ractual := actual.StripMarkers(); ractual != expectedWithoutMarkers {
			t.Errorf("expected:\n%s\n\ngot:\n%s", expectedWithoutMarkers, ractual)
		}
	})

	t.Run("multiline", func(t *testing.T) {
		var b StringBuilder
		b.SafeString("\nsafe\n")
		b.WriteString("\nunsafe\n")
		b.SafeString("\n")
		b.WriteByte('\n')
		b.SafeString("\n")
		b.WriteRune('\n')
		actual := b.RedactableString()
		const expected = "\nsafe\n\n‹unsafe›\n\n\n\n\n"
		if actual != expected {
			t.Errorf("expected:\n%q\n\ngot:\n%q", expected, actual)
		}
		const expectedWithoutMarkers = "\nsafe\n\nunsafe\n\n\n\n\n"
		if ractual := actual.StripMarkers(); ractual != expectedWithoutMarkers {
			t.Errorf("expected:\n%q\n\ngot:\n%q", expectedWithoutMarkers, ractual)
		}
	})
}
