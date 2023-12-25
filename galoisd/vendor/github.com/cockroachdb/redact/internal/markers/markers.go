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

package markers

import i "github.com/cockroachdb/redact/interfaces"

// RedactableString is a string that contains a mix of safe and unsafe
// bits of data, but where it is known that unsafe bits are enclosed
// by redaction markers ‹ and ›, and occurrences of the markers
// inside the original data items have been escaped.
//
// Instances of RedactableString should not be constructed directly;
// instead use the facilities from print.go (Sprint, Sprintf)
// or the methods below.
type RedactableString string

// StripMarkers removes the redaction markers from the
// RedactableString. This returns an unsafe string where all safe and
// unsafe bits are mixed together.
func (s RedactableString) StripMarkers() string {
	return ReStripMarkers.ReplaceAllString(string(s), "")
}

// Redact replaces all occurrences of unsafe substrings by the
// “Redacted” marker, ‹×›. The result string is still safe.
func (s RedactableString) Redact() RedactableString {
	return RedactableString(ReStripSensitive.ReplaceAllString(string(s), RedactedS))
}

// ToBytes converts the string to a byte slice.
func (s RedactableString) ToBytes() RedactableBytes {
	return RedactableBytes([]byte(string(s)))
}

// SafeFormat formats the redactable safely.
func (s RedactableString) SafeFormat(sp i.SafePrinter, _ rune) {
	// As per annotateArgs() in markers_internal_print.go,
	// we consider the redactable string not further formattable.
	sp.Print(s)
}

// RedactableBytes is like RedactableString but is a byte slice.
//
// Instances of RedactableBytes should not be constructed directly;
// instead use the facilities from print.go (Sprint, Sprintf)
// or the methods below.
type RedactableBytes []byte

// StripMarkers removes the redaction markers from the
// RedactableBytes. This returns an unsafe string where all safe and
// unsafe bits are mixed together.
func (s RedactableBytes) StripMarkers() []byte {
	return ReStripMarkers.ReplaceAll([]byte(s), nil)
}

// Redact replaces all occurrences of unsafe substrings by the
// “Redacted” marker, ‹×›.
func (s RedactableBytes) Redact() RedactableBytes {
	return RedactableBytes(ReStripSensitive.ReplaceAll(s, RedactedBytes))
}

// ToString converts the byte slice to a string.
func (s RedactableBytes) ToString() RedactableString {
	return RedactableString(string([]byte(s)))
}

// SafeFormat formats the redactable safely.
func (s RedactableBytes) SafeFormat(sp i.SafePrinter, _ rune) {
	// As per annotateArgs() in markers_internal_print.go,
	// we consider the redactable bytes not further formattable.
	sp.Print(s)
}

// StartMarker returns the start delimiter for an unsafe string.
func StartMarker() []byte { return []byte(StartS) }

// EndMarker returns the end delimiter for an unsafe string.
func EndMarker() []byte { return []byte(EndS) }

// RedactedMarker returns the special string used by Redact.
func RedactedMarker() []byte { return []byte(RedactedS) }

// EscapeMarkers escapes the special delimiters from the provided
// byte slice.
func EscapeMarkers(s []byte) []byte {
	return ReStripMarkers.ReplaceAll(s, EscapeMarkBytes)
}
