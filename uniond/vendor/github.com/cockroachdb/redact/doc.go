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

// Package redact provides facilities for separating “safe” and
// “unsafe” pieces of data when logging and constructing error object.
//
// An item is said to be “safe” if it is proven to not contain
// PII or otherwise confidential information that should not escape
// the boundaries of the current system, for example via telemetry
// or crash reporting. Conversely, data is considered “unsafe”
// until/unless it is known to be “safe”.
//
// Example use:
//
//     redactable := redact.Sprintf("hello %s", "universe")
//
//     // At this point, 'redactable' contains "hello ‹universe›".
//
//     // This prints "hello universe":
//     fmt.Println(redactable.StripMarkers())
//
//     // This reports "hello ‹×›":
//     fmt.Println(redactable.Redact())
//
//
// When defining your own custom types, you can define
// a SafeFormat method, implementing the redact.SafeFormatter
// interface in a way you'd otherwise implement fmt.Formatter.
// This is then recognized by this package's API automatically.
//
// Alternatively:
//
// - you can implement the SafeValue interface, which tells the
// redact package to always the default formatting of a type
// as safe and thus not included inside redaction markers.
//
// - you can include a value within redact.Safe() and redact.Unsafe()
// in redact.Sprintf / redact.Fprintf calls, to force
// the omission or inclusion of redaction markers.
package redact
